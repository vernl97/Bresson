// SPDX-License-Identifier: GPL-3.0-or-later
//! Pass registration for multi-pass Plustek USB scans.
//!
//! The carriage re-homes (AGOHOME) between colour, IR, and ME passes, so
//! secondary frames can land a few pixels off the reference.

use ndarray::{Array2, Array3, ArrayView2, Axis};

use crate::error::Result;
use crate::registration::{
    RegistrationMode, RegistrationOptions, Resampler, apply_transform, band_shift_profile, estimate_banded, estimate_translation, phase_correlate_shift,
    warp_shift,
};

const ALIGN_PROBE_WIDTH: usize = 1024;
const REFINE_ROI_SIDE: usize = 512;
const REFINE_MAX_RESIDUAL: f64 = 2.0;
const ALIGN_MIN_RESPONSE: f64 = 0.03;
const ALIGN_SANITY_MAX_FRAC: f64 = 0.5;
const ALIGN_BAND_COUNT: usize = 8;
const ALIGN_BAND_MIN_HEIGHT: usize = 256;
const ALIGN_BAND_OUTLIER_PX: f64 = 8.0;

pub type Shift2D = (f64, f64);

/// Always available (pure Rust registration).
pub fn native_align_available() -> bool {
    true
}

pub fn default_options() -> RegistrationOptions {
    RegistrationOptions {
        coarse_target_px: ALIGN_PROBE_WIDTH,
        refine_roi_side: REFINE_ROI_SIDE,
        refine_max_residual: REFINE_MAX_RESIDUAL,
        min_response: ALIGN_MIN_RESPONSE,
        sanity_max_frac: ALIGN_SANITY_MAX_FRAC,
        band_count: ALIGN_BAND_COUNT,
        band_min_height: ALIGN_BAND_MIN_HEIGHT,
        band_outlier_px: ALIGN_BAND_OUTLIER_PX,
        ..Default::default()
    }
}

fn phase(r: ArrayView2<f32>, m: ArrayView2<f32>, scale: f64) -> (f64, f64, f64) {
    phase_correlate_shift(r, m, scale, 1)
}

/// Per-row-band profile for Multi-Pass.
pub fn band_shift_profile_default(reference: &Array3<u16>, moving: &Array3<u16>, n_bands: usize) -> Option<(f64, Vec<f64>)> {
    let opt = RegistrationOptions { band_count: n_bands, ..default_options() };
    band_shift_profile(reference.view(), moving.view(), &opt, Some(&phase))
}

/// Estimate `(dx, dy)` (sub-pixel) for `moving` → `reference`.
pub fn estimate_pass_shift(reference: &Array3<u16>, moving: &Array3<u16>) -> Shift2D {
    let r = estimate_translation(reference.view(), moving.view(), RegistrationMode::MultiExposure, &default_options(), None);
    if r.accepted { r.shift() } else { (0.0, 0.0) }
}

/// Align `moving` onto `reference`; return aligned array and shift used.
pub fn align_pass_to_reference(reference: &Array3<u16>, moving: &Array3<u16>, shift: Option<Shift2D>) -> Result<(Array3<u16>, Shift2D)> {
    if moving.is_empty() || reference.dim().0 != moving.dim().0 || reference.dim().1 != moving.dim().1 {
        return Ok((moving.clone(), (0.0, 0.0)));
    }
    let (dx, dy) = shift.unwrap_or_else(|| estimate_pass_shift(reference, moving));
    if dx.abs() < 1e-9 && dy.abs() < 1e-9 {
        return Ok((moving.clone(), (0.0, 0.0)));
    }
    Ok((warp_shift(moving, dx, dy, Resampler::Linear), (dx, dy)))
}

/// Row-banded Multi-Pass alignment (production default).
pub fn align_pass_to_reference_banded(reference: &Array3<u16>, moving: &Array3<u16>) -> Result<(Array3<u16>, Shift2D)> {
    if moving.is_empty() || reference.dim().0 != moving.dim().0 || reference.dim().1 != moving.dim().1 {
        return Ok((moving.clone(), (0.0, 0.0)));
    }
    let r = estimate_banded(reference.view(), moving.view(), &default_options(), Some(&phase));
    if !r.accepted {
        return align_pass_to_reference(reference, moving, None);
    }
    Ok((apply_transform(moving, &r.transform, Resampler::Linear), r.shift()))
}

fn ir3(ir: &Array2<u16>) -> Array3<u16> {
    ir.clone().insert_axis(Axis(2))
}

/// IR→RGB alignment via structural / luminance registration.
pub fn align_ir_to_rgb(rgb: &Array3<u16>, ir: &Array2<u16>) -> Result<Array2<u16>> {
    Ok(align_ir_to_rgb_result(rgb, ir)?.0)
}

/// Like [`align_ir_to_rgb`] but also returns the shift actually applied.
pub fn align_ir_to_rgb_result(rgb: &Array3<u16>, ir: &Array2<u16>) -> Result<(Array2<u16>, Shift2D)> {
    let ir_v = ir3(ir);
    let r = estimate_translation(rgb.view(), ir_v.view(), RegistrationMode::RgbIr, &default_options(), None);
    let (out, shift) = if !r.accepted {
        // Fall back to plain luminance translation (historical path).
        align_pass_to_reference_mono(rgb, &ir_v)?
    } else {
        (apply_transform(&ir_v, &r.transform, Resampler::Linear), r.shift())
    };
    Ok((out.index_axis_move(Axis(2), 0), shift))
}

/// `align_pass_to_reference(rgb, ir)` with a mono moving plane.
fn align_pass_to_reference_mono(reference: &Array3<u16>, moving: &Array3<u16>) -> Result<(Array3<u16>, Shift2D)> {
    if moving.is_empty() || reference.dim().0 != moving.dim().0 || reference.dim().1 != moving.dim().1 {
        return Ok((moving.clone(), (0.0, 0.0)));
    }
    let r = estimate_translation(reference.view(), moving.view(), RegistrationMode::MultiExposure, &default_options(), None);
    let (dx, dy) = if r.accepted { r.shift() } else { (0.0, 0.0) };
    if dx.abs() < 1e-9 && dy.abs() < 1e-9 {
        return Ok((moving.clone(), (0.0, 0.0)));
    }
    Ok((warp_shift(moving, dx, dy, Resampler::Linear), (dx, dy)))
}
