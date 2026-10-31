// SPDX-License-Identifier: GPL-3.0-or-later
//! Frame-adaptive ME long-pass exposure selection.
//!
//! The short colour plane proposes a useful long `REG_EXPOSURE`; a separate
//! safety clamp enforces the validated envelope.

use ndarray::{ArrayView3, Axis};

use crate::error::{Error, Result};
use crate::num::{U16Hist, py_round};

pub const ALGORITHM_VERSION: &str = "1";
const FULL_SCALE: f64 = 65535.0;
const EPSILON: f64 = 1.0;
/// Soft clip threshold for predicted long-pass saturation fractions.
const CLIP_DN: f64 = 0.95 * FULL_SCALE;

/// How the ME long pass picks its exposure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MeLongMode {
    /// Meter the short pass.
    #[default]
    Adaptive,
    /// The model's `me_fixed_long_exposure` for every frame.
    Fixed,
}

impl MeLongMode {
    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "adaptive" => Ok(Self::Adaptive),
            "fixed" => Ok(Self::Fixed),
            _ => Err(Error::Value(format!("me_long_mode must be one of ('adaptive', 'fixed'), got {s:?}"))),
        }
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Adaptive => "adaptive",
            Self::Fixed => "fixed",
        }
    }
}

/// Result of adaptive (or fixed) long-pass exposure selection.
#[derive(Debug, Clone, PartialEq)]
pub struct MeExposureDecision {
    pub proposed: i64,
    pub selected: i64,
    pub reason: String,
    pub dense_p05: [f64; 3],
    pub predicted_clip: [f64; 3],
    pub algorithm_version: &'static str,
}

fn channel_hists(rgb: ArrayView3<u16>) -> [U16Hist; 3] {
    [0, 1, 2].map(|c| U16Hist::new(rgb.index_axis(Axis(2), c).iter().copied()))
}

/// Per-channel dense percentile of `max(DN - black, 0)`.
pub fn dense_percentiles(rgb: ArrayView3<u16>, black_level: f64, dense_percentile: f64) -> [f64; 3] {
    if rgb.dim().2 < 3 || rgb.len() == 0 {
        return [0.0; 3];
    }
    let h = channel_hists(rgb);
    [0, 1, 2].map(|c| h[c].percentile_mapped(dense_percentile, |v| (v - black_level).max(0.0)))
}

/// Propose a long exposure from per-channel dense percentiles.
pub fn choose_long_exposure(rgb: ArrayView3<u16>, short_exposure: i64, black_level: f64, dense_percentile: f64, target_dense_dn: f64) -> Result<f64> {
    if rgb.dim().2 < 3 {
        return Err(Error::Value("short_rgb must be HxWx3".into()));
    }
    if short_exposure <= 0 {
        return Err(Error::Value("short_exposure must be positive".into()));
    }
    if rgb.len() == 0 {
        return Err(Error::Value("short_rgb is empty".into()));
    }
    let dense = dense_percentiles(rgb, black_level, dense_percentile);
    let ratio = dense.iter().map(|&d| target_dense_dn / d.max(EPSILON)).fold(f64::MIN, f64::max);
    Ok(short_exposure as f64 * ratio)
}

/// Fraction of pixels predicted at/above soft-clip for each RGB channel.
pub fn predicted_clip_fractions(rgb: ArrayView3<u16>, short_exposure: i64, long_exposure: i64, black_level: f64) -> [f64; 3] {
    let (h, w, c) = rgb.dim();
    if c < 3 || short_exposure <= 0 || h * w == 0 {
        return [0.0; 3];
    }
    let scale = long_exposure as f64 / short_exposure as f64;
    let n = (h * w) as f64;
    [0, 1, 2].map(|ch| {
        let k = rgb.index_axis(Axis(2), ch).iter().filter(|&&v| black_level + (v as f64 - black_level).max(0.0) * scale >= CLIP_DN).count();
        k as f64 / n
    })
}

/// Clamp a proposed long exposure into the validated safety envelope.
pub fn clamp_long_exposure(proposed: f64, short_exposure: i64, adaptive_min: i64, adaptive_max: i64, hardware_max: i64, max_ratio: f64) -> (i64, String) {
    let safe_max = hardware_max.min(adaptive_max).min(1.max(py_round(short_exposure as f64 * max_ratio)));
    let lo = adaptive_min;
    if lo > safe_max {
        return (safe_max, "clamped to validated hardware maximum (min>max)".into());
    }
    if !proposed.is_finite() {
        return (lo, "fallback non-finite proposal → adaptive minimum".into());
    }
    let raw = py_round(proposed);
    if raw > safe_max {
        return (safe_max, "clamped to validated hardware maximum".into());
    }
    if raw < lo {
        return (lo, "clamped to adaptive minimum".into());
    }
    (raw, "adaptive".into())
}

/// Envelope parameters for [`select_long_exposure`] (Python keyword defaults).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LongExposureParams {
    pub black_level: f64,
    pub dense_percentile: f64,
    pub target_dense_dn: f64,
    pub adaptive_min: i64,
    pub adaptive_max: i64,
    pub hardware_max: i64,
    pub max_ratio: f64,
    pub default_long: i64,
}

impl Default for LongExposureParams {
    fn default() -> Self {
        Self {
            black_level: 0.0,
            dense_percentile: 5.0,
            target_dense_dn: 10000.0,
            adaptive_min: 42000,
            adaptive_max: 85000,
            hardware_max: 85000,
            max_ratio: 7.0,
            default_long: 42000,
        }
    }
}

/// Choose and clamp a long exposure; fall back to `default_long` on error.
pub fn select_long_exposure(rgb: ArrayView3<u16>, short_exposure: i64, p: &LongExposureParams) -> MeExposureDecision {
    let p05 = dense_percentiles(rgb, p.black_level, p.dense_percentile);
    let fallback = || {
        let selected = p.default_long;
        MeExposureDecision {
            proposed: selected,
            selected,
            reason: "fallback".into(),
            dense_p05: p05,
            predicted_clip: predicted_clip_fractions(rgb, short_exposure, selected, p.black_level),
            algorithm_version: ALGORITHM_VERSION,
        }
    };
    let proposed_f = match choose_long_exposure(rgb, short_exposure, p.black_level, p.dense_percentile, p.target_dense_dn) {
        Ok(v) if v.is_finite() && v > 0.0 => v,
        _ => return fallback(),
    };
    let (selected, reason) = clamp_long_exposure(proposed_f, short_exposure, p.adaptive_min, p.adaptive_max, p.hardware_max, p.max_ratio);
    MeExposureDecision {
        proposed: py_round(proposed_f),
        selected,
        reason,
        dense_p05: p05,
        predicted_clip: predicted_clip_fractions(rgb, short_exposure, selected, p.black_level),
        algorithm_version: ALGORITHM_VERSION,
    }
}
