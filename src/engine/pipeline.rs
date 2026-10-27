//! Negative preparation (scan → density), analysis and rendering.

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use super::color;
use super::curve::{self, ChannelLevels, CurveLut, Levels};
use super::decode::{self, SourceImage};
use super::dodge::{self, Dodge};
use super::look::{self, Look};
use super::presets;
use super::settings::{AnalysisKey, FilmBase, ImageType, OutputSpace, RollMode, Settings, WbMode};
use super::sidecar::RollData;
use super::skin::{self, SkinTone};

/// Long edge of the analysis rendition.
pub const ANALYSIS_SIZE: usize = 600;
/// Film base is mapped to this linear level after neutralisation.
const BASE_LEVEL: f32 = 0.97;
/// Base saturation in density. Crosstalk correction followed by per-channel
/// tone curves as steep as the plugin's profiles prints about 20 % more
/// chroma than the scene had; this brings the chart's colours back to their
/// true chroma (ratio 1.04, the plugin prints 0.96) and gives the lowest
/// colour error against them.
const DENSITY_SATURATION: f32 = 0.85;
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
    /// The neutral axis the levels were fitted to, when the frame had one.
    #[serde(default)]
    pub axis: Option<NeutralAxis>,
    /// The skin the midtone key was moved towards, when the frame had any.
    #[serde(default)]
    pub skin: Option<SkinTone>,
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
    let f = (1.0 + presets::pre_saturation(s.pre_saturation) / 100.0) * DENSITY_SATURATION;
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

/// The scan source implied by a file: demosaiced (LinearRaw) DNGs and
/// files written by scanner software are VueScan / SilverFast DNGs, other
/// raw files digital camera captures, bitmaps TIF scans.
pub fn detect_source(src: &SourceImage) -> super::settings::Source {
    use super::settings::Source;
    let sw = src.info.software.to_ascii_lowercase();
    let scanner = ["vuescan", "silverfast"].iter().any(|s| sw.contains(s));
    if src.is_raw && (src.info.linear_raw || scanner) {
        Source::VueScanRaw
    } else if src.is_raw {
        Source::DigitalCamera
    } else {
        Source::TifScan
    }
}

/// Build the negative's density image from linear source pixels: flat
/// field, camera matrix, film base neutralisation, `log10`, crosstalk and
/// calibration.
pub fn prepare_negative(src: &SourceImage, s: &Settings, base: [f32; 3]) -> Negative {
    let flat = s
        .flat_field
        .as_deref()
        .and_then(super::flat::get)
        .map(|f| f.apply(&src.data, src.width, src.height));
    let (d, w, h) = decode::geometry(
        flat.as_deref().unwrap_or(&src.data),
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
    let base = if !src.is_raw && s.source == super::settings::Source::LinearScan {
        base.map(color::srgb_encode)
    } else {
        base
    };
    let (mul, level) = if edit_only {
        ([1.0; 3], 1.0)
    } else {
        (base_multipliers(src, s, base), BASE_LEVEL)
    };
    let to_lin = src.to_melissa;
    // Bitmaps are decoded as sRGB-encoded; linear (gamma 1.0) scans hold
    // linear values, so undo that decode.
    let linear_scan = !src.is_raw && s.source == super::settings::Source::LinearScan;
    let m = density_matrix(s);
    let mono = s.is_monochrome();
    let yw = DENSITY_WEIGHTS;
    let data = d
        .par_iter()
        .map(|p| {
            let p = if linear_scan {
                p.map(color::srgb_encode)
            } else {
                *p
            };
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

/// The 600px, border-cropped, blurred analysis buffer. Film rebate, holder
/// and light past the film are trimmed off the edges first (see
/// [`frame_insets`]), then the border buffer.
pub fn analysis_buffer(neg: &Negative, s: &Settings) -> Negative {
    let (d, w, h) = decode::downscale(&neg.data, neg.width, neg.height, ANALYSIS_SIZE);
    let (d, w, h) = if s.image_type == ImageType::EditOnly {
        (d, w, h)
    } else {
        decode::crop(&d, w, h, frame_insets(&d, w, h))
    };
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

/// Whether a density is not image: unexposed film (within 0.05 D of the
/// base), light past the film (thinner than the base), or the holder (at
/// least `opaque` in every channel).
fn not_image(p: [f32; 3], opaque: f32) -> bool {
    p.iter().all(|d| d.abs() < 0.05)
        || p.iter().any(|&d| d < -0.15)
        || p.iter().all(|&d| d > opaque)
}

/// Density from which an edge counts as holder: within 0.2 D of the
/// frame's densest area. A holder or mask reads as dense as the scan can
/// measure, which for many scanners and cameras is only 2–2.5 D.
fn opaque_density(d: &[[f32; 3]]) -> f32 {
    let mut v: Vec<f32> = d.iter().map(|p| p[0].min(p[1]).min(p[2])).collect();
    (quantile(&mut v, 0.999) as f32 - 0.2).max(1.4)
}

/// Insets (percent: left, top, right, bottom) that trim the rows and columns
/// along the edges that are mostly not image, up to 20 % per side, plus two
/// lines for the soft edge between film border and image.
fn frame_insets(d: &[[f32; 3]], w: usize, h: usize) -> [f32; 4] {
    let opaque = opaque_density(d);
    let line_is_border = |pixels: &mut dyn Iterator<Item = [f32; 3]>| {
        let (mut n, mut border) = (0usize, 0usize);
        for p in pixels {
            n += 1;
            border += not_image(p, opaque) as usize;
        }
        n > 0 && border * 2 >= n
    };
    let (mx, my) = (w / 10, h / 10);
    let row = |y: usize| line_is_border(&mut (mx..w - mx).map(|x| d[y * w + x]));
    let col = |x: usize| line_is_border(&mut (my..h - my).map(|y| d[y * w + x]));
    let run = |n: usize, f: &dyn Fn(usize) -> bool| match (0..n / 5).take_while(|&i| f(i)).count() {
        0 => 0,
        k => k + 2,
    };
    let left = run(w, &|i| col(i));
    let right = run(w, &|i| col(w - 1 - i));
    let top = run(h, &|i| row(i));
    let bottom = run(h, &|i| row(h - 1 - i));
    let pct = |v: usize, n: usize| v as f32 * 100.0 / n as f32;
    [pct(left, w), pct(top, h), pct(right, w), pct(bottom, h)]
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
    // On a negative nothing in the image is thinner than the film base:
    // such pixels are flare or light past the film, not shadows.
    let film = |p: &[f32; 3]| t != ImageType::Negative || p.iter().all(|&d| d > -0.03);
    std::array::from_fn(|c| {
        let mut v: Vec<f32> = buf.data.iter().filter(|p| film(p)).map(|p| p[c]).collect();
        // Negatives print thin black, positives print thin white.
        let (lo_q, hi_q) = match t {
            ImageType::Positive => (white_t, 1.0 - black_t),
            _ => (black_t, 1.0 - white_t),
        };
        let lo = quantile(&mut v, lo_q);
        let hi = quantile(&mut v, hi_q).max(lo + 0.05);
        let mid = quantile(&mut v, 0.5);
        ChannelLevels {
            lo,
            hi,
            mid,
            bow: 0.0,
        }
    })
}

/// Median position (share of the uncapped range) below which a frame is
/// mostly dark, as at night.
const DARK_SHARE: f64 = 0.25;
/// Density range (D) between a frame's 97th percentile and its white end
/// that marks the area above the white-end cap as a lit subject.
const LIT_SPREAD: f64 = 0.1;

/// Smallest green density range skin is looked for in: a frame mostly at
/// its black end (a flash at night) has its white end capped close to the
/// black, with a flashed face far beyond it.
const SKIN_RANGE: f64 = 0.8;

/// The image's neutral axis: per channel `D_c = a + b · D_green`, plus
/// `bend_c · p(D_green)` where the film's layers cross over (their toes and
/// shoulders differ, so a grey's balance drifts from shadows to highlights).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NeutralAxis {
    pub fit: [(f64, f64); 3],
    /// Mean density of the tonal steps (a point on the line).
    pub centre: [f64; 3],
    /// Tonal steps (of 10) with enough near-neutral pixels.
    pub steps: usize,
    /// Curvature of each channel's neutral curve (density per D² of green).
    #[serde(default)]
    pub bend: [f64; 3],
    /// `[m, u, v]` of the quadratic `p(g) = (g − m)² − u·(g − m) − v`,
    /// orthogonal to the line over the tonal steps, so the bend leaves the
    /// fitted line as it is.
    #[serde(default)]
    pub basis: [f64; 3],
}

impl NeutralAxis {
    fn p(&self, g: f64) -> f64 {
        let [m, u, v] = self.basis;
        let x = g - m;
        x * x - u * x - v
    }

    /// The grey of the axis at green density `g`.
    fn at(&self, g: f64) -> [f64; 3] {
        std::array::from_fn(|c| self.fit[c].0 + self.fit[c].1 * g + self.bend[c] * self.p(g))
    }
}

/// Fewest tonal steps from which a frame's neutral axis may bend.
const BEND_STEPS: usize = 6;
/// Largest departure (D) of a neutral curve from its straight line.
const MAX_BEND_D: f64 = 0.05;

/// The [`ChannelLevels::bow`] of a neutral curve with curvature `bend`
/// (density per D² of green) over green span `span_g` and channel span
/// `span_c`, bounded so the curve stays within [`MAX_BEND_D`] of the line.
fn bow_for(bend: f64, span_g: f64, span_c: f64) -> f64 {
    let span_c = span_c.max(1e-3);
    let limit = curve::MAX_BOW.min(4.0 * MAX_BEND_D / span_c);
    (-bend * span_g * span_g / span_c).clamp(-limit, limit)
}

/// Analysed levels: green's own density range, with red and blue placed on
/// the image's neutral axis (see [`neutral_axis`]) so their slopes match the
/// film's per-layer H&D slopes. Keeps each channel's own range when the
/// image has too little near-neutral content.
pub fn levels_from_buffer(buf: &Negative, t: ImageType) -> (Levels, Option<NeutralAxis>) {
    let mut l = channel_quantiles(buf, t);
    if t == ImageType::Negative {
        // A light source or specular in the frame is far denser than the
        // scene around it and would print the whole scene dark: keep green's
        // white end within twice its shadow-to-median span above the median,
        // and shorten every channel's range by the same fraction so neutrals
        // stay neutral. A lit subject in a mostly dark frame (a flash at
        // night) is different: it spans many tones above that cap, and only
        // its top 3 % of the frame may clip (Fuji counts an extreme from 3 %
        // of the area). A light source is one plateau of density; a holder
        // or mask left in the frame is left out, as dense as the scan reads.
        let g = l[1];
        let mut cap = (g.mid + 2.0 * (g.mid - g.lo)).max(g.mid + 0.1);
        let opaque = opaque_density(&buf.data);
        let mut v: Vec<f32> = buf
            .data
            .iter()
            .filter(|p| p.iter().all(|&d| d > -0.03) && p[1] < opaque)
            .map(|p| p[1])
            .collect();
        let (top, hi) = (quantile(&mut v, 0.97), quantile(&mut v, 0.9995));
        let dark = g.mid - g.lo < DARK_SHARE * (hi - g.lo);
        if dark && top > cap && hi - top > LIT_SPREAD {
            cap = top;
        }
        let f = ((cap - g.lo) / (g.hi - g.lo).max(1e-6)).min(1.0);
        for c in l.iter_mut() {
            c.hi = c.lo + (c.hi - c.lo) * f;
        }
    }
    let own = l;
    let mut axis = None;
    for _ in 0..3 {
        let Some(fit) = neutral_axis(buf, &l, t) else {
            break;
        };
        for c in [0, 2] {
            let b = fit.fit[c].1;
            // Keep the slope within reach of the channel's own range.
            let own_b = (own[c].hi - own[c].lo) / (own[1].hi - own[1].lo).max(1e-3);
            let b = b.clamp(own_b * 0.8, own_b * 1.25);
            let at = |g: f64| fit.centre[c] + b * (g - fit.centre[1]) + fit.bend[c] * fit.p(g);
            let (lo, hi) = (at(l[1].lo), at(l[1].hi).max(at(l[1].lo) + 0.05));
            l[c] = ChannelLevels {
                lo,
                hi,
                mid: own[c].mid,
                bow: bow_for(fit.bend[c], l[1].hi - l[1].lo, hi - lo),
            };
        }
        axis = Some(fit);
    }
    (l, axis)
}

/// The neutral axis of an image: near-neutral pixels (normalised channels
/// within 0.1 of each other under the current levels `l`), binned by green
/// density into ten tonal steps, and a line `D_c = a + b · D_green` through
/// each step's grey (see [`neutral_cluster`]) for each channel. Every step weighs the same,
/// so a large area of one colour and tone can bias at most one step. With
/// [`BEND_STEPS`] steps or more the line also bends: a quadratic term fitted
/// through the steps, like the gradation-balance curves a lab scanner keeps
/// per film (Fuji US 6,160,634, EP 0 612 183). `None` without near-neutral
/// pixels in at least four steps.
fn neutral_axis(buf: &Negative, l: &Levels, t: ImageType) -> Option<NeutralAxis> {
    const BINS: usize = 10;
    // Near-neutral pixels per tonal step, with their colour offset from the
    // current axis (red and blue against green, normalised).
    let mut bins: Vec<Vec<([f32; 3], [f64; 2])>> = vec![Vec::new(); BINS];
    for p in &buf.data {
        let x: [f64; 3] = std::array::from_fn(|c| normalised(l, t, c, p[c] as f64));
        let (lo, hi) = x
            .iter()
            .fold((f64::MAX, f64::MIN), |(a, b), &v| (a.min(v), b.max(v)));
        if hi - lo < 0.1 && (0.03..0.97).contains(&x[1]) {
            bins[((x[1] * BINS as f64) as usize).min(BINS - 1)]
                .push((*p, [x[0] - x[1], x[2] - x[1]]));
        }
    }
    let min_count = (buf.data.len() / 500).max(8);
    let steps: Vec<[f64; 3]> = bins
        .into_iter()
        .filter(|b| b.len() >= min_count)
        .map(|b| {
            neutral_cluster(&b, min_count).unwrap_or_else(|| {
                std::array::from_fn(|c| median(b.iter().map(|p| p.0[c] as f64).collect()))
            })
        })
        .collect();
    let mut steps = steps;
    if steps.len() < 4 {
        return None;
    }
    // A step whose near-neutral pixels are mostly one coloured object sits
    // off the axis: drop the worst step while it is further off than the
    // rest justify (EP 0 612 183's band around the axis).
    let mut fit = line_fit(&steps);
    while steps.len() > 4 {
        let off: Vec<f64> = steps
            .iter()
            .map(|s| [0, 2].map(|c| (s[c] - fit[c].0 - fit[c].1 * s[1]).abs()))
            .map(|[r, b]| r.max(b))
            .collect();
        let (worst, &max) = off
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(b.1))
            .unwrap();
        if max <= (3.0 * 1.4826 * median(off.clone())).max(0.02) {
            break;
        }
        steps.remove(worst);
        fit = line_fit(&steps);
    }
    let n = steps.len() as f64;
    let mut axis = NeutralAxis {
        fit,
        centre: std::array::from_fn(|c| steps.iter().map(|s| s[c]).sum::<f64>() / n),
        steps: steps.len(),
        bend: [0.0; 3],
        basis: [0.0; 3],
    };
    if steps.len() >= BEND_STEPS {
        let m = axis.centre[1];
        let x: Vec<f64> = steps.iter().map(|s| s[1] - m).collect();
        let x2: f64 = x.iter().map(|v| v * v).sum();
        let x3: f64 = x.iter().map(|v| v * v * v).sum();
        if x2 > 1e-6 {
            axis.basis = [m, x3 / x2, x2 / n];
            let p: Vec<f64> = steps.iter().map(|s| axis.p(s[1])).collect();
            let pp: f64 = p.iter().map(|v| v * v).sum();
            // Bend only as far as the steps reach across green's range: a
            // curve fitted to part of the range must not be extrapolated.
            let (g0, g1) = steps
                .iter()
                .fold((f64::MAX, f64::MIN), |(a, b), s| (a.min(s[1]), b.max(s[1])));
            let reach = (g1 - g0) / (l[1].hi - l[1].lo).max(1e-3);
            let trust = ((reach - 0.5) / 0.3).clamp(0.0, 1.0);
            if pp > 1e-12 {
                for c in [0, 2] {
                    axis.bend[c] =
                        trust * steps.iter().zip(&p).map(|(s, p)| s[c] * p).sum::<f64>() / pp;
                }
            }
        }
    }
    Some(axis)
}

/// The grey of one tonal step: the cluster of near-neutral pixels around
/// the current axis, found by shifting a shrinking window from the axis to
/// the densest colour nearby. A coloured surface that is lit across many
/// tones (skin, sand, wood) can outnumber the greys in every step and pull
/// a plain median onto itself; the window stays on the greys as long as
/// they form a cluster of their own. Median densities of the cluster, or
/// `None` when no cluster of `min_count / 2` pixels sits on the axis or at
/// the step's median colour (the step then takes its plain median).
fn neutral_cluster(px: &[([f32; 3], [f64; 2])], min_count: usize) -> Option<[f64; 3]> {
    let inside = |centre: [f64; 2], h: f64| {
        px.iter()
            .filter(move |(_, o)| (o[0] - centre[0]).abs() < h && (o[1] - centre[1]).abs() < h)
    };
    let shift = |mut centre: [f64; 2], hs: &[f64]| {
        for &h in hs {
            let (sum, n) = inside(centre, h).fold(([0.0; 2], 0usize), |(s, n), (_, o)| {
                ([s[0] + o[0], s[1] + o[1]], n + 1)
            });
            if n == 0 {
                break;
            }
            centre = sum.map(|v| v / n as f64);
        }
        centre
    };
    let enough = |n: usize| n >= (min_count / 2).max(8);
    // From the axis; when no greys sit near it, from the step's median.
    let mut centre = shift([0.0; 2], &[0.03, 0.03, 0.02, 0.02]);
    if !enough(inside(centre, 0.02).count()) {
        let m = [0, 1].map(|k| median(px.iter().map(|(_, o)| o[k]).collect()));
        centre = shift(m, &[0.05, 0.03, 0.02, 0.02]);
    }
    let cluster: Vec<[f32; 3]> = inside(centre, 0.02).map(|(p, _)| *p).collect();
    enough(cluster.len())
        .then(|| std::array::from_fn(|c| median(cluster.iter().map(|p| p[c] as f64).collect())))
}

/// Least-squares lines `D_c = a + b · D_green` through tonal steps.
fn line_fit(steps: &[[f64; 3]]) -> [(f64, f64); 3] {
    let n = steps.len() as f64;
    std::array::from_fn(|c| {
        let mx = steps.iter().map(|s| s[1]).sum::<f64>() / n;
        let my = steps.iter().map(|s| s[c]).sum::<f64>() / n;
        let sxx: f64 = steps.iter().map(|s| (s[1] - mx).powi(2)).sum();
        let sxy: f64 = steps.iter().map(|s| (s[1] - mx) * (s[c] - my)).sum();
        let b = if sxx > 1e-9 { sxy / sxx } else { 1.0 };
        (my - b * mx, b)
    })
}

/// Normalised density (0 prints black … 1 prints white).
fn normalised(l: &Levels, t: ImageType, c: usize, d: f64) -> f64 {
    let x = l[c].normalise(d);
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
        bow: 0.0,
    }; 3]
}

/// Full conversion analysis of a prepared negative. Edit Only frames skip it.
pub fn analyze(neg: &Negative, s: &Settings, base: [f32; 3], ev: Option<f64>) -> Analysis {
    analyze_skin(neg, s, base, ev, true)
}

/// [`analyze`], optionally without keying the midtone on skin.
fn analyze_skin(
    neg: &Negative,
    s: &Settings,
    base: [f32; 3],
    ev: Option<f64>,
    use_skin: bool,
) -> Analysis {
    let (mut levels, auto_wb, axis, skin) = if s.image_type == ImageType::EditOnly {
        (edit_only_levels(), AutoWb::default(), None, None)
    } else {
        let buf = analysis_buffer(neg, s);
        let (levels, axis) = levels_from_buffer(&buf, s.image_type);
        let wb = auto_wb(&buf, &levels, axis.as_ref(), s.image_type);
        let skin = (use_skin && s.image_type == ImageType::Negative)
            .then(|| {
                let f = (SKIN_RANGE / (levels[1].hi - levels[1].lo).max(1e-3)).max(1.0);
                let wide = levels.map(|c| ChannelLevels {
                    hi: c.lo + (c.hi - c.lo) * f,
                    ..c
                });
                // Back to the frame's own levels.
                skin::find(&buf, &wide).map(|sk| SkinTone {
                    tone: levels[1].normalise(wide[1].density(sk.tone)),
                    ..sk
                })
            })
            .flatten();
        (levels, wb, axis, skin)
    };
    if let Some(sk) = &skin {
        levels = skin::keyed(&levels, sk);
    }
    Analysis {
        levels,
        auto_wb,
        base,
        key: s.analysis_key(),
        ev,
        axis,
        skin,
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

/// A roll's colour balance: every channel's density on the roll's neutral
/// axis, `D_c = offset_c + slope_c · D_green + bend_c · (D_green − pivot)²`.
/// The slopes and bends are the film's (its layers' H&D gamma ratios and
/// crossover), the offsets the roll's filtration.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RollBalance {
    pub slope: [f64; 3],
    pub offset: [f64; 3],
    #[serde(default)]
    pub bend: [f64; 3],
    /// Green density the bends are centred on.
    #[serde(default)]
    pub pivot: f64,
    /// How far from the pivot (D of green) the bends were measured; they
    /// level off beyond. `None` follows them everywhere.
    #[serde(default)]
    pub reach: Option<f64>,
}

impl RollBalance {
    /// The bend's departure at green `g`; it levels off beyond its reach
    /// rather than growing without the frames to show it.
    fn bent(&self, c: usize, g: f64) -> f64 {
        let r = self.reach.unwrap_or(f64::INFINITY);
        self.bend[c] * (g - self.pivot).clamp(-r, r).powi(2)
    }

    fn at(&self, c: usize, g: f64) -> f64 {
        self.offset[c] + self.slope[c] * g + self.bent(c, g)
    }

    /// Offsets that put the axis through the grey `point`.
    fn through(&self, point: [f64; 3]) -> [f64; 3] {
        std::array::from_fn(|c| point[c] - self.slope[c] * point[1] - self.bent(c, point[1]))
    }

    /// Levels for green levels `g`, red and blue on the roll's axis.
    fn levels(&self, g: ChannelLevels) -> Levels {
        std::array::from_fn(|c| {
            let (lo, hi) = (
                self.at(c, g.lo),
                self.at(c, g.hi).max(self.at(c, g.lo) + 0.05),
            );
            ChannelLevels {
                lo,
                hi,
                mid: self.at(c, g.mid),
                bow: bow_for(self.bend[c], g.hi - g.lo, hi - lo),
            }
        })
    }

    /// The same balance for a scan `d` denser in every channel.
    fn shifted(&self, d: f64) -> Self {
        Self {
            offset: std::array::from_fn(|c| self.offset[c] + d * (1.0 - self.slope[c])),
            pivot: self.pivot + d,
            ..*self
        }
    }
}

/// A roll built by "Convert as roll".
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RollAggregate {
    /// The roll's levels at one exposure for the whole roll.
    pub levels: Levels,
    pub balance: RollBalance,
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
        ..c
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

/// A frame's neutral axis as (slopes at a grey on it, that grey, bends):
/// the fitted one, or the line through its channel ranges when it had too
/// little near-neutral content. Bends only from enough tonal steps.
fn frame_axis(a: &Analysis) -> ([f64; 3], [f64; 3], Option<[f64; 3]>) {
    match &a.axis {
        Some(x) => {
            let bend = (x.steps >= BEND_STEPS).then_some(x.bend);
            let k = bend.unwrap_or([0.0; 3]);
            // p'(m) = −u at the steps' mean green.
            let slope = std::array::from_fn(|c| x.fit[c].1 - k[c] * x.basis[1]);
            (slope, x.at(x.centre[1]), bend)
        }
        None => {
            let l = &a.levels;
            let span_g = (l[1].hi - l[1].lo).max(1e-3);
            (
                std::array::from_fn(|c| (l[c].hi - l[c].lo) / span_g),
                std::array::from_fn(|c| l[c].mid),
                None,
            )
        }
    }
}

/// Roll aggregation, the way a lab prints a roll: one colour balance for
/// all frames. The channel slopes and crossover belong to the film, so the
/// roll takes the median of the frames' fitted slopes and bends; a single
/// frame's fit also follows its light (open shade turns its shadows blue). The roll's filtration is
/// the median of the frames' axis offsets under those slopes, leaving out
/// frames whose balance is far from the rest (robust outlier test, at
/// least 0.05 D off). Slopes, bends and filtration come only from frames
/// with a neutral axis of their own (all frames when fewer than two have
/// one): a frame without one has only its colour-biased channel ranges to
/// offer. The roll's one-exposure levels take the median green range. With
/// a `common_ev`, frames are first moved onto that exposure. Returns the
/// roll and the excluded frame indices.
pub fn aggregate_roll(frames: &[Analysis], common_ev: Option<f64>) -> (RollAggregate, Vec<usize>) {
    let t = frames[0].key.image_type;
    let shift = |a: &Analysis| match (common_ev, a.ev) {
        (Some(c), Some(e)) => -ev_density(e - c),
        _ => 0.0,
    };
    let axes: Vec<_> = frames
        .iter()
        .map(|a| {
            let (slope, grey, bend) = frame_axis(a);
            (slope, grey.map(|v| v + shift(a)), bend)
        })
        .collect();
    let mut reliable: Vec<usize> = (0..frames.len())
        .filter(|&i| frames[i].auto_wb.confidence >= LOW_CONFIDENCE)
        .collect();
    if reliable.len() < 2 {
        reliable = (0..frames.len()).collect();
    }
    let bends: Vec<[f64; 3]> = reliable.iter().filter_map(|&i| axes[i].2).collect();
    let bend: [f64; 3] = std::array::from_fn(|c| median(bends.iter().map(|b| b[c]).collect()));
    let pivot = median(frames.iter().map(|a| a.levels[1].mid + shift(a)).collect());
    // The bends were measured over the frames' green ranges: half the
    // typical range either side of the pivot.
    let reach = median(
        frames
            .iter()
            .map(|a| (a.levels[1].hi - a.levels[1].lo) / 2.0)
            .collect(),
    );
    // Each frame's slope at its own grey, carried along the roll's bend to
    // the pivot.
    let shape = RollBalance {
        slope: std::array::from_fn(|c| {
            median(
                reliable
                    .iter()
                    .map(|&i| {
                        let (s, g, _) = &axes[i];
                        s[c] - 2.0 * bend[c] * (g[1] - pivot)
                    })
                    .collect(),
            )
        }),
        offset: [0.0; 3],
        bend,
        pivot,
        reach: Some(reach),
    };
    let offsets: Vec<[f64; 3]> = axes.iter().map(|a| shape.through(a.1)).collect();
    let mut excluded: Vec<usize> = Vec::new();
    if reliable.len() >= 3 {
        for c in [0, 2] {
            let v: Vec<f64> = reliable.iter().map(|&i| offsets[i][c]).collect();
            excluded.extend(outliers(&v, 0.05).into_iter().map(|k| reliable[k]));
        }
        excluded.sort_unstable();
        excluded.dedup();
    }
    let kept: Vec<usize> = (0..frames.len())
        .filter(|i| !excluded.contains(i))
        .collect();
    let balanced: Vec<usize> = reliable
        .iter()
        .copied()
        .filter(|i| !excluded.contains(i))
        .collect();
    let balance = RollBalance {
        offset: std::array::from_fn(|c| {
            median(balanced.iter().map(|&i| offsets[i][c]).collect())
        }),
        ..shape
    };
    let green = |f: fn(&ChannelLevels) -> f64| {
        median(
            kept.iter()
                .map(|&i| f(&frames[i].levels[1]) + shift(&frames[i]))
                .collect(),
        )
    };
    let levels = balance.levels(ChannelLevels {
        lo: green(|c| c.lo),
        hi: green(|c| c.hi),
        mid: green(|c| c.mid),
        bow: 0.0,
    });
    let mean: [f64; 3] = std::array::from_fn(|c| {
        kept.iter()
            .map(|&i| frames[i].auto_wb.mean[c] + shift(&frames[i]))
            .sum::<f64>()
            / kept.len() as f64
    });
    let grey = balance.levels(levels[1]).map(|c| c.mid);
    let confidence = kept
        .iter()
        .map(|&i| frames[i].auto_wb.confidence)
        .fold(0.0, f32::max);
    let auto_wb = auto_wb_from(&levels, t, grey, mean, confidence);
    (
        RollAggregate {
            levels,
            balance,
            auto_wb,
            common_ev,
        },
        excluded,
    )
}

/// Below this auto WB confidence a frame's own colour balance is not
/// trusted; from [`HIGH_CONFIDENCE`] it is trusted fully, and in between
/// the frame's balance is blended with the roll's.
pub const LOW_CONFIDENCE: f32 = 0.4;
const HIGH_CONFIDENCE: f32 = 0.6;

/// How far Lab mode trusts a frame's own colour balance (0..1).
fn lab_trust(f: &Analysis) -> f64 {
    ((f.auto_wb.confidence - LOW_CONFIDENCE) / (HIGH_CONFIDENCE - LOW_CONFIDENCE)).clamp(0.0, 1.0)
        as f64
}

/// The roll's balance moved onto this frame's scan exposure.
fn roll_balance_for(frame: Option<&Analysis>, roll: &RollData) -> (RollBalance, f64) {
    let d = match (roll.common_ev, frame.and_then(|a| a.ev)) {
        (Some(c), Some(e)) => ev_density(e - c),
        _ => 0.0,
    };
    (roll.balance.shifted(d), d)
}

/// Pick the levels a frame renders with: its own analysis, or the roll's.
/// In Lab mode a frame with a reliable neutral axis prints exactly as it
/// would alone (the light changes from frame to frame: daylight, shade,
/// tungsten); one with too few neutral tones to judge takes the film's
/// slopes and the roll's balance at its own density range, and in between
/// the two are blended. Darkroom mode prints every frame with the roll's
/// one balance and one exposure.
pub fn levels_for(
    s: &Settings,
    frame: Option<&Analysis>,
    roll: Option<&RollData>,
) -> Option<Levels> {
    let Some(roll) = roll.filter(|_| s.use_roll) else {
        return frame.map(|a| a.levels);
    };
    let (balance, d) = roll_balance_for(frame, roll);
    let Some(f) = frame.filter(|_| s.roll_mode == RollMode::Lab) else {
        return Some(balance.levels(shift_levels(&roll.levels, d)[1]));
    };
    let trust = lab_trust(f);
    let from_roll = balance.levels(f.levels[1]);
    let mix = |a: f64, b: f64| a + trust * (b - a);
    Some(std::array::from_fn(|c| {
        let (r, o) = (from_roll[c], f.levels[c]);
        ChannelLevels {
            lo: mix(r.lo, o.lo),
            hi: mix(r.hi, o.hi),
            mid: mix(r.mid, o.mid),
            bow: mix(r.bow, o.bow),
        }
    }))
}

/// The auto WB candidates for the levels [`levels_for`] gives: the frame's
/// own, the roll's in Darkroom mode, and in Lab mode the frame's grey
/// blended with the roll's as far as the frame is trusted.
pub fn auto_wb_for(
    s: &Settings,
    frame: Option<&Analysis>,
    roll: Option<&RollData>,
) -> Option<AutoWb> {
    let Some(roll) = roll.filter(|_| s.use_roll) else {
        return frame.map(|a| a.auto_wb);
    };
    let Some(f) = frame.filter(|_| s.roll_mode == RollMode::Lab) else {
        return roll.auto_wb.or_else(|| frame.map(|a| a.auto_wb));
    };
    let trust = lab_trust(f);
    if trust >= 1.0 {
        return Some(f.auto_wb);
    }
    let l = levels_for(s, Some(f), Some(roll))?;
    let (balance, _) = roll_balance_for(frame, roll);
    let roll_grey = balance.levels(f.levels[1]).map(|c| c.mid);
    let grey = std::array::from_fn(|c| roll_grey[c] + trust * (f.auto_wb.grey[c] - roll_grey[c]));
    Some(auto_wb_from(
        &l,
        f.key.image_type,
        grey,
        f.auto_wb.mean,
        f.auto_wb.confidence,
    ))
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
    dodge: Option<Dodge>,
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
            dodge: levels.and_then(|l| Dodge::new(s, l)),
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

    /// What a GPU renderer needs to draw the same image: the per-channel
    /// curve tables (also for the unconverted view), the look, and the
    /// linear MelissaRGB → sRGB matrix of a display preview.
    pub fn gpu_parts(&self) -> (&CurveLut, Option<&Look>, [[f32; 3]; 3]) {
        (&self.lut, self.converted.then_some(&self.look), self.out_m)
    }

    /// The dodge applied before the curves (see [`dodge`]), if any.
    pub fn dodge(&self) -> Option<&Dodge> {
        self.dodge.as_ref()
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
        let mut out: Vec<[f32; 3]> = match &self.dodge {
            Some(dg) => neg
                .data
                .par_iter()
                .zip(dodge::low_pass(neg))
                .map(|(p, low)| {
                    let o = dg.offset(low);
                    let d = std::array::from_fn(|c| p[c] + o * dg.gain[c]);
                    self.output(self.positive(d))
                })
                .collect(),
            None => neg
                .data
                .par_iter()
                .map(|p| self.output(self.positive(*p)))
                .collect(),
        };
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
    fn detect_source_from_the_file() {
        use super::super::settings::Source;
        let mut src = super::super::testchart::negative(0.0).src;
        assert_eq!(detect_source(&src), Source::TifScan);
        src.is_raw = true;
        assert_eq!(detect_source(&src), Source::DigitalCamera);
        src.info.software = "VueScan 9.8.45".into();
        assert_eq!(detect_source(&src), Source::VueScanRaw);
        src.info.software.clear();
        src.info.linear_raw = true;
        assert_eq!(detect_source(&src), Source::VueScanRaw);
    }

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

    fn frame(lo: [f64; 3], hi: [f64; 3]) -> Analysis {
        Analysis {
            levels: std::array::from_fn(|c| ChannelLevels {
                lo: lo[c],
                hi: hi[c],
                mid: (lo[c] + hi[c]) / 2.0,
                bow: 0.0,
            }),
            auto_wb: AutoWb::default(),
            base: [1.0; 3],
            key: Settings::default().analysis_key(),
            ev: None,
            axis: None,
            skin: None,
        }
    }

    fn roll_of(agg: &RollAggregate, frames: usize) -> RollData {
        RollData {
            id: "r".into(),
            levels: agg.levels,
            balance: agg.balance,
            base: [1.0; 3],
            frames,
            excluded: vec![],
            auto_wb: Some(agg.auto_wb),
            common_ev: agg.common_ev,
        }
    }

    #[test]
    fn roll_takes_the_film_slopes_and_drops_outliers() {
        // Six frames of one film at different exposures, and one whose
        // blue is far off (different light, or a different film).
        // Red = 0.9 · green − 0.035, blue = 1.1 · green + 0.035.
        let on_axis = |lo_g: f64, hi_g: f64, blue: f64| {
            frame(
                [0.9 * lo_g - 0.035, lo_g, 1.1 * lo_g + blue],
                [0.9 * hi_g - 0.035, hi_g, 1.1 * hi_g + blue],
            )
        };
        let mut frames: Vec<Analysis> = (0..6)
            .map(|i| {
                let k = i as f64 * 0.05;
                on_axis(0.15 + k, 1.1 + k, 0.035)
            })
            .collect();
        frames.push(on_axis(0.15, 1.1, 0.285));
        let (r, excluded) = aggregate_roll(&frames, None);
        assert_eq!(excluded, vec![6]);
        // Slopes: red 0.9, blue 1.1 of green's span; the median ignores the
        // outlier.
        assert!((r.balance.slope[0] - 0.9).abs() < 1e-6, "{:?}", r.balance);
        assert!((r.balance.slope[2] - 1.1).abs() < 1e-6, "{:?}", r.balance);
        // The roll's axis passes through every regular frame's levels.
        let l = r.balance.levels(frames[3].levels[1]);
        for c in 0..3 {
            assert!((l[c].lo - frames[3].levels[c].lo).abs() < 1e-6, "{l:?}");
        }
    }

    /// Frames without a neutral axis of their own (a sunset's channel
    /// ranges) do not move the film's slopes or the roll's filtration.
    #[test]
    fn roll_balance_ignores_frames_without_an_axis() {
        let on_axis = |lo_g: f64, hi_g: f64| {
            let mut f = frame(
                [0.9 * lo_g - 0.035, lo_g, 1.1 * lo_g + 0.035],
                [0.9 * hi_g - 0.035, hi_g, 1.1 * hi_g + 0.035],
            );
            f.auto_wb.confidence = 0.8;
            f
        };
        let mut frames: Vec<Analysis> = (0..3)
            .map(|i| on_axis(0.15 + i as f64 * 0.05, 1.1 + i as f64 * 0.05))
            .collect();
        for _ in 0..3 {
            let mut f = frame([0.05, 0.2, 0.4], [1.3, 1.0, 0.9]);
            f.auto_wb.confidence = 0.2;
            frames.push(f);
        }
        let (r, excluded) = aggregate_roll(&frames, None);
        assert!(excluded.is_empty(), "{excluded:?}");
        assert!((r.balance.slope[0] - 0.9).abs() < 1e-6, "{:?}", r.balance);
        assert!((r.balance.slope[2] - 1.1).abs() < 1e-6, "{:?}", r.balance);
        assert!((r.balance.offset[0] + 0.035).abs() < 1e-6, "{:?}", r.balance);
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
        s.roll_mode = RollMode::Darkroom;
        assert_eq!(get(&s), agg.levels);
        s.roll_mode = RollMode::Lab;
        let lab = get(&s);
        // Green is the frame's own range, red and blue on the roll's axis.
        assert_eq!(lab[1].lo, frames[1].levels[1].lo);
        assert_eq!(lab[1].hi, frames[1].levels[1].hi);
        // Too few neutral tones (confidence 0): the roll's balance.
        assert_eq!(lab, agg.balance.levels(frames[1].levels[1]));
        // A frame with a reliable neutral axis prints as it would alone.
        let mut own = frames[1].clone();
        own.auto_wb.confidence = 1.0;
        assert_eq!(levels_for(&s, Some(&own), Some(&roll)).unwrap(), own.levels);
        assert_eq!(auto_wb_for(&s, Some(&own), Some(&roll)).unwrap(), own.auto_wb);
        // Half trusted: halfway between the roll's balance and its own.
        own.auto_wb.confidence = (LOW_CONFIDENCE + HIGH_CONFIDENCE) / 2.0;
        let half = levels_for(&s, Some(&own), Some(&roll)).unwrap();
        for c in 0..3 {
            let want = (lab[c].lo + own.levels[c].lo) / 2.0;
            assert!((half[c].lo - want).abs() < 1e-6, "{half:?}");
        }
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
        let (agg, excluded) = aggregate_roll(&frames, Some(c));
        assert!(excluded.is_empty());
        assert!((agg.levels[0].lo - 0.2).abs() < 1e-6, "{:?}", agg.levels);
        let roll = roll_of(&agg, 3);
        let s = Settings {
            use_roll: true,
            roll_mode: RollMode::Darkroom,
            ..Settings::default()
        };
        // Rendering that frame moves the roll back onto its own exposure.
        let l = levels_for(&s, Some(&frames[2]), Some(&roll)).unwrap();
        for c in 0..3 {
            assert!((l[c].hi - frames[2].levels[c].hi).abs() < 1e-6, "{l:?}");
        }
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

    /// A smooth density ramp renders to a smooth 16-bit gradient.
    #[test]
    fn gradients_stay_smooth_in_16_bit() {
        let l = [ChannelLevels {
            lo: 0.1,
            hi: 1.2,
            mid: 0.6,
            bow: 0.0,
        }; 3];
        let n = 65536;
        let neg = Negative {
            width: n,
            height: 1,
            data: (0..n)
                .map(|i| [(0.1 + 1.1 * i as f64 / (n - 1) as f64) as f32; 3])
                .collect(),
        };
        let s = chart_settings();
        for space in [
            OutputSpace::Srgb,
            OutputSpace::AdobeRgb,
            OutputSpace::ProPhoto,
        ] {
            let px = Renderer::new(&s, Some(&l), space).render(&neg, 0.0);
            let q: Vec<u16> = px.iter().map(|p| (p[1] * 65535.0).round() as u16).collect();
            let levels = q.windows(2).filter(|w| w[0] != w[1]).count();
            let jump = q.windows(2).map(|w| w[1].abs_diff(w[0])).max().unwrap();
            assert!(levels > 30000, "{space:?}: only {levels} distinct steps");
            assert!(jump < 40, "{space:?}: jump of {jump} / 65535");
        }
    }

    /// The same negative scanned to a linear and to a gamma-encoded TIFF
    /// gives the same densities once the source says which it is.
    #[test]
    fn linear_scans_read_as_linear() {
        use crate::engine::settings::Source;
        let chart = crate::engine::testchart::negative(0.0);
        let dir = std::env::temp_dir().join(format!("nlp_linear_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let write = |name: &str, f: fn(f32) -> f32| {
            let p = dir.join(name);
            let px: Vec<[f32; 3]> = chart
                .src
                .data
                .iter()
                .map(|v| color::mul3(&chart.src.to_melissa, *v).map(f))
                .collect();
            let icc = color::icc_for(OutputSpace::Srgb);
            crate::engine::export::write_tiff(
                &p,
                chart.src.width,
                chart.src.height,
                &px,
                true,
                false,
                &icc,
                &Default::default(),
                &Default::default(),
            )
            .unwrap();
            decode::load(&p, None).unwrap()
        };
        let encoded = write("encoded.tif", color::srgb_encode);
        let linear = write("linear.tif", |v| v);
        let _ = std::fs::remove_dir_all(&dir);
        let density = |src: &SourceImage, source: Source| {
            let s = Settings {
                source,
                ..Settings::default()
            };
            prepare_negative(src, &s, resolve_base(src, &s, None))
        };
        let a = density(&encoded, Source::TifScan);
        let b = density(&linear, Source::LinearScan);
        let worst = a
            .data
            .iter()
            .zip(&b.data)
            .map(|(x, y)| (0..3).map(|c| (x[c] - y[c]).abs()).fold(0.0f32, f32::max))
            .fold(0.0f32, f32::max);
        assert!(worst < 0.01, "density differs by {worst}");
        // Read as sRGB, the linear file's densities come out about 2.2× too large.
        let wrong = density(&linear, Source::TifScan);
        let p = chart.patches[21].pixels[0];
        assert!(wrong.data[p][1] > a.data[p][1] * 1.5);
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
        let wb = auto_wb_for(&s, Some(a), roll).unwrap();
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

    /// With no border buffer the analysis still finds the image inside the
    /// film rebate instead of treating unexposed film as the shadows.
    #[test]
    fn rebate_is_kept_out_of_the_analysis() {
        use crate::engine::testchart::{measure, negative};
        let chart = negative(0.0);
        let s = Settings {
            border_buffer: 0.0,
            ..chart_settings()
        };
        let a = analyse_chart(&chart, &s);
        assert!(a.levels.iter().all(|c| c.lo > 0.15), "{:?}", a.levels);
        let m = measure(&chart, &convert_chart(&chart, &s, &a, None));
        // With the rebate in the analysis: neutral C* 3.21, full dE00 10.17.
        assert!(m.neutral_cast < 0.5, "neutral C* {}", m.neutral_cast);
    }

    /// Neither a scanner holder that reads only ~2.2 D nor a lamp in the
    /// frame may set the white point and print the scene dark.
    #[test]
    fn holder_and_light_sources_keep_the_scene_bright() {
        use crate::engine::testchart::{negative, patch_lab, with_holder, with_light};
        // No border buffer: real scans' holders are wider than the chart's.
        let s = Settings {
            border_buffer: 0.0,
            ..chart_settings()
        };
        let grey_l = |chart: &crate::engine::testchart::Chart| {
            let a = analyse_chart(chart, &s);
            patch_lab(chart, &convert_chart(chart, &s, &a, None))[21][0]
        };
        let normal = grey_l(&negative(0.0));
        let mut holder = negative(0.0);
        with_holder(&mut holder, 2.2);
        let mut lamp = negative(0.0);
        let free = with_light(&mut lamp, 0.0);
        assert!(free.contains(&21));
        let (h, l) = (grey_l(&holder), grey_l(&lamp));
        eprintln!("mid grey L*: normal {normal:.1}, holder {h:.1}, lamp {l:.1}");
        assert!((h - normal).abs() < 3.0, "holder: L* {h} vs {normal}");
        assert!((l - normal).abs() < 6.0, "lamp: L* {l} vs {normal}");
    }

    /// A flat-field frame divides out uneven illumination.
    #[test]
    fn flat_field_removes_uneven_light() {
        use crate::engine::testchart::{flat_frame, measure, negative, vignette};
        let dir = std::env::temp_dir().join(format!("nlp_flat_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let flat_path = dir.join("flat.tif");
        let flat = flat_frame(0.35);
        let px: Vec<[f32; 3]> = flat
            .data
            .iter()
            .map(|p| p.map(color::srgb_encode))
            .collect();
        crate::engine::export::write_tiff(
            &flat_path,
            flat.width,
            flat.height,
            &px,
            true,
            false,
            &color::icc_for(OutputSpace::Srgb),
            &Default::default(),
            &Default::default(),
        )
        .unwrap();
        let mut chart = negative(0.0);
        vignette(&mut chart, 0.35);
        let without = chart_settings();
        let with = Settings {
            flat_field: Some(flat_path.clone()),
            ..chart_settings()
        };
        let run = |s: &Settings| {
            let a = analyse_chart(&chart, s);
            measure(&chart, &convert_chart(&chart, s, &a, None))
        };
        let (m0, m1) = (run(&without), run(&with));
        let _ = std::fs::remove_dir_all(&dir);
        eprintln!(
            "uneven light: tone-matched {:.2} -> {:.2}, neutral C* {:.2} -> {:.2} (max {:.2} -> {:.2})",
            m0.tone_matched_de,
            m1.tone_matched_de,
            m0.neutral_cast,
            m1.neutral_cast,
            m0.neutral_cast_max,
            m1.neutral_cast_max
        );
        // Even light: neutral C* 0.31 (max 0.94).
        assert!(
            m0.neutral_cast_max > 1.5,
            "the falloff shows without correction"
        );
        assert!(m1.neutral_cast < 0.4, "neutral C* {}", m1.neutral_cast);
    }

    /// Auto-Neutral on scenes dominated by one colour (foliage, sunset sky,
    /// skin-toned interior), also when the colour is lit across many tones
    /// and so reaches every tonal step (skin, sand, wood, a red rug, grass):
    /// the grey ramp and neutral patches must stay grey.
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
        let mut shaded = 0.0f64;
        for (name, field) in [
            ("skin", [62.0, 13.0, 17.0]),
            ("sand", [70.0, 3.0, 18.0]),
            ("wood", [48.0, 10.0, 24.0]),
            ("rug", [40.0, 45.0, 25.0]),
            ("grass", [50.0, -20.0, 25.0]),
        ] {
            let chart = crate::engine::testchart::dominated_shaded(0.0, field, 30.0);
            let a = analyse_chart(&chart, &s);
            let (mean, max) = neutral_cast(&chart, &convert_chart(&chart, &s, &a, None));
            eprintln!("shaded {name}: neutral C* {mean:.2} (max {max:.2})");
            shaded = shaded.max(mean);
        }
        eprintln!("shaded worst neutral C* {shaded:.2}");
        // Median per tonal step: skin 7.70, sand 8.21, wood 3.21, rug 0.58,
        // grass 3.68.
        assert!(shaded < 0.6, "shaded scene neutral C* {shaded}");
        // Plugin-port engine: foliage 1.01, sunset 1.67, interior 8.78,
        // sky 15.40.
        assert!(worst < 0.5, "dominated scene neutral C* {worst}");
    }

    /// Conversion quality on the physically modelled chart: colour accuracy,
    /// neutrality, and consistency of a roll shot at −1, 0 and +1 EV.
    /// On a film whose layers cross over (soft blue toe, early red
    /// shoulder) a straight neutral axis leaves the shadows and highlights
    /// tinted; the bent one keeps the whole grey scale neutral, in a single
    /// frame and across a roll.
    #[test]
    fn crossover_film_stays_neutral() {
        use crate::engine::testchart::{CROSSOVER, measure, negative_on};
        let s = chart_settings();
        let straight = |a: &Analysis| Analysis {
            levels: a.levels.map(|c| ChannelLevels { bow: 0.0, ..c }),
            ..a.clone()
        };
        let charts: Vec<_> = [-1.0, 0.0, 1.0]
            .into_iter()
            .map(|ev| negative_on(&CROSSOVER, ev))
            .collect();
        let analyses: Vec<_> = charts.iter().map(|c| analyse_chart(c, &s)).collect();
        let cast = |c, a: &Analysis, roll| measure(c, &convert_chart(c, &s, a, roll)).neutral_cast;
        let bent = charts.iter().zip(&analyses).map(|(c, a)| cast(c, a, None));
        let flat = charts
            .iter()
            .zip(&analyses)
            .map(|(c, a)| cast(c, &straight(a), None));
        let (bent, flat) = (bent.fold(0.0, f64::max), flat.fold(0.0, f64::max));
        let (agg, _) = aggregate_roll(&analyses, None);
        let roll = RollData {
            base: analyses[1].base,
            ..roll_of(&agg, 3)
        };
        let rolled = charts
            .iter()
            .zip(&analyses)
            .map(|(c, a)| cast(c, a, Some(&roll)))
            .fold(0.0, f64::max);
        eprintln!("crossover: neutral C* straight {flat:.2}, bent {bent:.2}, roll {rolled:.2}");
        assert!(bent < 0.3 * flat, "bent {bent} vs straight {flat}");
        assert!(rolled < 0.25 * flat, "roll {rolled} vs straight {flat}");
    }

    /// Backlit against a bright sky or flashed at night, a print exposed
    /// for the frame's median gets the face wrong; keyed on the skin it
    /// lands near where a plain background prints it.
    #[test]
    fn faces_print_at_their_density() {
        use crate::engine::testchart::{patch_lab, portrait};
        let s = chart_settings();
        let face = [62.0, 14.0, 17.0];
        let face_l = |back: [f64; 3], skin: bool| {
            let chart = portrait(0.0, back, face);
            let base = resolve_base(&chart.src, &s, None);
            let neg = prepare_negative(&chart.src, &s, base);
            let a = analyze_skin(&neg, &s, base, None, skin);
            (
                patch_lab(&chart, &convert_chart(&chart, &s, &a, None))[0][0],
                a.skin,
            )
        };
        let (reference, _) = face_l([50.0, 0.0, 0.0], false);
        let (keyed_ref, found) = face_l([50.0, 0.0, 0.0], true);
        eprintln!("face L* on grey {reference:.1}, keyed {keyed_ref:.1}, skin {found:?}");
        assert!(found.is_some(), "the face is found");
        assert!(
            (keyed_ref - reference).abs() < 4.0,
            "plain background moves"
        );
        for (name, back) in [
            ("backlit", [92.0, -2.0, -10.0]),
            ("night", [10.0, 0.0, 0.0]),
        ] {
            let (median, _) = face_l(back, false);
            let (keyed, skin) = face_l(back, true);
            eprintln!("{name}: face L* median {median:.1}, keyed {keyed:.1}, skin {skin:?}");
            assert!(
                (keyed - reference).abs() < 0.6 * (median - reference).abs(),
                "{name}: keyed {keyed} median {median} reference {reference}"
            );
        }
    }

    /// Hyper Tone lifts a dark subject against a bright sky and keeps the
    /// shading inside it; a scene whose large areas sit near its key is
    /// left as it was.
    #[test]
    fn hyper_tone_dodges_large_areas_only() {
        use crate::engine::testchart::{patch_lab, portrait};
        let s = chart_settings();
        let dodged = Settings {
            hyper_tone_highlights: 100.0,
            hyper_tone_shadows: 100.0,
            ..chart_settings()
        };
        let render = |chart: &crate::engine::testchart::Chart, s: &Settings| {
            let base = resolve_base(&chart.src, s, None);
            let neg = prepare_negative(&chart.src, s, base);
            let a = analyze_skin(&neg, s, base, None, false);
            convert_chart(chart, s, &a, None)
        };
        // L* across the face (shaded 6 L*): right edge minus left edge.
        let shading = |chart: &crate::engine::testchart::Chart, px: &[[f32; 3]]| {
            let w = crate::engine::testchart::WIDTH;
            let face = &chart.patches[0].pixels;
            let (x0, x1) = face
                .iter()
                .fold((usize::MAX, 0), |(a, b), &i| (a.min(i % w), b.max(i % w)));
            let side = |x: usize| {
                let v: Vec<[f32; 3]> = face
                    .iter()
                    .filter(|&&i| (i % w).abs_diff(x) < 3)
                    .map(|&i| px[i])
                    .collect();
                let n = v.len() as f32;
                let m: [f32; 3] = std::array::from_fn(|c| v.iter().map(|p| p[c]).sum::<f32>() / n);
                color::xyz_to_lab(
                    color::mat_vec(
                        &color::rgb_to_xyz(&color::SRGB),
                        m.map(|v| color::srgb_decode(v) as f64),
                    ),
                    color::D65,
                )[0]
            };
            side(x1) - side(x0)
        };
        let face_l =
            |chart: &crate::engine::testchart::Chart, px: &[[f32; 3]]| patch_lab(chart, px)[0][0];
        let chart = portrait(0.0, [92.0, -2.0, -10.0], [40.0, 12.0, 15.0]);
        let (before, after) = (render(&chart, &s), render(&chart, &dodged));
        let (l0, l1) = (face_l(&chart, &before), face_l(&chart, &after));
        // The face printed undodged with the exposure that makes it as light.
        let (mut lo, mut hi) = (0.0f32, 5.0f32);
        for _ in 0..20 {
            let e = (lo + hi) / 2.0;
            let lit = Settings {
                print_exposure: e,
                ..chart_settings()
            };
            if face_l(&chart, &render(&chart, &lit)) < l1 {
                lo = e
            } else {
                hi = e
            }
        }
        let lit = render(
            &chart,
            &Settings {
                print_exposure: lo,
                ..chart_settings()
            },
        );
        let (k0, k1) = (shading(&chart, &lit), shading(&chart, &after));
        eprintln!(
            "hyper tone: dark face L* {l0:.1} -> {l1:.1}; shading {k1:.2} L*, printed lighter {k0:.2} L*"
        );
        assert!(l1 > l0 + 4.0, "the face is dodged");
        assert!(k1 > 0.8 * k0, "the face keeps its shading");
        let chart = portrait(0.0, [50.0, 0.0, 0.0], [55.0, 12.0, 15.0]);
        let (l0, l1) = (
            patch_lab(&chart, &render(&chart, &s))[0][0],
            patch_lab(&chart, &render(&chart, &dodged))[0][0],
        );
        eprintln!("hyper tone: face on grey L* {l0:.1} -> {l1:.1}");
        assert!(
            (l1 - l0).abs() < 1.5,
            "a scene near its key moves {l0} -> {l1}"
        );
    }

    /// A flash at night: lit things far brighter than the dark scene keep
    /// their tones instead of printing as white blobs, even without skin
    /// to go by. Here the lit subject is the grey ramp (many tones) on a
    /// near-black field; most of its steps must stay below white.
    #[test]
    fn lit_subjects_at_night_keep_their_tones() {
        use crate::engine::testchart::{dominated, patch_lab};
        let s = chart_settings();
        let chart = dominated(0.0, [6.0, 0.0, 0.0]);
        let base = resolve_base(&chart.src, &s, None);
        let neg = prepare_negative(&chart.src, &s, base);
        let a = analyze_skin(&neg, &s, base, None, false);
        let lab = patch_lab(&chart, &convert_chart(&chart, &s, &a, None));
        let open = lab[..12].iter().filter(|l| l[0] < 97.0).count();
        eprintln!(
            "lit subject at night: {open} of 12 ramp steps below L* 97: {:?}",
            lab[..12].iter().map(|l| l[0].round()).collect::<Vec<_>>()
        );
        assert!(open >= 7, "the lit ramp blows out");
    }

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
        // Plugin-port engine: colour dE00 5.21, tone-matched 5.00, neutral
        // C* 3.22 (max 6.86), frame spread 2.78, roll spread 3.70, roll C*
        // 4.66. Since printing with the plugin's (brighter, steeper) tone
        // curves, colour dE00 against the chart's true colours is the gate;
        // tone-matched dE00 assumes the curves' extra chroma is wanted and is
        // only shown. The bounds only tighten from here.
        assert!(m.color_de < 5.1, "colour dE00 {}", m.color_de);
        assert!(
            m.tone_matched_de < 4.5,
            "tone-matched dE00 {}",
            m.tone_matched_de
        );
        assert!(m.neutral_cast < 0.4, "neutral C* {}", m.neutral_cast);
        assert!(
            m.neutral_cast_max < 1.0,
            "neutral C* max {}",
            m.neutral_cast_max
        );
        assert!(frame_spread < 0.65, "frame spread {frame_spread}");
        assert!(roll_spread < 0.65, "roll spread {roll_spread}");
        assert!(roll_cast < 0.5, "roll neutral C* {roll_cast}");
    }

    /// Lab mode never prints a frame that can judge its own balance worse
    /// than converting it alone, and frames that can't stay near neutral.
    #[test]
    fn roll_lab_matches_single_frames() {
        use crate::engine::testchart::{dominated, negative, neutral_cast};
        let s = chart_settings();
        let mut charts: Vec<_> = [-1.0, 0.0, 1.0].into_iter().map(negative).collect();
        charts.push(dominated(0.0, [62.0, 30.0, 55.0]));
        charts.push(dominated(0.0, [43.46, -32.0, 38.0]));
        let mut analyses: Vec<_> = charts.iter().map(|c| analyse_chart(c, &s)).collect();
        // The foliage frame as if it had too few neutral tones to judge.
        analyses[4].auto_wb.confidence = 0.2;
        let (agg, _) = aggregate_roll(&analyses, None);
        let roll = RollData {
            base: analyses[1].base,
            ..roll_of(&agg, charts.len())
        };
        for (i, (c, a)) in charts.iter().zip(&analyses).enumerate() {
            let single = convert_chart(c, &s, a, None);
            let rolled = convert_chart(c, &s, a, Some(&roll));
            let (cs, cr) = (neutral_cast(c, &single).0, neutral_cast(c, &rolled).0);
            eprintln!(
                "frame {i}: confidence {:.1}, neutral C* single {cs:.2}, roll {cr:.2}",
                a.auto_wb.confidence
            );
            if lab_trust(a) >= 1.0 {
                let diff = single
                    .iter()
                    .zip(&rolled)
                    .flat_map(|(p, q)| (0..3).map(move |k| (p[k] - q[k]).abs()))
                    .fold(0.0f32, f32::max);
                assert!(diff < 1e-4, "frame {i} differs by {diff}");
            } else {
                assert!(cr < 0.5, "frame {i}: roll {cr} vs single {cs}");
            }
        }
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
