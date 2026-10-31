// SPDX-License-Identifier: GPL-3.0-or-later
//! Multi-exposure SNR / IVW merge in linear film-negative scanner space.
//!
//! The arithmetic follows numpy's dtype promotion in the Python original:
//! short-pass terms stay float32, the long-pass variance is float64 (the
//! shift variance factor is a numpy float64 scalar), and the result is
//! truncated to uint16.

use ndarray::{Array3, ArrayView3, s};

use crate::error::{Error, Result};
use crate::num::{lstsq_line, median, median_f32, quantiles_sorted_f32, sort_f32, linspace};
use crate::pass_align::align_pass_to_reference;

const FULL_SCALE: f64 = 65535.0;
pub const SNR_FLOOR: f64 = 0.002 * FULL_SCALE;
pub const SNR_CLIP_START: f64 = 0.80 * FULL_SCALE;
pub const SNR_CLIP_END: f64 = 0.95 * FULL_SCALE;
pub const SNR_ALPHA: f64 = 1.0;
/// ~64 DN read noise.
pub const SNR_BETA: f64 = 4096.0;
const NOISE_FIT_BINS: usize = 24;
const NOISE_FIT_MIN_PER_BIN: usize = 200;
const NOISE_FIT_MIN_BINS: usize = 4;
const NOISE_FIT_ALPHA_MIN: f64 = 1e-3;
const NOISE_FIT_ALPHA_MAX: f64 = 100.0;
const NOISE_FIT_BETA_MIN: f64 = 1.0;
const NOISE_FIT_BETA_MAX: f64 = 1e7;
const SNR_Z_LO: f64 = 3.0;
const SNR_Z_HI: f64 = 5.0;
const MERGE_CHUNK_ROWS: usize = 128;
const STATS_MAX_SIDE: usize = 1024;
/// Luma disagreement (short scale) above which IVW is suppressed.
pub const LUMA_DISAGREE_TAU: f64 = 300.0;
const IVW_CHANNEL_SPREAD_TAU: f64 = 150.0;

/// Mean short/long IVW weights and related diagnostics.
#[derive(Debug, Clone, PartialEq)]
pub struct FusionStats {
    pub mean_short_weight: f64,
    pub mean_long_weight: f64,
    pub zero_weight_pixels: usize,
    pub total_pixels: usize,
    pub mean_residual_confidence: Option<f64>,
    pub exposure_ratio_used: Option<f64>,
    pub exposure_ratio_per_channel: Option<Vec<f64>>,
    pub noise_alpha: Option<Vec<f64>>,
    pub noise_beta: Option<Vec<f64>>,
    pub noise_fitted: Option<Vec<bool>>,
}

impl FusionStats {
    pub fn zero_weight_fraction(&self) -> f64 {
        if self.total_pixels == 0 { 0.0 } else { self.zero_weight_pixels as f64 / self.total_pixels as f64 }
    }
}

/// Diagnostics for [`merge_n_passes`] (same-exposure frame stacking).
#[derive(Debug, Clone, PartialEq)]
pub struct PassMergeStats {
    pub n_frames: usize,
    pub total_pixels: usize,
    pub mean_confidence: f64,
    pub outlier_pixels: usize,
    pub zero_weight_pixels: usize,
}

impl PassMergeStats {
    pub fn zero_weight_fraction(&self) -> f64 {
        if self.total_pixels == 0 { 0.0 } else { self.zero_weight_pixels as f64 / self.total_pixels as f64 }
    }
    pub fn outlier_fraction(&self) -> f64 {
        if self.total_pixels == 0 { 0.0 } else { self.outlier_pixels as f64 / self.total_pixels as f64 }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MergeStats {
    Fusion(FusionStats),
    Pass(PassMergeStats),
}

#[derive(Debug, Clone)]
pub struct MergeResult {
    pub rgb: Array3<u16>,
    pub fusion_stats: Option<MergeStats>,
}

/// Estimate Poisson–Gaussian `α`, `β` from flat (or near-flat) frames.
pub fn estimate_pg_noise_params(flats: &[ArrayView3<u16>], patch: usize) -> (f64, f64) {
    let (mut means, mut vars) = (Vec::new(), Vec::new());
    for flat in flats {
        let (h, w, c) = flat.dim();
        let lum = |y: usize, x: usize| (0..c).map(|k| flat[[y, x, k]] as f64).sum::<f64>() / c as f64;
        if h < patch || w < patch {
            continue;
        }
        let mut y = 0;
        while y + patch <= h {
            let mut x = 0;
            while x + patch <= w {
                let vals: Vec<f64> = (y..y + patch).flat_map(|yy| (x..x + patch).map(move |xx| (yy, xx))).map(|(yy, xx)| lum(yy, xx)).collect();
                let m = vals.iter().sum::<f64>() / vals.len() as f64;
                let v = vals.iter().map(|t| (t - m).powi(2)).sum::<f64>() / vals.len() as f64;
                means.push(m);
                vars.push(v);
                x += patch;
            }
            y += patch;
        }
    }
    if means.len() < 8 {
        return (SNR_ALPHA, SNR_BETA);
    }
    let (m, v): (Vec<f64>, Vec<f64>) =
        means.iter().zip(&vars).filter(|(m, v)| **m > 50.0 && **m < 0.9 * FULL_SCALE && v.is_finite()).map(|(a, b)| (*a, *b)).unzip();
    if m.len() < 8 {
        return (SNR_ALPHA, SNR_BETA);
    }
    let coef = lstsq_line(&m, &v, &vec![1.0; m.len()]);
    (coef[0].max(1e-6), coef[1].max(1.0))
}

/// Fuse short and long RGB negatives into one uint16 frame (SNR / IVW).
pub fn merge_exposures(short: &Array3<u16>, long: &Array3<u16>, exposure_short: i64, exposure_long: i64, align_shift: Option<(f64, f64)>, alpha: Option<f64>, beta: Option<f64>) -> Result<Array3<u16>> {
    Ok(merge_exposures_result(short, long, exposure_short, exposure_long, align_shift, alpha, beta)?.rgb)
}

/// Like [`merge_exposures`] but returns weight stats.
pub fn merge_exposures_result(short: &Array3<u16>, long: &Array3<u16>, exposure_short: i64, exposure_long: i64, align_shift: Option<(f64, f64)>, alpha: Option<f64>, beta: Option<f64>) -> Result<MergeResult> {
    if short.dim() != long.dim() || short.dim().2 != 3 {
        return Err(Error::Value(format!("expected matching HxWx3 arrays, got {:?} and {:?}", short.dim(), long.dim())));
    }
    if exposure_long <= 0 || exposure_short <= 0 {
        return Err(Error::Value("exposure values must be positive".into()));
    }
    let (long_a, shift_used) = align_pass_to_reference(short, long, align_shift)?;
    let usb_ratio = exposure_long as f64 / exposure_short as f64;
    let (rgb, stats) = merge_snr(short, &long_a, usb_ratio, linear_shift_var_scale(shift_used), alpha, beta);
    Ok(MergeResult { rgb, fusion_stats: Some(MergeStats::Fusion(stats)) })
}

/// Stride down large frames for global ratio / median estimates (`a[::sy, ::sx]`).
fn subsample(a: &Array3<u16>) -> Array3<f32> {
    let (h, w, _) = a.dim();
    let sy = (h / STATS_MAX_SIDE).max(1);
    let sx = (w / STATS_MAX_SIDE).max(1);
    a.slice(s![..;sy, ..;sx, ..]).mapv(|v| v as f32)
}

/// Variance factor of white noise after a bilinear shift by `shift`.
pub fn linear_shift_var_scale(shift: (f64, f64)) -> f64 {
    let mut scale = 1.0;
    for sv in [shift.0, shift.1] {
        let t = sv - sv.floor();
        scale *= (1.0 - t).powi(2) + t.powi(2);
    }
    scale
}

fn smoothstep01(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

/// Noise-floor ramp × soft saturation ramp on raw (unnormalized) DN (float32).
fn smooth_confidence(raw: f32, floor: f64, clip_start: f64, clip_end: f64) -> f32 {
    let fl = floor as f32;
    let floor_w = ((raw - fl) / floor.max(1e-12) as f32).clamp(0.0, 1.0);
    let t = (raw - clip_start as f32) / (clip_end - clip_start).max(1e-12) as f32;
    floor_w * (1.0 - smoothstep01(t))
}

/// 1 for |z|<=z_lo, smooth decay to 0 by z_hi (stored float32).
fn residual_confidence(z: f64) -> f32 {
    let az = z.abs();
    if az >= SNR_Z_HI {
        0.0
    } else if az > SNR_Z_LO {
        ((SNR_Z_HI - az) / (SNR_Z_HI - SNR_Z_LO).max(1e-12)) as f32
    } else {
        1.0
    }
}

/// Robust per-channel median long/short ratio on mid-tone pixels.
fn estimate_exposure_ratio(a: &Array3<f32>, b: &Array3<f32>, usb_ratio: f64) -> [f32; 3] {
    let lo = (0.01 * FULL_SCALE) as f32;
    let hi = (0.85 * FULL_SCALE) as f32;
    let (h, w, _) = a.dim();
    let mean3 = |arr: &Array3<f32>, y: usize, x: usize| (arr[[y, x, 0]] + arr[[y, x, 1]] + arr[[y, x, 2]]) / 3.0;
    let mut lum_ratios = Vec::new();
    for y in 0..h {
        for x in 0..w {
            let (la, lb) = (mean3(a, y, x), mean3(b, y, x));
            if la > lo && la < hi && lb > lo && lb < hi {
                lum_ratios.push(lb / la.max(1e-12));
            }
        }
    }
    let fallback = if lum_ratios.len() >= 100 { median_f32(lum_ratios) } else { usb_ratio as f32 };
    let mut ratios = [fallback; 3];
    for (c, r) in ratios.iter_mut().enumerate() {
        let mut v = Vec::new();
        for y in 0..h {
            for x in 0..w {
                let (ac, bc) = (a[[y, x, c]], b[[y, x, c]]);
                if ac > lo && ac < hi && bc > lo && bc < hi {
                    v.push(bc / ac.max(1e-12));
                }
            }
        }
        if v.len() >= 1000 {
            *r = median_f32(v);
        }
    }
    ratios
}

/// Per-channel Poisson-Gaussian `α`, `β` from the aligned pass pair; NaN where the fit fails.
fn fit_pg_noise_from_pair(a: &Array3<f32>, b: &Array3<f32>, r: [f32; 3], long_var_scale: f64, floor: f64, clip_start: f64) -> ([f64; 3], [f64; 3]) {
    let mut alpha = [f64::NAN; 3];
    let mut beta = [f64::NAN; 3];
    let cs = clip_start as f32;
    for c in 0..3 {
        let rc = r[c] as f64;
        let rc32 = rc as f32;
        let (mut level, mut d) = (Vec::new(), Vec::new());
        for (ac, bc) in a.slice(s![.., .., c]).iter().zip(b.slice(s![.., .., c]).iter()) {
            let l = bc / rc32;
            if l > floor as f32 && *bc < cs && *ac < cs {
                level.push(l);
                d.push(ac - l);
            }
        }
        if level.len() < NOISE_FIT_MIN_BINS * NOISE_FIT_MIN_PER_BIN {
            continue;
        }
        let sorted = sort_f32(level.clone());
        let edges = quantiles_sorted_f32(&sorted, &linspace(0.0, 1.0, NOISE_FIT_BINS + 1));
        let idx: Vec<usize> = level
            .iter()
            .map(|&l| {
                let pos = edges.partition_point(|&e| e <= l as f64) as isize - 1;
                pos.clamp(0, NOISE_FIT_BINS as isize - 1) as usize
            })
            .collect();
        let (mut xs, mut vs, mut ns) = (Vec::new(), Vec::new(), Vec::new());
        for k in 0..NOISE_FIT_BINS {
            let dk: Vec<f32> = idx.iter().zip(&d).filter(|(i, _)| **i == k).map(|(_, &v)| v).collect();
            let n = dk.len();
            if n < NOISE_FIT_MIN_PER_BIN {
                continue;
            }
            let med = median_f32(dk.clone());
            let mad = median_f32(dk.iter().map(|v| (v - med).abs()).collect()) as f64;
            let lv: Vec<f32> = idx.iter().zip(&level).filter(|(i, _)| **i == k).map(|(_, &v)| v).collect();
            xs.push(median_f32(lv) as f64);
            vs.push((1.4826 * mad).powi(2));
            ns.push(n as f64);
        }
        if xs.len() < NOISE_FIT_MIN_BINS {
            continue;
        }
        let mut pred: Vec<f64> = vs.iter().map(|v| v.max(1e-6)).collect();
        let mut coef = [0.0; 2];
        for _ in 0..3 {
            let sw: Vec<f64> = ns.iter().zip(&pred).map(|(n, p)| n.sqrt() / p).collect();
            coef = lstsq_line(&xs, &vs, &sw);
            pred = xs.iter().map(|x| (coef[0] * x + coef[1]).max(1e-6)).collect();
        }
        let a_fit = coef[0] / (1.0 + long_var_scale / rc);
        let b_fit = coef[1] / (1.0 + long_var_scale / (rc * rc));
        if !(a_fit.is_finite() && b_fit.is_finite()) || !(NOISE_FIT_ALPHA_MIN..=NOISE_FIT_ALPHA_MAX).contains(&a_fit) || b_fit > NOISE_FIT_BETA_MAX {
            continue;
        }
        alpha[c] = a_fit;
        // A slightly negative intercept is fit noise around a tiny read noise.
        beta[c] = b_fit.max(NOISE_FIT_BETA_MIN);
    }
    (alpha, beta)
}

/// Per-pixel model shared by the z-median estimate and the band merge.
struct Px {
    r: [f32; 3],
    alpha: [f32; 3],
    beta: [f32; 3],
    /// `long_var_scale / (r*r)` in float64.
    vb_scale: [f64; 3],
    floor: f64,
    clip_start: f64,
    clip_end: f64,
}

/// One pixel's quantities: (xa, xb, ca, cb, va, vb, lum_xa, lum_xb, va_lum, vb_lum, has).
#[allow(clippy::type_complexity)]
fn pixel_terms(p: &Px, a: [f32; 3], b: [f32; 3]) -> ([f32; 3], [f32; 3], [f32; 3], [f32; 3], [f32; 3], [f64; 3], f32, f32, f32, f64, bool) {
    let xb = [0, 1, 2].map(|c| b[c] / p.r[c]);
    let ca = a.map(|v| smooth_confidence(v, p.floor, p.clip_start, p.clip_end));
    let cb = b.map(|v| smooth_confidence(v, p.floor, p.clip_start, p.clip_end));
    let va = [0, 1, 2].map(|c| p.alpha[c] * a[c].max(0.0) + p.beta[c]);
    let vb = [0, 1, 2].map(|c| (p.alpha[c] * b[c].max(0.0) + p.beta[c]) as f64 * p.vb_scale[c]);
    let conf = [0, 1, 2].map(|c| ca[c].min(cb[c]));
    let wsum = conf[0] + conf[1] + conf[2];
    let has = wsum > 1e-6;
    let wsafe = wsum.max(1e-6);
    let lum_xa = (conf[0] * a[0] + conf[1] * a[1] + conf[2] * a[2]) / wsafe;
    let lum_xb = (conf[0] * xb[0] + conf[1] * xb[1] + conf[2] * xb[2]) / wsafe;
    let va_lum = (conf[0] * va[0] + conf[1] * va[1] + conf[2] * va[2]) / wsafe;
    let vb_lum = (conf[0] as f64 * vb[0] + conf[1] as f64 * vb[1] + conf[2] as f64 * vb[2]) / wsafe as f64;
    (xb, ca, cb, conf, va, vb, lum_xa, lum_xb, va_lum, vb_lum, has)
}

fn z_of(lum_xa: f32, lum_xb: f32, va_lum: f32, vb_lum: f64) -> f64 {
    (lum_xa - lum_xb) as f64 / (va_lum as f64 + vb_lum).max(1e-12).sqrt()
}

/// Clipping-aware inverse-variance fusion in short-exposure radiometric scale.
fn merge_snr(short: &Array3<u16>, long: &Array3<u16>, usb_ratio: f64, long_var_scale: f64, alpha: Option<f64>, beta: Option<f64>) -> (Array3<u16>, FusionStats) {
    let (floor, clip_start, clip_end) = (SNR_FLOOR, SNR_CLIP_START, SNR_CLIP_END);
    let a_sub = subsample(short);
    let b_sub = subsample(long);
    let r = estimate_exposure_ratio(&a_sub, &b_sub, usb_ratio).map(|v| v.max(1e-12));
    let (alpha_c, beta_c, fitted) = match (alpha, beta) {
        (Some(a), Some(b)) => ([a; 3], [b; 3], [false; 3]),
        _ => {
            let (fa, fb) = fit_pg_noise_from_pair(&a_sub, &b_sub, r, long_var_scale, floor, clip_start);
            let fitted = fa.map(f64::is_finite);
            ([0, 1, 2].map(|c| if fitted[c] { fa[c] } else { SNR_ALPHA }), [0, 1, 2].map(|c| if fitted[c] { fb[c] } else { SNR_BETA }), fitted)
        }
    };
    let px = Px {
        r,
        alpha: alpha_c.map(|v| v as f32),
        beta: beta_c.map(|v| v as f32),
        vb_scale: r.map(|rv| long_var_scale / (rv * rv) as f64),
        floor,
        clip_start,
        clip_end,
    };
    // Global residual-gate median on the strided subsample.
    let mut zs = Vec::new();
    let (hs, ws, _) = a_sub.dim();
    for y in 0..hs {
        for x in 0..ws {
            let a = [0, 1, 2].map(|c| a_sub[[y, x, c]]);
            let b = [0, 1, 2].map(|c| b_sub[[y, x, c]]);
            let t = pixel_terms(&px, a, b);
            if t.10 {
                zs.push(z_of(t.6, t.7, t.8, t.9));
            }
        }
    }
    let z_median = if zs.is_empty() { 0.0 } else { median(zs) };

    let (h, w, _) = short.dim();
    let mut out = Array3::<u16>::zeros((h, w, 3));
    let (mut wa_sum, mut wb_sum, mut c_res_sum) = (0.0f64, 0.0f64, 0.0f64);
    let mut zero_count = 0usize;
    let cs32 = clip_start as f32;
    for y0 in (0..h).step_by(MERGE_CHUNK_ROWS) {
        let y1 = h.min(y0 + MERGE_CHUNK_ROWS);
        // Band sums accumulate per band as numpy's `float(wa.sum())` does.
        let (mut wa_band, mut wb_band, mut cres_band) = (0.0f64, 0.0f64, 0.0f64);
        for y in y0..y1 {
            for x in 0..w {
                let a = [0, 1, 2].map(|c| short[[y, x, c]] as f32);
                let b = [0, 1, 2].map(|c| long[[y, x, c]] as f32);
                let (xb, ca, cb, conf, va, vb, lum_xa, lum_xb, va_lum, vb_lum, has) = pixel_terms(&px, a, b);
                let z_local = z_of(lum_xa, lum_xb, va_lum, vb_lum) - z_median;
                let c_res = residual_confidence(z_local);
                let gate = (conf[0] + conf[1] + conf[2]) / 3.0;
                let c_res_eff = 1.0 - gate * (1.0 - c_res);
                let wa = [0, 1, 2].map(|c| ca[c] / va[c].max(1e-12));
                let wb = [0, 1, 2].map(|c| cb[c] as f64 / vb[c].max(1e-12));
                // `wa * xa` is float32 in numpy; `wb` (and so the sum) is float64.
                let ivw = [0, 1, 2].map(|c| ((wa[c] * a[c]) as f64 + wb[c] * xb[c] as f64) / (wa[c] as f64 + wb[c]).max(1e-12));
                let lum_diff = if has { (lum_xa - lum_xb).abs() as f64 } else { 0.0 };
                let spread = ivw.iter().cloned().fold(f64::MIN, f64::max) - ivw.iter().cloned().fold(f64::MAX, f64::min);
                let misaligned = lum_diff > LUMA_DISAGREE_TAU && spread > IVW_CHANNEL_SPREAD_TAU;
                let mut all_zero = true;
                for c in 0..3 {
                    let prefer = if wa[c] as f64 >= wb[c] { a[c] } else { xb[c] };
                    // `(1 - c) * prefer` is float32 in numpy, added to the float64 IVW term.
                    let merged = c_res_eff as f64 * ivw[c] + ((1.0 - c_res_eff) * prefer) as f64;
                    let mut o = if misaligned { a[c] as f64 } else { merged };
                    let both_zero = ca[c] + cb[c] <= 1e-6;
                    all_zero &= both_zero;
                    if both_zero {
                        o = if a[c] >= cs32 { a[c] as f64 } else if b[c] < cs32 { xb[c] as f64 } else { 0.0 };
                    }
                    out[[y, x, c]] = o.clamp(0.0, 65535.0) as u16;
                    wa_band += wa[c] as f64;
                    wb_band += wb[c];
                }
                if all_zero {
                    zero_count += 1;
                }
                cres_band += c_res_eff as f64;
            }
        }
        wa_sum += wa_band;
        wb_sum += wb_band;
        c_res_sum += cres_band;
    }
    let n_weights = (h * w * 3).max(1) as f64;
    let total = h * w;
    let stats = FusionStats {
        mean_short_weight: wa_sum / n_weights,
        mean_long_weight: wb_sum / n_weights,
        zero_weight_pixels: zero_count,
        total_pixels: total,
        mean_residual_confidence: Some(c_res_sum / total.max(1) as f64),
        exposure_ratio_used: Some((r[0] + r[1] + r[2]) as f64 / 3.0),
        exposure_ratio_per_channel: Some(r.iter().map(|&v| v as f64).collect()),
        noise_alpha: Some(px.alpha.iter().map(|&v| v as f64).collect()),
        noise_beta: Some(px.beta.iter().map(|&v| v as f64).collect()),
        noise_fitted: Some(fitted.to_vec()),
    };
    (out, stats)
}

/// Stack N same-exposure captures for an SNR gain (no exposure fusion).
///
/// Frames must already be aligned to a common reference.
pub fn merge_n_passes(frames: &[Array3<u16>], alpha: Option<f64>, beta: Option<f64>) -> Result<MergeResult> {
    let Some(reference) = frames.first() else {
        return Err(Error::Value("merge_n_passes needs >= 1 frame, got 0".into()));
    };
    let alpha = alpha.unwrap_or(SNR_ALPHA) as f32;
    let beta = beta.unwrap_or(SNR_BETA) as f32;
    if reference.dim().2 != 3 {
        return Err(Error::Value(format!("expected HxWx3 arrays, got {:?}", reference.dim())));
    }
    for (i, f) in frames.iter().enumerate().skip(1) {
        if f.dim() != reference.dim() {
            return Err(Error::Value(format!("frame {i} shape {:?} does not match frame 0 shape {:?}", f.dim(), reference.dim())));
        }
    }
    let (h, w, _) = reference.dim();
    let total = h * w;
    if frames.len() == 1 {
        return Ok(MergeResult {
            rgb: reference.clone(),
            fusion_stats: Some(MergeStats::Pass(PassMergeStats { n_frames: 1, total_pixels: total, mean_confidence: 1.0, outlier_pixels: 0, zero_weight_pixels: 0 })),
        });
    }
    let n = frames.len();
    let mut out = Array3::<u16>::zeros((h, w, 3));
    let (mut conf_sum, mut zero_count, mut outlier_count) = (0.0f64, 0usize, 0usize);
    let mut lum = vec![0f32; n];
    let mut xs = vec![[0f32; 3]; n];
    let mut cs = vec![[0f32; 3]; n];
    for y0 in (0..h).step_by(MERGE_CHUNK_ROWS) {
        let y1 = h.min(y0 + MERGE_CHUNK_ROWS);
        let mut band_conf = 0.0f64;
        for y in y0..y1 {
            for x in 0..w {
                for (i, f) in frames.iter().enumerate() {
                    xs[i] = [0, 1, 2].map(|c| f[[y, x, c]] as f32);
                    cs[i] = xs[i].map(|v| smooth_confidence(v, SNR_FLOOR, SNR_CLIP_START, SNR_CLIP_END));
                    lum[i] = (xs[i][0] + xs[i][1] + xs[i][2]) / 3.0;
                }
                let luma_median = median_f32(lum.clone());
                let mut outlier_any = false;
                let mut weights = vec![[0f32; 3]; n];
                for i in 0..n {
                    let is_outlier = ((lum[i] - luma_median).abs() as f64) > LUMA_DISAGREE_TAU;
                    outlier_any |= is_outlier;
                    weights[i] = [0, 1, 2].map(|c| {
                        let variance = alpha * xs[i][c].max(0.0) + beta;
                        if is_outlier { 0.0 } else { cs[i][c] / variance.max(1e-12) }
                    });
                }
                let mut no_weight_all = true;
                for c in 0..3 {
                    let mut acc = weights[0][c] * xs[0][c];
                    let mut w_sum = weights[0][c];
                    let mut all_zero_conf = cs[0][c] <= 1e-6;
                    for i in 1..n {
                        acc += weights[i][c] * xs[i][c];
                        w_sum += weights[i][c];
                        all_zero_conf &= cs[i][c] <= 1e-6;
                    }
                    let mut merged = acc / w_sum.max(1e-12);
                    let no_weight = w_sum <= 1e-9;
                    no_weight_all &= no_weight;
                    if no_weight || all_zero_conf {
                        // np.mean over the stacked frame axis (float32 pairwise for small n).
                        merged = xs.iter().map(|v| v[c]).sum::<f32>() / n as f32;
                    }
                    out[[y, x, c]] = merged.clamp(0.0, 65535.0) as u16;
                    band_conf += (cs.iter().map(|v| v[c]).sum::<f32>() / n as f32) as f64;
                }
                if no_weight_all {
                    zero_count += 1;
                }
                if outlier_any {
                    outlier_count += 1;
                }
            }
        }
        conf_sum += band_conf;
    }
    Ok(MergeResult {
        rgb: out,
        fusion_stats: Some(MergeStats::Pass(PassMergeStats {
            n_frames: n,
            total_pixels: total,
            mean_confidence: conf_sum / (total * 3).max(1) as f64,
            outlier_pixels: outlier_count,
            zero_weight_pixels: zero_count,
        })),
    })
}
