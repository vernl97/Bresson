//! Conversion check on real scans: converts every roll folder the way the
//! app's automatic convert does (auto-crop, roll in Lab mode, auto WB) and
//! measures the prints. Real scans have no reference colours, so the
//! measures are about health and consistency: how sure the analysis was,
//! how much clips, the cast of the near-neutral areas per tone, chroma and
//! exposure, and how much they vary across a roll. Stock-typical casts are
//! film character, not errors: compare against a baseline, don't gate on
//! absolute values.
//!
//! ```text
//! NLP_SCAN_DIRS=<roll>:<roll>|<folder of rolls> \
//! NLP_SCAN_OUT=<dir> NLP_SCAN_BASELINE=<old stats.json> \
//!   cargo test --release real_scans -- --ignored --nocapture
//! ```
//!
//! Every folder with scans directly inside is a roll; a folder without any
//! is searched one level down. Sidecars are neither read nor written.
//! `NLP_SCAN_OUT` gets `stats.json` and a JPEG of each print; with
//! `NLP_SCAN_BASELINE` the frames that moved are listed.

use std::path::{Path, PathBuf};

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use super::color;
use super::decode::{self, SourceImage};
use super::frame;
use super::pipeline::{self, Analysis, Renderer};
use super::settings::{OutputSpace, Settings};
use super::sidecar::RollData;

/// Long edge the app analyses at (`app::jobs::PROXY_SIZE`).
const PROXY: usize = 1024;
/// Decodes at once (a full-size scanner DNG is several hundred MB in memory).
const PARALLEL: usize = 4;
/// Chroma below which a print pixel counts as near-neutral.
const NEUTRAL_C: f64 = 12.0;
/// Changes from the baseline worth listing.
const MOVED_CAST: f64 = 1.5;
const MOVED_L: f64 = 2.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FrameStats {
    roll: String,
    file: String,
    /// Neutral-axis confidence (0..1) and tonal steps.
    confidence: f32,
    steps: usize,
    skin: bool,
    /// Excluded from the roll's balance as an outlier.
    excluded: bool,
    /// L* percentiles 2 / 50 / 98.
    l: [f64; 3],
    /// Share (%) of pixels clipped to black / white in any channel.
    clip: [f64; 2],
    /// Mean (a*, b*) of near-neutral pixels: shadows (L* 15–40), mids
    /// (40–70), highlights (70–92), all; 0 where a band has none.
    cast: [[f64; 2]; 4],
    /// Share (%) of the frame's pixels that are near-neutral, per band.
    neutral: [f64; 4],
    /// Mean and 95th percentile chroma.
    chroma: [f64; 2],
}

impl FrameStats {
    /// The cast of a band, or `None` without near-neutral pixels.
    fn band(&self, b: usize) -> Option<[f64; 2]> {
        (self.neutral[b] > 0.0).then_some(self.cast[b])
    }

    fn cast_c(&self) -> f64 {
        self.band(3).map_or(f64::NAN, hypot)
    }
    /// Highlight cast against shadow cast (crossover).
    fn crossover(&self) -> f64 {
        match (self.band(0), self.band(2)) {
            (Some(s), Some(h)) => hypot([h[0] - s[0], h[1] - s[1]]),
            _ => f64::NAN,
        }
    }
}

fn hypot(v: [f64; 2]) -> f64 {
    (v[0] * v[0] + v[1] * v[1]).sqrt()
}

fn scans_in(dir: &Path) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = std::fs::read_dir(dir)
        .map(|r| r.filter_map(|e| e.ok()).map(|e| e.path()).collect())
        .unwrap_or_default();
    v.retain(|p| p.is_file() && decode::is_supported(p));
    v.sort();
    v
}

/// Roll folders under each given folder.
fn rolls(dirs: &[PathBuf]) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for d in dirs {
        if !scans_in(d).is_empty() {
            out.push(d.clone());
            continue;
        }
        let mut subs: Vec<PathBuf> = std::fs::read_dir(d)
            .map(|r| r.filter_map(|e| e.ok()).map(|e| e.path()).collect())
            .unwrap_or_default();
        subs.retain(|p| p.is_dir() && !scans_in(p).is_empty());
        subs.sort();
        out.extend(subs);
    }
    out
}

/// Convert one roll as the app's automatic convert does.
fn convert_roll(files: &[PathBuf]) -> Vec<(PathBuf, Settings, Analysis, SourceImage)> {
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(PARALLEL)
        .build()
        .unwrap();
    let loaded: Vec<(PathBuf, SourceImage, Settings)> = pool.install(|| {
        files
            .par_iter()
            .filter_map(|f| match decode::load(f, Some(PROXY)) {
                Ok(src) => {
                    let mut s = Settings {
                        source: pipeline::detect_source(&src),
                        auto_crop: true,
                        ..Settings::default()
                    };
                    if let Some(d) = frame::detect(&src, &s, pipeline::resolve_base(&src, &s, None))
                    {
                        (s.crop, s.straighten) = (d.crop, d.straighten);
                    }
                    Some((f.clone(), src, s))
                }
                Err(e) => {
                    eprintln!("skip {}: {e:#}", f.display());
                    None
                }
            })
            .collect()
    });
    let bases: Vec<[f32; 3]> = loaded
        .iter()
        .map(|(_, src, s)| pipeline::resolve_base(src, s, None))
        .collect();
    // The roll's base: the median of the frames'.
    let roll_base: [f32; 3] = std::array::from_fn(|c| {
        let mut v: Vec<f32> = bases.iter().map(|b| b[c]).collect();
        v.sort_by(f32::total_cmp);
        v[v.len() / 2]
    });
    let as_roll = loaded.len() > 1;
    loaded
        .into_par_iter()
        .zip(bases)
        .map(|((f, src, s), base)| {
            let base = if as_roll { roll_base } else { base };
            let neg = pipeline::prepare_negative(&src, &s, base);
            let a = pipeline::analyze(&neg, &s, base, pipeline::frame_ev(&src.info));
            (f, s, a, src)
        })
        .collect()
}

fn percentile(v: &mut [f64], q: f64) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    let i = ((v.len() - 1) as f64 * q).round() as usize;
    *v.select_nth_unstable_by(i, f64::total_cmp).1
}

/// Measure a print (encoded sRGB).
fn measure(px: &[[f32; 3]]) -> ([f64; 3], [f64; 2], [[f64; 2]; 4], [f64; 4], [f64; 2]) {
    let labs: Vec<[f64; 3]> = px
        .par_iter()
        .map(|p| {
            let lin = p.map(|v| color::srgb_decode(v.clamp(0.0, 1.0)) as f64);
            color::rgb_to_lab(&color::SRGB, lin)
        })
        .collect();
    let n = px.len().max(1) as f64;
    let mut ls: Vec<f64> = labs.iter().map(|l| l[0]).collect();
    let l = [0.02, 0.5, 0.98].map(|q| percentile(&mut ls, q));
    let black = px.iter().filter(|p| p.iter().any(|&v| v <= 0.002)).count() as f64;
    let white = px.iter().filter(|p| p.iter().any(|&v| v >= 0.998)).count() as f64;
    let bands = [(15.0, 40.0), (40.0, 70.0), (70.0, 92.0), (15.0, 92.0)];
    let found = bands.map(|(lo, hi)| {
        let (mut s, mut k) = ([0.0; 2], 0usize);
        for p in &labs {
            if (lo..hi).contains(&p[0]) && hypot([p[1], p[2]]) < NEUTRAL_C {
                s[0] += p[1];
                s[1] += p[2];
                k += 1;
            }
        }
        (s.map(|v| v / k.max(1) as f64), k as f64 * 100.0 / n)
    });
    let (cast, neutral) = (found.map(|f| f.0), found.map(|f| f.1));
    let mut cs: Vec<f64> = labs.iter().map(|p| hypot([p[1], p[2]])).collect();
    let mean_c = cs.iter().sum::<f64>() / n;
    let chroma = [mean_c, percentile(&mut cs, 0.95)];
    (
        l,
        [black * 100.0 / n, white * 100.0 / n],
        cast,
        neutral,
        chroma,
    )
}

fn save_jpeg(path: &Path, px: &[[f32; 3]], w: usize, h: usize) {
    let buf: Vec<u8> = px
        .iter()
        .flat_map(|p| p.map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8))
        .collect();
    if let Some(img) = image::RgbImage::from_raw(w as u32, h as u32, buf) {
        let _ = img.save(path);
    }
}

fn median(mut v: Vec<f64>) -> f64 {
    v.retain(|x| x.is_finite());
    percentile(&mut v, 0.5)
}

/// Spread of a measure across a roll: the range of its middle 80 %.
fn spread(mut v: Vec<f64>) -> f64 {
    v.retain(|x| x.is_finite());
    percentile(&mut v.clone(), 0.9) - percentile(&mut v, 0.1)
}

fn report(stats: &[FrameStats]) {
    eprintln!(
        "\n{:<28} {:>3} {:>5} {:>5} {:>5} {:>6} {:>6} {:>6} {:>6} {:>6} {:>6} {:>6} {:>6}",
        "roll",
        "n",
        "conf",
        "lowc",
        "excl",
        "L50",
        "L50sd",
        "clipW",
        "cast",
        "castsd",
        "xover",
        "C*",
        "C*95"
    );
    let mut names: Vec<&str> = stats.iter().map(|s| s.roll.as_str()).collect();
    names.dedup();
    let row = |name: &str, f: Vec<&FrameStats>| {
        let low = f
            .iter()
            .filter(|s| s.confidence < pipeline::LOW_CONFIDENCE)
            .count();
        let excl = f.iter().filter(|s| s.excluded).count();
        let col = |g: &dyn Fn(&FrameStats) -> f64| f.iter().map(|s| g(s)).collect::<Vec<f64>>();
        // Cast spread: how far the frames' (a*, b*) scatter.
        let all = |k: usize| col(&|s| s.band(3).map_or(f64::NAN, |c| c[k]));
        let (ca, cb) = (all(0), all(1));
        let cast_sd = hypot([spread(ca), spread(cb)]);
        eprintln!(
            "{:<28} {:>3} {:>5.2} {:>5} {:>5} {:>6.1} {:>6.1} {:>6.2} {:>6.2} {:>6.2} {:>6.2} {:>6.1} {:>6.1}",
            name.chars().take(28).collect::<String>(),
            f.len(),
            median(col(&|s| s.confidence as f64)),
            low,
            excl,
            median(col(&|s| s.l[1])),
            spread(col(&|s| s.l[1])),
            median(col(&|s| s.clip[1])),
            median(col(&|s| s.cast_c())),
            cast_sd,
            median(col(&|s| s.crossover())),
            median(col(&|s| s.chroma[0])),
            median(col(&|s| s.chroma[1])),
        );
    };
    for name in &names {
        row(name, stats.iter().filter(|s| s.roll == *name).collect());
    }
    row("ALL", stats.iter().collect());
    let dark: Vec<String> = stats
        .iter()
        .filter(|s| s.clip[0] > 50.0)
        .map(|s| format!("{}/{} ({:.0} % black)", s.roll, s.file, s.clip[0]))
        .collect();
    if !dark.is_empty() {
        eprintln!(
            "frames more than half clipped to black: {}",
            dark.join(", ")
        );
    }
    eprintln!(
        "conf: median axis confidence; lowc: frames below {} confidence; excl: roll outliers; \
         L50/L50sd: median L* and its spread; clipW: % pixels clipped white; cast: C* of \
         near-neutral areas; castsd: spread of the cast across the roll; xover: highlight vs \
         shadow cast; C*/C*95: mean / 95th-percentile chroma. Spreads are 10th–90th percentile \
         ranges.",
        pipeline::LOW_CONFIDENCE
    );
}

fn compare(stats: &[FrameStats], baseline: &[FrameStats]) {
    let mut moved = Vec::new();
    let (mut d_cast, mut d_l, mut d_c, mut n) = (0.0, 0.0, 0.0, 0usize);
    for s in stats {
        let Some(b) = baseline
            .iter()
            .find(|b| b.roll == s.roll && b.file == s.file)
        else {
            continue;
        };
        let dc = match (s.band(3), b.band(3)) {
            (Some(x), Some(y)) => hypot([x[0] - y[0], x[1] - y[1]]),
            _ => f64::NAN,
        };
        let dl = s.l[1] - b.l[1];
        if dc.is_finite() {
            d_cast += dc;
        }
        d_l += dl.abs();
        d_c += s.chroma[0] - b.chroma[0];
        n += 1;
        if dc > MOVED_CAST || dl.abs() > MOVED_L {
            moved.push(format!(
                "  {}/{}: cast moved {dc:.2} (a* {:+.1} b* {:+.1} -> a* {:+.1} b* {:+.1}), L50 {:+.1}",
                s.roll, s.file, b.cast[3][0], b.cast[3][1], s.cast[3][0], s.cast[3][1], dl
            ));
        }
    }
    let k = n.max(1) as f64;
    eprintln!(
        "\nagainst baseline ({n} frames): mean cast change {:.2}, mean |L50| change {:.2}, \
         mean chroma change {:+.2}; {} frames moved more than {MOVED_CAST} C* or {MOVED_L} L*",
        d_cast / k,
        d_l / k,
        d_c / k,
        moved.len()
    );
    for m in moved {
        eprintln!("{m}");
    }
}

#[test]
#[ignore]
fn real_scans() {
    let Ok(dirs) = std::env::var("NLP_SCAN_DIRS") else {
        eprintln!("set NLP_SCAN_DIRS to scan folders (separated by ':')");
        return;
    };
    let dirs: Vec<PathBuf> = dirs
        .split(':')
        .filter(|d| !d.is_empty())
        .map(PathBuf::from)
        .collect();
    let out = std::env::var("NLP_SCAN_OUT").ok().map(PathBuf::from);
    let mut stats = Vec::new();
    for roll_dir in rolls(&dirs) {
        let files = scans_in(&roll_dir);
        let roll_name = roll_dir.file_name().unwrap().to_string_lossy().to_string();
        eprintln!("{roll_name}: {} scans", files.len());
        let frames = convert_roll(&files);
        let analyses: Vec<Analysis> = frames.iter().map(|f| f.2.clone()).collect();
        let roll = (frames.len() > 1).then(|| {
            let (agg, excluded) = pipeline::aggregate_roll(&analyses, None);
            (
                RollData {
                    id: "check".into(),
                    levels: agg.levels,
                    balance: agg.balance,
                    base: analyses[0].base,
                    frames: frames.len(),
                    excluded: vec![],
                    auto_wb: Some(agg.auto_wb),
                    common_ev: agg.common_ev,
                },
                excluded,
            )
        });
        let jpeg_dir = out.as_ref().map(|o| o.join(&roll_name));
        if let Some(d) = &jpeg_dir {
            std::fs::create_dir_all(d).unwrap();
        }
        let roll_stats: Vec<FrameStats> = frames
            .par_iter()
            .enumerate()
            .map(|(i, (path, s, a, src))| {
                let mut s = s.clone();
                s.use_roll = roll.is_some();
                let rd = roll.as_ref().map(|r| &r.0);
                let wb = pipeline::auto_wb_for(&s, Some(a), rd).unwrap_or(a.auto_wb);
                if let Some((t, n)) = pipeline::wb_for_mode(s.wb_mode, &wb) {
                    (s.wb_temp, s.wb_tint) = (t, n);
                }
                let levels = pipeline::levels_for(&s, Some(a), rd).unwrap();
                let neg = pipeline::prepare_negative(src, &s, a.base);
                let px = Renderer::new(&s, Some(&levels), OutputSpace::Srgb).render(&neg, 0.0);
                let file = path.file_name().unwrap().to_string_lossy().to_string();
                if let Some(d) = &jpeg_dir {
                    save_jpeg(&d.join(format!("{file}.jpg")), &px, neg.width, neg.height);
                }
                let (l, clip, cast, neutral, chroma) = measure(&px);
                FrameStats {
                    roll: roll_name.clone(),
                    file,
                    confidence: a.auto_wb.confidence,
                    steps: a.axis.map_or(0, |x| x.steps),
                    skin: a.skin.is_some(),
                    excluded: roll.as_ref().is_some_and(|r| r.1.contains(&i)),
                    l,
                    clip,
                    cast,
                    neutral,
                    chroma,
                }
            })
            .collect();
        stats.extend(roll_stats);
    }
    report(&stats);
    if let Ok(b) = std::env::var("NLP_SCAN_BASELINE") {
        match std::fs::read_to_string(&b).map(|t| serde_json::from_str::<Vec<FrameStats>>(&t)) {
            Ok(Ok(baseline)) => compare(&stats, &baseline),
            _ => eprintln!("could not read baseline {b}"),
        }
    }
    if let Some(o) = &out {
        std::fs::create_dir_all(o).unwrap();
        let p = o.join("stats.json");
        std::fs::write(&p, serde_json::to_string_pretty(&stats).unwrap()).unwrap();
        eprintln!("\nwrote {} and the prints", p.display());
    }
}
