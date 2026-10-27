//! Negative preparation (scan → density), analysis and rendering.

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use super::color;
use super::curve::{self, ChannelLevels, CurveLut, Levels};
use super::dcp;
use super::decode::{self, SourceImage};
use super::look::{self, Look};
use super::presets;
use super::settings::{AnalysisKey, FilmBase, ImageType, OutputSpace, RollMode, Settings, WbMode};
use super::sidecar::RollData;

/// Long edge of the analysis rendition.
pub const ANALYSIS_SIZE: usize = 600;
/// Film base is mapped to this linear level after neutralisation.
const BASE_LEVEL: f32 = 0.97;
/// Channel weights for monochrome density and for saturation in density.
const DENSITY_WEIGHTS: [f32; 3] = [0.30, 0.55, 0.15];

/// Residual unwanted absorption of C-41 dyes after the masking couplers, as
/// a camera sees it: measured density per unit of dye (rows R/G/B channel,
/// columns C/M/Y dye). Mostly the magenta dye's blue absorption and the
/// cyan dye's green absorption.
const C41_CROSSTALK: color::Mat3 = [[1.00, 0.06, 0.02], [0.10, 1.00, 0.05], [0.04, 0.15, 1.00]];

/// The prepared negative: per-channel density above the film base
/// (`log10(base / transmittance)`), dye crosstalk removed. Not clamped:
/// anything thinner than the base is negative.
#[derive(Clone)]
pub struct Negative {
    pub width: usize,
    pub height: usize,
    pub data: Vec<[f32; 3]>,
}

/// Auto white-balance candidates as (temp, tint) in CC.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct AutoWb {
    pub neutral: (f32, f32),
    pub warm: (f32, f32),
    pub cool: (f32, f32),
    pub avg: (f32, f32),
    /// Grey density the neutral balance prints grey; roll analysis averages it.
    #[serde(default)]
    pub grey: [f64; 3],
    /// Mean density of all image pixels.
    #[serde(default)]
    pub mean: [f64; 3],
    /// 0..1: how much near-neutral content, across how many tones, the
    /// neutral estimate rests on.
    #[serde(default)]
    pub confidence: f32,
}

/// Per-frame result of "Convert".
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Analysis {
    pub levels: Levels,
    pub auto_wb: AutoWb,
    /// Film base colour (source space) used for neutralisation.
    pub base: [f32; 3],
    pub key: AnalysisKey,
    /// Exposure value of the capture, when the EXIF has it.
    #[serde(default)]
    pub ev: Option<f64>,
}

// ---- film base -------------------------------------------------------------------

/// Estimate the film base (brightest, unexposed area) in source space:
/// the average of pixels between the 98.5th and 99.5th brightness percentile.
/// Always uses the entire image (uncropped) so that film borders remain
/// available for mask neutralisation even when a crop has been set.
pub fn auto_base(src: &SourceImage, _s: &Settings) -> [f32; 3] {
    let d = &src.data;
    let mut sums: Vec<(f32, usize)> = d
        .iter()
        .enumerate()
        .map(|(i, p)| (p[0] + p[1] + p[2], i))
        .collect();
    if sums.is_empty() {
        return [1.0; 3];
    }
    sums.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    let n = sums.len();
    let lo = ((n as f32 * 0.985) as usize).min(n - 1);
    let hi = ((n as f32 * 0.995) as usize).clamp(lo + 1, n);
    let mut acc = [0.0f64; 3];
    for &(_, i) in &sums[lo..hi] {
        for c in 0..3 {
            acc[c] += d[i][c] as f64;
        }
    }
    let k = (hi - lo) as f64;
    acc.map(|v| ((v / k) as f32).max(1e-4))
}

/// Resolve the base colour for a frame (roll base takes precedence for Auto).
pub fn resolve_base(src: &SourceImage, s: &Settings, roll_base: Option<[f32; 3]>) -> [f32; 3] {
    match s.film_base {
        FilmBase::Sampled => s.film_base_sample.map(|v| v.max(1e-4)),
        FilmBase::Auto => roll_base.unwrap_or_else(|| auto_base(src, s)),
        FilmBase::AsShot | FilmBase::Off => auto_base(src, s),
    }
}

/// Source-space multipliers from a base colour.
pub fn base_multipliers(src: &SourceImage, s: &Settings, base: [f32; 3]) -> [f32; 3] {
    match s.film_base {
        FilmBase::Auto | FilmBase::Sampled => base.map(|b| BASE_LEVEL / b.max(1e-4)),
        FilmBase::AsShot => {
            let wb = src.as_shot_wb;
            let m = (0..3)
                .map(|c| base[c] * wb[c])
                .fold(0.0f32, f32::max)
                .max(1e-4);
            wb.map(|v| v * BASE_LEVEL / m)
        }
        FilmBase::Off => [1.0; 3],
    }
}

/// Colour model calibration and pre-saturation, applied in density: hue
/// shifts mix each primary towards a neighbour, saturation scales the
/// distance from the neutral axis (channel separation).
fn calibration_matrix(s: &Settings) -> [[f32; 3]; 3] {
    let cal = presets::calibration(s.color_model, s.source);
    let yw = DENSITY_WEIGHTS;
    const K: f32 = 0.25 / 100.0;
    // Primary columns.
    let mut cols = [[1.0f32, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    let hue = [cal.red_hue, cal.green_hue, cal.blue_hue];
    let sat = [cal.red_sat, cal.green_sat, cal.blue_sat];
    // (towards for +hue, towards for −hue)
    let dirs = [(1usize, 2usize), (2, 0), (1, 0)];
    for p in 0..3 {
        let h = hue[p];
        if h > 0.0 {
            cols[p][dirs[p].0] += h * K;
        } else if h < 0.0 {
            cols[p][dirs[p].1] += -h * K;
        }
        let y = cols[p][0] * yw[0] + cols[p][1] * yw[1] + cols[p][2] * yw[2];
        let f = 1.0 + sat[p] / 100.0;
        for c in 0..3 {
            cols[p][c] = y + (cols[p][c] - y) * f;
        }
    }
    let mut m = [[0.0f32; 3]; 3];
    for r in 0..3 {
        for p in 0..3 {
            m[r][p] = cols[p][r];
        }
        let sum: f32 = m[r].iter().sum();
        if sum.abs() > 1e-6 {
            for v in m[r].iter_mut() {
                *v /= sum;
            }
        }
    }
    let f = 1.0 + presets::pre_saturation(s.pre_saturation) / 100.0;
    let sat_m: [[f32; 3]; 3] = std::array::from_fn(|r| {
        std::array::from_fn(|c| yw[c] * (1.0 - f) + if r == c { f } else { 0.0 })
    });
    mat_mul_f32(&sat_m, &m)
}

fn mat_mul_f32(a: &[[f32; 3]; 3], b: &[[f32; 3]; 3]) -> [[f32; 3]; 3] {
    std::array::from_fn(|r| std::array::from_fn(|c| (0..3).map(|k| a[r][k] * b[k][c]).sum()))
}

/// Dye crosstalk correction: the inverse of the dye mixing, with its rows
/// normalised so a neutral density stays neutral.
fn crosstalk_correction() -> [[f32; 3]; 3] {
    let mut m = color::mat_inv(&C41_CROSSTALK);
    for row in m.iter_mut() {
        let sum: f64 = row.iter().sum();
        for v in row.iter_mut() {
            *v /= sum;
        }
    }
    color::to_f32(&m)
}

/// Density-space matrix of a frame: crosstalk correction (colour negatives),
/// then the colour model calibration.
fn density_matrix(s: &Settings) -> [[f32; 3]; 3] {
    let cal = calibration_matrix(s);
    if s.image_type == ImageType::Negative && !s.is_monochrome() {
        mat_mul_f32(&cal, &crosstalk_correction())
    } else {
        cal
    }
}

/// The plugin's Negative Lab camera profile for this source, if one matches.
/// Only used to recognise scanner DNGs.
pub fn camera_profile(src: &SourceImage) -> Option<std::sync::Arc<dcp::DcpProfile>> {
    if src.is_raw {
        dcp::profile_for(&src.info)
    } else {
        None
    }
}

/// The scan source implied by a file: VueScan / SilverFast DNGs (scanner
/// profile or scanner software), otherwise digital camera; bitmaps are TIF scans.
pub fn detect_source(src: &SourceImage) -> super::settings::Source {
    use super::settings::Source;
    if !src.is_raw {
        return Source::TifScan;
    }
    let scanner_id = |s: &str| {
        let l = s.to_lowercase();
        l.contains("vuescan") || l.contains("silverfast")
    };
    let by_profile = camera_profile(src).map(|p| {
        scanner_id(&p.camera)
            || p.path.components().any(|c| {
                c.as_os_str()
                    .to_string_lossy()
                    .eq_ignore_ascii_case("Vuescan Profiles")
            })
    });
    match by_profile {
        Some(true) => Source::VueScanRaw,
        Some(false) => Source::DigitalCamera,
        None if src.info.camera_ids.iter().any(|s| scanner_id(s)) => Source::VueScanRaw,
        None => Source::DigitalCamera,
    }
}

/// Build the negative's density image from linear source pixels: camera
/// matrix, film base neutralisation, `log10`, crosstalk and calibration.
pub fn prepare_negative(src: &SourceImage, s: &Settings, base: [f32; 3]) -> Negative {
    let (d, w, h) = decode::geometry(
        &src.data,
        src.width,
        src.height,
        s.rotation,
        s.flip_h,
        s.straighten,
        s.crop,
    );
    let edit_only = s.image_type == ImageType::EditOnly;
    // Already positive scans keep their own levels: density is −log10 of
    // the linear value.
    let (mul, level) = if edit_only {
        ([1.0; 3], 1.0)
    } else {
        (base_multipliers(src, s, base), BASE_LEVEL)
    };
    let to_lin = src.to_melissa;
    let m = density_matrix(s);
    let mono = s.is_monochrome();
    let yw = DENSITY_WEIGHTS;
    let data = d
        .par_iter()
        .map(|p| {
            let t = color::mul3(&to_lin, [p[0] * mul[0], p[1] * mul[1], p[2] * mul[2]]);
            let dens = t.map(|v| (level / v.max(1e-5)).log10());
            if edit_only {
                return dens;
            }
            let dens = color::mul3(&m, dens);
            if mono {
                let y = dens[0] * yw[0] + dens[1] * yw[1] + dens[2] * yw[2];
                [y, y, y]
            } else {
                dens
            }
        })
        .collect();
    Negative {
        width: w,
        height: h,
        data,
    }
}

// ---- analysis ------------------------------------------------------------------

/// The 600px, border-cropped, blurred analysis buffer.
pub fn analysis_buffer(neg: &Negative, s: &Settings) -> Negative {
    let (d, w, h) = decode::downscale(&neg.data, neg.width, neg.height, ANALYSIS_SIZE);
    let b = s.border_buffer.clamp(0.0, 40.0);
    let ins = [b, b, b, b];
    let (d, w, h) = decode::crop(&d, w, h, ins);
    let d = look::gaussian_blur_3(&d, w, h, 2.0);
    Negative {
        width: w,
        height: h,
        data: d,
    }
}

/// Value at quantile `q` (0..1) of `v` (reordered in place).
fn quantile(v: &mut [f32], q: f64) -> f64 {
    if v.is_empty() {
        return 0.0;
    }
    let i = ((v.len() - 1) as f64 * q.clamp(0.0, 1.0)).round() as usize;
    let (_, x, _) = v.select_nth_unstable_by(i, |a, b| a.total_cmp(b));
    *x as f64
}

/// Density range of each channel: exact quantiles at the clip thresholds
/// and the median, independently per channel.
fn channel_quantiles(buf: &Negative, t: ImageType) -> Levels {
    let (white_t, black_t) = presets::clip_thresholds(t);
    std::array::from_fn(|c| {
        let mut v: Vec<f32> = buf.data.iter().map(|p| p[c]).collect();
        // Negatives print thin black, positives print thin white.
        let (lo_q, hi_q) = match t {
            ImageType::Positive => (white_t, 1.0 - black_t),
            _ => (black_t, 1.0 - white_t),
        };
        let lo = quantile(&mut v, lo_q);
        let hi = quantile(&mut v, hi_q).max(lo + 0.05);
        let mid = quantile(&mut v, 0.5);
        ChannelLevels { lo, hi, mid }
    })
}

/// The image's neutral axis: per channel `D_c = a + b · D_green`.
#[derive(Debug, Clone, Copy)]
pub struct NeutralAxis {
    pub fit: [(f64, f64); 3],
    /// Tonal steps (of 10) with enough near-neutral pixels.
    pub steps: usize,
}

impl NeutralAxis {
    /// The grey of the axis at green density `g`.
    fn at(&self, g: f64) -> [f64; 3] {
        self.fit.map(|(a, b)| a + b * g)
    }
}

/// Analysed levels: green's own density range, with red and blue placed on
/// the image's neutral axis (see [`neutral_axis`]) so their slopes match the
/// film's per-layer H&D slopes. Keeps each channel's own range when the
/// image has too little near-neutral content.
pub fn levels_from_buffer(buf: &Negative, t: ImageType) -> (Levels, Option<NeutralAxis>) {
    let mut l = channel_quantiles(buf, t);
    let own = l;
    let mut axis = None;
    for _ in 0..3 {
        let Some(fit) = neutral_axis(buf, &l, t) else {
            break;
        };
        for c in [0, 2] {
            let (a, b) = fit.fit[c];
            // Keep the fitted slope within reach of the channel's own range.
            let own_b = (own[c].hi - own[c].lo) / (own[1].hi - own[1].lo).max(1e-3);
            let b = b.clamp(own_b * 0.8, own_b * 1.25);
            let at = |g: f64| a + b * g;
            l[c] = ChannelLevels {
                lo: at(l[1].lo),
                hi: at(l[1].hi).max(at(l[1].lo) + 0.05),
                mid: own[c].mid,
            };
        }
        axis = Some(fit);
    }
    (l, axis)
}

/// The neutral axis of an image: near-neutral pixels (normalised channels
/// within 0.1 of each other under the current levels `l`), binned by green
/// density into ten tonal steps, and a line `D_c = a + b · D_green` through
/// the steps' median densities for each channel. Every step weighs the same,
/// so a large area of one colour and tone can bias at most one step. `None`
/// without near-neutral pixels in at least four steps.
fn neutral_axis(buf: &Negative, l: &Levels, t: ImageType) -> Option<NeutralAxis> {
    const BINS: usize = 10;
    let mut bins: Vec<Vec<[f32; 3]>> = vec![Vec::new(); BINS];
    for p in &buf.data {
        let x: [f64; 3] = std::array::from_fn(|c| normalised(l, t, c, p[c] as f64));
        let (lo, hi) = x
            .iter()
            .fold((f64::MAX, f64::MIN), |(a, b), &v| (a.min(v), b.max(v)));
        if hi - lo < 0.1 && (0.03..0.97).contains(&x[1]) {
            bins[((x[1] * BINS as f64) as usize).min(BINS - 1)].push(*p);
        }
    }
    let min_count = (buf.data.len() / 500).max(8);
    let steps: Vec<[f64; 3]> = bins
        .into_iter()
        .filter(|b| b.len() >= min_count)
        .map(|b| std::array::from_fn(|c| median(b.iter().map(|p| p[c] as f64).collect())))
        .collect();
    if steps.len() < 4 {
        return None;
    }
    let n = steps.len() as f64;
    let fit = std::array::from_fn(|c| {
        let mx = steps.iter().map(|s| s[1]).sum::<f64>() / n;
        let my = steps.iter().map(|s| s[c]).sum::<f64>() / n;
        let sxx: f64 = steps.iter().map(|s| (s[1] - mx).powi(2)).sum();
        let sxy: f64 = steps.iter().map(|s| (s[1] - mx) * (s[c] - my)).sum();
        let b = if sxx > 1e-9 { sxy / sxx } else { 1.0 };
        (my - b * mx, b)
    });
    Some(NeutralAxis {
        fit,
        steps: steps.len(),
    })
}

/// Normalised density (0 prints black … 1 prints white).
fn normalised(l: &Levels, t: ImageType, c: usize, d: f64) -> f64 {
    let x = (d - l[c].lo) / (l[c].hi - l[c].lo).max(1e-6);
    if t == ImageType::Positive { 1.0 - x } else { x }
}

/// Mean density of the image pixels (gray world), and of the pixels that
/// are near-neutral under `l`.
fn image_means(buf: &Negative, l: &Levels, t: ImageType) -> ([f64; 3], Option<[f64; 3]>) {
    let (mut all, mut grey) = (([0.0; 3], 0usize), ([0.0; 3], 0usize));
    for p in &buf.data {
        let d = p.map(|v| v as f64);
        let x: [f64; 3] = std::array::from_fn(|c| normalised(l, t, c, d[c]));
        if !x.iter().all(|v| (0.02..0.98).contains(v)) {
            continue;
        }
        let spread =
            x.iter().fold(f64::MIN, |a, &b| a.max(b)) - x.iter().fold(f64::MAX, |a, &b| a.min(b));
        for (acc, ok) in [(&mut all, true), (&mut grey, spread < 0.1)] {
            if ok {
                for c in 0..3 {
                    acc.0[c] += d[c];
                }
                acc.1 += 1;
            }
        }
    }
    let mean = |(v, n): ([f64; 3], usize)| (n > 0).then(|| v.map(|x| x / n as f64));
    let mid = [0, 1, 2].map(|c| l[c].mid);
    (mean(all).unwrap_or(mid), mean(grey))
}

/// White balance (temp, tint) that prints density `d` neutral.
fn wb_for_density(l: &Levels, t: ImageType, d: [f64; 3]) -> (f32, f32) {
    curve::balance_wb(std::array::from_fn(|c| normalised(l, t, c, d[c])))
}

/// Auto WB candidates from the grey and mean densities.
fn auto_wb_from(
    l: &Levels,
    t: ImageType,
    grey: [f64; 3],
    mean: [f64; 3],
    confidence: f32,
) -> AutoWb {
    let neutral = wb_for_density(l, t, grey);
    AutoWb {
        neutral,
        warm: (neutral.0 + 6.0, neutral.1 + 1.5),
        cool: (neutral.0 - 6.0, neutral.1),
        avg: wb_for_density(l, t, mean),
        grey,
        mean,
        confidence,
    }
}

/// Auto colour balance. Auto-Neutral prints the image's neutral axis grey
/// at its median tone; without a neutral axis it falls back to the mean of
/// the near-neutral pixels, then to the channel medians, with a confidence
/// that says so. Auto-AVG is the plain gray-world mean of all pixels.
pub fn auto_wb(buf: &Negative, l: &Levels, axis: Option<&NeutralAxis>, t: ImageType) -> AutoWb {
    let (mean, near) = image_means(buf, l, t);
    let (grey, confidence) = match (axis, near) {
        (Some(a), _) => (a.at(l[1].mid), a.steps as f32 / 10.0),
        (None, Some(g)) => (g, 0.2),
        (None, None) => ([0, 1, 2].map(|c| l[c].mid), 0.0),
    };
    auto_wb_from(l, t, grey, mean, confidence)
}

/// Levels that Edit Only frames get instead of an analysis.
pub fn edit_only_levels() -> Levels {
    [ChannelLevels {
        lo: 0.0,
        hi: 1.0,
        mid: 0.5,
    }; 3]
}

/// Full conversion analysis of a prepared negative. Edit Only frames skip it.
pub fn analyze(neg: &Negative, s: &Settings, base: [f32; 3], ev: Option<f64>) -> Analysis {
    let (levels, auto_wb) = if s.image_type == ImageType::EditOnly {
        (edit_only_levels(), AutoWb::default())
    } else {
        let buf = analysis_buffer(neg, s);
        let (levels, axis) = levels_from_buffer(&buf, s.image_type);
        let wb = auto_wb(&buf, &levels, axis.as_ref(), s.image_type);
        (levels, wb)
    };
    Analysis {
        levels,
        auto_wb,
        base,
        key: s.analysis_key(),
        ev,
    }
}

/// (temp, tint) for a WB dropdown choice; `None` keeps the current values.
pub fn wb_for_mode(mode: WbMode, a: &AutoWb) -> Option<(f32, f32)> {
    if let Some((magenta, yellow)) = presets::film_character(mode) {
        return Some((yellow, magenta));
    }
    match mode {
        WbMode::AutoNeutral => Some(a.neutral),
        WbMode::AutoWarm => Some(a.warm),
        WbMode::AutoCool => Some(a.cool),
        WbMode::AutoAvg => Some(a.avg),
        WbMode::AutoMix => Some(((a.neutral.0 + a.warm.0) / 2.0, a.neutral.1)),
        _ => None,
    }
}

// ---- roll ---------------------------------------------------------------------------

/// A roll built by "Convert as roll".
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RollAggregate {
    /// Median density range of the frames.
    pub levels: Levels,
    pub auto_wb: AutoWb,
    /// The roll's most common exposure; `None` without exposure data.
    pub common_ev: Option<f64>,
}

/// Snap to thirds of a stop.
fn round_ev(ev: f64) -> f64 {
    let base = ev.floor();
    let f = ev - base;
    if f <= 1.0 / 6.0 {
        base
    } else if f <= 0.5 {
        base + 1.0 / 3.0
    } else if f <= 5.0 / 6.0 {
        base + 2.0 / 3.0
    } else {
        base + 1.0
    }
}

/// `log2(N²/t) + log2(iso/100)`, rounded to thirds.
pub fn exposure_value(t: f64, n: f64, iso: f64) -> f64 {
    round_ev((n * n / t).log2() + (iso / 100.0).log2())
}

/// The frame's exposure value from its EXIF.
pub fn frame_ev(info: &decode::SourceInfo) -> Option<f64> {
    info.exposure.map(|[t, n, iso]| exposure_value(t, n, iso))
}

/// The most frequent exposure (ties go to the first frame).
pub fn common_ev(frames: &[Analysis]) -> Option<f64> {
    let evs: Vec<f64> = frames.iter().filter_map(|a| a.ev).collect();
    let count = |e: f64| evs.iter().filter(|&&x| (x - e).abs() < 1e-9).count();
    evs.iter()
        .copied()
        .fold(None, |best: Option<f64>, e| match best {
            Some(b) if count(b) >= count(e) => Some(b),
            _ => Some(e),
        })
}

/// Largest exposure difference from the common one; above 1 EV the app
/// asks whether to continue without exposure data.
pub fn ev_spread(frames: &[Analysis], common: f64) -> f64 {
    frames
        .iter()
        .filter_map(|a| a.ev)
        .map(|e| (e - common).abs())
        .fold(0.0, f64::max)
}

/// Density offset of a scan made `ev` stops from the common exposure: one
/// stop less light reads `log10 2` denser against a shared base.
fn ev_density(ev: f64) -> f64 {
    ev * std::f64::consts::LOG10_2
}

fn shift_levels(l: &Levels, d: f64) -> Levels {
    l.map(|c| ChannelLevels {
        lo: c.lo + d,
        hi: c.hi + d,
        mid: c.mid + d,
    })
}

fn median(mut v: Vec<f64>) -> f64 {
    if v.is_empty() {
        return 0.0;
    }
    v.sort_by(f64::total_cmp);
    let n = v.len();
    if n % 2 == 1 {
        v[n / 2]
    } else {
        (v[n / 2 - 1] + v[n / 2]) / 2.0
    }
}

/// Indices of outliers in `v`: further from the median than 3.5 robust
/// standard deviations (1.4826 · MAD), and at least `floor` away.
fn outliers(v: &[f64], floor: f64) -> Vec<usize> {
    let m = median(v.to_vec());
    let mad = median(v.iter().map(|x| (x - m).abs()).collect());
    let limit = (3.5 * 1.4826 * mad).max(floor);
    (0..v.len()).filter(|&i| (v[i] - m).abs() > limit).collect()
}

/// Roll aggregation. With a `common_ev`, each frame is first moved onto that
/// exposure. Frames whose colour balance (green and blue range relative to
/// red) is far from the rest of the roll (robust outlier test, at least
/// 0.05 D off) are left out; the roll's range is the median of the rest.
/// Returns the roll and the excluded frame indices.
pub fn aggregate_roll(frames: &[Analysis], common_ev: Option<f64>) -> (RollAggregate, Vec<usize>) {
    let t = frames[0].key.image_type;
    let shift = |a: &Analysis| match (common_ev, a.ev) {
        (Some(c), Some(e)) => -ev_density(e - c),
        _ => 0.0,
    };
    let adj: Vec<Levels> = frames
        .iter()
        .map(|a| shift_levels(&a.levels, shift(a)))
        .collect();
    let features: Vec<[f64; 4]> = adj
        .iter()
        .map(|l| {
            [
                l[1].lo - l[0].lo,
                l[2].lo - l[0].lo,
                l[1].hi - l[0].hi,
                l[2].hi - l[0].hi,
            ]
        })
        .collect();
    let mut excluded: Vec<usize> = Vec::new();
    if frames.len() >= 3 {
        for k in 0..4 {
            let v: Vec<f64> = features.iter().map(|f| f[k]).collect();
            excluded.extend(outliers(&v, 0.05));
        }
        excluded.sort_unstable();
        excluded.dedup();
    }
    let kept: Vec<usize> = (0..frames.len())
        .filter(|i| !excluded.contains(i))
        .collect();
    let levels: Levels = std::array::from_fn(|c| ChannelLevels {
        lo: median(kept.iter().map(|&i| adj[i][c].lo).collect()),
        hi: median(kept.iter().map(|&i| adj[i][c].hi).collect()),
        mid: median(kept.iter().map(|&i| adj[i][c].mid).collect()),
    });
    let avg = |f: &dyn Fn(&AutoWb) -> [f64; 3]| -> [f64; 3] {
        let mut m = [0.0; 3];
        for &i in &kept {
            let v = f(&frames[i].auto_wb);
            for c in 0..3 {
                m[c] += (v[c] + shift(&frames[i])) / kept.len() as f64;
            }
        }
        m
    };
    let confidence = kept
        .iter()
        .map(|&i| frames[i].auto_wb.confidence)
        .fold(0.0, f32::max);
    let auto_wb = auto_wb_from(&levels, t, avg(&|w| w.grey), avg(&|w| w.mean), confidence);
    (
        RollAggregate {
            levels,
            auto_wb,
            common_ev,
        },
        excluded,
    )
}

/// Pick the levels a frame renders with: its own analysis, or the roll's
/// according to the Roll Process. Pro Lab keeps the roll's colour balance
/// but places it on the frame's own overall density range; Darkroom Paper
/// prints every frame with the roll's levels; Basic Lab is halfway between
/// the two. A film exposure change moves each channel's density in
/// proportion to its H&D slope, i.e. its density span, so Pro Lab moves the
/// roll's ends by the same fraction of each channel's span.
pub fn levels_for(
    s: &Settings,
    frame: Option<&Analysis>,
    roll: Option<&RollData>,
) -> Option<Levels> {
    let Some(roll) = roll.filter(|_| s.use_roll) else {
        return frame.map(|a| a.levels);
    };
    // Move the roll onto this frame's exposure.
    let roll_levels = match (roll.common_ev, frame.and_then(|a| a.ev)) {
        (Some(c), Some(e)) => shift_levels(&roll.levels, ev_density(e - c)),
        _ => roll.levels,
    };
    let Some(frame) = frame.filter(|_| s.roll_mode != RollMode::DarkroomPaper) else {
        return Some(roll_levels);
    };
    let span = |c: usize| (roll_levels[c].hi - roll_levels[c].lo).max(1e-3);
    let mean = |f: &dyn Fn(usize) -> f64| (f(0) + f(1) + f(2)) / 3.0;
    let k = if s.roll_mode == RollMode::BasicLab {
        0.5
    } else {
        1.0
    };
    let dlo = k * mean(&|c| (frame.levels[c].lo - roll_levels[c].lo) / span(c));
    let dhi = k * mean(&|c| (frame.levels[c].hi - roll_levels[c].hi) / span(c));
    Some(std::array::from_fn(|c| {
        let r = roll_levels[c];
        ChannelLevels {
            lo: r.lo + dlo * span(c),
            hi: r.hi + dhi * span(c),
            mid: r.mid + (dlo + dhi) / 2.0 * span(c),
        }
    }))
}

// ---- rendering --------------------------------------------------------------------

/// Everything needed to turn negative densities into output pixels.
pub struct Renderer {
    lut: CurveLut,
    converted: bool,
    look: Look,
    out_m: [[f32; 3]; 3],
    out_enc: color::EncodeLut,
    sharpen: Option<presets::SharpenDef>,
}

impl Renderer {
    /// `levels == None` renders the negative as scanned (unconverted).
    pub fn new(s: &Settings, levels: Option<&Levels>, space: OutputSpace) -> Self {
        let (out_m, f) = color::output_transform(space);
        Self {
            lut: match levels {
                Some(l) => CurveLut::new(l, s),
                None => CurveLut::negative_view(),
            },
            converted: levels.is_some(),
            look: Look::new(s),
            out_m,
            out_enc: color::EncodeLut::new(f),
            sharpen: presets::sharpening(s.sharpening),
        }
    }

    /// Scale sharpening for the image size (relative to 24 MP; strength
    /// capped at 150 %, radius at 1–3 px).
    pub fn with_megapixels(mut self, mp: f32) -> Self {
        if let Some(sd) = &mut self.sharpen {
            let f = (mp - 24.0) / 2.0 / 24.0 + 1.0;
            let sharpness = (sd.lr_sharpness * f).round().min(150.0);
            sd.amount *= sharpness / sd.lr_sharpness;
            sd.radius = (sd.radius * f).clamp(1.0, 3.0);
        }
        self
    }

    pub fn curve_lut(&self) -> Option<&CurveLut> {
        self.converted.then_some(&self.lut)
    }

    pub fn uses_lut_table(&self) -> bool {
        self.look.uses_table()
    }

    /// Encoded MelissaRGB positive (before output conversion).
    #[inline]
    pub fn positive(&self, d: [f32; 3]) -> [f32; 3] {
        let p = [
            self.lut.apply(0, d[0]),
            self.lut.apply(1, d[1]),
            self.lut.apply(2, d[2]),
        ];
        if self.converted {
            self.look.apply(p)
        } else {
            p
        }
    }

    #[inline]
    pub fn output(&self, mel_enc: [f32; 3]) -> [f32; 3] {
        let dec = color::srgb_decode_lut();
        let lin = [
            dec.get(mel_enc[0]),
            dec.get(mel_enc[1]),
            dec.get(mel_enc[2]),
        ];
        let o = color::mul3(&self.out_m, lin);
        [
            self.out_enc.get(o[0]),
            self.out_enc.get(o[1]),
            self.out_enc.get(o[2]),
        ]
    }

    /// Render a whole buffer to encoded output pixels. `sharpen_scale` is the
    /// ratio of this buffer to full resolution (0 disables sharpening).
    pub fn render(&self, neg: &Negative, sharpen_scale: f32) -> Vec<[f32; 3]> {
        let mut out: Vec<[f32; 3]> = neg
            .data
            .par_iter()
            .map(|p| self.output(self.positive(*p)))
            .collect();
        if let (Some(sd), true) = (self.sharpen, sharpen_scale > 0.0 && self.converted) {
            look::sharpen(&mut out, neg.width, neg.height, sd, sharpen_scale);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levels_find_the_density_range() {
        let data: Vec<[f32; 3]> = (0..1000)
            .map(|i| {
                let v = 0.2 + i as f32 / 999.0;
                [v, v + 0.1, v + 0.3]
            })
            .collect();
        let buf = Negative {
            width: 1000,
            height: 1,
            data,
        };
        let (l, _) = levels_from_buffer(&buf, ImageType::Negative);
        assert!(
            (l[0].lo - 0.2).abs() < 0.01 && (l[0].hi - 1.2).abs() < 0.01,
            "{l:?}"
        );
        assert!((l[2].mid - 1.0).abs() < 0.01, "{l:?}");
    }

    #[test]
    fn vuescan_dng_is_detected_as_scanner() {
        let p = std::path::Path::new("../testphoto.dng");
        if !p.exists() {
            return;
        }
        let src = decode::load(p, Some(400)).unwrap();
        assert_eq!(
            detect_source(&src),
            crate::engine::settings::Source::VueScanRaw
        );
        let mut cam = src.clone();
        cam.info.camera_ids = vec!["Nikon D750".into()];
        cam.info.make = "Nikon".into();
        cam.info.model = "D750".into();
        assert_eq!(
            detect_source(&cam),
            crate::engine::settings::Source::DigitalCamera
        );
        let mut unknown = src.clone();
        unknown.info.camera_ids = vec!["Mystery Cam 1".into()];
        unknown.info.make = "Mystery".into();
        unknown.info.model = "Cam 1".into();
        assert_eq!(
            detect_source(&unknown),
            crate::engine::settings::Source::DigitalCamera
        );
    }

    fn frame(lo: [f64; 3], hi: [f64; 3]) -> Analysis {
        Analysis {
            levels: std::array::from_fn(|c| ChannelLevels {
                lo: lo[c],
                hi: hi[c],
                mid: (lo[c] + hi[c]) / 2.0,
            }),
            auto_wb: AutoWb::default(),
            base: [1.0; 3],
            key: Settings::default().analysis_key(),
            ev: None,
        }
    }

    fn roll_of(agg: &RollAggregate, frames: usize) -> RollData {
        RollData {
            id: "r".into(),
            levels: agg.levels,
            base: [1.0; 3],
            frames,
            excluded: vec![],
            auto_wb: Some(agg.auto_wb),
            common_ev: agg.common_ev,
        }
    }

    #[test]
    fn roll_takes_the_median_and_drops_outliers() {
        let mut frames: Vec<Analysis> = (0..6)
            .map(|i| {
                let k = i as f64 * 0.01;
                frame([0.10 + k, 0.15 + k, 0.20 + k], [1.0 + k, 1.1 + k, 1.2 + k])
            })
            .collect();
        // A frame with a very different colour balance.
        frames.push(frame([0.10, 0.45, 0.20], [1.0, 1.4, 1.2]));
        let (r, excluded) = aggregate_roll(&frames, None);
        assert_eq!(excluded, vec![6]);
        assert!((r.levels[0].lo - 0.125).abs() < 0.006, "{:?}", r.levels);
        assert!((r.levels[1].hi - 1.125).abs() < 0.006, "{:?}", r.levels);
    }

    #[test]
    fn roll_process_modes() {
        let frames = [
            frame([0.10, 0.15, 0.20], [1.00, 1.10, 1.20]),
            frame([0.20, 0.27, 0.36], [1.20, 1.28, 1.42]),
        ];
        let (agg, _) = aggregate_roll(&frames, None);
        let roll = roll_of(&agg, 2);
        let mut s = Settings {
            use_roll: true,
            ..Settings::default()
        };
        let get = |s: &Settings| levels_for(s, Some(&frames[1]), Some(&roll)).unwrap();
        s.roll_mode = RollMode::DarkroomPaper;
        assert_eq!(get(&s), agg.levels);
        s.roll_mode = RollMode::ProLab;
        let pro = get(&s);
        // Every channel moves by the same fraction of its span: the roll's
        // colour balance on the frame's overall range.
        let frac = |c: usize, f: fn(&ChannelLevels) -> f64| {
            (f(&pro[c]) - f(&agg.levels[c])) / (agg.levels[c].hi - agg.levels[c].lo)
        };
        assert!((frac(0, |c| c.lo) - frac(2, |c| c.lo)).abs() < 1e-9);
        assert!((frac(1, |c| c.hi) - frac(2, |c| c.hi)).abs() < 1e-9);
        assert!(frac(0, |c| c.lo) > 0.0, "the frame is denser than the roll");
        s.roll_mode = RollMode::BasicLab;
        let basic = get(&s);
        assert!((basic[0].lo - (agg.levels[0].lo + pro[0].lo) / 2.0).abs() < 1e-9);
        s.use_roll = false;
        assert_eq!(get(&s), frames[1].levels);
    }

    #[test]
    fn exposure_value_rounds_to_thirds() {
        // f/8, 1/125 s, ISO 100: log2(64 · 125) = 12.97 → 13.
        assert_eq!(exposure_value(1.0 / 125.0, 8.0, 100.0), 13.0);
        assert_eq!(round_ev(10.1), 10.0);
        assert!((round_ev(10.3) - (10.0 + 1.0 / 3.0)).abs() < 1e-12);
        assert!((round_ev(10.7) - (10.0 + 2.0 / 3.0)).abs() < 1e-12);
        assert_eq!(round_ev(10.9), 11.0);
    }

    #[test]
    fn roll_exposure_matching() {
        let third = 1.0 / 3.0;
        let shift = third * std::f64::consts::LOG10_2;
        let mut frames = [
            frame([0.2; 3], [1.2; 3]),
            frame([0.2; 3], [1.2; 3]),
            frame([0.2 + shift; 3], [1.2 + shift; 3]),
        ];
        frames[0].ev = Some(12.0);
        frames[1].ev = Some(12.0);
        frames[2].ev = Some(12.0 + third);
        let c = common_ev(&frames).unwrap();
        assert_eq!(c, 12.0);
        assert!(ev_spread(&frames, c) < 1.0);
        // The +1/3 EV scan reads denser; moved onto the common exposure it
        // matches the others exactly.
        let (agg, _) = aggregate_roll(&frames, Some(c));
        assert!((agg.levels[0].lo - 0.2).abs() < 1e-6, "{:?}", agg.levels);
        let roll = roll_of(&agg, 3);
        let s = Settings {
            use_roll: true,
            roll_mode: RollMode::DarkroomPaper,
            ..Settings::default()
        };
        // Rendering that frame moves the roll back onto its own exposure.
        let l = levels_for(&s, Some(&frames[2]), Some(&roll)).unwrap();
        assert!((l[0].hi - frames[2].levels[0].hi).abs() < 1e-6);
        // More than 1 EV apart asks before matching.
        frames[2].ev = Some(13.5);
        assert!(ev_spread(&frames, common_ev(&frames).unwrap()) > 1.0);
    }

    /// A synthetic colour negative (orange mask, per-channel H&D gamma, so
    /// the plain inversion has a colour cast) of a mostly neutral scene.
    fn synthetic_negative() -> (SourceImage, Vec<usize>) {
        let (w, h) = (300usize, 200usize);
        let base = [0.92f32, 0.62, 0.38];
        let gamma = [0.58f32, 0.66, 0.74];
        let mut data = vec![[0.0f32; 3]; w * h];
        let mut grey = Vec::new();
        for y in 0..h {
            for x in 0..w {
                let border = x < 10 || x >= w - 10 || y < 10 || y >= h - 10;
                let t = (x - 10.min(x)) as f32 / (w - 20) as f32;
                let scene: Option<[f32; 3]> = if border {
                    None
                } else if y < 100 {
                    Some([t * t * 0.8 + 0.02; 3])
                } else if y < 150 {
                    Some(match (x * 4) / w {
                        0 => [0.45, 0.32, 0.25],
                        1 => [0.12, 0.25, 0.55],
                        2 => [0.15, 0.35, 0.10],
                        _ => [0.60, 0.10, 0.08],
                    })
                } else {
                    Some([0.18; 3])
                };
                if !border && y >= 160 && y < 180 && x > 40 && x < w - 40 {
                    grey.push(y * w + x);
                }
                data[y * w + x] = match scene {
                    None => base,
                    Some(e) => std::array::from_fn(|c| {
                        base[c] * ((e[c] + 0.004) / 0.004).powf(-gamma[c] * 0.42)
                    }),
                };
            }
        }
        let src = SourceImage {
            width: w,
            height: h,
            data,
            to_melissa: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            info: Default::default(),
            as_shot_wb: [1.0; 3],
            is_raw: false,
        };
        (src, grey)
    }

    fn cast(px: &[[f32; 3]], idx: &[usize]) -> f32 {
        let mut m = [0.0f32; 3];
        for &i in idx {
            for c in 0..3 {
                m[c] += px[i][c] / idx.len() as f32;
            }
        }
        (m[0] - m[1]).abs() + (m[2] - m[1]).abs()
    }

    #[test]
    fn auto_neutral_keeps_greys_neutral() {
        let (src, grey) = synthetic_negative();
        let mut s = Settings::default();
        s.lut = crate::engine::settings::Lut::None;
        let base = resolve_base(&src, &s, None);
        let neg = prepare_negative(&src, &s, base);
        let a = analyze(&neg, &s, base, None);
        let (t, n) = wb_for_mode(WbMode::AutoNeutral, &a.auto_wb).unwrap();
        let mut wb = s.clone();
        wb.wb_temp = t;
        wb.wb_tint = n;
        let px = Renderer::new(&wb, Some(&a.levels), OutputSpace::Srgb).render(&neg, 0.0);
        let after = cast(&px, &grey);
        assert!(after < 0.01, "auto WB {:?}: cast {after}", (t, n));
    }

    #[test]
    fn base_is_zero_density() {
        let (src, _) = synthetic_negative();
        let s = Settings::default();
        let base = resolve_base(&src, &s, None);
        let neg = prepare_negative(&src, &s, base);
        // The border is unexposed base.
        assert!(
            neg.data[0].iter().all(|d| d.abs() < 0.02),
            "{:?}",
            neg.data[0]
        );
        // Image areas are denser.
        assert!(neg.data[150 * 300 + 150].iter().all(|d| *d > 0.1));
    }

    #[test]
    fn lab_keys_shift_towards_their_colour() {
        use crate::engine::settings::lab::{self, Channel};
        let (src, grey) = synthetic_negative();
        let mut s = Settings::default();
        s.lut = crate::engine::settings::Lut::None;
        let base = resolve_base(&src, &s, None);
        let neg = prepare_negative(&src, &s, base);
        let a = analyze(&neg, &s, base, None);
        let mean = |s: &Settings| {
            let px = Renderer::new(s, Some(&a.levels), OutputSpace::Srgb).render(&neg, 0.0);
            let mut m = [0.0f32; 3];
            for &i in &grey {
                for c in 0..3 {
                    m[c] += px[i][c] / grey.len() as f32;
                }
            }
            m
        };
        let m0 = mean(&s);
        // +C / +M / +Y take away red / green / blue; +D darkens.
        for (ch, c) in [(Channel::C, 0), (Channel::M, 1), (Channel::Y, 2)] {
            let mut t = s.clone();
            lab::nudge(&mut t, ch, 5);
            let m = mean(&t);
            let rel = |m: [f32; 3]| m[c] - (m[0] + m[1] + m[2]) / 3.0;
            assert!(rel(m) < rel(m0), "{ch:?}: {m0:?} -> {m:?}");
        }
        let mut t = s.clone();
        lab::nudge(&mut t, Channel::D, 3);
        let m = mean(&t);
        assert!(m[1] < m0[1], "D+: {m0:?} -> {m:?}");
    }

    #[test]
    fn edit_only_skips_analysis() {
        let (src, _) = synthetic_negative();
        let mut s = Settings::default();
        s.image_type = ImageType::EditOnly;
        let base = resolve_base(&src, &s, None);
        let neg = prepare_negative(&src, &s, base);
        let a = analyze(&neg, &s, base, None);
        assert_eq!(a.levels, edit_only_levels());
        assert_eq!(a.auto_wb, AutoWb::default());
    }

    /// Convert a test chart the way the app does: analyse, apply the auto
    /// WB of the frame's (or roll's) WB mode, render to sRGB.
    fn convert_chart(
        chart: &crate::engine::testchart::Chart,
        s: &Settings,
        a: &Analysis,
        roll: Option<&RollData>,
    ) -> Vec<[f32; 3]> {
        let neg = prepare_negative(&chart.src, s, a.base);
        let mut s = s.clone();
        s.use_roll = roll.is_some();
        let wb = roll.and_then(|r| r.auto_wb).unwrap_or(a.auto_wb);
        if let Some((t, n)) = wb_for_mode(s.wb_mode, &wb) {
            (s.wb_temp, s.wb_tint) = (t, n);
        }
        let l = levels_for(&s, Some(a), roll).unwrap();
        Renderer::new(&s, Some(&l), OutputSpace::Srgb).render(&neg, 0.0)
    }

    fn chart_settings() -> Settings {
        Settings {
            lut: crate::engine::settings::Lut::None,
            ..Settings::default()
        }
    }

    fn analyse_chart(chart: &crate::engine::testchart::Chart, s: &Settings) -> Analysis {
        let base = resolve_base(&chart.src, s, None);
        analyze(&prepare_negative(&chart.src, s, base), s, base, None)
    }

    /// Mean over the 24 patches of the largest colour difference (lightness
    /// ignored) between any two frames.
    fn roll_spread(chart: &crate::engine::testchart::Chart, frames: &[Vec<[f32; 3]>]) -> f64 {
        use crate::engine::testchart::{color_de, patch_lab};
        let labs: Vec<_> = frames.iter().map(|f| patch_lab(chart, f)).collect();
        let mut sum = 0.0;
        for p in 0..24 {
            let mut worst = 0.0f64;
            for a in &labs {
                for b in &labs {
                    worst = worst.max(color_de(a[p], b[p]));
                }
            }
            sum += worst;
        }
        sum / 24.0
    }

    /// Auto-Neutral on scenes dominated by one colour (foliage, sunset sky,
    /// skin-toned interior): the grey ramp and neutral patches must stay grey.
    #[test]
    fn dominated_scenes_stay_neutral() {
        use crate::engine::testchart::{dominated, neutral_cast};
        let s = chart_settings();
        let mut worst = 0.0f64;
        for (name, field) in [
            ("foliage", [43.46, -32.0, 38.0]),
            ("sunset", [62.0, 30.0, 55.0]),
            ("interior", [60.0, 14.0, 20.0]),
            ("sky", [60.0, -6.0, -38.0]),
        ] {
            let chart = dominated(0.0, field);
            let a = analyse_chart(&chart, &s);
            let (mean, max) = neutral_cast(&chart, &convert_chart(&chart, &s, &a, None));
            eprintln!("dominated {name}: neutral C* {mean:.2} (max {max:.2})");
            if std::env::var("NLP_CHART_DEBUG").is_ok() {
                eprintln!("  levels {:?}\n  wb {:?}", a.levels, a.auto_wb);
                let own = channel_quantiles(
                    &analysis_buffer(&prepare_negative(&chart.src, &s, a.base), &s),
                    s.image_type,
                );
                eprintln!("  own {own:?}");
            }
            worst = worst.max(mean);
        }
        eprintln!("dominated worst neutral C* {worst:.2}");
        // Plugin-port engine: foliage 1.01, sunset 1.67, interior 8.78,
        // sky 15.40.
        assert!(worst < 0.6, "dominated scene neutral C* {worst}");
    }

    /// Conversion quality on the physically modelled chart: colour accuracy,
    /// neutrality, and consistency of a roll shot at −1, 0 and +1 EV.
    #[test]
    fn chart_quality() {
        use crate::engine::testchart::{measure, negative};
        let s = chart_settings();
        let chart = negative(0.0);
        let a = analyse_chart(&chart, &s);
        let out = convert_chart(&chart, &s, &a, None);
        let m = measure(&chart, &out);
        if std::env::var("NLP_CHART_DEBUG").is_ok() {
            let labs = crate::engine::testchart::patch_lab(&chart, &out);
            for (p, l) in chart.patches.iter().zip(&labs).take(24) {
                eprintln!(
                    "ref {:6.1} {:6.1} {:6.1}  out {:6.1} {:6.1} {:6.1}  dE {:5.2}",
                    p.lab[0],
                    p.lab[1],
                    p.lab[2],
                    l[0],
                    l[1],
                    l[2],
                    crate::engine::testchart::color_de(p.lab, *l)
                );
            }
        }

        let charts: Vec<_> = [-1.0, 0.0, 1.0].into_iter().map(negative).collect();
        let analyses: Vec<_> = charts.iter().map(|c| analyse_chart(c, &s)).collect();
        let single: Vec<_> = charts
            .iter()
            .zip(&analyses)
            .map(|(c, a)| convert_chart(c, &s, a, None))
            .collect();
        let (agg, _) = aggregate_roll(&analyses, None);
        let roll = RollData {
            base: analyses[1].base,
            ..roll_of(&agg, 3)
        };
        let rolled: Vec<_> = charts
            .iter()
            .zip(&analyses)
            .map(|(c, a)| convert_chart(c, &s, a, Some(&roll)))
            .collect();
        let frame_spread = roll_spread(&chart, &single);
        let roll_spread = roll_spread(&chart, &rolled);
        let roll_cast = rolled
            .iter()
            .map(|f| measure(&chart, f).neutral_cast)
            .fold(0.0, f64::max);
        eprintln!(
            "chart: colour dE00 {:.2}, tone-matched dE00 {:.2}, full dE00 {:.2}, \
             neutral C* {:.2} (max {:.2}); frame spread {:.2}, roll spread {:.2}, \
             roll neutral C* {:.2}",
            m.color_de,
            m.tone_matched_de,
            m.full_de,
            m.neutral_cast,
            m.neutral_cast_max,
            frame_spread,
            roll_spread,
            roll_cast
        );
        // Plugin-port engine baseline: tone-matched 5.00, neutral C* 3.22
        // (max 6.86), frame spread 2.78, roll spread 3.70, roll C* 4.66. The
        // bounds only tighten. Plain colour dE (5.21 before) is shown but
        // only loosely bounded: it also scores the tone curve's contrast.
        assert!(
            m.tone_matched_de < 2.7,
            "tone-matched dE00 {}",
            m.tone_matched_de
        );
        assert!(m.color_de < 5.5, "colour dE00 {}", m.color_de);
        assert!(m.neutral_cast < 0.5, "neutral C* {}", m.neutral_cast);
        assert!(
            m.neutral_cast_max < 1.2,
            "neutral C* max {}",
            m.neutral_cast_max
        );
        assert!(frame_spread < 0.7, "frame spread {frame_spread}");
        assert!(roll_spread < 1.6, "roll spread {roll_spread}");
        assert!(roll_cast < 0.8, "roll neutral C* {roll_cast}");
    }

    #[test]
    fn auto_base_samples_entire_image_regardless_of_crop() {
        let mut data = vec![[0.1f32, 0.1, 0.1]; 100 * 100];
        // Put unexposed film base in the border (outer 5 rows)
        for y in 0..5 {
            for x in 0..100 {
                data[y * 100 + x] = [0.9f32, 0.6, 0.4];
            }
        }
        let src = SourceImage {
            width: 100,
            height: 100,
            data,
            to_melissa: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            info: Default::default(),
            as_shot_wb: [1.0; 3],
            is_raw: false,
        };

        let uncropped = Settings::default();
        let mut cropped = Settings::default();
        // Crop out the top 20% containing the border
        cropped.crop = [20.0, 10.0, 10.0, 10.0];

        let base_uncropped = auto_base(&src, &uncropped);
        let base_cropped = auto_base(&src, &cropped);

        assert_eq!(
            base_uncropped, base_cropped,
            "auto_base must look at the entire image even when cropped"
        );
        assert!((base_cropped[0] - 0.9).abs() < 1e-3);
    }
}
