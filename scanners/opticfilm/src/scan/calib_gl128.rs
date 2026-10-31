// SPDX-License-Identifier: GPL-3.0-or-later
//! Capture-derived AFE search + ASIC shading table helpers for GL128 SE.
//!
//! SilverFast choreography + shading pack/unpack, SilverFast's coarse AFE
//! calibration, and the ASIC shading table at `0x10014000`. Pure functions
//! only — hardware I/O lives on the GL128 driver. The dichotomy helpers
//! further down serve GL845.

use ndarray::{Array2, Array3, ArrayView2, Axis, s};

use crate::device::model::lperiod_for_line;
use crate::error::{Error, Result};
use crate::num::{mean, percentile_sorted, py_round, sorted_f64, to_u16};

/// RGB column triple (dark, white or gain terms).
pub type Rgb = [i64; 3];

pub const FE_GAIN_R: u8 = 0x02;
pub const FE_GAIN_G: u8 = 0x03;
pub const FE_GAIN_B: u8 = 0x04;
pub const FE_OFFSET_R: u8 = 0x05;
pub const FE_OFFSET_G: u8 = 0x06;
pub const FE_OFFSET_B: u8 = 0x07;
pub const FE_GAIN_INDICES: [u8; 3] = [FE_GAIN_R, FE_GAIN_G, FE_GAIN_B];
pub const FE_OFFSET_INDICES: [u8; 3] = [FE_OFFSET_R, FE_OFFSET_G, FE_OFFSET_B];

/// One AFE probe line: 512 px × 3 ch × 2 bytes (16-bit).
pub const AFE_STRIP_PIXELS: usize = 512;
pub const AFE_STRIP_BYTES: usize = AFE_STRIP_PIXELS * 3 * 2;
/// Capture AFE window in native sensor units.
pub const AFE_STRPIXEL: i64 = 0x40;
pub const AFE_ENDPIXEL: i64 = 0x240;
/// Wider mid-search read observed between offset and gain phases.
pub const AFE_WIDE_BYTES: usize = 62268;
pub const AFE_WIDE_PIXELS: usize = AFE_WIDE_BYTES / 6;
pub const AFE_WIDE_ENDPIXEL: i64 = AFE_STRPIXEL + AFE_WIDE_PIXELS as i64;

/// ASIC shading coefficient window.
pub const AHB_SHADING: u32 = 0x1001_4000;
pub const SHADING_RECORD_BYTES: usize = 12;
pub const SHADING_PAIR_BYTES: usize = 4;
/// DVDSET indexes only the first 126 pairs of each 512-byte block; the last
/// two pairs are padding the vendor fills with `gain = 0`.
pub const SHADING_BLOCK_BYTES: usize = 512;
pub const SHADING_BLOCK_PAIRS: usize = SHADING_BLOCK_BYTES / SHADING_PAIR_BYTES;
pub const SHADING_BLOCK_DATA_PAIRS: usize = 126;
pub const SHADING_BLOCK_PAD_PAIRS: usize = SHADING_BLOCK_PAIRS - SHADING_BLOCK_DATA_PAIRS;
/// Gain fixed point: `0x2000` = 1.0.
pub const SHADING_GAIN_UNITY: i64 = 0x2000;
pub const SHADING_GAIN_TARGET: i64 = 0xFFFF;
pub const SHADING_GAIN_MIN: i64 = SHADING_GAIN_UNITY / 4;
pub const SHADING_GAIN_MAX: i64 = SHADING_GAIN_UNITY * 4;
/// Capture shading acquires use 128 lines × window_width × RGB16.
pub const SHADING_LINES: usize = 128;

/// Shading table width in pixels by scan DPI, for passes without an image window.
pub const SHADING_WIDTH_BY_DPI: [(i64, i64); 11] = [
    (150, 852),
    (300, 852),
    (600, 852),
    (720, 1021),
    (900, 1277),
    (1200, 1728),
    (1440, 2042),
    (1800, 2478),
    (2400, 3406),
    (3600, 4956),
    (7200, 10223),
];

pub const AFE_OFFSET_TARGET: i64 = 0x1000;
pub const AFE_GAIN_TARGET: i64 = 0xD000;
pub const AFE_OFFSET_MAX: i64 = 0xFF;
pub const AFE_GAIN_MAX: i64 = 0x3F;
pub const AFE_DICHOTOMY_ITERS: i64 = 9;
/// Sign bit of the 9-bit sign-magnitude FE offset code.
pub const AFE_OFFSET_SIGN: i64 = 0x100;

/// Optical-black columns 2..56 of the narrow strip.
pub const AFE_OPTICAL_BLACK_COLUMNS: std::ops::Range<usize> = 2..56;
pub const AFE_OFFSET_PROBE_LOW: i64 = 0x80;
pub const AFE_OFFSET_PROBE_HIGH: i64 = 0xFF;
pub const AFE_PRE_OFFSET_BLACK: f64 = 48.0;
pub const AFE_BLACK_TARGET_COLOR: f64 = 0x500 as f64;
pub const AFE_BLACK_TARGET_IR: f64 = 0.0;
pub const AFE_SF_WHITE_TARGETS_COLOR: [f64; 3] = [47000.0, 53575.0, 53575.0];
pub const AFE_SF_WHITE_TARGETS_IR: [f64; 3] = [51800.0, 51800.0, 51800.0];
pub const AFE_WHITE_TARGETS_COLOR: [f64; 3] = [50518.0, 55518.0, 56428.0];
pub const AFE_WHITE_TARGETS_IR: [f64; 3] = [52313.0, 52639.0, 52515.0];
pub const AFE_MIN_COUNTS_PER_OFFSET: f64 = 4.0;
pub const AFE_FALLBACK_PRE_OFFSETS: [i64; 3] = [-95, -82, -81];
pub const AFE_FALLBACK_OFFSETS_COLOR: [i64; 3] = [15, 23, 19];
pub const AFE_FALLBACK_OFFSETS_IR: [i64; 3] = [-31, -20, -28];
pub const AFE_STRIP_DUMMY_PRE: i64 = 0x02;
pub const AFE_STRIP_DUMMY_WIDE: i64 = 0x17;

/// Line period and `0x2B` dummy for one AFE strip acquire.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AfeStripTiming {
    pub lperiod: i64,
    pub dummy: i64,
}

/// SF's AFE strip timing for a scan at `scan_dpi` (ASIC dpi).
pub fn afe_strip_timing(scan_dpi: i64, _wide: bool, pre_offset: bool) -> AfeStripTiming {
    if pre_offset {
        return AfeStripTiming { lperiod: lperiod_for_line(AFE_STRIP_PIXELS as f64, scan_dpi), dummy: AFE_STRIP_DUMMY_PRE };
    }
    AfeStripTiming { lperiod: lperiod_for_line(AFE_WIDE_PIXELS as f64, scan_dpi), dummy: AFE_STRIP_DUMMY_WIDE }
}

/// Whether AFE codes searched at `searched_dpi` match `scan_dpi`'s strip timing.
pub fn afe_timing_matches(searched_dpi: Option<i64>, scan_dpi: i64) -> bool {
    let Some(d) = searched_dpi else { return false };
    [(false, true), (true, false)]
        .iter()
        .all(|&(wide, pre)| afe_strip_timing(d, wide, pre) == afe_strip_timing(scan_dpi, wide, pre))
}

fn u16le(b: &[u8], off: usize) -> i64 {
    u16::from_le_bytes([b[off], b[off + 1]]) as i64
}

fn u16_plane(strip: &[u8], pixels: usize) -> Result<Vec<i64>> {
    let need = pixels * 2;
    if strip.len() < need {
        return Err(Error::Value(format!("strip too short for plane: {} < {need}", strip.len())));
    }
    Ok((0..need).step_by(2).map(|i| u16le(strip, i)).collect())
}

fn lag_corr(samples: &[i64], lag: usize) -> f64 {
    if lag == 0 || samples.len() <= lag {
        return 0.0;
    }
    let a: Vec<f64> = samples[..samples.len() - lag].iter().map(|&v| v as f64).collect();
    let b: Vec<f64> = samples[lag..].iter().map(|&v| v as f64).collect();
    let n = a.len() as f64;
    let ma = a.iter().sum::<f64>() / n;
    let mb = b.iter().sum::<f64>() / n;
    let num: f64 = a.iter().zip(&b).map(|(x, y)| (x - ma) * (y - mb)).sum();
    let da: f64 = a.iter().map(|x| (x - ma).powi(2)).sum();
    let db: f64 = b.iter().map(|y| (y - mb).powi(2)).sum();
    let denom = (da * db).sqrt();
    if denom <= 1e-9 { 0.0 } else { num / denom }
}

/// True if `strip` looks planar (`RRR…`) rather than chunky `RGBRGB…`.
pub fn choose_usb_planar(strip: &[u8], pixels: usize) -> bool {
    if strip.len() < pixels * 6 {
        return true;
    }
    let plane = u16_plane(strip, pixels).expect("length checked");
    let c1 = lag_corr(&plane, 1);
    let c3 = lag_corr(&plane, 3);
    !(c3 > c1 + 0.08)
}

/// Signed offset (-255..+255) to the 9-bit sign-magnitude FE code.
pub fn encode_afe_offset(value: i64) -> i64 {
    let v = value.clamp(-AFE_OFFSET_MAX, AFE_OFFSET_MAX);
    if v < 0 { AFE_OFFSET_SIGN | -v } else { v }
}

/// 9-bit sign-magnitude FE code to a signed offset (`0x100` is -0 → 0).
pub fn decode_afe_offset(code: i64) -> i64 {
    let magnitude = code & AFE_OFFSET_MAX;
    if code & AFE_OFFSET_SIGN != 0 { -magnitude } else { magnitude }
}

/// AD9826 PGA codes that bring wide-strip `means` (taken at gain 0) to `targets`.
pub fn coarse_gains_from_wide_means(means: [f64; 3], targets: [f64; 3], gain_max: i64) -> [i64; 3] {
    let mut out = [0; 3];
    for c in 0..3 {
        let code = (75.6 - 75.6 * means[c] / targets[c]).floor() as i64;
        out[c] = code.clamp(0, gain_max);
    }
    out
}

/// Mean R/G/B of the optical-black columns of a narrow (chunky) AFE strip.
pub fn optical_black_means(strip: &[u8]) -> Result<[f64; 3]> {
    let cols = AFE_OPTICAL_BLACK_COLUMNS;
    let need = cols.end * 6;
    if strip.len() < need {
        return Err(Error::Value(format!("strip too short for optical black: {} < {need}", strip.len())));
    }
    let mut sums = [0i64; 3];
    for x in cols.clone() {
        for (c, s) in sums.iter_mut().enumerate() {
            *s += u16le(strip, x * 6 + 2 * c);
        }
    }
    let n = cols.len() as f64;
    Ok([sums[0] as f64 / n, sums[1] as f64 / n, sums[2] as f64 / n])
}

/// Signed offsets that put optical black at `target`, from a two-point probe.
/// A channel whose probe is unusable is `None`.
pub fn offsets_for_black_level(low: [f64; 3], high: [f64; 3], target: f64, low_code: i64, high_code: i64) -> [Option<i64>; 3] {
    let steps = (high_code - low_code) as f64;
    let mut out = [None; 3];
    for c in 0..3 {
        let per_code = (high[c] - low[c]) / steps;
        if per_code < AFE_MIN_COUNTS_PER_OFFSET {
            continue;
        }
        let value = py_round(low_code as f64 + (target - low[c]) / per_code);
        out[c] = Some(value.clamp(-AFE_OFFSET_MAX, AFE_OFFSET_MAX));
    }
    out
}

/// Higher is better for photo-like structure vs 1-pixel columnar rainbow.
pub fn rgb_layout_score(rgb: &Array3<u16>) -> f64 {
    let (h, w, _) = rgb.dim();
    if h < 1 || w < 2 {
        return 0.0;
    }
    let luma: Array2<f32> = rgb.map(|&v| v as f32).mean_axis(Axis(2)).expect("3 channels");
    let a: Vec<f64> = luma.slice(s![.., ..-1]).iter().map(|&v| v as f64).collect();
    let b: Vec<f64> = luma.slice(s![.., 1..]).iter().map(|&v| v as f64).collect();
    let ma = mean(a.iter().copied());
    let mb = mean(b.iter().copied());
    let (mut num, mut da, mut db) = (0.0, 0.0, 0.0);
    for (x, y) in a.iter().zip(&b) {
        num += (x - ma) * (y - mb);
        da += (x - ma).powi(2);
        db += (y - mb).powi(2);
    }
    let denom = (da * db).sqrt();
    if denom <= 1e-6 { 0.0 } else { num / denom }
}

/// Settled FE codes for R,G,B: 6-bit PGA gains and 9-bit sign-magnitude offsets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AfeFrontend {
    pub offsets: [i64; 3],
    pub gains: [i64; 3],
}

impl AfeFrontend {
    /// `(fe_index, value)` pairs in capture order (gains, then offsets).
    pub fn as_fe_writes(&self) -> Vec<(u8, u16)> {
        let mut out = Vec::with_capacity(6);
        for (i, g) in FE_GAIN_INDICES.iter().zip(self.gains) {
            out.push((*i, (g & 0xFFFF) as u16));
        }
        for (i, o) in FE_OFFSET_INDICES.iter().zip(self.offsets) {
            out.push((*i, (o & 0xFFFF) as u16));
        }
        out
    }
}

/// Parameters for the runtime dichotomy AFE search.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AfeSearchConfig {
    pub offset_target: i64,
    pub gain_target: i64,
    pub offset_max: i64,
    pub gain_max: i64,
    pub iterations: i64,
    /// On this FE, raising the offset code *raises* strip means.
    pub offset_increases_mean: bool,
    pub gain_increases_mean: bool,
    pub tolerance: f64,
}

impl Default for AfeSearchConfig {
    fn default() -> Self {
        Self {
            offset_target: AFE_OFFSET_TARGET,
            gain_target: AFE_GAIN_TARGET,
            offset_max: AFE_OFFSET_MAX,
            gain_max: AFE_GAIN_MAX,
            iterations: AFE_DICHOTOMY_ITERS,
            offset_increases_mean: true,
            gain_increases_mean: true,
            tolerance: 512.0,
        }
    }
}

/// One observed FE programming step from the capture timeline.
#[derive(Debug, Clone, PartialEq)]
pub struct AfeSearchPhase {
    pub name: &'static str,
    pub offsets: Option<[i64; 3]>,
    pub gains: Option<[i64; 3]>,
    pub strip_bytes: Option<usize>,
    pub note: &'static str,
}

const fn phase(name: &'static str, offsets: Option<[i64; 3]>, gains: Option<[i64; 3]>, strip_bytes: Option<usize>, note: &'static str) -> AfeSearchPhase {
    AfeSearchPhase { name, offsets, gains, strip_bytes, note }
}

/// Capture-derived phase list (session 03). Offset codes `>= 0x100` are negative.
pub const AFE_SEARCH_PHASES_CAPTURE: [AfeSearchPhase; 14] = [
    phase("init_zeros", Some([0, 0, 0]), Some([0, 0, 0]), None, ""),
    phase("offset_probe_0x80", Some([0x80, 0x80, 0x80]), Some([0, 0, 0]), Some(AFE_STRIP_BYTES), "first 3072-byte strip"),
    phase("offset_probe_0xff", Some([0xFF, 0xFF, 0xFF]), None, Some(AFE_STRIP_BYTES), ""),
    phase("offset_negative_estimate", Some([0x016A, 0x0142, 0x015B]), None, None, "session-03 example (-106, -66, -91); values are unit-specific"),
    phase("wide_gain_strip", None, Some([0, 0, 0]), Some(AFE_WIDE_BYTES), "62268-byte read before coarse gains"),
    phase("gain_coarse", None, Some([0x24, 0x1A, 0x21]), None, "session-03 example coarse gains"),
    phase("offset_reprobe_0x80", Some([0x80, 0x80, 0x80]), None, Some(AFE_STRIP_BYTES), ""),
    phase("offset_reprobe_0xff", Some([0xFF, 0xFF, 0xFF]), None, Some(AFE_STRIP_BYTES), ""),
    phase("offset_negative_small", Some([0x115, 0x113, 0x113]), None, None, ""),
    phase("offset_low", Some([0x0C, 0x16, 0x10]), None, None, ""),
    phase("shading_upload_unity_white", None, None, None, "AHB 0x10014000 with white=0x2000; dark from dark strip"),
    phase("offset_settle", Some([0x12, 0x1E, 0x17]), None, None, ""),
    phase("shading_upload_measured_white", None, None, None, "AHB 0x10014000 with per-pixel white terms"),
    phase("gain_fine", None, Some([0x26, 0x1E, 0x25]), None, ""),
];

/// Total `(dark, gain)` pairs for `entries` pixels, block padding included.
pub fn shading_pair_count(entries: usize) -> usize {
    let data = entries * 3;
    data + SHADING_BLOCK_PAD_PAIRS * (data / SHADING_BLOCK_DATA_PAIRS)
}

/// AHB declared size for `entries` pixels of shading coefficients.
pub fn declared_shading_size(entries: usize) -> usize {
    SHADING_PAIR_BYTES * shading_pair_count(entries)
}

/// Pixels covered by a declared AHB shading upload size (inverse of above).
pub fn shading_entry_count(declared_size: usize) -> Result<usize> {
    if declared_size % SHADING_PAIR_BYTES != 0 {
        return Err(Error::Value(format!("Shading declared_size={declared_size} is not a whole number of 4-byte pairs")));
    }
    let estimate = declared_size * SHADING_BLOCK_DATA_PAIRS / (SHADING_PAIR_BYTES * 3 * SHADING_BLOCK_PAIRS);
    for c in estimate.saturating_sub(2)..estimate + 3 {
        if declared_shading_size(c) == declared_size {
            return Ok(c);
        }
    }
    Err(Error::Value(format!("Shading declared_size={declared_size} does not match any pixel count")))
}

/// Declared AHB shading table width for `resolution` dpi.
pub fn shading_width_for_resolution(resolution: i64) -> i64 {
    if let Some(&(_, w)) = SHADING_WIDTH_BY_DPI.iter().find(|e| e.0 == resolution) {
        return w;
    }
    if resolution < 600 {
        return 852;
    }
    py_round(2478.0 * resolution as f64 / 1800.0).max(1)
}

/// USB pixels per shading line for an image window.
pub fn shading_acquire_width(strpixel: i64, endpixel: i64, dpiset: i64, optical_resolution: i64) -> Result<i64> {
    let span = endpixel - strpixel;
    if span <= 0 {
        return Err(Error::Value(format!("shading window empty: {strpixel}..{endpixel}")));
    }
    let dpi = (dpiset * 6).max(1);
    let factor = (optical_resolution / dpi).max(1);
    Ok((span / factor).max(1))
}

/// Native-dpi samples per USB pixel for `DPISET`.
pub fn shading_native_factor(dpiset: i64, optical_resolution: i64) -> i64 {
    let dpi = (dpiset * 6).max(1);
    (optical_resolution / dpi).max(1)
}

pub const IR_SHADING_WHITE_SPREAD_MAX: i64 = 500;
pub const IR_SHADING_DARK_MAX: i64 = 64;
pub const IR_SHADING_WHITE_MEAN_MIN: f64 = 10000.0;
pub const IR_SHADING_WHITE_CLIP_LEVEL: i64 = 60000;
pub const IR_SHADING_WHITE_CLIP_FRAC: f64 = 0.90;
pub const IR_SHADING_DROPOUT_PERIOD: i64 = 126;
pub const IR_SHADING_DROPOUT_MIN_RUNS: usize = 4;
pub const COLOR_SHADING_DARK_MEAN_MAX: f64 = 3000.0;
pub const COLOR_SHADING_WHITE_MEAN_MIN: f64 = 20000.0;
pub const COLOR_SHADING_WHITE_CLIP_LEVEL: i64 = 65000;
pub const COLOR_SHADING_WHITE_CLIP_FRAC: f64 = 0.05;
pub const COLOR_SHADING_CHANNEL_WHITE_MIN: f64 = 8000.0;
pub const COLOR_SHADING_GAIN_MEDIAN_MIN: f64 = 1.02;
pub const COLOR_SHADING_GAIN_MEDIAN_MAX: f64 = 3.0;
pub const COLOR_SHADING_GAIN_CLAMP_FRAC: f64 = 0.02;
pub const COLOR_SHADING_FILM_R_FRAC: f64 = 0.55;
pub const COLOR_SHADING_FILM_GB_MIN: f64 = 12000.0;
pub const COLOR_SHADING_DARK_SETTLE_S: f64 = 0.5;
pub const IR_SHADING_DARK_SETTLE_S: f64 = 0.5;
pub const COLOR_SHADING_LAYOUT_MEAN_IMPROVE_MIN: f64 = 5000.0;

/// Mean of all channel samples across columns.
pub fn shading_columns_mean(cols: &[Rgb]) -> f64 {
    if cols.is_empty() {
        return 0.0;
    }
    cols.iter().flatten().sum::<i64>() as f64 / (cols.len() * 3).max(1) as f64
}

/// Choose dark-column USB layout for shading averages.
pub fn pick_shading_dark_layout(primary: &[Rgb], alternate: &[Rgb], primary_is_planar: bool, dark_mean_max: f64, mean_improve_min: f64) -> (Vec<Rgb>, bool) {
    let mean_a = shading_columns_mean(primary);
    let mean_b = shading_columns_mean(alternate);
    let use_alt = mean_b < mean_a && (mean_b <= dark_mean_max || (mean_a - mean_b) >= mean_improve_min);
    let chosen = if use_alt { alternate } else { primary };
    let planar = if use_alt { !primary_is_planar } else { primary_is_planar };
    (chosen.to_vec(), planar)
}

fn channel_mean(rows: &[Rgb], c: usize) -> f64 {
    rows.iter().map(|r| r[c]).sum::<i64>() as f64 / rows.len() as f64
}

/// True when a home white strip looks like colour-neg orange mask, not clear TA.
pub fn color_shading_looks_like_film(white: &[Rgb]) -> bool {
    if white.is_empty() {
        return false;
    }
    let (wr, wg, wb) = (channel_mean(white, 0), channel_mean(white, 1), channel_mean(white, 2));
    let gb = wg.min(wb);
    gb >= COLOR_SHADING_FILM_GB_MIN && wr < gb * COLOR_SHADING_FILM_R_FRAC
}

/// Python `f"{x:.0f}"`: round half to even.
fn f0(x: f64) -> String {
    format!("{}", x.round_ties_even() as i64)
}

/// `(ok, reason)` for the raw strips a colour shading table is built from.
pub fn validate_color_shading_strip(dark: &[Rgb], white: &[Rgb], acquire_width: Option<i64>) -> (bool, String) {
    if dark.is_empty() || white.is_empty() {
        return (false, "empty dark/white".into());
    }
    if dark.len() != white.len() {
        return (false, format!("dark/white length mismatch {}!={}", dark.len(), white.len()));
    }
    let (mut dark, mut white) = (dark, white);
    if let Some(aw) = acquire_width.filter(|&a| a > 0) {
        let n = (aw as usize).min(white.len());
        if n < 8 {
            return (false, format!("acquire_width too small ({n})"));
        }
        dark = &dark[..n];
        white = &white[..n];
    }
    let dark_mean = shading_columns_mean(dark);
    let white_mean = shading_columns_mean(white);
    if white_mean < COLOR_SHADING_WHITE_MEAN_MIN {
        return (false, format!("white mean {} < {}", f0(white_mean), COLOR_SHADING_WHITE_MEAN_MIN as i64));
    }
    let n_samples = white.len() * 3;
    let clipped = white.iter().flatten().filter(|&&v| v >= COLOR_SHADING_WHITE_CLIP_LEVEL).count();
    if clipped as f64 > n_samples as f64 * COLOR_SHADING_WHITE_CLIP_FRAC {
        return (false, format!("white clipped at the rail ({clipped}/{n_samples} ≥ {COLOR_SHADING_WHITE_CLIP_LEVEL})"));
    }
    if color_shading_looks_like_film(white) {
        return (false, "white strip looks like film (not clear home field)".into());
    }
    for ch in 0..3 {
        let w = channel_mean(white, ch);
        if w < COLOR_SHADING_CHANNEL_WHITE_MIN {
            return (false, format!("ch{ch} white {} < {}", f0(w), COLOR_SHADING_CHANNEL_WHITE_MIN as i64));
        }
    }
    let floor = (COLOR_SHADING_WHITE_MEAN_MIN as i64 / 4) as i64;
    let zero_cols: Vec<i64> = white
        .iter()
        .enumerate()
        .filter(|(_, r)| *r.iter().min().unwrap() < floor)
        .map(|(i, _)| i as i64)
        .collect();
    if zero_cols.len() >= IR_SHADING_DROPOUT_MIN_RUNS && has_periodic_dropouts(&zero_cols, IR_SHADING_DROPOUT_PERIOD) {
        return (false, format!("periodic white dropouts (n={}, period≈{IR_SHADING_DROPOUT_PERIOD})", zero_cols.len()));
    }
    if dark_mean > COLOR_SHADING_DARK_MEAN_MAX {
        return (true, format!("ok (soft dark mean {}, white {})", f0(dark_mean), f0(white_mean)));
    }
    (true, "ok".into())
}

/// `(ok, reason)` for computed DVDSET gains before arming them.
pub fn validate_color_shading_gains(gains: &[Rgb]) -> (bool, String) {
    let mut flat: Vec<i64> = gains.iter().flatten().copied().collect();
    if flat.is_empty() {
        return (false, "empty gains".into());
    }
    flat.sort_unstable();
    let median = flat[flat.len() / 2] as f64 / SHADING_GAIN_UNITY as f64;
    if median < COLOR_SHADING_GAIN_MEDIAN_MIN {
        return (false, format!("median gain {median:.3}x < {COLOR_SHADING_GAIN_MEDIAN_MIN}x (white already at target)"));
    }
    if median > COLOR_SHADING_GAIN_MEDIAN_MAX {
        return (false, format!("median gain {median:.3}x > {COLOR_SHADING_GAIN_MEDIAN_MAX:.1}x (white too dim to flatten)"));
    }
    let clamped = flat.iter().filter(|&&v| v <= SHADING_GAIN_MIN || v >= SHADING_GAIN_MAX).count();
    if clamped as f64 > flat.len() as f64 * COLOR_SHADING_GAIN_CLAMP_FRAC {
        return (false, format!("{clamped}/{} gains hit a clamp ({SHADING_GAIN_MIN}..{SHADING_GAIN_MAX})", flat.len()));
    }
    (true, format!("ok (median gain {median:.3}x)"))
}

/// `(ok, reason)` for a just-built IR shading table.
///
/// Pass `raw_white` (pre-equalize strip) so bar/period checks still see
/// channel dropouts that [`equalize_ir_white_columns`] would hide.
pub fn validate_ir_shading_table(dark: &[Rgb], white: &[Rgb], acquire_width: Option<i64>, raw_white: Option<&[Rgb]>) -> (bool, String) {
    if dark.is_empty() || white.is_empty() {
        return (false, "empty dark/white".into());
    }
    if dark.len() != white.len() {
        return (false, format!("dark/white length mismatch {}!={}", dark.len(), white.len()));
    }
    let (mut dark, mut white, mut raw_white) = (dark, white, raw_white);
    if let Some(aw) = acquire_width.filter(|&a| a > 0) {
        let n = (aw as usize).min(white.len());
        if n < 8 {
            return (false, format!("acquire_width too small ({n})"));
        }
        dark = &dark[..n];
        white = &white[..n];
        raw_white = raw_white.map(|r| &r[..n.min(r.len())]);
    }
    let dark_peak = dark.iter().flatten().copied().max().unwrap();
    if dark_peak > IR_SHADING_DARK_MAX {
        return (false, format!("dark peak {dark_peak} > {IR_SHADING_DARK_MAX}"));
    }
    let w0 = white[0];
    let head_spread = w0.iter().max().unwrap() - w0.iter().min().unwrap();
    if head_spread > IR_SHADING_WHITE_SPREAD_MAX {
        return (false, format!("white head spread {head_spread} > {IR_SHADING_WHITE_SPREAD_MAX}"));
    }
    let whites: Vec<i64> = white.iter().flatten().copied().collect();
    let mean_white = whites.iter().sum::<i64>() as f64 / whites.len() as f64;
    if mean_white < IR_SHADING_WHITE_MEAN_MIN {
        return (false, format!("white mean {} < {}", f0(mean_white), IR_SHADING_WHITE_MEAN_MIN as i64));
    }
    let clipped = whites.iter().filter(|&&v| v > IR_SHADING_WHITE_CLIP_LEVEL).count();
    if clipped as f64 >= whites.len() as f64 * IR_SHADING_WHITE_CLIP_FRAC {
        return (false, format!("white clipped ({clipped}/{} > {IR_SHADING_WHITE_CLIP_LEVEL})", whites.len()));
    }
    let bar_white = raw_white.unwrap_or(white);
    let bar_min = bar_white.iter().flatten().copied().min();
    if bar_min.is_some_and(|m| m <= 0) {
        let zero_cols: Vec<i64> = bar_white
            .iter()
            .enumerate()
            .filter(|(_, r)| r.iter().any(|&c| c <= 0))
            .map(|(i, _)| i as i64)
            .collect();
        if zero_cols.len() >= IR_SHADING_DROPOUT_MIN_RUNS && has_periodic_dropouts(&zero_cols, IR_SHADING_DROPOUT_PERIOD) {
            return (false, format!("periodic zero whites (n={}, period≈{IR_SHADING_DROPOUT_PERIOD})", zero_cols.len()));
        }
        if zero_cols.len() > 8.max(bar_white.len() / 20) {
            return (false, format!("too many zero white columns ({})", zero_cols.len()));
        }
    }
    let wmax = *whites.iter().max().unwrap();
    let wmin = *whites.iter().min().unwrap();
    let global_spread = wmax - wmin;
    if global_spread > 40000 && wmin < 256 {
        return (false, format!("pathological white range spread={global_spread}"));
    }
    (true, "ok".into())
}

/// True when zero-column starts cluster on a repeating period.
fn has_periodic_dropouts(zero_cols: &[i64], period: i64) -> bool {
    if zero_cols.len() < IR_SHADING_DROPOUT_MIN_RUNS {
        return false;
    }
    let mut starts = vec![zero_cols[0]];
    for &i in &zero_cols[1..] {
        let last = *starts.last().unwrap();
        if i != last + 1 && i != last {
            starts.push(i);
        }
    }
    if starts.len() < IR_SHADING_DROPOUT_MIN_RUNS {
        return false;
    }
    let diffs: Vec<i64> = starts.windows(2).map(|w| w[1] - w[0]).collect();
    let near = diffs.iter().filter(|&&d| (d - period).abs() <= 2 || (d - period / 2).abs() <= 2).count();
    near >= 3.max(diffs.len() / 2)
}

/// Broadcast per-column mean R/G/B to `(m, m, m)` for IR shading.
pub fn equalize_ir_white_columns(white: &[Rgb]) -> Vec<Rgb> {
    white
        .iter()
        .map(|r| {
            let m = py_round(r.iter().sum::<i64>() as f64 / 3.0).clamp(0, 65535);
            [m, m, m]
        })
        .collect()
}

/// Edge-padded box filter (float32 in, float32 out); `half_window <= 0` copies.
pub fn box_smooth_profile(profile: &[f32], half_window: i64) -> Vec<f32> {
    if half_window <= 0 || profile.is_empty() {
        return profile.to_vec();
    }
    let h = half_window as usize;
    let k = 2 * h + 1;
    let n = profile.len();
    let ext = |i: isize| profile[(i.clamp(0, n as isize - 1)) as usize] as f64;
    let mut c = Vec::with_capacity(n + 2 * h + 1);
    c.push(0.0f64);
    let mut acc = 0.0f64;
    for i in -(h as isize)..(n + h) as isize {
        acc += ext(i);
        c.push(acc);
    }
    (0..n).map(|i| ((c[i + k] - c[i]) / k as f64) as f32).collect()
}

/// Host IR flatten: `out[x] = ir[x] * target / white[x]` (per column).
///
/// By default the white profile is low-pass filtered so only slow
/// illumination falloff is corrected. `smooth_half = Some(0)` uses the raw profile.
pub fn flatten_ir_columns(ir: ArrayView2<u16>, white: &[i64], target: Option<f64>, smooth_half: Option<i64>) -> Result<Array2<u16>> {
    let width = ir.ncols();
    if white.len() < width {
        return Err(Error::Value(format!("white profile length {} < image width {width}", white.len())));
    }
    let w_raw: Vec<f32> = white[..width].iter().map(|&v| v.max(1) as f32).collect();
    let half = smooth_half.unwrap_or(if width < 64 { 0 } else { 8.max(width as i64 / 40) });
    let w: Vec<f32> = box_smooth_profile(&w_raw, half).into_iter().map(|v| v.max(1.0)).collect();
    let mut tgt = target.unwrap_or_else(|| mean(w.iter().map(|&v| v as f64)) as f32 as f64);
    if tgt <= 0.0 {
        tgt = 1.0;
    }
    let gain: Vec<f32> = w.iter().map(|&wv| tgt as f32 / wv).collect();
    Ok(Array2::from_shape_fn(ir.dim(), |(y, x)| to_u16((ir[[y, x]] as f32 * gain[x]) as f64)))
}

pub const IR_SIDECAR_TARGET_LEVEL: f64 = 56000.0;
pub const IR_FLATTEN_BORDER_INSET: f64 = 0.04;
pub const IR_FLATTEN_MAX_GAIN: f64 = 4.0;

/// Per-column percentile of a 2-D float32 plane (`np.percentile(axis=0)`).
pub fn column_percentile(arr: &Array2<f32>, q: f64) -> Vec<f32> {
    arr.axis_iter(Axis(1)).map(|col| crate::num::percentile_sorted_f32(&crate::num::sort_f32(col.to_vec()), q)).collect()
}

/// Per-image IR flatten for a SilverFast-like iSRD sidecar.
pub fn flatten_ir_image_columns(ir: ArrayView2<u16>, percentile: f64, target: Option<f64>, smooth_half: Option<i64>, border_inset: f64, max_gain: f64) -> Result<Array2<u16>> {
    let (height, width) = ir.dim();
    if height < 1 || width < 1 {
        return Err(Error::Value(format!("empty infrared plane: ({height}, {width})")));
    }
    let arr = ir.map(|&v| v as f32);
    let mut levels = column_percentile(&arr, percentile);
    let inset = border_inset.clamp(0.0, 0.2);
    let cut = if width >= 16 { py_round(width as f64 * inset).max(0) as usize } else { 0 };
    if cut * 2 < width {
        let lo = levels[cut];
        let hi = levels[width - cut - 1];
        for v in &mut levels[..cut] {
            *v = lo;
        }
        for v in &mut levels[width - cut..] {
            *v = hi;
        }
    }
    let half = smooth_half.unwrap_or(if width < 64 { 0 } else { 8.max(width as i64 / 40) });
    let levels: Vec<f32> = box_smooth_profile(&levels, half).into_iter().map(|v| v.max(1.0)).collect();
    let mean_of = |s: &[f32]| mean(s.iter().map(|&v| v as f64)) as f32 as f64;
    let mut tgt_base = match target {
        Some(t) => t,
        None if cut * 2 < width => mean_of(&levels[cut..width - cut]),
        None => mean_of(&levels),
    };
    if tgt_base <= 0.0 {
        tgt_base = 1.0;
    }
    let gain: Vec<f32> = levels
        .iter()
        .map(|&l| {
            let g = tgt_base as f32 / l;
            if max_gain > 0.0 { g.min(max_gain as f32) } else { g }
        })
        .collect();
    Ok(Array2::from_shape_fn((height, width), |(y, x)| to_u16((arr[[y, x]] * gain[x]) as f64)))
}

/// Edge-padded box mean via integral image (float64 in, float32 out).
pub fn box_mean_2d(a: &Array2<f64>, half_y: usize, half_x: usize) -> Array2<f32> {
    if half_y == 0 && half_x == 0 {
        return a.map(|&v| v as f32);
    }
    let (h, w) = a.dim();
    let (ph, pw) = (h + 2 * half_y, w + 2 * half_x);
    let mut integral = Array2::<f64>::zeros((ph + 1, pw + 1));
    // padded.cumsum(0).cumsum(1)
    let pad = |y: usize, x: usize| {
        let yy = (y as isize - half_y as isize).clamp(0, h as isize - 1) as usize;
        let xx = (x as isize - half_x as isize).clamp(0, w as isize - 1) as usize;
        a[[yy, xx]]
    };
    let mut col = Array2::<f64>::zeros((ph, pw));
    for x in 0..pw {
        let mut acc = 0.0;
        for y in 0..ph {
            acc += pad(y, x);
            col[[y, x]] = acc;
        }
    }
    for y in 0..ph {
        let mut acc = 0.0;
        for x in 0..pw {
            acc += col[[y, x]];
            integral[[y + 1, x + 1]] = acc;
        }
    }
    let (ky, kx) = (2 * half_y + 1, 2 * half_x + 1);
    let area = (ky * kx) as f64;
    Array2::from_shape_fn((h, w), |(y, x)| {
        let t = integral[[y + ky, x + kx]] - integral[[y, x + kx]] - integral[[y + ky, x]] + integral[[y, x]];
        (t / area) as f32
    })
}

pub const MILD_IR_GHOST_FADE_STRENGTH: f64 = 0.35;

/// Gently fade dye/ghost structure and deepen sharp dark defects (IR preview).
pub fn mild_ir_ghost_fade(plane_u16: ArrayView2<u16>, strength: f64) -> Array2<u16> {
    let (height, width) = plane_u16.dim();
    if height < 8 || width < 8 {
        return plane_u16.to_owned();
    }
    let plane = plane_u16.map(|&v| v as f32);
    let s = strength.clamp(0.0, 1.0);
    let film_hy = 12.max(90.min(height / 12));
    let film_hx = 16.max(140.min(width / 10));
    let low = box_mean_2d(&plane.map(|&v| v as f64), film_hy, film_hx);
    let low_mean = mean(low.iter().map(|&v| v as f64)) as f32 as f64;
    let film_retain = 1.0 - 0.55 * s;
    let dark_boost = 1.0 + 1.4 * s;
    let bright_keep = 1.0 - 0.35 * s;
    Array2::from_shape_fn((height, width), |(y, x)| {
        let l = low[[y, x]] as f64;
        let residual = (plane[[y, x]] - low[[y, x]]) as f64;
        let base = low_mean + (l - low_mean) * film_retain;
        to_u16(base + residual.min(0.0) * dark_boost + residual.max(0.0) * bright_keep)
    })
}

pub const IR_DETECTION_MIN: i64 = 0;
pub const IR_DETECTION_MAX: i64 = 20;
const IR_DET20_LO: f64 = 96.0;
const IR_DET20_HI: f64 = 99.25;
const IR_DET0_LO: f64 = 99.0;
const IR_DET0_HI: f64 = 99.85;

/// Map SilverFast-like Detection `0..20` → `(dip_lo, dip_hi)` enhance percentiles.
pub fn detection_to_enhance_params(detection: i64) -> Result<(f64, f64)> {
    if !(IR_DETECTION_MIN..=IR_DETECTION_MAX).contains(&detection) {
        return Err(Error::Value(format!("detection must be {IR_DETECTION_MIN}..{IR_DETECTION_MAX}, got {detection}")));
    }
    let t = detection as f64 / IR_DETECTION_MAX as f64;
    Ok((IR_DET0_LO + t * (IR_DET20_LO - IR_DET0_LO), IR_DET0_HI + t * (IR_DET20_HI - IR_DET0_HI)))
}

struct DipMaps {
    film: Array2<f32>,
    local: Array2<f32>,
    dip: Array2<f32>,
    center: Array2<bool>,
}

fn ir_dip_maps(arr: &Array2<f32>) -> DipMaps {
    let (height, width) = arr.dim();
    let film_hy = 8.max(80.min(height / 20));
    let film_hx = 12.max(120.min(width / 16));
    let local_hy = 1.max(3.min(height / 200));
    let local_hx = 4.max(12.min(width / 120));
    let a64 = arr.map(|&v| v as f64);
    let film = box_mean_2d(&a64, film_hy, film_hx);
    let local = box_mean_2d(&a64, local_hy, local_hx);
    let dip = Array2::from_shape_fn((height, width), |p| (local[p] - arr[p]).max(0.0));
    let (y0, y1) = ((height as f64 * 0.08) as usize, (height as f64 * 0.92) as usize);
    let (x0, x1) = ((width as f64 * 0.08) as usize, (width as f64 * 0.92) as usize);
    let center = Array2::from_shape_fn((height, width), |(y, x)| y >= y0 && y < y1 && x >= x0 && x < x1 && arr[[y, x]] > 10000.0);
    DipMaps { film, local, dip, center }
}

fn punch_strength(dip: &Array2<f32>, sample: &[f64], lo_p: f64, hi_p: f64) -> Array2<f32> {
    let s = if sample.len() < 64 { sorted_f64(dip.iter().map(|&v| v as f64)) } else { sorted_f64(sample.iter().copied()) };
    let lo = percentile_sorted(&s, lo_p);
    let hi = percentile_sorted(&s, hi_p).max(lo + 1.0);
    dip.map(|&d| {
        let st = ((d as f64 - lo) / (hi - lo)).clamp(0.0, 1.0);
        (st * st * (3.0 - 2.0 * st)) as f32
    })
}

/// Auto Detection: max real-defect hits, minimize ghost false positives.
pub fn estimate_ir_detection(ir_flat: ArrayView2<u16>) -> i64 {
    let arr = ir_flat.map(|&v| v as f32);
    if arr.nrows() < 8 || arr.ncols() < 8 {
        return 15;
    }
    let m = ir_dip_maps(&arr);
    let mut sample: Vec<f64> = m.dip.iter().zip(m.center.iter()).filter(|(_, c)| **c).map(|(&d, _)| d as f64).collect();
    if sample.len() < 64 {
        sample = m.dip.iter().map(|&v| v as f64).collect();
    }
    let ss = sorted_f64(sample.iter().copied());
    let (p80, p96, p995) = (percentile_sorted(&ss, 80.0), percentile_sorted(&ss, 96.0), percentile_sorted(&ss, 99.5));
    let true_pool = Array2::from_shape_fn(m.dip.dim(), |p| m.center[p] && m.dip[p] as f64 >= p995);
    let ghost_pool = Array2::from_shape_fn(m.dip.dim(), |p| m.center[p] && m.dip[p] as f64 >= p80 && m.dip[p] as f64 <= p96);
    let n_true = true_pool.iter().filter(|&&b| b).count();
    let n_ghost = ghost_pool.iter().filter(|&&b| b).count();
    if n_true < 1 {
        return 10;
    }
    let frac = |strength: &Array2<f32>, pool: &Array2<bool>, n: usize| {
        if n == 0 {
            return 0.0;
        }
        strength.iter().zip(pool.iter()).filter(|(s, p)| **p && **s >= 0.5).count() as f64 / n as f64
    };
    let (mut best_d, mut best_score) = (0, -1e9);
    for d in IR_DETECTION_MIN..=IR_DETECTION_MAX {
        let (lo, hi) = detection_to_enhance_params(d).expect("in range");
        let strength = punch_strength(&m.dip, &sample, lo, hi);
        let score = frac(&strength, &true_pool, n_true) - 2.0 * frac(&strength, &ghost_pool, n_ghost);
        if score > best_score + 1e-9 || ((score - best_score).abs() <= 1e-9 && d < best_d) {
            best_score = score;
            best_d = d;
        }
    }
    best_d
}

/// Wash film ghost and punch local dark defects toward SilverFast IR levels.
#[allow(clippy::too_many_arguments)]
pub fn enhance_ir_defect_contrast(ir: ArrayView2<u16>, detection: Option<i64>, target: f64, film_retain: f64, dip_lo_percentile: Option<f64>, dip_hi_percentile: Option<f64>, defect_floor: f64) -> Result<Array2<u16>> {
    let (lo_p, hi_p) = match detection {
        Some(d) => detection_to_enhance_params(d)?,
        None => (dip_lo_percentile.unwrap_or(IR_DET20_LO), dip_hi_percentile.unwrap_or(IR_DET20_HI)),
    };
    let arr = ir.map(|&v| v as f32);
    let (height, width) = arr.dim();
    if height < 8 || width < 8 {
        return Ok(arr.map(|&v| to_u16(v as f64)));
    }
    let m = ir_dip_maps(&arr);
    let film_mean = mean(m.film.iter().map(|&v| v as f64)) as f32 as f64;
    let sample: Vec<f64> = m.dip.iter().zip(m.center.iter()).filter(|(_, c)| **c).map(|(&d, _)| d as f64).collect();
    let strength = punch_strength(&m.dip, &sample, lo_p, hi_p);
    Ok(Array2::from_shape_fn((height, width), |p| {
        let film = m.film[p] as f64;
        let base = target + (film - film_mean) * film_retain + (m.local[p] as f64 - film) * (film_retain * 0.35);
        let st = (strength[p] as f64).max(((8000.0 - arr[p] as f64) / 8000.0).clamp(0.0, 1.0));
        to_u16(base * (1.0 - st) + defect_floor * st)
    }))
}

/// Per-column mean R/G/B from `lines` of RGB16 (planar `RRR…GGG…BBB…` or chunky).
pub fn average_rgb16_columns(raw: &[u8], pixels: usize, lines: usize, planar: bool) -> Result<Vec<Rgb>> {
    let row_bytes = pixels * 6;
    let need = row_bytes * lines;
    if raw.len() < need {
        return Err(Error::Value(format!("shading raw too short: {} < {need}", raw.len())));
    }
    let mut sums = vec![[0i64; 3]; pixels];
    let plane = pixels * 2;
    for line in 0..lines {
        let base = line * row_bytes;
        for (x, s) in sums.iter_mut().enumerate() {
            for (c, sc) in s.iter_mut().enumerate() {
                let off = if planar { base + c * plane + x * 2 } else { base + x * 6 + 2 * c };
                *sc += u16le(raw, off);
            }
        }
    }
    let inv = 1.0 / lines as f64;
    Ok(sums.iter().map(|s| [py_round(s[0] as f64 * inv), py_round(s[1] as f64 * inv), py_round(s[2] as f64 * inv)]).collect())
}

/// Mean R/G/B from a 16-bit LE strip (AFE probe line).
pub fn channel_means_u16(strip: &[u8], pixels: usize, planar: bool) -> Result<[f64; 3]> {
    let expected = pixels * 6;
    if strip.len() < expected {
        return Err(Error::Value(format!("strip too short: {} < {expected}", strip.len())));
    }
    let mut sums = [0i64; 3];
    for (c, s) in sums.iter_mut().enumerate() {
        for i in 0..pixels {
            *s += if planar { u16le(strip, c * pixels * 2 + i * 2) } else { u16le(strip, i * 6 + 2 * c) };
        }
    }
    let p = pixels as f64;
    Ok([sums[0] as f64 / p, sums[1] as f64 / p, sums[2] as f64 / p])
}

/// Broadcast channel means across X — first-upload shape in sessions 03/04.
pub fn constant_dark_from_columns(dark: &[Rgb]) -> Result<Vec<Rgb>> {
    if dark.is_empty() {
        return Err(Error::Value("dark_per_pixel is empty".into()));
    }
    let n = dark.len() as f64;
    let m = [0, 1, 2].map(|c| py_round(dark.iter().map(|r| r[c]).sum::<i64>() as f64 / n));
    Ok(vec![m; dark.len()])
}

/// Fixed-point gain that lifts `white_unity` to `target`.
pub fn shading_gain_for_white(white_unity: f64, target: i64) -> i64 {
    let denom = white_unity.max(1.0);
    py_round(target as f64 * SHADING_GAIN_UNITY as f64 / denom).clamp(SHADING_GAIN_MIN, SHADING_GAIN_MAX)
}

/// Per-pixel RGB gains for a post-unity DVDSET white strip.
pub fn shading_gains_from_white(white: &[Rgb], target: i64) -> Vec<Rgb> {
    white.iter().map(|r| r.map(|v| shading_gain_for_white(v as f64, target))).collect()
}

/// Pack the second shading upload: measured dark plus computed gains.
pub fn build_measured_shading_table(dark: &[Rgb], white: &[Rgb], flatten_dark: bool, declared_size: Option<usize>, target: i64) -> Result<Vec<u8>> {
    let dark_rows = if flatten_dark { constant_dark_from_columns(dark)? } else { dark.to_vec() };
    let declared = declared_size.unwrap_or_else(|| declared_shading_size(white.len()));
    pack_shading_table(&dark_rows, &shading_gains_from_white(white, target), Some(declared))
}

/// Pack per-pixel RGB dark/gain u16 terms into the ASIC shading blob.
pub fn pack_shading_table(dark: &[Rgb], gain: &[Rgb], declared_size: Option<usize>) -> Result<Vec<u8>> {
    if dark.len() != gain.len() {
        return Err(Error::Value("dark/gain length mismatch".into()));
    }
    let mut out = Vec::new();
    let mut pairs = 0usize;
    let mut last_dark: u16 = 0;
    for (d, g) in dark.iter().zip(gain) {
        for ch in 0..3 {
            if pairs != 0 && pairs % SHADING_BLOCK_DATA_PAIRS == 0 {
                for _ in 0..SHADING_BLOCK_PAD_PAIRS {
                    out.extend_from_slice(&last_dark.to_le_bytes());
                    out.extend_from_slice(&[0, 0]);
                }
            }
            last_dark = (d[ch] & 0xFFFF) as u16;
            out.extend_from_slice(&last_dark.to_le_bytes());
            out.extend_from_slice(&((g[ch] & 0xFFFF) as u16).to_le_bytes());
            pairs += 1;
        }
    }
    if let Some(ds) = declared_size {
        if out.len() > ds {
            return Err(Error::Value(format!("payload {} exceeds declared_size {ds}", out.len())));
        }
        out.resize(ds, 0);
    }
    Ok(out)
}

/// One unpacked shading pixel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShadingEntry {
    pub dark: Vec<i64>,
    pub gain: Vec<i64>,
}

/// Parse a shading AHB payload into per-pixel `{dark, gain}` rows (pad pairs skipped).
pub fn unpack_shading_table(payload: &[u8]) -> Vec<ShadingEntry> {
    let mut pairs = Vec::new();
    let mut off = 0;
    while off + SHADING_PAIR_BYTES <= payload.len() {
        if (off / SHADING_PAIR_BYTES) % SHADING_BLOCK_PAIRS < SHADING_BLOCK_DATA_PAIRS {
            pairs.push((u16le(payload, off), u16le(payload, off + 2)));
        }
        off += SHADING_PAIR_BYTES;
    }
    let mut entries = Vec::new();
    let mut i = 0;
    while i + 2 < pairs.len() {
        let trio = &pairs[i..i + 3];
        entries.push(ShadingEntry { dark: trio.iter().map(|p| p.0).collect(), gain: trio.iter().map(|p| p.1).collect() });
        i += 3;
    }
    entries
}

/// First shading upload shape: measured dark, gain fixed at unity (`0x2000`).
pub fn make_unity_white_table(dark: &[Rgb], declared_size: Option<usize>, flatten_dark: bool) -> Result<Vec<u8>> {
    let dark_rows = if flatten_dark { constant_dark_from_columns(dark)? } else { dark.to_vec() };
    let gain = vec![[SHADING_GAIN_UNITY; 3]; dark_rows.len()];
    let declared = declared_size.unwrap_or_else(|| declared_shading_size(dark_rows.len()));
    pack_shading_table(&dark_rows, &gain, Some(declared))
}

/// One binary-search step. Returns `(new_low, new_high, next_code)`.
pub fn dichotomy_bracket_update(low: i64, high: i64, current: i64, mean: f64, target: f64, code_increases_mean: bool) -> (i64, i64, i64) {
    let (mut low, mut high) = (low, high);
    if high <= low {
        return (low, high, low);
    }
    if code_increases_mean {
        if mean > target { high = current } else { low = current }
    } else if mean > target {
        low = current;
    } else {
        high = current;
    }
    if high <= low + 1 {
        return (low, high, low);
    }
    let mut next = (low + high).div_euclid(2);
    if next == current {
        next = if high > current { current + 1 } else { low.max(current - 1) };
    }
    (low, high, next)
}

/// Dichotomy per RGB channel with a shared strip acquire via `apply`.
#[allow(clippy::too_many_arguments)]
pub fn search_afe_codes(
    initial: [i64; 3],
    code_max: i64,
    target: f64,
    iterations: i64,
    tolerance: f64,
    code_increases_mean: bool,
    apply: &mut dyn FnMut([i64; 3]) -> Result<[f64; 3]>,
    code_min: i64,
) -> Result<[i64; 3]> {
    let lo0 = code_min.max(0);
    let hi0 = code_max.max(lo0);
    let mut lows = [lo0; 3];
    let mut highs = [hi0; 3];
    let mut codes = initial.map(|v| v.clamp(lo0, hi0));
    for _ in 0..iterations.max(1) {
        let means = apply(codes)?;
        let mut done = true;
        for c in 0..3 {
            if (means[c] - target).abs() <= tolerance {
                continue;
            }
            done = false;
            let (l, h, n) = dichotomy_bracket_update(lows[c], highs[c], codes[c], means[c], target, code_increases_mean);
            lows[c] = l;
            highs[c] = h;
            codes[c] = n.clamp(lo0, hi0);
        }
        if done {
            break;
        }
    }
    Ok(codes)
}

/// Search offsets then gains so strip channel means approach the targets.
pub fn run_afe_dichotomy(measure: &mut dyn FnMut(AfeFrontend) -> Result<[f64; 3]>, config: Option<AfeSearchConfig>, start: Option<AfeFrontend>) -> Result<AfeFrontend> {
    let cfg = config.unwrap_or_default();
    let fe0 = start.unwrap_or(AfeFrontend { offsets: [0; 3], gains: [0; 3] });
    let offsets = search_afe_codes(
        fe0.offsets,
        cfg.offset_max,
        cfg.offset_target as f64,
        cfg.iterations,
        cfg.tolerance,
        cfg.offset_increases_mean,
        &mut |o| measure(AfeFrontend { offsets: o, gains: fe0.gains }),
        0,
    )?;
    // Start gain search near the capture mid probe (0x80), not zero.
    let gain0 = if fe0.gains.iter().any(|&g| g != 0) { fe0.gains } else { [0x80; 3] };
    let gains = search_afe_codes(
        gain0,
        cfg.gain_max,
        cfg.gain_target as f64,
        cfg.iterations,
        cfg.tolerance,
        cfg.gain_increases_mean,
        &mut |g| measure(AfeFrontend { offsets, gains: g }),
        0,
    )?;
    Ok(AfeFrontend { offsets, gains })
}
