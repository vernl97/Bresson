// SPDX-License-Identifier: GPL-3.0-or-later
//! Port of `tests/registration/`: phase correlation, resampling, pyramid,
//! RGB↔IR seed rules and end-to-end synthetic registration. Inputs use the
//! fixture generator's integer-hash noise instead of numpy's RNG; the bounds
//! are the Python ones. `test_dft.py` checks Guizar against a numpy-only
//! brute-force helper and is not ported (Guizar runs in every test below).

mod common;

use common::noise;
use ndarray::{Array1, Array2, Array3, ArrayView3, Axis, s};
use opticfilm::pass_align::{align_pass_to_reference, align_pass_to_reference_banded, estimate_pass_shift};
use opticfilm::registration::*;

/// Integers in `[lo, hi)` from the deterministic noise.
fn ints(shape: (usize, usize, usize), lo: f64, hi: f64, seed: u64) -> Array3<u16> {
    let n = noise(shape.0 * shape.1 * shape.2, seed);
    Array3::from_shape_vec(shape, n.iter().map(|v| (lo + (v + 1.0) / 2.0 * (hi - lo)).floor() as u16).collect()).unwrap()
}

fn plane(a: &Array3<u16>) -> Array2<f32> {
    a.index_axis(Axis(2), 0).mapv(f32::from)
}

fn shift(a: &Array3<u16>, dx: f64, dy: f64) -> Array3<u16> {
    warp_shift(a, dx, dy, Resampler::Cubic)
}

// --- test_phase.py ---

#[test]
fn optimal_dft_size_powers() {
    assert_eq!(optimal_dft_size(1024), 1024);
    assert_eq!(optimal_dft_size(7), 8);
    assert_eq!(optimal_dft_size(1025), 1080);
}

#[test]
fn hanning_window_shape_and_peak() {
    let win = hanning_window(32, 48);
    assert_eq!(win.dim(), (32, 48));
    assert!(win.iter().all(|&v| (0.0..=1.0 + 1e-5).contains(&v)));
    assert!(win[[16, 24]] > win[[0, 0]]);
}

#[test]
fn phase_correlate_shift_recovers_warp() {
    let base = ints((96, 96, 1), 1000.0, 20000.0, 1);
    // Created with the inverse remap so the estimate recovers the apply shift (+3,-2).
    let mov = shift(&base, -3.0, 2.0);
    let (dx, dy, resp) = phase_correlate_shift(plane(&base).view(), plane(&mov).view(), 1.0, 16);
    assert!(resp > 0.05, "{resp}");
    assert!((dx - 3.0).abs() < 0.5 && (dy + 2.0).abs() < 0.5, "({dx}, {dy})");
}

#[test]
fn phase_correlate_rejects_empty() {
    let e = Array2::<f32>::zeros((0, 0));
    let (dx, dy, resp, _) = phase_correlate(e.view(), e.view(), None, 16);
    assert_eq!((dx, dy, resp), (0.0, 0.0, 0.0));
}

/// Guizar upsample must agree with the 5×5 centroid (same sign, ≤0.15 px).
#[test]
fn guizar_matches_centroid_sign_and_subpixel() {
    let base = ints((96, 96, 1), 10000.0, 50000.0, 21);
    let truth = (1.35, -0.72);
    let mov = shift(&base, -truth.0, -truth.1);
    let (b, m) = (plane(&base), plane(&mov));
    let (dx_c, dy_c, _) = phase_correlate_shift(b.view(), m.view(), 1.0, 1);
    let (dx_g, dy_g, _) = phase_correlate_shift(b.view(), m.view(), 1.0, 16);
    assert!((dx_c - truth.0).abs() < 0.35 && (dy_c - truth.1).abs() < 0.35, "centroid ({dx_c}, {dy_c})");
    assert!(dx_g * dx_c > 0.0 && dy_g * dy_c > 0.0);
    assert!((dx_g - dx_c).abs() < 0.15 && (dy_g - dy_c).abs() < 0.15, "guizar ({dx_g}, {dy_g}) vs ({dx_c}, {dy_c})");
    assert!((dx_g - truth.0).abs() < 0.2 && (dy_g - truth.1).abs() < 0.2, "guizar ({dx_g}, {dy_g})");
}

// --- test_resample.py ---

#[test]
fn integer_warp_exact_overlap() {
    let src = ints((32, 40, 3), 0.0, 65535.0, 3);
    let (dx, dy) = (3i64, -2i64);
    let out = warp_integer(&src, dx, dy);
    // out[y,x] = src[y-dy, x-dx] away from the border fill, which copies from ~2|shift| deep.
    let (y0, y1, x0, x1) = (2 * dy.abs(), 32 - 2 * dy.abs(), 2 * dx.abs(), 40 - 2 * dx.abs());
    let r = |a: i64, b: i64| a as isize..b as isize;
    assert_eq!(out.slice(s![r(y0, y1), r(x0, x1), ..]), src.slice(s![r(y0 - dy, y1 - dy), r(x0 - dx, x1 - dx), ..]));
}

#[test]
fn cubic_preserves_constant_field() {
    let out = warp_cubic(&Array3::from_elem((48, 48, 1), 12345), 1.3, -0.7);
    let inner = out.slice(s![10..40, 10..40, ..]);
    assert!(inner.iter().all(|&v| (12344..=12346).contains(&v)));
}

#[test]
fn cubic_clips_to_uint16_rails() {
    // u16 output cannot leave the rails; the check is that Keys overshoot does not wrap.
    let out = warp_cubic(&Array3::from_elem((32, 32, 1), 65535), 0.5, 0.5);
    assert!(out.slice(s![4..28, 4..28, ..]).iter().all(|&v| v == 65535));
}

#[test]
fn fill_shift_border_uses_interior() {
    let mut out = Array3::<u16>::zeros((10, 10, 1));
    out.slice_mut(s![.., 2.., ..]).fill(100);
    out.slice_mut(s![.., 4, ..]).fill(50); // donor at 2*|dx|
    fill_shift_border(&mut out, 2.0, 0.0);
    assert!(out.slice(s![.., ..2, ..]).iter().all(|&v| v == 50));
}

#[test]
fn warp_row_shifts_identity() {
    let src = ints((64, 32, 1), 0.0, 1000.0, 4);
    assert_eq!(warp_row_shifts(&src, 0.0, &[0.0; 64]), src);
}

#[test]
fn warp_shift_zero_is_identity() {
    let arr = Array3::from_shape_vec((4, 4, 1), (0..16).collect()).unwrap();
    assert_eq!(shift(&arr, 0.0, 0.0), arr);
    assert_eq!(warp_shift(&arr, 0.0, 0.0, Resampler::Linear), arr);
}

/// warp_shift with -S creates the misregistration; the estimate is ≈ S, and applying it aligns.
#[test]
fn register_apply_round_trip_reduces_residual() {
    let reference = ints((256, 320, 3), 2000.0, 40000.0, 9);
    let truth = (4.0, -3.0);
    let mis = shift(&reference, -truth.0, -truth.1);
    let before = luma_residual(reference.view(), mis.view(), 0.05);
    let (aligned, result) = register_and_align(&reference, &mis, RegistrationMode::MultiExposure, &RegistrationOptions::default(), None);
    assert!(result.accepted, "{}", result.reason);
    let (dx, dy) = result.shift();
    assert!((dx - truth.0).abs() < 0.5 && (dy - truth.1).abs() < 0.5, "({dx}, {dy})");
    let after = luma_residual(reference.view(), aligned.view(), 0.05);
    assert!(after < 0.25 * before, "{before} -> {after}");
}

// --- test_pyramid.py ---

#[test]
fn block_mean_all_65535_no_overflow() {
    let out = block_mean(Array3::from_elem((64, 64, 1), 65535).view(), 4);
    assert_eq!(out.dim(), (16, 16));
    assert!(out.iter().all(|&v| (v - 65535.0).abs() < 1e-3));
}

#[test]
fn block_mean_trims_remainder() {
    let img = Array3::from_shape_fn((70, 65, 1), |(y, x, _)| (y * 65 + x) as u16);
    assert_eq!(block_mean(img.view(), 4).dim(), (17, 16));
}

#[test]
fn downsample_plane_to_width_identity_when_small() {
    let (coarse, scale) = downsample_plane_to_width(&Array2::ones((40, 50)), 1024);
    assert_eq!(scale, 1.0);
    assert_eq!(coarse.dim(), (40, 50));
}

// --- test_rgb_ir_seed.py ---

#[test]
fn rgb_ir_seed_axes() {
    let close = |(a, b): (f64, f64), (x, y): (f64, f64)| (a - x).abs() < 1e-9 && (b - y).abs() < 1e-9;
    // Prefers luma when the axes agree.
    assert!(close(rgb_ir_merge_seed_axes(-0.40, -0.12, -0.35, -0.05), (-0.35, -0.05)));
    // 1800 IR case: luma pulled to −1.95, gradient nearer the true −1.5.
    assert!(close(rgb_ir_merge_seed_axes(-1.43, -0.13, -1.95, -0.01), (-1.43, -0.01)));
    // 3600 IR case: luma residual small, gradient still biased.
    assert!(close(rgb_ir_merge_seed_axes(-1.03, 0.09, -0.27, -0.08), (-0.27, -0.08)));
    // Keep the seed axis when the refine grows its magnitude.
    assert!(close(rgb_ir_keep_seed_axes(-1.43, -0.01, -1.90, -0.02), (-1.43, -0.02)));
}

// --- test_rgb_ir_refine.py ---

/// Mono IR from RGB luma through `f`, as HxWx1.
fn ir_from(rgb: &Array3<u16>, f: impl Fn(f64) -> f64) -> Array3<u16> {
    let (h, w, _) = rgb.dim();
    Array3::from_shape_fn((h, w, 1), |(y, x, _)| {
        let luma = (rgb[[y, x, 0]] as f64 + 2.0 * rgb[[y, x, 1]] as f64 + rgb[[y, x, 2]] as f64) * 0.25;
        f(luma).clamp(0.0, 65535.0) as u16
    })
}

/// Tone-mapped mono "IR" with a known fractional shift; refine lands ≤0.15 px.
#[test]
fn rgb_ir_recovers_fractional_y_shift() {
    let rgb = ints((512, 640, 3), 2000.0, 45000.0, 11);
    let ir = ir_from(&rgb, |l| (l / 65535.0).sqrt() * 65535.0);
    let truth = (2.35, -1.40);
    let mis = shift(&ir, -truth.0, -truth.1);
    let result = register(rgb.view(), mis.view(), RegistrationMode::RgbIr, None, &RegistrationOptions::default(), None);
    assert!(result.accepted, "{}", result.reason);
    let (dx, dy) = result.shift();
    assert!((dx - truth.0).abs() <= 0.15 && (dy - truth.1).abs() <= 0.15, "({dx}, {dy})");
}

#[test]
fn rgb_ir_near_zero_on_aligned_pair() {
    // Large enough that refine runs (the engine skips refine below 512²).
    let rgb = ints((640, 768, 3), 3000.0, 40000.0, 12);
    let ir = ir_from(&rgb, |l| l * 0.9 + 1000.0);
    let result = register(rgb.view(), ir.view(), RegistrationMode::RgbIr, None, &RegistrationOptions::default(), None);
    assert!(result.accepted, "{}", result.reason);
    let (dx, dy) = result.shift();
    assert!(dx.abs() <= 0.15 && dy.abs() <= 0.15, "({dx}, {dy})");
}

// --- test_synthetic.py ---

#[test]
fn estimate_pass_shift_recovers_known_translation() {
    let base = ints((96, 96, 3), 1000.0, 20000.0, 1);
    let (dx, dy) = estimate_pass_shift(&base, &shift(&base, -3.0, 2.0));
    assert!((dx - 3.0).abs() < 0.6 && (dy + 2.0).abs() < 0.6, "({dx}, {dy})");
}

#[test]
fn register_rejects_featureless() {
    let blank = Array3::from_elem((128, 128, 3), 1000u16);
    let result = register(blank.view(), blank.view(), RegistrationMode::MultiExposure, None, &RegistrationOptions::default(), None);
    assert!(!result.accepted || result.shift() == (0.0, 0.0));
}

#[test]
fn register_and_align_me_gain_change() {
    let short = ints((128, 128, 3), 2000.0, 12000.0, 7);
    let long = shift(&short.mapv(|v| (v as f64 * 3.0).min(65535.0) as u16), -2.0, -1.0);
    let (aligned, result) = register_and_align(&short, &long, RegistrationMode::MultiExposure, &RegistrationOptions::default(), None);
    assert!(result.accepted, "{}", result.reason);
    let (dx, dy) = result.shift();
    assert!((dx - 2.0).abs() < 0.75 && (dy - 1.0).abs() < 0.75, "({dx}, {dy})");
    assert_eq!(aligned.dim(), short.dim());
}

fn residual(a: ArrayView3<u16>, b: ArrayView3<u16>) -> f64 {
    a.iter().zip(b.iter()).map(|(&x, &y)| (x as f64 - y as f64).abs()).sum::<f64>() / a.len() as f64
}

#[test]
fn banded_beats_whole_frame_on_progressive_drift() {
    let (h, w) = (2400, 300);
    let reference = ints((h, w, 3), 1000.0, 40000.0, 5);
    let true_dy: Array1<f64> = Array1::linspace(0.0, 40.0, h);
    let moving = Array3::from_shape_fn((h, w, 3), |(y, x, c)| {
        let src = (y as f64 - true_dy[y].round_ties_even()).clamp(0.0, (h - 1) as f64) as usize;
        reference[[src, x, c]]
    });
    let (warped, (dx, dy_center)) = align_pass_to_reference_banded(&reference, &moving).unwrap();
    assert!(dx.abs() < 1.0, "{dx}");
    assert!((dy_center.abs() - 20.0).abs() < 3.0, "{dy_center}");
    let (whole, _) = align_pass_to_reference(&reference, &moving, None).unwrap();
    let band = 200;
    for (y0, y1) in [(0, band), (h - band, h)] {
        let r = |img: &Array3<u16>| residual(reference.slice(s![y0..y1, .., ..]), img.slice(s![y0..y1, .., ..]));
        assert!(r(&warped) < 0.75 * r(&whole), "rows {y0}..{y1}");
    }
}
