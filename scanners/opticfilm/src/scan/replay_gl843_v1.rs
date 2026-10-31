// SPDX-License-Identifier: GPL-3.0-or-later
//! Replay a recorded OpticFilm 7600i v1 job (`control` / `write` / `read` / `delay` ops).
//!
//! Recorded poll counts are replaced by live waits: write acknowledge, bulk
//! completion, motor idle before a start, FEEDFSH after the job's own move,
//! BUFEMPTY before image reads. Profiles stay untyped JSON, as in Python.

use std::collections::{BTreeMap, HashMap, HashSet};

use base64::Engine;
use serde_json::{Value, json};

use super::calib_gl843_v1::Calibration;
use crate::asic::gl843_v1::Gl843V1Usb;
use crate::clock::{monotonic, sleep};
use crate::difflib::{SequenceMatcher, Tag};
use crate::error::{Error, Result};
use crate::num::median;
use crate::usb::transport::ControlData;

pub const STATUS: i64 = 0x41;
pub const MOTORENB: i64 = 0x01;
pub const FEEDFSH: i64 = 0x20;
pub const BUFEMPTY: i64 = 0x40;
pub const CHUNK: usize = 0x40000;
pub const PACKET: usize = 512;
pub const THROUGHPUT_GRACE_S: f64 = 6.0;
pub const THROUGHPUT_MIN: f64 = 0.9;
pub const SHADING_UNITY: i64 = 0x2000;
/// SilverFast's infrared shading: no dark offset, gain = target * unity / white.
pub const IR_SHADING_TARGET: i64 = 0x13000;

fn b64() -> base64::engine::GeneralPurpose {
    base64::engine::general_purpose::STANDARD
}

fn vi(v: &Value) -> i64 {
    v.as_i64().or_else(|| v.as_f64().map(|f| f as i64)).or_else(|| v.as_bool().map(i64::from)).unwrap_or(0)
}

fn data(op: &Value) -> Vec<i64> {
    op["data"].as_array().map(|a| a.iter().map(vi).collect()).unwrap_or_default()
}

fn kind(op: &Value) -> &str {
    op["kind"].as_str().unwrap_or("")
}

fn is_write83(op: &Value) -> bool {
    kind(op) == "control" && vi(&op["rt"]) == 0x40 && vi(&op["value"]) == 0x83 && data(op).len() > 1
}

fn decode(s: &Value) -> Result<Vec<u8>> {
    b64().decode(s.as_str().unwrap_or("")).map_err(|e| Error::Asic(format!("bad base64 in replay profile: {e}")))
}

fn with(op: &Value, key: &str, v: Value) -> Value {
    let mut o = op.clone();
    o[key] = v;
    o
}

fn addr_op() -> Value {
    json!({"kind": "control", "rt": 0x40, "request": 0x0C, "value": 0x83, "index": 0, "data": [STATUS]})
}

fn ack_op() -> Value {
    json!({"kind": "control", "rt": 0xC0, "request": 0x0C, "value": 0x8E, "index": 0x20, "length": 1})
}

fn writes_motor_start(op: &Value) -> bool {
    let d = data(op);
    vi(&op["rt"]) == 0x40 && vi(&op["value"]) == 0x83 && d.len() > 1 && d.chunks_exact(2).any(|p| p[0] == 0x0F && p[1] == 1)
}

fn table(s: &Value) -> Result<Vec<i64>> {
    Ok(decode(s)?.chunks_exact(2).map(|p| p[0] as i64 | ((p[1] as i64) << 8)).collect())
}

fn table_b64(words: impl Iterator<Item = i64>) -> Value {
    let b: Vec<u8> = words.flat_map(|v| [(v & 0xFF) as u8, (v >> 8) as u8]).collect();
    json!(b64().encode(b))
}

/// Python `round(x, n)` for the JSON scan summary.
fn round_to(x: f64, n: i32) -> f64 {
    crate::num::py_round_to(x, n)
}

fn as_num(x: f64) -> Value {
    if x.fract() == 0.0 && x.abs() < 1e15 { json!(x as i64) } else { json!(x) }
}

/// Set the main scan's CCD dummy lines (`recorded` / `fewer` / `none`) or exposure ×k.
pub fn prepare_profile(profile: &Value, dummy_lines: &str, exposure_multiplier: i64, single_sample: bool) -> Result<Value> {
    if !matches!(dummy_lines, "recorded" | "fewer" | "none") || !(1..=4).contains(&exposure_multiplier) {
        return Err(Error::Value("dummy_lines must be recorded/fewer/none and exposure_multiplier 1-4".into()));
    }
    let recorded = vi(&profile["scan"]["lineSel"]);
    let line_sel = if exposure_multiplier > 1 || dummy_lines == "none" {
        0
    } else if dummy_lines == "fewer" {
        (recorded - 1).max(0)
    } else {
        recorded
    };
    if line_sel == recorded && exposure_multiplier == 1 && !single_sample {
        return Ok(profile.clone());
    }
    let k = exposure_multiplier;
    let ops = profile["ops"].as_array().cloned().unwrap_or_default();
    let main = vi(&profile["mainFrame"]);
    let mframe = &profile["frames"][main as usize];
    let factor = k as f64 * (line_sel + 1) as f64 / (recorded + 1) as f64;
    let first_read = ops
        .iter()
        .position(|o| kind(o) == "read" && vi(&o["frame"]) == main)
        .ok_or_else(|| Error::Asic("profile has no recognisable main-scan start".into()))?;
    let start = (0..first_read).rev().find(|&i| writes_motor_start(&ops[i])).map_or(-1, |i| i as i64);
    let prev_read = if start > 0 { (0..start as usize).rev().find(|&i| kind(&ops[i]) == "read").map_or(-1, |i| i as i64) } else { -1 };
    if !(start > prev_read && prev_read >= 0) {
        return Err(Error::Asic("profile has no recognisable main-scan start".into()));
    }
    let start = start as usize;
    let mut slot: Option<i64> = None;
    let mut tables = Vec::new();
    for (i, o) in ops.iter().enumerate().take(start) {
        if is_write83(o) {
            for p in data(o).chunks_exact(2) {
                if p[0] == 0x5B {
                    slot = if p[1] & 0x40 != 0 { Some((p[1] >> 3) & 7) } else { None };
                }
            }
        }
        if kind(o) == "write" && i as i64 > prev_read && slot.is_some_and(|s| s <= 2) {
            tables.push(i);
        }
    }
    if tables.is_empty() {
        return Err(Error::Asic("profile has no main-scan motor table".into()));
    }
    let cruise = *table(&ops[tables[0]]["data"])?.last().unwrap();
    let nxt_f = cruise as f64 * factor / if single_sample { 2.0 } else { 1.0 };
    if nxt_f != nxt_f.trunc() || !(nxt_f > 0.0 && nxt_f <= 65535.0) {
        return Err(Error::Asic("dummy line change does not give a whole motor period".into()));
    }
    let nxt = nxt_f as i64;
    let line_reg = mframe["regs"].get("30").map(vi);
    let Some(line_reg) = line_reg.filter(|&l| l & 0x0F == recorded) else {
        return Err(Error::Asic("recorded main-scan LINESEL not found".into()));
    };
    let mut new_ops = ops.clone();
    for &i in &tables {
        let t = table(&ops[i]["data"])?;
        if *t.last().unwrap() != cruise {
            return Err(Error::Asic("main-scan tables disagree".into()));
        }
        new_ops[i] = with(&ops[i], "data", table_b64(t.into_iter().map(|v| if v == cruise { nxt } else { v })));
    }
    let lp = vi(&profile["scan"]["lPeriod"]) * k;
    let mut changed: Vec<(i64, i64)> = vec![(0x1E, (line_reg & 0xF0) | line_sel)];
    if k > 1 {
        changed.extend([(0x38, lp >> 8), (0x39, lp & 0xFF), (0x20, 0x08)]);
    } else if line_sel != recorded {
        let maxwd = vi(&mframe["pixels"]) * 6 >> 1;
        changed.extend([(0x35, maxwd >> 16), (0x36, (maxwd >> 8) & 0xFF), (0x37, maxwd & 0xFF), (0x20, 0x10), (0x02, vi(&mframe["regs"]["2"]) | 0x40)]);
    }
    let mut lines = vi(&mframe["lines"]);
    if single_sample {
        lines /= 2;
        changed.extend([(0x25, (2 * lines) >> 16), (0x26, (2 * lines >> 8) & 0xFF), (0x27, 2 * lines & 0xFF)]);
        let n = vi(&mframe["pixels"]) * lines * 6;
        if vi(&new_ops[first_read]["length"]) != vi(&mframe["bytes"]) {
            return Err(Error::Asic("main frame is not read in one piece".into()));
        }
        new_ops[first_read] = with(&new_ops[first_read], "length", json!(n));
        let setup = (0..first_read).rev().find(|&i| vi(&new_ops[i]["value"]) == 0x82).expect("bulk setup");
        new_ops[setup] = with_length(&new_ops[setup], n);
    }
    let d = data(&new_ops[start]);
    let mut nd: Vec<i64> = Vec::new();
    for p in d.chunks_exact(2) {
        if p[0] == 0x0F {
            nd.extend(changed.iter().flat_map(|&(a, b)| [a, b]));
        }
        nd.extend([p[0], p[1]]);
    }
    new_ops[start] = with(&new_ops[start], "data", json!(nd));
    let mut frames = profile["frames"].as_array().cloned().unwrap_or_default();
    let mut regs = mframe["regs"].clone();
    for &(a, v) in &changed {
        regs[a.to_string()] = json!(v);
    }
    let mut f = frames[main as usize].clone();
    f["lines"] = json!(lines);
    f["bytes"] = json!(vi(&mframe["pixels"]) * lines * 6);
    f["regs"] = regs;
    frames[main as usize] = f;
    let s0 = &profile["scan"];
    let mut scan = s0.clone();
    scan["lineSel"] = json!(line_sel);
    scan["lPeriod"] = json!(lp);
    let c02 = changed.iter().find(|e| e.0 == 0x02).map_or(0, |e| e.1);
    scan["backtracking"] = json!(c02 & 0x40 == 0);
    scan["lineSeconds"] = as_num(round_to(s0["lineSeconds"].as_f64().unwrap_or(0.0) * factor, 6));
    scan["seconds"] = as_num(round_to(s0["seconds"].as_f64().unwrap_or(0.0) * factor * lines as f64 / vi(&mframe["lines"]) as f64, 1));
    scan["bytesPerSecond"] = json!(crate::num::py_round(s0["bytesPerSecond"].as_f64().unwrap_or(0.0) / factor));
    let yres = if single_sample { vi(&profile["yres"]) / 2 } else { vi(&profile["yres"]) };
    let mut out = profile.clone();
    out["ops"] = json!(new_ops);
    out["frames"] = json!(frames);
    out["scan"] = scan;
    out["yres"] = json!(yres);
    out["motorCruise"] = json!({"recorded": cruise, "used": nxt});
    Ok(out)
}

/// Collapse 5+ identical recorded status polls with MOTORENB before the first motor start.
pub fn collapse_recorded_waits(ops: &[Value]) -> Vec<Value> {
    let first = ops.iter().position(|o| kind(o) == "control" && writes_motor_start(o)).map_or(-1, |i| i as i64);
    let triplet = |i: usize| -> Option<i64> {
        if i + 2 >= ops.len() {
            return None;
        }
        let (a, b, c) = (&ops[i], &ops[i + 1], &ops[i + 2]);
        let all_control = kind(a) == "control" && kind(b) == "control" && kind(c) == "control";
        if all_control
            && data(a) == vec![STATUS]
            && a.get("data").is_some()
            && vi(&a["value"]) == 0x83
            && vi(&b["value"]) == 0x8E
            && vi(&b["index"]) == 0x20
            && vi(&c["value"]) == 0x84
            && c.get("register").map(vi) == Some(STATUS)
            && c["expected"].get(0).map(vi).is_some_and(|e| e & MOTORENB != 0)
        {
            return Some(vi(&c["expected"][0]));
        }
        None
    };
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < ops.len() {
        let e = if (i as i64) < first { triplet(i) } else { None };
        let Some(e) = e else {
            out.push(ops[i].clone());
            i += 1;
            continue;
        };
        let mut n = 1;
        while ((i + 3 * n) as i64) < first && triplet(i + 3 * n) == Some(e) {
            n += 1;
        }
        if n < 5 {
            out.extend_from_slice(&ops[i..i + 3 * n]);
        } else {
            out.push(ops[i].clone());
            out.push(ops[i + 1].clone());
            let mut c = ops[i + 2].clone();
            c["expected"] = json!([e & !MOTORENB]);
            c["waitMotorIdle"] = json!(n);
            out.push(c);
        }
        i += 3 * n;
    }
    if out.len() == ops.len() { ops.to_vec() } else { out }
}

/// GL843 shading upload: dark and gain words per pixel, 42 pixels per 512 bytes.
pub fn shading_table(gains: &[[i64; 3]], dark: [i64; 3]) -> Vec<u8> {
    let n = gains.len().div_ceil(42);
    let mut words = vec![[0u16; 6]; n * 42];
    for (x, w) in words.iter_mut().enumerate() {
        for c in 0..3 {
            w[2 * c] = dark[c] as u16;
            if x < gains.len() {
                w[2 * c + 1] = gains[x][c] as u16;
            }
        }
    }
    shading_blocks_words(&words)
}

fn shading_blocks_words(words: &[[u16; 6]]) -> Vec<u8> {
    let n = words.len().div_ceil(42);
    let mut out = vec![0u8; n * 512];
    for (x, w) in words.iter().enumerate() {
        let base = (x / 42) * 512 + (x % 42) * 12;
        for (k, v) in w.iter().enumerate() {
            out[base + 2 * k..base + 2 * k + 2].copy_from_slice(&v.to_le_bytes());
        }
    }
    out
}

/// Per-channel float64 column means of RGB16 lines `(pixels, 3)`.
fn column_means(white: &[u8], pixels: usize) -> Vec<[f64; 3]> {
    let lines = white.len() / (pixels * 6);
    let mut sums = vec![[0f64; 3]; pixels];
    for y in 0..lines {
        for (x, s) in sums.iter_mut().enumerate() {
            for (c, sc) in s.iter_mut().enumerate() {
                let i = ((y * pixels + x) * 3 + c) * 2;
                *sc += u16::from_le_bytes([white[i], white[i + 1]]) as f64;
            }
        }
    }
    sums.into_iter().map(|s| s.map(|v| v / lines as f64)).collect()
}

/// Infrared shading from a white frame read at unity gain.
pub fn white_shading(white: &[u8], pixels: usize) -> Vec<u8> {
    let gains: Vec<[i64; 3]> = column_means(white, pixels)
        .into_iter()
        .map(|m| m.map(|v| ((IR_SHADING_TARGET * SHADING_UNITY) as f64 / v.max(1.0)).round_ties_even().clamp(0.0, 65535.0) as i64))
        .collect();
    shading_table(&gains, [0; 3])
}

/// A shading table's per-pixel words with `extra` pixels in front.
fn widen_words(table: &[u8], pixels: usize, extra: usize) -> Vec<[u16; 6]> {
    let all: Vec<u16> = table.chunks_exact(2).map(|p| u16::from_le_bytes([p[0], p[1]])).collect();
    let mut words: Vec<[u16; 6]> = Vec::new();
    for blk in all.chunks(256) {
        for px in blk[..252.min(blk.len())].chunks_exact(6) {
            words.push([px[0], px[1], px[2], px[3], px[4], px[5]]);
        }
    }
    words.truncate(pixels);
    let reference: Vec<[f64; 6]> = (0..2)
        .map(|p| {
            let rows: Vec<&[u16; 6]> = words.iter().take(16).skip(p).step_by(2).collect();
            [0, 1, 2, 3, 4, 5].map(|k| median(rows.iter().map(|r| r[k] as f64)))
        })
        .collect();
    let mut out: Vec<[u16; 6]> = (0..extra)
        .map(|i| {
            let r = reference[(i as i64 - extra as i64).rem_euclid(2) as usize];
            r.map(|v| v.round_ties_even() as u16)
        })
        .collect();
    out.extend(words);
    out
}

/// A recorded final shading table with `extra` pixels added in front, every
/// gain computed from the white frame for the per-channel target the recorded gains give.
pub fn widened_shading(table: &[u8], extra: usize, white: &[u8], pixels: usize) -> Vec<u8> {
    let mean: Vec<[f64; 3]> = column_means(white, pixels).into_iter().map(|m| m.map(|v| v.max(1.0))).collect();
    let mut words = widen_words(table, pixels - extra, extra);
    let target = [0, 1, 2].map(|c| median(words.iter().zip(&mean).skip(extra).map(|(w, m)| w[2 * c + 1] as f64 * m[c])));
    for (w, m) in words.iter_mut().zip(&mean) {
        for c in 0..3 {
            w[2 * c + 1] = (target[c] / m[c]).round_ties_even().clamp(0.0, 65535.0) as u16;
        }
    }
    shading_blocks_words(&words)
}

/// A bulk set-up (`0x82`) for `n` bytes.
fn with_length(setup: &Value, n: i64) -> Value {
    let mut d: Vec<i64> = data(setup).into_iter().take(4).collect();
    d.extend((n as u32).to_le_bytes().iter().map(|&b| b as i64));
    with(setup, "data", json!(d))
}

fn reg(f: &Value, k: &str) -> i64 {
    vi(&f["regs"][k])
}

/// Start the dark, white and main frames at sensor pixel `start` instead of the recorded STRPIXEL.
pub fn widen_window(profile: &Value, start: i64) -> Result<Value> {
    let main = vi(&profile["mainFrame"]) as usize;
    let pf = profile["frames"].as_array().cloned().unwrap_or_default();
    let mf = &pf[main];
    let old = reg(mf, "48") << 8 | reg(mf, "49");
    let step = 1200 / (reg(mf, "44") << 8 | reg(mf, "45"));
    // An even pixel count: the chip sends no odd last pixel.
    let extra = crate::num::floor_div(crate::num::floor_div(old - start, step), 2) * 2;
    if extra <= 0 {
        return Ok(profile.clone());
    }
    let new = old - extra * step;
    let mut frames = Vec::new();
    for f in &pf {
        let mut f = f.clone();
        if reg(&f, "48") << 8 | reg(&f, "49") == old {
            let px = vi(&f["pixels"]) + extra;
            f["pixels"] = json!(px);
            f["bytes"] = json!(px * vi(&f["lines"]) * 6);
            f["regs"]["48"] = json!(new >> 8);
            f["regs"]["49"] = json!(new & 0xFF);
        }
        frames.push(f);
    }
    let white = vi(&profile["lamp"]["shading"]["frame"]);
    let pixels = vi(&frames[white as usize]["pixels"]);
    let size = (pixels + 41) / 42 * 512;
    let mut ops = profile["ops"].as_array().cloned().unwrap_or_default();
    let mut regs: HashMap<i64, i64> = HashMap::new();
    let mut header: i64 = -1;
    let mut headers: HashMap<usize, i64> = HashMap::new();
    let mut uploads: [Vec<usize>; 2] = [Vec::new(), Vec::new()];
    let mut white_read = false;
    for i in 0..ops.len() {
        let o = ops[i].clone();
        if kind(&o) == "control" && vi(&o["rt"]) == 0x40 && vi(&o["value"]) == 0x82 {
            header = i as i64;
        } else if is_write83(&o) {
            let mut pairs: Vec<(i64, i64)> = Vec::new();
            for p in data(&o).chunks_exact(2) {
                match pairs.iter_mut().find(|e| e.0 == p[0]) {
                    Some(e) => e.1 = p[1],
                    None => pairs.push((p[0], p[1])),
                }
                regs.insert(p[0], p[1]);
            }
            let g = |k| pairs.iter().find(|e| e.0 == k).map(|e| e.1);
            if (g(0x30), g(0x31)) == (Some(old >> 8), Some(old & 0xFF)) {
                for (k, v) in [(0x30, new >> 8), (0x31, new & 0xFF)] {
                    pairs.iter_mut().find(|e| e.0 == k).unwrap().1 = v;
                }
                ops[i] = with(&o, "data", json!(pairs.iter().flat_map(|&(a, b)| [a, b]).collect::<Vec<_>>()));
            }
        } else if kind(&o) == "read" {
            let fr = vi(&o["frame"]) as usize;
            white_read |= fr as i64 == white;
            let n = vi(&frames[fr]["bytes"]);
            let recorded = vi(&pf[fr]["bytes"]);
            if n != recorded {
                if vi(&o["length"]) != recorded {
                    return Err(Error::Asic("widened frame is not read in one piece".into()));
                }
                ops[i] = with(&o, "length", json!(n));
                ops[header as usize] = with_length(&ops[header as usize], n);
            }
        } else if kind(&o) == "write" && (regs.get(&0x5B), regs.get(&0x5C)) == (Some(&0x10), Some(&0x00)) {
            uploads[white_read as usize].push(i);
            headers.insert(i, header);
        }
    }
    // A set-up counts the table without its last block's padding.
    let used = pixels / 42 * 512 + pixels % 42 * 12;
    for live in [false, true] {
        let idx = uploads[live as usize].clone();
        if idx.is_empty() {
            return Err(Error::Asic("shading uploads not found".into()));
        }
        let mut lengths: Vec<i64> = idx
            .iter()
            .map(|&i| ops[i].get("length").map(vi).filter(|&l| l != 0).map_or_else(|| decode(&ops[i]["data"]).map(|d| d.len() as i64), Ok))
            .collect::<Result<_>>()?;
        let last_h = headers[idx.last().unwrap()];
        let last: Vec<usize> = idx.iter().copied().filter(|i| headers[i] == last_h).collect();
        let nl = lengths.len();
        lengths[nl - 1] = size - lengths[..nl - 1].iter().sum::<i64>();
        let tail = used - (lengths.iter().sum::<i64>() - lengths[nl - last.len()..].iter().sum::<i64>());
        if lengths[nl - 1] <= 0 || tail <= 0 {
            return Err(Error::Asic("widened shading does not fit the recorded uploads".into()));
        }
        ops[last_h as usize] = with_length(&ops[last_h as usize], tail);
        let mut recorded = Vec::new();
        for &i in &idx {
            if ops[i].get("data").is_some() {
                recorded.extend(decode(&ops[i]["data"])?);
            }
        }
        let tbl = if live { Vec::new() } else { shading_blocks_words(&widen_words(&recorded, (pixels - extra) as usize, extra as usize)) };
        let mut at = 0i64;
        for (&i, &n) in idx.iter().zip(&lengths) {
            if ops[i].get("shading").is_some() {
                ops[i] = with(&ops[i], "length", json!(n));
            } else if live {
                ops[i] = json!({"kind": "write", "length": n, "shading": {
                    "frame": white, "offset": at, "widen": {"table": b64().encode(&recorded), "extra": extra}}});
            } else {
                let (a, b) = (at as usize, ((at + n) as usize).min(tbl.len()));
                ops[i] = with(&ops[i], "data", json!(b64().encode(&tbl[a.min(b)..b])));
            }
            at += n;
        }
    }
    let mut scan = profile["scan"].clone();
    scan["bytesPerSecond"] = json!(crate::num::py_round(
        profile["scan"]["bytesPerSecond"].as_f64().unwrap_or(0.0) * vi(&frames[main]["pixels"]) as f64 / vi(&pf[main]["pixels"]) as f64
    ));
    let mut out = profile.clone();
    out["ops"] = json!(ops);
    out["frames"] = json!(frames);
    out["scan"] = scan;
    Ok(out)
}

/// Canonical text of a JSON value with Python equality (`1 == 1.0 == True`).
fn canon(v: Option<&Value>) -> String {
    match v {
        None | Some(Value::Null) => "None".into(),
        Some(Value::Bool(b)) => (*b as i64).to_string(),
        Some(Value::Number(n)) => match n.as_f64() {
            Some(f) if f.fract() == 0.0 => (f as i64).to_string(),
            Some(f) => format!("{f:?}"),
            None => n.to_string(),
        },
        Some(Value::String(s)) => format!("{s:?}"),
        Some(Value::Array(a)) => format!("({})", a.iter().map(|x| canon(Some(x))).collect::<Vec<_>>().join(",")),
        Some(o) => o.to_string(),
    }
}

fn full(op: &Value) -> String {
    if kind(op) == "control" {
        let tail = if vi(&op["rt"]) == 0x40 { canon(op.get("data")) } else { canon(op.get("register")) };
        return format!("C|{}|{}|{}|{tail}", canon(op.get("rt")), canon(op.get("value")), canon(op.get("index")));
    }
    format!("K|{}|{}|{}|{}", canon(op.get("kind")), canon(op.get("frame")), canon(op.get("ms")), canon(op.get("timedStop")))
}

fn shape(op: &Value) -> String {
    if is_write83(op) {
        let d = data(op);
        let addrs: Vec<String> = d.iter().step_by(2).map(|v| v.to_string()).collect();
        return format!("S83|({})", addrs.join(","));
    }
    if kind(op) == "control" {
        return full(op);
    }
    format!("K|{}|{}|{}", canon(op.get("kind")), canon(op.get("frame")), canon(op.get("timedStop")))
}

fn is_poll(op: &Value) -> bool {
    kind(op) == "control" && (vi(&op["rt"]) == 0xC0 || data(op) == vec![STATUS])
}

/// The infrared job at `color`'s resolution: `color` with the changes from
/// `color_ref` to `ir_ref`, and shading from the job's white frame.
pub fn derive_infrared(color: &Value, color_ref: &Value, ir_ref: &Value) -> Result<Value> {
    let ops = color["ops"].as_array().cloned().unwrap_or_default();
    let refo = color_ref["ops"].as_array().cloned().unwrap_or_default();
    let ir = ir_ref["ops"].as_array().cloned().unwrap_or_default();
    let mut at: HashMap<usize, usize> = HashMap::new();
    let (sa, sb): (Vec<String>, Vec<String>) = (refo.iter().map(shape).collect(), ops.iter().map(shape).collect());
    for (tag, a1, a2, b1, b2) in SequenceMatcher::new(&sa, &sb).get_opcodes() {
        if tag == Tag::Equal || (tag == Tag::Replace && a2 - a1 == b2 - b1) {
            for i in 0..a2 - a1 {
                at.insert(a1 + i, b1 + i);
            }
        }
    }
    let (fa, fb): (Vec<String>, Vec<String>) = (refo.iter().map(full).collect(), ir.iter().map(full).collect());
    let mut edits: Vec<(usize, usize, Vec<Value>)> = Vec::new();
    for (tag, a1, a2, b1, b2) in SequenceMatcher::new(&fa, &fb).get_opcodes() {
        let changed: Vec<&Value> = refo[a1..a2].iter().chain(&ir[b1..b2]).collect();
        if tag == Tag::Equal || changed.iter().all(|o| is_poll(o)) || changed.iter().all(|o| kind(o) == "delay") {
            continue;
        }
        let mut idx: Vec<Option<usize>> = (a1..a2).map(|i| at.get(&i).copied()).collect();
        if tag == Tag::Delete {
            idx.retain(Option::is_some);
            if idx.is_empty() {
                continue;
            }
        }
        let start = if let Some(first) = idx.first() { *first } else { at.get(&a1).copied() };
        let contiguous = start.is_some_and(|s| idx.iter().enumerate().all(|(k, v)| *v == Some(s + k)));
        let Some(start) = start.filter(|_| contiguous) else {
            return Err(Error::Asic(format!("infrared change at vendor op {a1} has no place in the {} dpi job", vi(&color["dpi"]))));
        };
        edits.push((start, start + idx.len(), ir[b1..b2].to_vec()));
    }
    let mut out = ops.clone();
    edits.sort_by(|a, b| b.0.cmp(&a.0));
    for (s, e, new) in edits {
        out.splice(s..e, new);
    }
    // Shading: unity gain while the white frame is read, then computed from it.
    let lamp = ir_ref["lamp"].clone();
    let white = vi(&lamp["shading"]["frame"]);
    let pixels = vi(&color["frames"][white as usize]["pixels"]) as usize;
    let unity = shading_table(&vec![[SHADING_UNITY; 3]; pixels], [0; 3]);
    let mut regs: HashMap<i64, i64> = HashMap::new();
    let mut frames_read: HashSet<i64> = HashSet::new();
    let mut offset: BTreeMap<bool, usize> = BTreeMap::new();
    for item in out.iter_mut() {
        let o = item.clone();
        if is_write83(&o) {
            for p in data(&o).chunks_exact(2) {
                regs.insert(p[0], p[1]);
            }
        } else if kind(&o) == "read" {
            frames_read.insert(vi(&o["frame"]));
        } else if kind(&o) == "write" && (regs.get(&0x5B), regs.get(&0x5C)) == (Some(&0x10), Some(&0x00)) {
            let n = decode(&o["data"])?.len();
            let live = frames_read.contains(&white);
            let o0 = *offset.get(&live).unwrap_or(&0);
            offset.insert(live, o0 + n);
            *item = if live {
                json!({"kind": "write", "length": n, "shading": {"frame": white, "offset": o0}})
            } else {
                json!({"kind": "write", "data": b64().encode(&unity[o0..(o0 + n).min(unity.len())])})
            };
        }
    }
    let vals: HashSet<usize> = offset.values().copied().collect();
    if vals != HashSet::from([unity.len()]) {
        return Err(Error::Asic("infrared shading uploads not found".into()));
    }
    let frames: Vec<Value> = color["frames"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .enumerate()
        .map(|(i, mut f)| {
            for k in ["3", "168"] {
                f["regs"][k] = ir_ref["frames"][i]["regs"][k].clone();
            }
            f
        })
        .collect();
    let mut res = color.clone();
    res["ops"] = json!(out);
    res["frames"] = json!(frames);
    res["lamp"] = lamp;
    res["name"] = json!(format!("Derived full frame {}, infrared", vi(&color["dpi"])));
    res["source"] = json!(format!("{} with the infrared changes of {}", color["source"].as_str().unwrap_or(""), ir_ref["source"].as_str().unwrap_or("")));
    Ok(res)
}

/// Refuse a profile whose frames are incomplete or whose image reads outlast the scan.
pub fn validate_profile(profile: &Value) -> Result<()> {
    let frames = profile["frames"].as_array().cloned().unwrap_or_default();
    let main = vi(&profile["mainFrame"]);
    let mut planned: HashMap<i64, i64> = HashMap::new();
    for op in profile["ops"].as_array().into_iter().flatten() {
        if kind(op) == "read" {
            let f = vi(&op["frame"]);
            let p = planned.entry(f).or_insert(0);
            *p += vi(&op["length"]);
            if *p > vi(&frames[f as usize]["bytes"]) {
                return Err(Error::Asic("invalid frame read budget".into()));
            }
        } else if kind(op) == "control" {
            let pm = planned.get(&main).copied().unwrap_or(0);
            if 0 < pm && pm < vi(&frames[main as usize]["bytes"]) {
                let d = data(op);
                let done = vi(&op["rt"]) == 0xC0 && vi(&op["value"]) == 0x8E && vi(&op["index"]) == 0x18;
                let shutdown = vi(&op["rt"]) == 0x40
                    && vi(&op["value"]) == 0x83
                    && d.len() > 1
                    && d.chunks_exact(2).any(|p| (p[0] == 1 && p[1] & 1 == 0) || (p[0] == 3 && p[1] & 0x10 == 0));
                if done || shutdown {
                    return Err(Error::Asic("main frame incomplete before transfer completion or shutdown".into()));
                }
            }
        }
    }
    for (i, f) in frames.iter().enumerate() {
        if planned.get(&(i as i64)).copied() != Some(vi(&f["bytes"])) {
            return Err(Error::Asic("incomplete frame in profile".into()));
        }
    }
    Ok(())
}

pub type CheckFn<'a> = &'a dyn Fn() -> Result<()>;

pub struct ReplayHooks<'a> {
    pub check: CheckFn<'a>,
    pub progress: Option<&'a mut dyn FnMut(i64, usize, usize)>,
    /// Called with `(frame, bytes)` when a calibration frame is complete.
    pub frame_done: Option<&'a mut dyn FnMut(i64, &[u8])>,
    pub timeout_s: f64,
    pub read_chunk: usize,
}

impl<'a> ReplayHooks<'a> {
    pub fn new(check: CheckFn<'a>) -> Self {
        Self { check, progress: None, frame_done: None, timeout_s: 60.0, read_chunk: CHUNK }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ReplayResult {
    pub main: Vec<u8>,
    pub frames: HashMap<i64, Vec<u8>>,
    pub positioning_stop_ms: Option<f64>,
}

struct Replay<'u, 'h> {
    usb: &'u mut Gl843V1Usb,
    h: ReplayHooks<'h>,
    regs: HashMap<i64, i64>,
    address: i64,
    status: Option<i64>,
    moved: Option<f64>,
}

impl Replay<'_, '_> {
    fn raw(&mut self, op: &Value) -> Result<Vec<u8>> {
        let (req, value, index) = (vi(&op["request"]) as u8, vi(&op["value"]) as u16, vi(&op["index"]) as u16);
        if vi(&op["rt"]) == 0x40 {
            let d = data(op);
            let bytes: Vec<u8> = d.iter().map(|&b| b as u8).collect();
            self.usb.transport.control_msg(0x40, req, value, index, ControlData::Out(&bytes), None)?;
            if value == 0x83 {
                if d.len() == 1 {
                    self.address = d[0];
                }
                for p in d.chunks_exact(2) {
                    self.regs.insert(p[0], p[1]);
                    self.usb.shadow.insert(p[0] as u8, p[1] as u8);
                }
            }
            return Ok(Vec::new());
        }
        let n = vi(&op["length"]) as usize;
        let r = self.usb.transport.control_msg(0xC0, req, value, index, ControlData::In(n), None)?;
        if r.len() != n {
            return Err(Error::Scan("short control read".into()));
        }
        if value == 0x84 {
            // The GL843 auto-increments the address after a read.
            if self.address == STATUS {
                self.status = Some(r[0] as i64);
            }
            self.address = (self.address + 1) & 0xFF;
        }
        Ok(r)
    }

    fn wait(&mut self, mut r: Vec<u8>, again: &mut dyn FnMut(&mut Self) -> Result<Vec<u8>>, ready: &dyn Fn(i64) -> bool, why: &str, pause: f64) -> Result<()> {
        let deadline = monotonic() + self.h.timeout_s;
        while !ready(r[0] as i64) {
            (self.h.check)()?;
            if monotonic() > deadline {
                return Err(Error::Scan(format!("timeout {why} (status 0x{:02x})", r[0])));
            }
            sleep(pause);
            r = again(self)?;
        }
        Ok(())
    }

    fn poll(&mut self, op: &Value, r: Vec<u8>, mask: i64, target: i64, why: &str) -> Result<()> {
        let op = op.clone();
        let mut status = move |me: &mut Self| -> Result<Vec<u8>> {
            me.raw(&addr_op())?;
            let a = me.raw(&ack_op())?;
            me.wait(a, &mut |m: &mut Self| m.raw(&ack_op()), &|s| s & 1 != 0, "waiting for write acknowledge", 0.005)?;
            me.raw(&op)
        };
        self.wait(r, &mut status, &|s| s & mask == target, why, 0.015)
    }

    fn control(&mut self, op: &Value) -> Result<()> {
        if vi(&op["rt"]) == 0x40 {
            let start = writes_motor_start(op);
            if start && let Some(s) = self.status.filter(|s| s & MOTORENB != 0) {
                let mut probe = ack_op();
                probe["index"] = json!(0);
                probe["value"] = json!(0x84);
                self.poll(&probe, vec![s as u8], MOTORENB, 0, "motor before start")?;
            }
            self.raw(op)?;
            if start {
                self.status = None;
                self.moved = Some(monotonic());
            }
            return Ok(());
        }
        let r = self.raw(op)?;
        let (value, index) = (vi(&op["value"]), vi(&op["index"]));
        let opc = op.clone();
        if value == 0x8E && index == 0x20 {
            self.wait(r, &mut |m: &mut Self| m.raw(&opc), &|s| s & 1 != 0, "waiting for write acknowledge", 0.005)?;
        } else if value == 0x8E && index == 0x18 {
            self.wait(r, &mut |m: &mut Self| m.raw(&opc), &|s| s & 0x0C == 0, "waiting for bulk completion", 0.02)?;
        } else if op.get("waitMotorIdle").is_some_and(|v| vi(v) != 0) {
            self.poll(op, r, MOTORENB, 0, "waiting for the previous move")?;
        } else if value == 0x84 && op.get("register").map(vi) == Some(STATUS) {
            let expected = vi(&op["expected"][0]);
            let scanning = self.regs.get(&1).copied().unwrap_or(0) & 1 != 0;
            if !scanning && expected & (FEEDFSH | MOTORENB) == FEEDFSH {
                // FEEDFSH latches the end of a move; before this job's own move it is the vendor's.
                if self.moved.is_none() {
                    self.poll(op, r, MOTORENB, 0, "waiting for the previous move")?;
                } else {
                    self.poll(op, r, FEEDFSH | MOTORENB, FEEDFSH, "waiting for the positioning move")?;
                }
            } else if scanning && expected & BUFEMPTY == 0 {
                self.poll(op, r, BUFEMPTY, 0, "waiting for scan data")?;
            }
        }
        Ok(())
    }
}

/// Replay a job; `calibrate` replaces recorded AFE/shading with live measurements.
pub fn run_profile(profile: &Value, usb: &mut Gl843V1Usb, hooks: ReplayHooks<'_>, calibrate: bool) -> Result<ReplayResult> {
    validate_profile(profile)?;
    let frames = profile["frames"].as_array().cloned().unwrap_or_default();
    let main = vi(&profile["mainFrame"]);
    let mut calibration = if calibrate { Some(Calibration::new(profile)?) } else { None };
    let lamp = profile.get("lamp").cloned().unwrap_or(json!({}));
    let lamp_frames: HashSet<i64> = ["line", "shading", "dark"].iter().filter_map(|k| lamp.get(*k).map(|l| vi(&l["frame"]))).collect();
    let mut calib_frames = lamp_frames.clone();
    if let Some(c) = &calibration {
        calib_frames.extend(c.first..=c.white);
    }
    // Without backtracking a slow host only pauses the carriage.
    let need = if profile["scan"].get("backtracking").is_none_or(|b| b.as_bool().unwrap_or(true)) { profile["scan"]["bytesPerSecond"].as_f64().unwrap_or(0.0) } else { 0.0 };
    let mut buffers: HashMap<i64, Vec<u8>> = calib_frames.iter().chain(std::iter::once(&main)).map(|&i| (i, vec![0u8; vi(&frames[i as usize]["bytes"]) as usize])).collect();
    let mut got: Vec<usize> = vec![0; frames.len()];
    let mut result = ReplayResult::default();
    let mut shading: HashMap<i64, Vec<u8>> = HashMap::new();
    let mut rp = Replay { usb, h: hooks, regs: HashMap::new(), address: 0, status: None, moved: None };
    let ops = collapse_recorded_waits(profile["ops"].as_array().map_or(&[][..], |v| v.as_slice()));
    for op in &ops {
        (rp.h.check)()?;
        match kind(op) {
            "delay" => {
                if !op.get("timedStop").is_some_and(|v| vi(v) != 0) {
                    sleep(op["ms"].as_f64().unwrap_or(0.0) / 1000.0);
                    continue;
                }
                // The vendor stops the first positioning move on the position-sensor event.
                let Some(moved) = rp.moved else {
                    return Err(Error::Asic("timed stop without a preceding motor start".into()));
                };
                let target = moved + op["ms"].as_f64().unwrap_or(0.0) / 1000.0;
                while monotonic() < target {
                    (rp.h.check)()?;
                    sleep((target - monotonic()).clamp(0.001, 0.02));
                }
                result.positioning_stop_ms = Some((monotonic() - moved) * 1000.0);
            }
            "control" => {
                (rp.h.check)()?;
                let op2 = match calibration.as_mut() {
                    Some(c) => c.control(op)?,
                    None => op.clone(),
                };
                rp.control(&op2)?;
            }
            "write" => {
                let payload = if calibration.is_some() && (rp.regs.get(&0x5B), rp.regs.get(&0x5C)) == (Some(&0x10), Some(&0)) {
                    let size = match op.get("length").map(vi).filter(|&l| l != 0) {
                        Some(l) => l as usize,
                        None => decode(&op["data"])?.len(),
                    };
                    calibration.as_mut().unwrap().upload(size)?
                } else if let Some(sh) = op.get("shading") {
                    let (f, start) = (vi(&sh["frame"]), vi(&sh["offset"]) as usize);
                    if !shading.contains_key(&f) {
                        let fdata = result.frames.get(&f).ok_or_else(|| Error::Scan(format!("shading frame {f} not read")))?;
                        let px = vi(&frames[f as usize]["pixels"]) as usize;
                        let t = match sh.get("widen") {
                            Some(w) => widened_shading(&decode(&w["table"])?, vi(&w["extra"]) as usize, fdata, px),
                            None => white_shading(fdata, px),
                        };
                        shading.insert(f, t);
                    }
                    let t = &shading[&f];
                    let end = (start + vi(&op["length"]) as usize).min(t.len());
                    t[start.min(end)..end].to_vec()
                } else {
                    decode(&op["data"])?
                };
                if rp.usb.transport.bulk_write(&payload, None)? != payload.len() {
                    return Err(Error::Scan("short bulk upload".into()));
                }
            }
            _ => {
                let i = vi(&op["frame"]);
                let fi = i as usize;
                let size = vi(&frames[fi]["bytes"]) as usize;
                let mut left = vi(&op["length"]) as usize;
                let mut asked = left;
                // Whole 512-byte packets, remainder on its own.
                let request = |asked: &mut usize, chunk: usize| {
                    let mut n = chunk.min(*asked);
                    if n >= PACKET {
                        n -= n % PACKET;
                    }
                    *asked -= n;
                    n
                };
                let mut n = request(&mut asked, rp.h.read_chunk);
                let mut part = rp.usb.transport.bulk_read(n, None)?;
                let mut t0: Option<f64> = None;
                let mut since = 0usize;
                while left > 0 {
                    (rp.h.check)()?;
                    if part.is_empty() || part.len() > n || got[fi] + part.len() > size {
                        return Err(Error::Scan(format!("frame {i}: bulk read returned {} of {n} bytes", part.len())));
                    }
                    left -= part.len();
                    asked += n - part.len(); // a short read is asked for again
                    let this = std::mem::take(&mut part);
                    if asked > 0 {
                        n = request(&mut asked, rp.h.read_chunk);
                        part = rp.usb.transport.bulk_read(n, None)?;
                    }
                    if let Some(b) = buffers.get_mut(&i) {
                        b[got[fi]..got[fi] + this.len()].copy_from_slice(&this);
                    }
                    got[fi] += this.len();
                    match t0 {
                        None => t0 = Some(monotonic()),
                        Some(t) if i == main => {
                            since += this.len();
                            let secs = monotonic() - t;
                            if secs > THROUGHPUT_GRACE_S && (since as f64 / secs) < need * THROUGHPUT_MIN {
                                return Err(Error::Scan(format!(
                                    "scan data at {:.2} MB/s, the scan needs {:.2} MB/s",
                                    since as f64 / secs / 1e6,
                                    need / 1e6
                                )));
                            }
                        }
                        _ => {}
                    }
                    if let Some(p) = rp.h.progress.as_mut() {
                        p(i, got[fi], size);
                    }
                }
                if i != main && got[fi] == size && let Some(b) = buffers.get(&i) {
                    result.frames.insert(i, b.clone());
                    if let Some(c) = calibration.as_mut() {
                        c.frame_done(i, b)?;
                    }
                    if lamp_frames.contains(&i)
                        && let Some(fd) = rp.h.frame_done.as_mut()
                    {
                        fd(i, b);
                    }
                }
            }
        }
    }
    for (i, f) in frames.iter().enumerate() {
        if got[i] as i64 != vi(&f["bytes"]) {
            return Err(Error::Scan("incomplete frame".into()));
        }
    }
    result.main = buffers.remove(&main).unwrap_or_default();
    Ok(result)
}
