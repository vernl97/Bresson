// SPDX-License-Identifier: GPL-3.0-or-later
//! Native pass registration (ME, Multi-Pass, RGB↔IR): windowed phase
//! correlation, Guizar-Sicairos subpixel refinement, banded Y profiles and
//! integer-exact / separable warps.
//!
//! Images are `HxWxC` uint16 (`C = 3` for RGB, `C = 1` for an IR plane).

use std::f64::consts::PI;

use log::{info, warn};
use ndarray::{Array2, Array3, ArrayView2, ArrayView3, s};
use num_complex::Complex;
use realfft::RealFftPlanner;
use rustfft::FftPlanner;

use crate::num::{lstsq_line, median, py_round};

// ---------------------------------------------------------------- types

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegistrationMode {
    MultiExposure,
    MultiPass,
    RgbIr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegistrationFailure {
    None,
    LowTexture,
    AmbiguousPeak,
    InsufficientOverlap,
    RegionalDisagreement,
    OutsideScannerPrior,
    CrossModalMismatch,
    NonTranslationalMotion,
    ResidualNotImproved,
    ShapeMismatch,
    PathologicalShift,
}

impl RegistrationFailure {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::LowTexture => "low_texture",
            Self::AmbiguousPeak => "ambiguous_peak",
            Self::InsufficientOverlap => "insufficient_overlap",
            Self::RegionalDisagreement => "regional_disagreement",
            Self::OutsideScannerPrior => "outside_scanner_prior",
            Self::CrossModalMismatch => "cross_modal_mismatch",
            Self::NonTranslationalMotion => "non_translational_motion",
            Self::ResidualNotImproved => "residual_not_improved",
            Self::ShapeMismatch => "shape_mismatch",
            Self::PathologicalShift => "pathlogical_shift",
        }
    }
}

/// Maps moving-image coordinates into the reference coordinate system.
#[derive(Debug, Clone, PartialEq)]
pub enum Transform {
    Translation {
        dx: f64,
        dy: f64,
    },
    /// Constant dx with a per-row dy profile (Multi-Pass banded path).
    LineWarp {
        dx: f64,
        dy_per_row: Vec<f64>,
    },
}

impl Transform {
    pub fn shift(&self) -> (f64, f64) {
        match self {
            Transform::Translation { dx, dy } => (*dx, *dy),
            Transform::LineWarp { dx, dy_per_row } => (
                *dx,
                dy_per_row.get(dy_per_row.len() / 2).copied().unwrap_or(0.0),
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RegionalShift {
    pub center_x: f64,
    pub center_y: f64,
    pub dx: f64,
    pub dy: f64,
    pub response: f64,
    pub accepted: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct RegistrationQuality {
    pub response: f64,
    pub peak_ratio: f64,
    pub regional_inliers: usize,
    pub regional_total: usize,
    pub regional_mad_px: f64,
    pub residual_before: f64,
    pub residual_after: f64,
    pub overlap_fraction: f64,
    pub prior_distance: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegistrationResult {
    pub transform: Transform,
    pub mode: RegistrationMode,
    pub accepted: bool,
    pub integer_exact: bool,
    pub quality: RegistrationQuality,
    pub regions: Vec<RegionalShift>,
    pub reason: String,
    pub failure: RegistrationFailure,
}

impl RegistrationResult {
    pub fn shift(&self) -> (f64, f64) {
        self.transform.shift()
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct RegistrationContext {
    pub model: Option<String>,
    pub dpi: Option<i64>,
    pub max_abs_dx: Option<f64>,
    pub max_abs_dy: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegistrationOptions {
    pub coarse_target_px: usize,
    pub refine_roi_side: usize,
    pub subpixel_factor: usize,
    pub max_candidates: usize,
    pub min_response: f64,
    pub sanity_max_frac: f64,
    pub refine_max_residual: f64,
    pub enable_subpixel: bool,
    pub guizar_max_roi_side: usize,
    pub enable_guizar_refine: bool,
    pub enable_regional: bool,
    pub snap_frac_eps: f64,
    pub band_count: usize,
    pub band_min_height: usize,
    pub band_outlier_px: f64,
    pub fractional_resampler: Resampler,
    pub warp_chunk_rows: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resampler {
    Cubic,
    Linear,
}

impl Default for RegistrationOptions {
    fn default() -> Self {
        Self {
            coarse_target_px: 1024,
            refine_roi_side: 512,
            subpixel_factor: 16,
            max_candidates: 5,
            min_response: 0.03,
            sanity_max_frac: 0.5,
            refine_max_residual: 2.0,
            enable_subpixel: true,
            guizar_max_roi_side: 512,
            enable_guizar_refine: true,
            enable_regional: false,
            snap_frac_eps: 0.08,
            band_count: 8,
            band_min_height: 256,
            band_outlier_px: 8.0,
            fractional_resampler: Resampler::Cubic,
            warp_chunk_rows: 512,
        }
    }
}

// ---------------------------------------------------------------- phase

/// Smallest `2^a 3^b 5^c` at least `n` (FFT-friendly pad length).
pub fn optimal_dft_size(n: usize) -> usize {
    let mut m = n.max(1);
    loop {
        let mut k = m;
        for p in [2, 3, 5] {
            while k % p == 0 {
                k /= p;
            }
        }
        if k == 1 {
            return m;
        }
        m += 1;
    }
}

/// Sqrt of separable Hann window, shape `(h, w)`, float32.
pub fn hanning_window(h: usize, w: usize) -> Array2<f32> {
    if h < 2 || w < 2 {
        return Array2::ones((h.max(1), w.max(1)));
    }
    let tau = 2.0 * PI;
    let c0 = tau / (w - 1) as f64;
    let c1 = tau / (h - 1) as f64;
    let wc: Vec<f64> = (0..w)
        .map(|i| 0.5 * (1.0 - (c0 * i as f64).cos()))
        .collect();
    let wr: Vec<f64> = (0..h)
        .map(|i| 0.5 * (1.0 - (c1 * i as f64).cos()))
        .collect();
    Array2::from_shape_fn((h, w), |(y, x)| (wr[y] * wc[x]).sqrt() as f32)
}

/// fftShift, 5×5 weighted centroid → `(dx, dy, response, peak_ratio)`.
fn fftshift_peak_coords(corr: &Array2<f32>) -> (f64, f64, f64, f64) {
    let (m, n) = corr.dim();
    let (ym, xm) = (m / 2, n / 2);
    // shifted[i, j] = corr[(i + m - m/2) % m, (j + n - n/2) % n]
    let sh = |i: usize, j: usize| corr[[(i + m - ym) % m, (j + n - xm) % n]];
    let (mut py, mut px, mut best) = (0, 0, f32::NEG_INFINITY);
    for i in 0..m {
        for j in 0..n {
            let v = sh(i, j);
            if v > best {
                best = v;
                py = i;
                px = j;
            }
        }
    }
    let (y0, y1) = (py.saturating_sub(2), (m - 1).min(py + 2));
    let (x0, x1) = (px.saturating_sub(2), (n - 1).min(px + 2));
    let (mut wsum, mut cxn, mut cyn) = (0.0f64, 0.0f64, 0.0f64);
    for y in y0..=y1 {
        for x in x0..=x1 {
            let v = sh(y, x) as f64;
            wsum += v;
            cxn += x as f64 * v;
            cyn += y as f64 * v;
        }
    }
    let weight_sum = wsum + f64::EPSILON;
    let cx = cxn / weight_sum;
    let cy = cyn / weight_sum;
    let response = weight_sum / (m * n) as f64;
    let mut second = f32::NEG_INFINITY;
    for i in 0..m {
        for j in 0..n {
            let masked = i + 1 >= py && i <= py + 1 && j + 1 >= px && j <= px + 1;
            if !masked {
                second = second.max(sh(i, j));
            }
        }
    }
    let first = sh(py, px) as f64;
    let second = second as f64;
    let peak_ratio = if first != 0.0 {
        (first - second) / (first.abs() + 1e-12)
    } else {
        0.0
    };
    (xm as f64 - cx, ym as f64 - cy, response, peak_ratio)
}

/// Real 2-D forward FFT of an `m×n` float32 plane → `m×(n/2+1)` complex64.
fn rfft2(a: &Array2<f32>) -> Array2<Complex<f32>> {
    let (m, n) = a.dim();
    let nc = n / 2 + 1;
    let mut rp = RealFftPlanner::<f32>::new();
    let r2c = rp.plan_fft_forward(n);
    let mut out = Array2::<Complex<f32>>::zeros((m, nc));
    let mut row_in = r2c.make_input_vec();
    let mut row_out = r2c.make_output_vec();
    for y in 0..m {
        row_in.copy_from_slice(a.row(y).as_slice().expect("contiguous"));
        r2c.process(&mut row_in, &mut row_out).expect("rfft");
        out.row_mut(y)
            .assign(&ndarray::ArrayView1::from(&row_out[..]));
    }
    let mut cp = FftPlanner::<f32>::new();
    let col = cp.plan_fft_forward(m);
    let mut buf = vec![Complex::new(0.0, 0.0); m];
    for x in 0..nc {
        for y in 0..m {
            buf[y] = out[[y, x]];
        }
        col.process(&mut buf);
        for y in 0..m {
            out[[y, x]] = buf[y];
        }
    }
    out
}

/// Inverse of [`rfft2`] with numpy `norm="forward"` (no 1/N scaling).
fn irfft2_forward_norm(spec: &Array2<Complex<f32>>, m: usize, n: usize) -> Array2<f32> {
    let nc = n / 2 + 1;
    let mut work = spec.clone();
    let mut cp = FftPlanner::<f32>::new();
    let col = cp.plan_fft_inverse(m);
    let mut buf = vec![Complex::new(0.0, 0.0); m];
    for x in 0..nc {
        for y in 0..m {
            buf[y] = work[[y, x]];
        }
        col.process(&mut buf);
        for y in 0..m {
            work[[y, x]] = buf[y];
        }
    }
    let mut rp = RealFftPlanner::<f32>::new();
    let c2r = rp.plan_fft_inverse(n);
    let mut out = Array2::<f32>::zeros((m, n));
    let mut row_in = c2r.make_input_vec();
    let mut row_out = c2r.make_output_vec();
    for y in 0..m {
        for x in 0..nc {
            row_in[x] = work[[y, x]];
        }
        // numpy discards the imaginary part of DC / Nyquist bins.
        row_in[0].im = 0.0;
        if n % 2 == 0 {
            row_in[nc - 1].im = 0.0;
        }
        c2r.process(&mut row_in, &mut row_out).expect("irfft");
        out.row_mut(y)
            .assign(&ndarray::ArrayView1::from(&row_out[..]));
    }
    out
}

/// Full complex 2-D FFT (float64) of a real plane.
fn fft2_f64(a: &Array2<f32>) -> Array2<Complex<f64>> {
    let (m, n) = a.dim();
    let mut out = a.mapv(|v| Complex::new(v as f64, 0.0));
    let mut p = FftPlanner::<f64>::new();
    let rows = p.plan_fft_forward(n);
    for mut r in out.rows_mut() {
        let mut v = r.to_vec();
        rows.process(&mut v);
        r.assign(&ndarray::ArrayView1::from(&v[..]));
    }
    let cols = p.plan_fft_forward(m);
    for mut c in out.columns_mut() {
        let mut v = c.to_vec();
        cols.process(&mut v);
        c.assign(&ndarray::ArrayView1::from(&v[..]));
    }
    out
}

/// Refine an integer correlation peak to a `1/upsample_factor` grid (Guizar-Sicairos).
pub fn refine_peak_guizar(
    cross: &Array2<Complex<f64>>,
    peak_y: i64,
    peak_x: i64,
    upsample_factor: usize,
    upsample_region: Option<f64>,
) -> (f64, f64) {
    let u = upsample_factor.max(1);
    if u == 1 {
        return (peak_x as f64, peak_y as f64);
    }
    let (h, w) = cross.dim();
    let uf = u as f64;
    let side = upsample_region.unwrap_or(uf * 1.5);
    let nout = ((side * 2.0).ceil() as usize) | 1;
    let dftshift = (nout as f64 / 2.0).trunc();
    let r_offset = dftshift - peak_y as f64 * uf;
    let c_offset = dftshift - peak_x as f64 * uf;
    let freq = |k: usize, len: usize| {
        if k >= len.div_ceil(2) {
            k as f64 - len as f64
        } else {
            k as f64
        }
    };
    let ys: Vec<f64> = (0..nout).map(|r| (r as f64 - r_offset) / uf).collect();
    let xs: Vec<f64> = (0..nout).map(|c| (c as f64 - c_offset) / uf).collect();
    let kern_y = Array2::from_shape_fn((nout, h), |(r, fy)| {
        Complex::from_polar(1.0, 2.0 * PI * ys[r] * freq(fy, h) / h as f64)
    });
    let kern_x = Array2::from_shape_fn((w, nout), |(fx, c)| {
        Complex::from_polar(1.0, 2.0 * PI * freq(fx, w) * xs[c] / w as f64)
    });
    let patch = kern_y.dot(cross).dot(&kern_x);
    let (mut pr, mut pc, mut best) = (0, 0, f64::NEG_INFINITY);
    for r in 0..nout {
        for c in 0..nout {
            if patch[[r, c]].re > best {
                best = patch[[r, c]].re;
                pr = r;
                pc = c;
            }
        }
    }
    let mut dy = (pr as f64 - dftshift) / uf + peak_y as f64;
    let mut dx = (pc as f64 - dftshift) / uf + peak_x as f64;
    let (wf, hf) = (w as f64, h as f64);
    if dx >= wf / 2.0 {
        dx -= wf;
    } else if dx < -wf / 2.0 {
        dx += wf;
    }
    if dy >= hf / 2.0 {
        dy -= hf;
    } else if dy < -hf / 2.0 {
        dy += hf;
    }
    (dx, dy)
}

/// Phase correlate `a` vs `b` → `(dx, dy, response, peak_ratio)` (shift of `b` relative to `a`).
pub fn phase_correlate(
    a: ArrayView2<f32>,
    b: ArrayView2<f32>,
    window: Option<&Array2<f32>>,
    upsample: usize,
) -> (f64, f64, f64, f64) {
    if a.dim() != b.dim() {
        return (0.0, 0.0, 0.0, 0.0);
    }
    let (h, w) = a.dim();
    if h < 2 || w < 2 {
        return (0.0, 0.0, 0.0, 0.0);
    }
    let own;
    let win = match window {
        Some(wn) => wn,
        None => {
            own = hanning_window(h, w);
            &own
        }
    };
    let (m, n) = (optimal_dft_size(h), optimal_dft_size(w));
    let mut wa = Array2::<f32>::zeros((m, n));
    let mut wb = Array2::<f32>::zeros((m, n));
    for y in 0..h {
        for x in 0..w {
            wa[[y, x]] = a[[y, x]] * win[[y, x]];
            wb[[y, x]] = b[[y, x]] * win[[y, x]];
        }
    }
    let fa = rfft2(&wa);
    let fb = rfft2(&wb);
    let mut cross = Array2::from_shape_fn(fa.dim(), |p| fa[p] * fb[p].conj());
    let mag = cross.mapv(|c| c.norm());
    let max_mag = mag.iter().cloned().fold(0.0f32, f32::max);
    let eps = f32::EPSILON * max_mag.max(1.0);
    for (c, &mg) in cross.iter_mut().zip(mag.iter()) {
        *c = if mg > eps {
            *c / mg
        } else {
            Complex::new(0.0, 0.0)
        };
    }
    let corr = irfft2_forward_norm(&cross, m, n);
    let (mut dx, mut dy, response, peak_ratio) = fftshift_peak_coords(&corr);
    if upsample > 1 {
        let (mut py, mut px, mut best) = (0, 0, f32::NEG_INFINITY);
        for ((y, x), &v) in corr.indexed_iter() {
            if v > best {
                best = v;
                py = y;
                px = x;
            }
        }
        let ix = if (px as f64) < n as f64 / 2.0 {
            px as i64
        } else {
            px as i64 - n as i64
        };
        let iy = if (py as f64) < m as f64 / 2.0 {
            py as i64
        } else {
            py as i64 - m as i64
        };
        let fa2 = fft2_f64(&wa);
        let fb2 = fft2_f64(&wb);
        let cross_full = Array2::from_shape_fn(fa2.dim(), |p| fa2[p] * fb2[p].conj());
        let (rdx, rdy) = refine_peak_guizar(&cross_full, iy, ix, upsample, None);
        dx = -rdx;
        dy = -rdy;
    }
    (dx, dy, response, peak_ratio)
}

/// Pass-align convention: `(dx, dy, response)` for `moving` → `reference`.
pub fn phase_correlate_shift(
    reference: ArrayView2<f32>,
    moving: ArrayView2<f32>,
    scale: f64,
    upsample: usize,
) -> (f64, f64, f64) {
    let (dx, dy, resp, _) = phase_correlate(moving, reference, None, upsample);
    (dx * scale, dy * scale, resp)
}

// ---------------------------------------------------------------- pyramid / planes

/// Average non-overlapping `factor×factor` blocks (and channels) to float32.
pub fn block_mean(image: ArrayView3<u16>, factor: usize) -> Array2<f32> {
    let (h0, w0, c) = image.dim();
    if factor <= 1 {
        return mono_f32(image);
    }
    let (h, w) = ((h0 / factor) * factor, (w0 / factor) * factor);
    if h == 0 || w == 0 {
        return mono_f32(image);
    }
    let count = (factor * factor * c) as f64;
    Array2::from_shape_fn((h / factor, w / factor), |(y, x)| {
        let mut s = 0.0f64;
        for dy in 0..factor {
            for dx in 0..factor {
                for k in 0..c {
                    s += image[[y * factor + dy, x * factor + dx, k]] as f64;
                }
            }
        }
        (s / count) as f32
    })
}

/// 2-D float block mean (`block_mean` on a plane).
fn block_mean_plane(plane: &Array2<f32>, factor: usize) -> Array2<f32> {
    let (h0, w0) = plane.dim();
    let (h, w) = ((h0 / factor) * factor, (w0 / factor) * factor);
    if factor <= 1 || h == 0 || w == 0 {
        return plane.clone();
    }
    let area = (factor * factor) as f64;
    Array2::from_shape_fn((h / factor, w / factor), |(y, x)| {
        let mut s = 0.0f64;
        for dy in 0..factor {
            for dx in 0..factor {
                s += plane[[y * factor + dy, x * factor + dx]] as f64;
            }
        }
        (s / area) as f32
    })
}

/// Single-channel plane as float32 (`C == 1`) or channel mean otherwise.
fn mono_f32(image: ArrayView3<u16>) -> Array2<f32> {
    let (h, w, c) = image.dim();
    if c == 1 {
        return Array2::from_shape_fn((h, w), |(y, x)| image[[y, x, 0]] as f32);
    }
    Array2::from_shape_fn((h, w), |(y, x)| {
        (0..c).map(|k| image[[y, x, k]] as f32).sum::<f32>() / c as f32
    })
}

fn coarse_factor(full_w: usize, target_w: usize) -> usize {
    let mut factor = if full_w > target_w {
        (py_round(full_w as f64 / target_w as f64)).max(1) as usize
    } else {
        1
    };
    while factor > 1 && (full_w / factor) < 32.max(target_w / 4) {
        factor -= 1;
    }
    factor
}

/// Like `downsample_to_width` for a 2-D plane; returns `(plane, full_w / coarse_w)`.
pub fn downsample_plane_to_width(plane: &Array2<f32>, target_w: usize) -> (Array2<f32>, f64) {
    let w = plane.ncols();
    if w <= target_w {
        return (plane.clone(), 1.0);
    }
    let factor = coarse_factor(w, target_w);
    let coarse = block_mean_plane(plane, factor);
    let scale = w as f64 / coarse.ncols() as f64;
    (coarse, scale)
}

/// `(R + 2G + B) / 4` as float32 (mono planes pass through).
pub fn structural_luminance(image: ArrayView3<u16>) -> Array2<f32> {
    let (h, w, c) = image.dim();
    if c == 1 {
        return mono_f32(image);
    }
    Array2::from_shape_fn((h, w), |(y, x)| {
        let (r, g, b) = (
            image[[y, x, 0]] as f32,
            image[[y, x, 1]] as f32,
            image[[y, x, 2]] as f32,
        );
        (r + 2.0 * g + b) * 0.25
    })
}

/// Centered-difference gradient magnitude, zero border.
fn gradient_magnitude(p: &Array2<f32>) -> Array2<f32> {
    let (h, w) = p.dim();
    if h < 3 || w < 3 {
        return p.mapv(f32::abs);
    }
    let mut out = Array2::<f32>::zeros((h, w));
    for y in 1..h - 1 {
        for x in 1..w - 1 {
            let gx = p[[y, x + 1]] - p[[y, x - 1]];
            let gy = p[[y + 1, x]] - p[[y - 1, x]];
            out[[y, x]] = gx.hypot(gy);
        }
    }
    out
}

/// Max channel gradient magnitude for RGB↔IR structural matching.
pub fn rgb_edge_energy(rgb: ArrayView3<u16>) -> Array2<f32> {
    let (h, w, c) = rgb.dim();
    let mut out = Array2::<f32>::from_elem((h, w), f32::NEG_INFINITY);
    for k in 0..c {
        let g = gradient_magnitude(&Array2::from_shape_fn((h, w), |(y, x)| {
            rgb[[y, x, k]] as f32
        }));
        out.zip_mut_with(&g, |o, &v| *o = o.max(v));
    }
    out
}

/// True where both frames have usable (non-clipped) content.
pub fn clip_mask(
    reference: ArrayView3<u16>,
    moving: ArrayView3<u16>,
    high: f64,
    low: f64,
) -> Array2<bool> {
    let r = structural_luminance(reference);
    let m = structural_luminance(moving);
    let (hi, lo) = (high as f32, low as f32);
    Array2::from_shape_fn(r.dim(), |p| {
        r[p] < hi && m[p] < hi && r[p] > lo && m[p] > lo
    })
}

fn crop2(a: &Array2<f32>, h: usize, w: usize) -> Array2<f32> {
    a.slice(s![..h, ..w]).to_owned()
}

/// Downsample first, then form registration planes. Returns `(ref, mov, scale)`.
pub fn coarse_registration_pair(
    reference: ArrayView3<u16>,
    moving: ArrayView3<u16>,
    mode: RegistrationMode,
    target_w: usize,
) -> (Array2<f32>, Array2<f32>, f64) {
    let full_w = reference.dim().1;
    let factor = coarse_factor(full_w, target_w);
    if mode == RegistrationMode::RgbIr {
        let (rc, mc) = if factor <= 1 {
            (
                structural_luminance(reference),
                structural_luminance(moving),
            )
        } else {
            (block_mean(reference, factor), block_mean(moving, factor))
        };
        let h = rc.nrows().min(mc.nrows());
        let w = rc.ncols().min(mc.ncols());
        let scale = if w > 0 { full_w as f64 / w as f64 } else { 1.0 };
        return (
            gradient_magnitude(&crop2(&rc, h, w)),
            gradient_magnitude(&crop2(&mc, h, w)),
            scale,
        );
    }
    let (rc, mc, scale) = if factor <= 1 {
        (
            structural_luminance(reference),
            structural_luminance(moving),
            1.0,
        )
    } else {
        let rc = block_mean(reference, factor);
        let mc = block_mean(moving, factor);
        let sc = full_w as f64 / rc.ncols() as f64;
        (rc, mc, sc)
    };
    let h = rc.nrows().min(mc.nrows());
    let w = rc.ncols().min(mc.ncols());
    let (mut rc, mut mc) = (crop2(&rc, h, w), crop2(&mc, h, w));
    if mode == RegistrationMode::MultiExposure {
        let (high, low) = ((0.98 * 65535.0) as f32, 200.0f32);
        for (r, m) in rc.iter_mut().zip(mc.iter_mut()) {
            let ok = *r < high && *m < high && *r > low && *m > low;
            let k = if ok { 1.0 } else { 0.0 };
            *r *= k;
            *m *= k;
        }
    }
    (rc, mc, scale)
}

/// Block-mean structural luma pair at the same scale as RGB↔IR coarse.
pub fn coarse_luma_registration_pair(
    reference: ArrayView3<u16>,
    moving: ArrayView3<u16>,
    target_w: usize,
) -> (Array2<f32>, Array2<f32>, f64) {
    let full_w = reference.dim().1;
    let factor = coarse_factor(full_w, target_w);
    let (rc, mc, scale) = if factor <= 1 {
        (
            structural_luminance(reference),
            structural_luminance(moving),
            1.0,
        )
    } else {
        let rc = block_mean(reference, factor);
        let mc = block_mean(moving, factor);
        let sc = full_w as f64 / rc.ncols() as f64;
        (rc, mc, sc)
    };
    let h = rc.nrows().min(mc.nrows());
    let w = rc.ncols().min(mc.ncols());
    (crop2(&rc, h, w), crop2(&mc, h, w), scale)
}

// ---------------------------------------------------------------- resample

/// Gather: `out[y, x] = arr[y + sy, x + sx]` on in-bounds overlap, zeros elsewhere.
fn ishift(arr: ArrayView3<u16>, sx: i64, sy: i64) -> Array3<u16> {
    let (h, w, c) = arr.dim();
    let (hi, wi) = (h as i64, w as i64);
    let mut out = Array3::<u16>::zeros((h, w, c));
    let (y0, y1) = (0.max(-sy), hi.min(hi - sy));
    let (x0, x1) = (0.max(-sx), wi.min(wi - sx));
    if y1 > y0 && x1 > x0 {
        out.slice_mut(s![y0 as usize..y1 as usize, x0 as usize..x1 as usize, ..])
            .assign(&arr.slice(s![
                (y0 + sy) as usize..(y1 + sy) as usize,
                (x0 + sx) as usize..(x1 + sx) as usize,
                ..
            ]));
    }
    out
}

/// Replace OOB strips with deeper interior (IR-safe; no edge-column widen).
pub fn fill_shift_border(out: &mut Array3<u16>, dx: f64, dy: f64) {
    let (h, w, _) = out.dim();
    let bx = dx.abs().ceil() as usize;
    let by = dy.abs().ceil() as usize;
    if bx > 0 && bx < w {
        if dx > 0.0 {
            let src = (w - 1).min(2 * bx);
            let col = out.slice(s![.., src..src + 1, ..]).to_owned();
            for x in 0..bx {
                out.slice_mut(s![.., x..x + 1, ..]).assign(&col);
            }
        } else if dx < 0.0 {
            let src = (w as i64 - 1 - 2 * bx as i64).max(0) as usize;
            let col = out.slice(s![.., src..src + 1, ..]).to_owned();
            for x in w - bx..w {
                out.slice_mut(s![.., x..x + 1, ..]).assign(&col);
            }
        }
    }
    if by > 0 && by < h {
        if dy > 0.0 {
            let src = (h - 1).min(2 * by);
            let row = out.slice(s![src..src + 1, .., ..]).to_owned();
            for y in 0..by {
                out.slice_mut(s![y..y + 1, .., ..]).assign(&row);
            }
        } else if dy < 0.0 {
            let src = (h as i64 - 1 - 2 * by as i64).max(0) as usize;
            let row = out.slice(s![src..src + 1, .., ..]).to_owned();
            for y in h - by..h {
                out.slice_mut(s![y..y + 1, .., ..]).assign(&row);
            }
        }
    }
}

/// Per-row generalization of [`fill_shift_border`].
pub fn fill_row_shift_border(out: &mut Array3<u16>, dx: f64, dy_per_row: &[f64]) {
    let (h, w, _) = out.dim();
    let bx = dx.abs().ceil() as usize;
    if bx > 0 && bx < w {
        if dx > 0.0 {
            let src = (w - 1).min(2 * bx);
            let col = out.slice(s![.., src..src + 1, ..]).to_owned();
            for x in 0..bx {
                out.slice_mut(s![.., x..x + 1, ..]).assign(&col);
            }
        } else {
            let src = (w as i64 - 1 - 2 * bx as i64).max(0) as usize;
            let col = out.slice(s![.., src..src + 1, ..]).to_owned();
            for x in w - bx..w {
                out.slice_mut(s![.., x..x + 1, ..]).assign(&col);
            }
        }
    }
    let valid: Vec<usize> = (0..h)
        .filter(|&y| {
            let sy = y as f64 - dy_per_row[y];
            sy >= 0.0 && sy <= (h - 1) as f64
        })
        .collect();
    if !valid.is_empty() && valid.len() < h {
        let (first, last) = (valid[0], *valid.last().unwrap());
        if first > 0 {
            let row = out.slice(s![first..first + 1, .., ..]).to_owned();
            for y in 0..first {
                out.slice_mut(s![y..y + 1, .., ..]).assign(&row);
            }
        }
        if last < h - 1 {
            let row = out.slice(s![last..last + 1, .., ..]).to_owned();
            for y in last + 1..h {
                out.slice_mut(s![y..y + 1, .., ..]).assign(&row);
            }
        }
    }
}

/// Integer remap translate: `out[y, x] = src[y - dy, x - dx]`.
pub fn warp_integer(mov: &Array3<u16>, dx: i64, dy: i64) -> Array3<u16> {
    if dx == 0 && dy == 0 {
        return mov.clone();
    }
    let mut out = ishift(mov.view(), -dx, -dy);
    fill_shift_border(&mut out, dx as f64, dy as f64);
    out
}

fn keys_cubic_weights(t: f64) -> [f64; 4] {
    let a = -0.5;
    let (t2, t3) = (t * t, t * t * t);
    [
        a * (t3 - 2.0 * t2 + t),
        (a + 2.0) * t3 - (a + 3.0) * t2 + 1.0,
        -(a + 2.0) * t3 + (2.0 * a + 3.0) * t2 - a * t,
        a * (-t3 + t2),
    ]
}

fn finalize(out: Array3<f32>, dx: f64, dy: f64) -> Array3<u16> {
    let mut o = out.mapv(|v| v.round_ties_even().clamp(0.0, 65535.0) as u16);
    fill_shift_border(&mut o, dx, dy);
    o
}

/// Separable Keys cubic remap: `out = src(x - dx, y - dy)`.
pub fn warp_cubic(mov: &Array3<u16>, dx: f64, dy: f64) -> Array3<u16> {
    if dx.abs() < 1e-9 && dy.abs() < 1e-9 {
        return mov.clone();
    }
    if (dx - dx.round_ties_even()).abs() < 1e-6 && (dy - dy.round_ties_even()).abs() < 1e-6 {
        return warp_integer(mov, py_round(dx), py_round(dy));
    }
    let (h, w, c) = mov.dim();
    let (sx, sy) = (-dx, -dy);
    let taps = |shift: f64, n: usize| -> (Vec<[usize; 4]>, [f32; 4]) {
        let base = shift.floor() as i64;
        let k = keys_cubic_weights(shift - base as f64).map(|v| v as f32);
        let idx = (0..n as i64)
            .map(|i| [-1, 0, 1, 2].map(|o| (i + base + o).clamp(0, n as i64 - 1) as usize))
            .collect();
        (idx, k)
    };
    let mut tmp = mov.mapv(|v| v as f32);
    if sx.abs() >= 1e-12 {
        let (ix, k) = taps(sx, w);
        let src = tmp.clone();
        for y in 0..h {
            for x in 0..w {
                for ch in 0..c {
                    let t = |j: usize| src[[y, ix[x][j], ch]];
                    tmp[[y, x, ch]] = k[0] * t(0) + k[1] * t(1) + k[2] * t(2) + k[3] * t(3);
                }
            }
        }
    }
    if sy.abs() >= 1e-12 {
        let (iy, k) = taps(sy, h);
        let src = tmp.clone();
        for y in 0..h {
            for x in 0..w {
                for ch in 0..c {
                    let t = |j: usize| src[[iy[y][j], x, ch]];
                    tmp[[y, x, ch]] = k[0] * t(0) + k[1] * t(1) + k[2] * t(2) + k[3] * t(3);
                }
            }
        }
    }
    finalize(tmp, dx, dy)
}

/// Bilinear remap: `out[y, x] = src[y - dy, x - dx]`.
pub fn warp_linear(mov: &Array3<u16>, dx: f64, dy: f64) -> Array3<u16> {
    if !dx.is_finite() || !dy.is_finite() || (dx.abs() < 1e-9 && dy.abs() < 1e-9) {
        return mov.clone();
    }
    if (dx - dx.round_ties_even()).abs() < 1e-6 && (dy - dy.round_ties_even()).abs() < 1e-6 {
        return warp_integer(mov, py_round(dx), py_round(dy));
    }
    let (gx, gy) = (-dx, -dy);
    let (ix, iy) = (gx.floor() as i64, gy.floor() as i64);
    let (fx, fy) = (gx - ix as f64, gy - iy as f64);
    let c00 = ishift(mov.view(), ix, iy).mapv(|v| v as f32);
    let out: Array3<f32> = if fx.abs() < 1e-6 && fy.abs() < 1e-6 {
        c00
    } else if fx.abs() < 1e-6 {
        let c01 = ishift(mov.view(), ix, iy + 1);
        let (w0, w1) = ((1.0 - fy) as f32, fy as f32);
        Array3::from_shape_fn(c00.dim(), |p| w0 * c00[p] + w1 * c01[p] as f32)
    } else if fy.abs() < 1e-6 {
        let c10 = ishift(mov.view(), ix + 1, iy);
        let (w0, w1) = ((1.0 - fx) as f32, fx as f32);
        Array3::from_shape_fn(c00.dim(), |p| w0 * c00[p] + w1 * c10[p] as f32)
    } else {
        let c10 = ishift(mov.view(), ix + 1, iy);
        let c01 = ishift(mov.view(), ix, iy + 1);
        let c11 = ishift(mov.view(), ix + 1, iy + 1);
        let w00 = ((1.0 - fx) * (1.0 - fy)) as f32;
        let w10 = (fx * (1.0 - fy)) as f32;
        let w01 = ((1.0 - fx) * fy) as f32;
        let w11 = (fx * fy) as f32;
        Array3::from_shape_fn(c00.dim(), |p| {
            w00 * c00[p] + w10 * c10[p] as f32 + w01 * c01[p] as f32 + w11 * c11[p] as f32
        })
    };
    finalize(out, dx, dy)
}

/// Remap-translate `mov` by `(dx, dy)` — linear or cubic.
pub fn warp_shift(mov: &Array3<u16>, dx: f64, dy: f64, method: Resampler) -> Array3<u16> {
    match method {
        Resampler::Cubic => warp_cubic(mov, dx, dy),
        Resampler::Linear => warp_linear(mov, dx, dy),
    }
}

/// Per-row dy + constant dx (remap `dst(x,y) = src(x - dx, y - dy[y])`).
pub fn warp_row_shifts(mov: &Array3<u16>, dx: f64, dy_per_row: &[f64]) -> Array3<u16> {
    let (h, w, c) = mov.dim();
    assert_eq!(
        dy_per_row.len(),
        h,
        "dy_per_row length must match image height"
    );
    let shift_x = -dx;
    let tmp: Array3<f32> = if shift_x.abs() < 1e-12 {
        mov.mapv(|v| v as f32)
    } else {
        let base = shift_x.floor() as i64;
        let frac = shift_x - base as f64;
        let (w0, w1) = ((1.0 - frac) as f32, frac as f32);
        let i0: Vec<usize> = (0..w as i64)
            .map(|i| (i + base).clamp(0, w as i64 - 1) as usize)
            .collect();
        let i1: Vec<usize> = (0..w as i64)
            .map(|i| (i + base + 1).clamp(0, w as i64 - 1) as usize)
            .collect();
        Array3::from_shape_fn((h, w, c), |(y, x, k)| {
            w0 * mov[[y, i0[x], k]] as f32 + w1 * mov[[y, i1[x], k]] as f32
        })
    };
    let mut out = Array3::<u16>::zeros((h, w, c));
    for y in 0..h {
        let src_y = y as f64 - dy_per_row[y];
        let y0 = src_y.floor() as i64;
        let f = (src_y - y0 as f64) as f32;
        let a0 = y0.clamp(0, h as i64 - 1) as usize;
        let a1 = (y0 + 1).clamp(0, h as i64 - 1) as usize;
        let (w0, w1) = (1.0 - f, f);
        for x in 0..w {
            for k in 0..c {
                let v = w0 * tmp[[a0, x, k]] + w1 * tmp[[a1, x, k]];
                out[[y, x, k]] = v.round_ties_even().clamp(0.0, 65535.0) as u16;
            }
        }
    }
    fill_row_shift_border(&mut out, dx, dy_per_row);
    out
}

/// Boolean mask of pixels that sample inside the moving image (remap).
pub fn valid_mask_translation(h: usize, w: usize, dx: f64, dy: f64) -> Array2<bool> {
    Array2::from_shape_fn((h, w), |(y, x)| {
        let (sx, sy) = (x as f64 - dx, y as f64 - dy);
        sx >= 0.0 && sx <= (w - 1) as f64 && sy >= 0.0 && sy <= (h - 1) as f64
    })
}

// ---------------------------------------------------------------- models / regions / validate

pub fn shift_is_pathological(
    dx: f64,
    dy: f64,
    full_w: usize,
    full_h: usize,
    max_frac: f64,
) -> bool {
    dx.abs() > max_frac * full_w as f64 || dy.abs() > max_frac * full_h as f64
}

pub type PhaseFn<'a> = &'a dyn Fn(ArrayView2<f32>, ArrayView2<f32>, f64) -> (f64, f64, f64);

/// Per-row-band `(dx, dy)` fit to a per-row `dy(y)` line → `(dx_median, dy_per_row)`.
pub fn band_shift_profile(
    reference: ArrayView3<u16>,
    moving: ArrayView3<u16>,
    opt: &RegistrationOptions,
    phase_fn: Option<PhaseFn>,
) -> Option<(f64, Vec<f64>)> {
    let n_bands = opt.band_count;
    if reference.dim().0 != moving.dim().0 || reference.dim().1 != moving.dim().1 {
        return None;
    }
    let (h, w, _) = reference.dim();
    if h < n_bands * opt.band_min_height {
        return None;
    }
    let default_fn =
        |r: ArrayView2<f32>, m: ArrayView2<f32>, sc: f64| phase_correlate_shift(r, m, sc, 1);
    let correlate: PhaseFn = phase_fn.unwrap_or(&default_fn);
    let band_h = h / n_bands;
    let (mut centers, mut dxs, mut dys) = (Vec::new(), Vec::new(), Vec::new());
    for i in 0..n_bands {
        let y0 = i * band_h;
        let y1 = if i == n_bands - 1 { h } else { y0 + band_h };
        let rb = band_luma(reference.slice(s![y0..y1, .., ..]), opt.coarse_target_px);
        let mb = band_luma(moving.slice(s![y0..y1, .., ..]), opt.coarse_target_px);
        let scale = if rb.ncols() == w {
            1.0
        } else {
            w as f64 / rb.ncols() as f64
        };
        let (dx, dy, resp) = correlate(rb.view(), mb.view(), scale);
        if resp < opt.min_response {
            continue;
        }
        centers.push((y0 + y1) as f64 / 2.0);
        dxs.push(dx);
        dys.push(dy);
    }
    if centers.len() < 3.max(n_bands / 2) {
        return None;
    }
    let fit = |c: &[f64], d: &[f64]| lstsq_line(c, d, &vec![1.0; c.len()]);
    let (mut c, mut d, mut dx_arr) = (centers, dys, dxs);
    let mut coef = fit(&c, &d);
    let resid = |c: &[f64], d: &[f64], k: [f64; 2]| -> Vec<f64> {
        c.iter()
            .zip(d)
            .map(|(x, y)| (y - (k[0] * x + k[1])).abs())
            .collect()
    };
    let mut res = resid(&c, &d, coef);
    let maxr = |r: &[f64]| r.iter().cloned().fold(f64::MIN, f64::max);
    while c.len() > 3 && maxr(&res) > opt.band_outlier_px {
        let m = maxr(&res);
        let keep: Vec<bool> = res.iter().map(|&r| r < m).collect();
        if keep.iter().filter(|&&k| k).count() < 3 {
            break;
        }
        let filt = |v: &[f64]| {
            v.iter()
                .zip(&keep)
                .filter(|(_, k)| **k)
                .map(|(x, _)| *x)
                .collect::<Vec<f64>>()
        };
        c = filt(&c);
        d = filt(&d);
        dx_arr = filt(&dx_arr);
        coef = fit(&c, &d);
        res = resid(&c, &d, coef);
    }
    let dy_per_row: Vec<f64> = (0..h).map(|y| coef[0] * y as f64 + coef[1]).collect();
    let (lo, hi) = (
        dy_per_row.iter().cloned().fold(f64::MAX, f64::min),
        dy_per_row.iter().cloned().fold(f64::MIN, f64::max),
    );
    if hi - lo > 0.5 * h as f64 {
        return None;
    }
    let dx_med = median(dx_arr);
    let max_abs = dy_per_row.iter().map(|v| v.abs()).fold(0.0, f64::max);
    if shift_is_pathological(dx_med, max_abs, w, h, opt.sanity_max_frac) {
        return None;
    }
    Some((dx_med, dy_per_row))
}

fn band_luma(image: ArrayView3<u16>, target_w: usize) -> Array2<f32> {
    downsample_plane_to_width(&mono_f32(image), target_w).0
}

/// Return `(y0, x0, y1, x1)` patches on a spatial grid.
pub fn select_patch_grid(
    h: usize,
    w: usize,
    n_y: usize,
    n_x: usize,
    patch: usize,
    margin: f64,
) -> Vec<(usize, usize, usize, usize)> {
    let patch = patch.min(h).min(w);
    if patch < 32 {
        return Vec::new();
    }
    let y_lo = (h as f64 * margin) as i64;
    let y_hi = (h as f64 * (1.0 - margin)) as i64 - patch as i64;
    let x_lo = (w as f64 * margin) as i64;
    let x_hi = (w as f64 * (1.0 - margin)) as i64 - patch as i64;
    if y_hi < y_lo || x_hi < x_lo {
        let y0 = (h - patch) / 2;
        let x0 = (w - patch) / 2;
        return vec![(y0, x0, y0 + patch, x0 + patch)];
    }
    let ys = crate::num::linspace(y_lo as f64, y_lo.max(y_hi) as f64, n_y);
    let xs = crate::num::linspace(x_lo as f64, x_lo.max(x_hi) as f64, n_x);
    let mut boxes = Vec::new();
    for &yf in &ys {
        for &xf in &xs {
            let y0 = py_round(yf).clamp(0, (h - patch) as i64) as usize;
            let x0 = py_round(xf).clamp(0, (w - patch) as i64) as usize;
            boxes.push((y0, x0, y0 + patch, x0 + patch));
        }
    }
    boxes
}

fn plane_std(p: ArrayView2<f32>) -> f64 {
    let n = p.len() as f64;
    let m = p.iter().map(|&v| v as f64).sum::<f64>() / n;
    (p.iter().map(|&v| (v as f64 - m).powi(2)).sum::<f64>() / n).sqrt()
}

/// Independent phase-correlate shifts on a spatial patch grid.
pub fn estimate_regional_shifts(
    ref_plane: &Array2<f32>,
    mov_plane: &Array2<f32>,
    min_response: f64,
    min_texture: f64,
    n_y: usize,
    n_x: usize,
    patch: usize,
) -> Vec<RegionalShift> {
    let (h, w) = ref_plane.dim();
    select_patch_grid(h, w, n_y, n_x, patch, 0.08)
        .into_iter()
        .map(|(y0, x0, y1, x1)| {
            let (cx, cy) = ((x0 + x1) as f64 / 2.0, (y0 + y1) as f64 / 2.0);
            if plane_std(ref_plane.slice(s![y0..y1, x0..x1])) < min_texture {
                return RegionalShift {
                    center_x: cx,
                    center_y: cy,
                    dx: 0.0,
                    dy: 0.0,
                    response: 0.0,
                    accepted: false,
                };
            }
            let (dx, dy, resp) = phase_correlate_shift(
                ref_plane.slice(s![y0..y1, x0..x1]),
                mov_plane.slice(s![y0..y1, x0..x1]),
                1.0,
                1,
            );
            RegionalShift {
                center_x: cx,
                center_y: cy,
                dx,
                dy,
                response: resp,
                accepted: resp >= min_response,
            }
        })
        .collect()
}

/// Median `(dx, dy)` of accepted regions + MAD; or `None` if too few.
pub fn consensus_translation(regions: &[RegionalShift]) -> Option<(f64, f64, f64, usize)> {
    let good: Vec<&RegionalShift> = regions.iter().filter(|r| r.accepted).collect();
    if good.len() < 2.max(regions.len().div_ceil(3)) {
        return None;
    }
    let dx = median(good.iter().map(|r| r.dx));
    let dy = median(good.iter().map(|r| r.dy));
    let mad = median(good.iter().map(|r| (r.dx - dx).hypot(r.dy - dy)));
    Some((dx, dy, mad, good.len()))
}

/// Mean absolute luma difference on a central inset.
pub fn luma_residual(a: ArrayView3<u16>, b: ArrayView3<u16>, inset: f64) -> f64 {
    let (ha, wa, _) = a.dim();
    let (y0, y1) = (
        (ha as f64 * inset) as usize,
        (ha as f64 * (1.0 - inset)) as usize,
    );
    let (x0, x1) = (
        (wa as f64 * inset) as usize,
        (wa as f64 * (1.0 - inset)) as usize,
    );
    let (la, lb) = if y1 <= y0 || x1 <= x0 {
        (structural_luminance(a), structural_luminance(b))
    } else {
        (
            structural_luminance(a.slice(s![y0..y1, x0..x1, ..])),
            structural_luminance(b.slice(s![y0..y1, x0..x1, ..])),
        )
    };
    la.iter()
        .zip(lb.iter())
        .map(|(x, y)| (*x as f64 - *y as f64).abs())
        .sum::<f64>()
        / la.len() as f64
}

pub fn should_snap_to_integer(dx: f64, dy: f64, eps: f64, regional_frac_consistent: bool) -> bool {
    regional_frac_consistent
        && (dx - dx.round_ties_even()).abs() < eps
        && (dy - dy.round_ties_even()).abs() < eps
}

pub fn classify_failure(
    response: f64,
    min_response: f64,
    pathological: bool,
    regional_mad: Option<f64>,
    shape_ok: bool,
    mode: RegistrationMode,
) -> RegistrationFailure {
    if !shape_ok {
        return RegistrationFailure::ShapeMismatch;
    }
    if pathological {
        return RegistrationFailure::PathologicalShift;
    }
    if response < min_response {
        return RegistrationFailure::LowTexture;
    }
    if regional_mad.is_some_and(|m| m > 2.0) {
        return if mode == RegistrationMode::RgbIr {
            RegistrationFailure::CrossModalMismatch
        } else {
            RegistrationFailure::RegionalDisagreement
        };
    }
    RegistrationFailure::None
}

// ---------------------------------------------------------------- engine

const RGB_IR_SEED_AGREE_PX: f64 = 0.5;
const RGB_IR_REFINE_KEEP_SEED_PX: f64 = 0.25;

#[doc(hidden)]
pub fn rgb_ir_merge_seed_axes(gdx: f64, gdy: f64, ldx: f64, ldy: f64) -> (f64, f64) {
    let one = |g: f64, l: f64| {
        if (l - g).abs() <= RGB_IR_SEED_AGREE_PX {
            l
        } else if g.abs() <= l.abs() {
            g
        } else {
            l
        }
    };
    (one(gdx, ldx), one(gdy, ldy))
}

#[doc(hidden)]
pub fn rgb_ir_keep_seed_axes(sdx: f64, sdy: f64, rdx: f64, rdy: f64) -> (f64, f64) {
    let one = |s: f64, r: f64| {
        if r.abs() > s.abs() + RGB_IR_REFINE_KEEP_SEED_PX {
            s
        } else {
            r
        }
    };
    (one(sdx, rdx), one(sdy, rdy))
}

fn zero_result(
    mode: RegistrationMode,
    reason: &str,
    failure: RegistrationFailure,
) -> RegistrationResult {
    RegistrationResult {
        transform: Transform::Translation { dx: 0.0, dy: 0.0 },
        mode,
        accepted: false,
        integer_exact: true,
        quality: RegistrationQuality {
            overlap_fraction: 1.0,
            ..Default::default()
        },
        regions: Vec::new(),
        reason: reason.into(),
        failure,
    }
}

/// Hierarchical phase-correlation translation (ME / RGB↔IR / MP fallback).
pub fn estimate_translation(
    reference: ArrayView3<u16>,
    moving: ArrayView3<u16>,
    mode: RegistrationMode,
    opt: &RegistrationOptions,
    context: Option<&RegistrationContext>,
) -> RegistrationResult {
    if reference.is_empty()
        || moving.is_empty()
        || reference.dim().0 != moving.dim().0
        || reference.dim().1 != moving.dim().1
    {
        return zero_result(mode, "shape mismatch", RegistrationFailure::ShapeMismatch);
    }
    let (full_h, full_w, _) = reference.dim();
    let (coarse_ref, coarse_mov, scale) =
        coarse_registration_pair(reference, moving, mode, opt.coarse_target_px);
    let (mut dx, mut dy, resp) =
        phase_correlate_shift(coarse_ref.view(), coarse_mov.view(), scale, 1);
    if !dx.is_finite() || !dy.is_finite() {
        return zero_result(mode, "non-finite shift", RegistrationFailure::LowTexture);
    }
    if resp < opt.min_response {
        warn!("registration: shift ({dx:.2}, {dy:.2}) low response ({resp:.3}) — reject");
        return zero_result(
            mode,
            &format!("low response {resp:.3}"),
            RegistrationFailure::LowTexture,
        );
    }
    if shift_is_pathological(dx, dy, full_w, full_h, opt.sanity_max_frac) {
        return zero_result(
            mode,
            "pathological shift",
            RegistrationFailure::PathologicalShift,
        );
    }
    let (mut seed_dx, mut seed_dy) = (dx, dy);
    let mut used_rgb_ir_seed = false;
    if mode == RegistrationMode::RgbIr && dx.abs().max(dy.abs()) <= opt.refine_max_residual + 0.5 {
        let (lr, lm, ls) = coarse_luma_registration_pair(reference, moving, opt.coarse_target_px);
        let (ldx, ldy, lresp) = phase_correlate_shift(lr.view(), lm.view(), ls, 1);
        if ldx.is_finite()
            && ldy.is_finite()
            && lresp >= opt.min_response
            && !shift_is_pathological(ldx, ldy, full_w, full_h, opt.sanity_max_frac)
        {
            (seed_dx, seed_dy) = rgb_ir_merge_seed_axes(seed_dx, seed_dy, ldx, ldy);
            used_rgb_ir_seed = true;
        }
    }
    (dx, dy) = refine_translation(reference, moving, (seed_dx, seed_dy), mode, opt);
    if used_rgb_ir_seed {
        if (dx - seed_dx).abs() > 0.25 || (dy - seed_dy).abs() > 0.25 {
            (dx, dy) = (seed_dx, seed_dy);
        } else {
            (dx, dy) = rgb_ir_keep_seed_axes(seed_dx, seed_dy, dx, dy);
        }
    }
    if shift_is_pathological(dx, dy, full_w, full_h, opt.sanity_max_frac) {
        return zero_result(
            mode,
            "refined pathological shift",
            RegistrationFailure::PathologicalShift,
        );
    }
    let mut regions = Vec::new();
    let (mut regional_mad, mut inliers) = (0.0, 0);
    let mut consensus = None;
    if opt.enable_regional {
        let patch = 128.min(coarse_ref.nrows()).min(coarse_ref.ncols());
        regions =
            estimate_regional_shifts(&coarse_ref, &coarse_mov, opt.min_response, 1.0, 3, 3, patch);
        for r in regions.iter_mut().filter(|r| r.accepted) {
            r.center_x *= scale;
            r.center_y *= scale;
            r.dx *= scale;
            r.dy *= scale;
        }
        consensus = consensus_translation(&regions);
        if let Some((cdx, cdy, mad, n)) = consensus {
            regional_mad = mad;
            inliers = n;
            if mad < 1.5 && n >= 3 && (cdx - dx).abs() < 2.0 && (cdy - dy).abs() < 2.0 {
                dx = 0.5 * (dx + cdx);
                dy = 0.5 * (dy + cdy);
            }
        }
    }
    if let Some(ctx) = context {
        if ctx.max_abs_dx.is_some_and(|m| dx.abs() > m) {
            return zero_result(
                mode,
                "outside prior dx",
                RegistrationFailure::OutsideScannerPrior,
            );
        }
        if ctx.max_abs_dy.is_some_and(|m| dy.abs() > m) {
            return zero_result(
                mode,
                "outside prior dy",
                RegistrationFailure::OutsideScannerPrior,
            );
        }
    }
    // RGB↔IR: never integer-snap (cross-modal peaks often sit ~0.5–1 px off).
    let mut integer_exact = false;
    if mode != RegistrationMode::RgbIr {
        integer_exact = should_snap_to_integer(dx, dy, opt.snap_frac_eps, true);
        if integer_exact {
            dx = dx.round_ties_even();
            dy = dy.round_ties_even();
        }
    }
    let failure = classify_failure(
        resp,
        opt.min_response,
        false,
        consensus.map(|_| regional_mad),
        true,
        mode,
    );
    let mut accepted = matches!(
        failure,
        RegistrationFailure::None | RegistrationFailure::RegionalDisagreement
    );
    if opt.enable_regional
        && failure == RegistrationFailure::RegionalDisagreement
        && regional_mad > 4.0
    {
        accepted = false;
    }
    let quality = RegistrationQuality {
        response: resp,
        regional_inliers: inliers,
        regional_total: regions.len(),
        regional_mad_px: regional_mad,
        overlap_fraction: (1.0 - dx.abs() / full_w as f64).max(0.0)
            * (1.0 - dy.abs() / full_h as f64).max(0.0),
        ..Default::default()
    };
    RegistrationResult {
        transform: Transform::Translation { dx, dy },
        mode,
        accepted,
        integer_exact,
        quality,
        regions,
        reason: if accepted {
            "ok".into()
        } else {
            failure.as_str().into()
        },
        failure: if accepted {
            RegistrationFailure::None
        } else {
            failure
        },
    }
}

/// Phase-correlate one integer-preshifted ROI; `None` if out of bounds.
#[allow(clippy::too_many_arguments)]
fn roi_residual(
    reference: ArrayView3<u16>,
    moving: ArrayView3<u16>,
    y0: i64,
    x0: i64,
    rh: i64,
    rw: i64,
    idx: i64,
    idy: i64,
    upsample: usize,
) -> Option<(f64, f64, f64)> {
    let (h, w, _) = reference.dim();
    let (h, w) = (h as i64, w as i64);
    let (mut my0, mut mx0) = (y0 - idy, x0 - idx);
    let (mut ty0, mut tx0) = (y0, x0);
    if my0 < 0 {
        ty0 -= my0;
        my0 = 0;
    }
    if mx0 < 0 {
        tx0 -= mx0;
        mx0 = 0;
    }
    let th = rh.min(h - ty0).min(h - my0);
    let tw = rw.min(w - tx0).min(w - mx0);
    if th < 64 || tw < 64 {
        return None;
    }
    let (ty0, tx0, my0, mx0, th, tw) = (
        ty0 as usize,
        tx0 as usize,
        my0 as usize,
        mx0 as usize,
        th as usize,
        tw as usize,
    );
    let r = structural_luminance(reference.slice(s![ty0..ty0 + th, tx0..tx0 + tw, ..]));
    let m = structural_luminance(moving.slice(s![my0..my0 + th, mx0..mx0 + tw, ..]));
    Some(phase_correlate_shift(r.view(), m.view(), 1.0, upsample))
}

/// Refine on a center ROI using integer-slice pre-shift (no full-frame warp).
fn refine_translation(
    reference: ArrayView3<u16>,
    moving: ArrayView3<u16>,
    coarse: (f64, f64),
    mode: RegistrationMode,
    opt: &RegistrationOptions,
) -> (f64, f64) {
    let (h, w, _) = reference.dim();
    if h * w < 512 * 512 {
        return coarse;
    }
    let (dx0, dy0) = coarse;
    let rh = opt.refine_roi_side.min(h);
    let rw = opt.refine_roi_side.min(w);
    let idx = py_round(dx0);
    let idy = py_round(dy0);
    let guizar_ok = opt.enable_subpixel
        && opt.enable_guizar_refine
        && mode != RegistrationMode::RgbIr
        && rh.max(rw) <= opt.guizar_max_roi_side;
    let upsample = if guizar_ok { opt.subpixel_factor } else { 1 };
    let cy = ((h - rh) / 2) as i64;
    let cx = ((w - rw) / 2) as i64;
    let (rh, rw) = (rh as i64, rw as i64);
    let origins: Vec<(i64, i64)> = if mode == RegistrationMode::RgbIr {
        let (hy, hx) = (rh / 2, rw / 2);
        vec![
            (cy, cx),
            (cy - hy, cx),
            (cy + hy, cx),
            (cy, cx - hx),
            (cy, cx + hx),
            (cy - hy, cx - hx),
            (cy - hy, cx + hx),
            (cy + hy, cx - hx),
            (cy + hy, cx + hx),
        ]
    } else {
        vec![(cy, cx)]
    };
    let mut samples = Vec::new();
    for (y0, x0) in origins {
        if y0 < 0 || x0 < 0 || y0 + rh > h as i64 || x0 + rw > w as i64 {
            continue;
        }
        let Some((ddx, ddy, resp)) =
            roi_residual(reference, moving, y0, x0, rh, rw, idx, idy, upsample)
        else {
            continue;
        };
        if resp < opt.min_response || ddx.abs().max(ddy.abs()) > opt.refine_max_residual {
            continue;
        }
        samples.push((ddx, ddy, resp));
    }
    if samples.is_empty() {
        return coarse;
    }
    let (ddx, ddy) = if samples.len() == 1 {
        (samples[0].0, samples[0].1)
    } else {
        let thr = median(samples.iter().map(|s| s.2));
        let kept: Vec<_> = {
            let k: Vec<_> = samples.iter().copied().filter(|s| s.2 >= thr).collect();
            if k.is_empty() { samples.clone() } else { k }
        };
        let (sdx, sdy) = (dx0 - idx as f64, dy0 - idy as f64);
        let near: Vec<_> = kept
            .iter()
            .copied()
            .filter(|s| (s.0 - sdx).abs() <= 0.75 && (s.1 - sdy).abs() <= 0.75)
            .collect();
        let use_ = if near.len() >= 2 { near } else { kept };
        (
            median(use_.iter().map(|s| s.0)),
            median(use_.iter().map(|s| s.1)),
        )
    };
    (idx as f64 + ddx, idy as f64 + ddy)
}

/// Multi-Pass row-banded registration (production default).
pub fn estimate_banded(
    reference: ArrayView3<u16>,
    moving: ArrayView3<u16>,
    opt: &RegistrationOptions,
    phase_fn: Option<PhaseFn>,
) -> RegistrationResult {
    let mode = RegistrationMode::MultiPass;
    if moving.is_empty()
        || reference.dim().0 != moving.dim().0
        || reference.dim().1 != moving.dim().1
    {
        return zero_result(mode, "shape mismatch", RegistrationFailure::ShapeMismatch);
    }
    let Some((dx, dy_per_row)) = band_shift_profile(reference, moving, opt, phase_fn) else {
        return estimate_translation(reference, moving, mode, opt, None);
    };
    let ok = |transform, integer_exact, reason: &str, quality| RegistrationResult {
        transform,
        mode,
        accepted: true,
        integer_exact,
        quality,
        regions: Vec::new(),
        reason: reason.into(),
        failure: RegistrationFailure::None,
    };
    if dx.abs() < 1e-9 && dy_per_row.iter().map(|v| v.abs()).fold(0.0, f64::max) < 1e-9 {
        let zero = dy_per_row.iter().map(|v| v * 0.0).collect();
        return ok(
            Transform::LineWarp {
                dx: 0.0,
                dy_per_row: zero,
            },
            true,
            "zero",
            RegistrationQuality {
                overlap_fraction: 1.0,
                ..Default::default()
            },
        );
    }
    let n = dy_per_row.len();
    info!(
        "registration: banded dx={dx:.2} dy(row)={:.2}..{:.2} (slope={:.4} px/row)",
        dy_per_row[0],
        dy_per_row[n - 1],
        (dy_per_row[n - 1] - dy_per_row[0]) / (reference.dim().0.max(2) - 1) as f64
    );
    ok(
        Transform::LineWarp { dx, dy_per_row },
        false,
        "banded",
        RegistrationQuality {
            overlap_fraction: 1.0,
            ..Default::default()
        },
    )
}

/// Estimate a transform mapping `moving` → `reference`.
pub fn register(
    reference: ArrayView3<u16>,
    moving: ArrayView3<u16>,
    mode: RegistrationMode,
    context: Option<&RegistrationContext>,
    opt: &RegistrationOptions,
    banded: Option<bool>,
) -> RegistrationResult {
    if banded.unwrap_or(mode == RegistrationMode::MultiPass) {
        return estimate_banded(reference, moving, opt, None);
    }
    estimate_translation(reference, moving, mode, opt, context)
}

/// Apply `transform` once to `moving` (never mutates input).
pub fn apply_transform(
    moving: &Array3<u16>,
    transform: &Transform,
    method: Resampler,
) -> Array3<u16> {
    match transform {
        Transform::LineWarp { dx, dy_per_row } => warp_row_shifts(moving, *dx, dy_per_row),
        Transform::Translation { dx, dy } => {
            if dx.abs() < 1e-9 && dy.abs() < 1e-9 {
                return moving.clone();
            }
            warp_shift(moving, *dx, *dy, method)
        }
    }
}

/// Estimate + apply; on reject returns `moving` unchanged.
pub fn register_and_align(
    reference: &Array3<u16>,
    moving: &Array3<u16>,
    mode: RegistrationMode,
    opt: &RegistrationOptions,
    banded: Option<bool>,
) -> (Array3<u16>, RegistrationResult) {
    let result = register(reference.view(), moving.view(), mode, None, opt, banded);
    if !result.accepted {
        return (moving.clone(), result);
    }
    (
        apply_transform(moving, &result.transform, Resampler::Linear),
        result,
    )
}
