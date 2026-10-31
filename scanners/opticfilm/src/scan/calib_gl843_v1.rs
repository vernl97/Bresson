// SPDX-License-Identifier: GPL-3.0-or-later
//! Live GL843/AD9826 calibration recovered from eight colour/IR USB jobs (7600i v1).

use ndarray::Array2;
use serde_json::{Value, json};

use crate::error::{Error, Result};
use crate::num::floor_div;

pub const UNITY: i64 = 8192;
pub const WHITE_RGB: [i64; 3] = [79380, 79119, 82739];

fn u16s(data: &[u8]) -> Vec<i64> {
    data.chunks_exact(2).map(|p| u16::from_le_bytes([p[0], p[1]]) as i64).collect()
}

/// 128 lines: discard eight samples at each end, then truncate the mean → `(pixels, 3)`.
pub fn reference(data: &[u8], pixels: usize) -> Array2<i64> {
    let a = u16s(data);
    Array2::from_shape_fn((pixels, 3), |(x, c)| {
        let mut col: Vec<i64> = (0..128).map(|y| a[(y * pixels + x) * 3 + c]).collect();
        col.sort_unstable();
        floor_div(col[8..120].iter().sum(), 112)
    })
}

pub fn dark_reference(data: &[u8], pixels: usize, stride: usize) -> Array2<i64> {
    let raw = reference(data, pixels);
    let mut smooth = Array2::<i64>::zeros(raw.dim());
    for parity in 0..stride {
        let idx: Vec<usize> = (parity..pixels).step_by(stride).collect();
        let n = idx.len();
        for c in 0..3 {
            let mut sums = vec![0i64; n + 1];
            for (k, &x) in idx.iter().enumerate() {
                sums[k + 1] = sums[k] + raw[[x, c]];
            }
            for (k, &x) in idx.iter().enumerate() {
                let end = (k + 100).min(n);
                smooth[[x, c]] = floor_div(sums[end] - sums[k], (end - k) as i64);
            }
        }
    }
    // Inclusive 64 fits every capture.
    Array2::from_shape_fn(raw.dim(), |p| if (raw[p] - smooth[p]).abs() > 64 { raw[p] } else { smooth[p] })
}

/// Dark and gain words for R, G, B per pixel, 42 pixels per 512-byte block.
pub fn shading_table(dark: &Array2<i64>, gain: &Array2<i64>) -> Vec<u8> {
    let px = dark.nrows();
    let n = (px + 41) / 42;
    let mut out = vec![0u8; n * 512];
    for x in 0..px {
        let (blk, k) = (x / 42, x % 42);
        for c in 0..3 {
            let base = blk * 512 + (k * 6 + c * 2) * 2;
            out[base..base + 2].copy_from_slice(&(dark[[x, c]] as u16).to_le_bytes());
            out[base + 2..base + 4].copy_from_slice(&(gain[[x, c]] as u16).to_le_bytes());
        }
    }
    out
}

pub fn initial_gain(data: &[u8]) -> Result<[i64; 3]> {
    let a = u16s(data);
    let len = a.len() / 3;
    let n = len / 4 * 4;
    let mut peak = [i64::MIN; 3];
    for b in 0..n / 4 {
        for c in 0..3 {
            let s: i64 = (0..4).map(|k| a[(b * 4 + k) * 3 + c]).sum();
            peak[c] = peak[c].max(floor_div(s, 4));
        }
    }
    if n < len {
        for c in 0..3 {
            let s: i64 = (n..len).map(|p| a[p * 3 + c]).sum();
            peak[c] = peak[c].max(floor_div(s, (len - n) as i64));
        }
    }
    if peak.contains(&0) {
        return Err(Error::Scan("zero lamp response during AFE calibration".into()));
    }
    Ok(peak.map(|p| floor_div(378 * (65535 - p), 5 * 65535).clamp(0, 63)))
}

fn vi(v: &Value) -> i64 {
    v.as_i64().or_else(|| v.as_f64().map(|f| f as i64)).unwrap_or(0)
}

/// Replace values in the existing probe/AFE/shading sequence, without extra USB commands.
pub struct Calibration {
    pub first: i64,
    pub white: i64,
    pub dark_frame: i64,
    pub pixels: usize,
    pub infrared: bool,
    pub stride: usize,
    pub phase: i64,
    probes: [Option<[i64; 3]>; 5],
    offset_writes: [i64; 3],
    pub table: Vec<u8>,
    table_offset: usize,
    zero_low: [i64; 3],
    zero_high: [i64; 3],
    dark_offset: [i64; 3],
    white_offset: [i64; 3],
    gain: [i64; 3],
    dark: Option<Array2<i64>>,
}

impl Calibration {
    pub fn new(profile: &Value) -> Result<Self> {
        let lamp = &profile["lamp"];
        let first = vi(&lamp["line"]["frame"]) - 2;
        let white = vi(&lamp["shading"]["frame"]);
        let dark_frame = vi(&lamp["dark"]["frame"]);
        if (dark_frame, white) != (first + 5, first + 6) {
            return Err(Error::Scan("unsupported GL843 calibration sequence".into()));
        }
        let wf = &profile["frames"][white as usize];
        Ok(Self {
            first,
            white,
            dark_frame,
            pixels: vi(&wf["pixels"]) as usize,
            infrared: vi(&wf["regs"]["168"]) & 4 != 0,
            stride: if vi(&profile["dpi"]) == 7200 { 2 } else { 1 },
            phase: -1,
            probes: [None; 5],
            offset_writes: [0; 3],
            table: Vec::new(),
            table_offset: 0,
            zero_low: [0; 3],
            zero_high: [0; 3],
            dark_offset: [0; 3],
            white_offset: [0; 3],
            gain: [0; 3],
            dark: None,
        })
    }

    pub fn frame_done(&mut self, i: i64, data: &[u8]) -> Result<()> {
        self.phase = i - self.first;
        let ph = self.phase;
        if matches!(ph, 0 | 1 | 3 | 4) {
            let a = u16s(data);
            let m = [0, 1, 2].map(|c| floor_div((19..51).map(|x| a[x * 3 + c]).sum(), 32));
            self.probes[ph as usize] = Some(m);
        }
        if ph == 1 || ph == 4 {
            let low = self.probes[ph as usize - 1].expect("probe");
            let high = self.probes[ph as usize].expect("probe");
            let span = [0, 1, 2].map(|c| high[c] - low[c]);
            if span.iter().any(|&s| s <= 0) {
                return Err(Error::Scan("nonpositive AFE offset response".into()));
            }
            let zero = [0, 1, 2].map(|c| 128 - floor_div(low[c] * 127, span[c]));
            if ph == 1 {
                self.zero_low = zero;
            } else {
                self.zero_high = zero;
                let dk = if self.infrared { 0 } else { 1280 };
                let wk = if self.infrared { 2048 } else { 256 };
                self.dark_offset = [0, 1, 2].map(|c| zero[c] + floor_div(dk * 127, span[c]));
                self.white_offset = [0, 1, 2].map(|c| self.dark_offset[c] + floor_div(wk * 127, span[c]));
            }
        } else if ph == 2 {
            self.gain = initial_gain(data)?;
        } else if i == self.dark_frame {
            let dark = dark_reference(data, self.pixels, self.stride);
            self.table = shading_table(&dark, &Array2::from_elem(dark.dim(), UNITY));
            self.dark = Some(dark);
        } else if i == self.white {
            let white = reference(data, self.pixels); // already dark-corrected by the chip
            if white.iter().any(|&v| v == 0) {
                return Err(Error::Scan("zero white reference during shading calibration".into()));
            }
            let dark = self.dark.as_ref().expect("dark before white");
            let gain = Array2::from_shape_fn(white.dim(), |(x, c)| {
                let target = if self.infrared { 77824 } else { WHITE_RGB[c] };
                floor_div(target * UNITY, white[[x, c]]).min(65535)
            });
            self.table = shading_table(dark, &gain);
        }
        self.table_offset = 0;
        Ok(())
    }

    pub fn control(&mut self, op: &Value) -> Result<Value> {
        let data: Vec<i64> = op["data"].as_array().map(|a| a.iter().map(vi).collect()).unwrap_or_default();
        if op["rt"].as_i64() != Some(0x40) || op["value"].as_i64() != Some(0x83) || data.first() != Some(&0x51) {
            return Ok(op.clone());
        }
        let mut d: Vec<(i64, i64)> = Vec::new();
        for p in data.chunks_exact(2) {
            match d.iter_mut().find(|e| e.0 == p[0]) {
                Some(e) => e.1 = p[1],
                None => d.push((p[0], p[1])),
            }
        }
        let get = |d: &Vec<(i64, i64)>, k: i64| d.iter().find(|e| e.0 == k).map(|e| e.1);
        let addr = get(&d, 0x51).unwrap_or(0);
        let mut value: Option<i64> = None;
        if (2..=4).contains(&addr) && self.phase >= 2 {
            let c = (addr - 2) as usize;
            let bump = if self.phase >= 6 && !self.infrared { c as i64 } else { 0 };
            value = Some(63.min(self.gain[c] + bump));
        } else if (5..=7).contains(&addr) {
            let c = (addr - 5) as usize;
            let offset = if self.phase == 1 {
                Some(self.zero_low[c])
            } else if self.phase == 4 {
                let o = if self.offset_writes[c] == 0 { self.zero_high[c] } else { self.dark_offset[c] };
                self.offset_writes[c] += 1;
                Some(o)
            } else if self.phase >= 5 {
                Some(self.white_offset[c])
            } else {
                None
            };
            if let Some(o) = offset {
                if o.abs() > 255 {
                    return Err(Error::Scan("AFE offset outside nine-bit sign-magnitude range".into()));
                }
                value = Some(if o < 0 { 256 | -o } else { o });
            }
        }
        let Some(v) = value else { return Ok(op.clone()) };
        for (k, nv) in [(0x3A, v >> 8), (0x3B, v & 255)] {
            match d.iter_mut().find(|e| e.0 == k) {
                Some(e) => e.1 = nv,
                None => d.push((k, nv)),
            }
        }
        let mut out = op.clone();
        out["data"] = json!(d.iter().flat_map(|&(a, b)| [a, b]).collect::<Vec<i64>>());
        Ok(out)
    }

    pub fn upload(&mut self, size: usize) -> Result<Vec<u8>> {
        let end = (self.table_offset + size).min(self.table.len());
        let out = self.table[self.table_offset.min(end)..end].to_vec();
        if out.len() != size {
            return Err(Error::Scan("incomplete live shading upload".into()));
        }
        self.table_offset += size;
        Ok(out)
    }
}
