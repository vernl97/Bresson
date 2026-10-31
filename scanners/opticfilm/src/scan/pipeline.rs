// SPDX-License-Identifier: GPL-3.0-or-later
//! Host-side image reconstruction (RGB16, shifts, stagger, shading).

use log::{debug, info};
use ndarray::{Array2, Array3, ArrayView3, Axis, s};

use super::geometry::ScanGeometry;
use crate::device::model::FilmModel;
use crate::error::{Error, Result};
use crate::num::{U16Hist, py_round};

/// After dark/white stretch, push frame highlights toward sensor white.
pub const HOST_CALIB_PEAK_PERCENTILE: f64 = 99.7;
pub const HOST_CALIB_PEAK_TARGET: i64 = 0xF000;
pub const HOST_CALIB_PEAK_TRIGGER: f64 = 0.85;
/// Cap post-ME makeup so already-bright pixels are not driven into hard clip.
pub const HOST_CALIB_HIGHLIGHT_CEILING: i64 = (0.92 * 65535.0) as i64;
pub const HOST_CALIB_HIGHLIGHT_PERCENTILE: f64 = 99.9;
/// Drop this fraction from each edge when estimating film highlights.
pub const HOST_CALIB_BORDER_INSET: f64 = 0.04;
const HOST_CALIB_CHUNK_ROWS: usize = 256;
const FS: f32 = 65535.0;
/// Linear-negative USB dummy is near-zero DN; film / gate is far brighter.
pub const USB_END_DUMMY_MEAN_MAX: f64 = 2048.0;

/// `HxWx3` uint16 image (rows, columns, R/G/B).
pub type Rgb16 = Array3<u16>;

fn column_mean(rgb: ArrayView3<u16>, x: usize, channels: usize) -> f64 {
    let (h, _, _) = rgb.dim();
    let mut s = 0u64;
    for y in 0..h {
        for c in 0..channels {
            s += rgb[[y, x, c]] as u64;
        }
    }
    s as f64 / (h * channels) as f64
}

/// Mean DN of the first/last `n` USB columns (all channels).
fn edge_column_means(rgb: &Rgb16, n: usize) -> (Vec<f64>, Vec<f64>) {
    let w = rgb.dim().1;
    if w < 2 {
        return (Vec::new(), Vec::new());
    }
    let count = n.min((w / 2).max(1));
    let head = (0..count).map(|i| column_mean(rgb.view(), i, 3)).collect();
    let tail = (0..count)
        .map(|i| column_mean(rgb.view(), w - count + i, 3))
        .collect();
    (head, tail)
}

fn fmt_means(values: &[f64]) -> String {
    let shown: Vec<String> = values.iter().take(8).map(|v| format!("{v:.0}")).collect();
    format!("[{}]", shown.join(", "))
}

/// Count ENDPIXEL-side columns that look like USB dummy, capped at `max_drop`.
pub fn count_usb_end_dummy_columns(rgb: &Rgb16, max_drop: i64, dummy_mean_max: f64) -> i64 {
    let w = rgb.dim().1 as i64;
    if max_drop <= 0 || w < 16 {
        return 0;
    }
    let cap = max_drop.min(w - 8);
    if cap <= 0 {
        return 0;
    }
    let channels = 3.min(rgb.dim().2);
    let mut dropped = 0;
    for i in 1..=cap {
        if column_mean(rgb.view(), (w - i) as usize, channels) > dummy_mean_max {
            break;
        }
        dropped = i;
    }
    dropped
}

/// Drop trailing columns past the programmed STR/END output width.
pub fn trim_to_optical_span(rgb: Rgb16, geometry: &ScanGeometry) -> Rgb16 {
    let expected = geometry.pixels as usize;
    if expected < 8 || rgb.dim().1 <= expected {
        return rgb;
    }
    info!(
        "trimmed optical span width {} → {expected} (USB wider than geometry.pixels)",
        rgb.dim().1
    );
    rgb.slice(s![.., ..expected, ..]).to_owned()
}

/// Per-channel `np.percentile(region, q, axis=(0, 1))` on u16 data.
pub fn channel_percentiles(rgb: ArrayView3<u16>, q: f64) -> [f64; 3] {
    [0, 1, 2].map(|c| U16Hist::new(rgb.index_axis(Axis(2), c).iter().copied()).percentile(q))
}

/// Convert raw scanner bytes into HxWx3 uint16 RGB.
pub struct ImagePipeline {
    pub model: FilmModel,
    /// Columns dropped from the USB ENDPIXEL side on the last `assemble`.
    pub last_usb_end_drop: i64,
}

/// `(ys, xs)` ranges for a centered inset, or `None` if too small.
pub fn inset_slice(h: usize, w: usize, inset: f64) -> Option<((usize, usize), (usize, usize))> {
    let frac = inset.clamp(0.0, 0.2);
    if frac <= 0.0 || h < 8 || w < 8 {
        return None;
    }
    let cut_h = py_round(h as f64 * frac).max(1) as usize;
    let cut_w = py_round(w as f64 * frac).max(1) as usize;
    if cut_h * 2 >= h || cut_w * 2 >= w {
        return None;
    }
    Some(((cut_h, h - cut_h), (cut_w, w - cut_w)))
}

impl ImagePipeline {
    pub fn new(model: FilmModel) -> Self {
        Self {
            model,
            last_usb_end_drop: 0,
        }
    }

    /// Decode USB RGB optical buffer → uint16 HxWx3.
    ///
    /// 8-bit streams are upsampled to 16-bit (`value * 257`). Layout is
    /// chunky `RGBRGB…` unless `planar` (or the model) says `RRR…GGG…BBB…`.
    pub fn decode_rgb(
        &self,
        raw: &[u8],
        geometry: &ScanGeometry,
        planar: Option<bool>,
    ) -> Result<Rgb16> {
        let expected = geometry.total_bytes() as usize;
        if raw.len() < expected {
            return Err(Error::Value(format!(
                "Short scan buffer: got {} want {expected}",
                raw.len()
            )));
        }
        let (h, w, c) = (
            geometry.optical_line_count as usize,
            geometry.pixels as usize,
            geometry.channels as usize,
        );
        let planar =
            planar.unwrap_or_else(|| self.model.gl128.as_ref().is_some_and(|g| g.usb_planar_rgb));
        let index = |y: usize, x: usize, ch: usize| {
            if planar {
                (y * c + ch) * w + x
            } else {
                (y * w + x) * c + ch
            }
        };
        match geometry.depth {
            8 => Ok(Array3::from_shape_fn((h, w, c), |(y, x, ch)| {
                raw[index(y, x, ch)] as u16 * 257
            })),
            16 => {
                let swap = self.model.swap_16bit_data.unwrap_or(false);
                Ok(Array3::from_shape_fn((h, w, c), |(y, x, ch)| {
                    let i = index(y, x, ch) * 2;
                    let v = u16::from_le_bytes([raw[i], raw[i + 1]]);
                    if swap { v.swap_bytes() } else { v }
                }))
            }
            d => Err(Error::Value(format!("Unsupported geometry depth {d}"))),
        }
    }

    /// Average USB rows down to `geometry.lines` (`optical_line_count // lines` per group).
    pub fn reduce_y_oversample(&self, rgb: Rgb16, geometry: &ScanGeometry) -> Result<Rgb16> {
        if geometry.lines <= 0 {
            return Ok(rgb);
        }
        let n = (geometry.optical_line_count / geometry.lines) as usize;
        if n <= 1 {
            return Ok(rgb);
        }
        let (height, width, channels) = rgb.dim();
        let groups = height / n;
        if groups < 1 {
            return Err(Error::Value(format!(
                "Buffer of {height} rows is shorter than one {n}-row group"
            )));
        }
        let out = Array3::from_shape_fn((groups, width, channels), |(g, x, c)| {
            let s: u32 = (0..n).map(|k| rgb[[g * n + k, x, c]] as u32).sum();
            ((s + (n / 2) as u32) / n as u32) as u16
        });
        debug!("averaged {height} rows -> {groups} (oversample={n})");
        Ok(out)
    }

    /// Block-average when the ASIC ran hotter than the requested PPI.
    pub fn apply_host_downsample(&self, rgb: Rgb16, geometry: &ScanGeometry) -> Rgb16 {
        let f = geometry.host_downsample.max(1) as usize;
        if f <= 1 {
            return rgb;
        }
        let (h, w, c) = rgb.dim();
        let (nh, nw) = ((h / f) * f, (w / f) * f);
        if nh == 0 || nw == 0 {
            return rgb;
        }
        let count = (f * f) as u32;
        let out = Array3::from_shape_fn((nh / f, nw / f, c), |(y, x, ch)| {
            let mut s = 0u32;
            for dy in 0..f {
                for dx in 0..f {
                    s += rgb[[y * f + dy, x * f + dx, ch]] as u32;
                }
            }
            ((s + count / 2) / count) as u16
        });
        debug!(
            "host downsample {h}x{w} -> {}x{} (factor={f})",
            out.dim().0,
            out.dim().1
        );
        out
    }

    /// Align R/G/B using genesys ld_shift scaled to yres.
    pub fn apply_line_shifts(&self, rgb: Rgb16, geometry: &ScanGeometry) -> Result<Rgb16> {
        let shifts = [geometry.shift_r, geometry.shift_g, geometry.shift_b].map(|v| v as usize);
        let mut out_h = (geometry.lines + geometry.num_staggered_lines) as usize;
        let max_shift = *shifts.iter().max().unwrap();
        if max_shift == 0 {
            let k = out_h.min(rgb.dim().0);
            return Ok(rgb.slice(s![..k, .., ..]).to_owned());
        }
        let (height, width, _) = rgb.dim();
        out_h = out_h.min(height.saturating_sub(max_shift));
        if height < max_shift || out_h < 1 {
            return Err(Error::Value(
                "Optical buffer shorter than required after shift".into(),
            ));
        }
        let out = Array3::from_shape_fn((out_h, width, 3), |(y, x, c)| rgb[[y + shifts[c], x, c]]);
        debug!("applied line shifts r/g/b={shifts:?}");
        Ok(out)
    }

    /// Unstagger alternating columns (7200 dpi `StaggerConfig{4,0}`).
    pub fn apply_y_stagger(&self, rgb: Rgb16, geometry: &ScanGeometry) -> Result<Rgb16> {
        let shifts = &geometry.stagger_y;
        if shifts.is_empty() || geometry.num_staggered_lines == 0 {
            let k = (geometry.lines as usize).min(rgb.dim().0);
            return Ok(rgb.slice(s![..k, .., ..]).to_owned());
        }
        let (height, width, _) = rgb.dim();
        let max_shift = *shifts.iter().max().unwrap() as usize;
        let out_h = (geometry.lines as usize).min(height.saturating_sub(max_shift));
        if height < max_shift || out_h < 1 {
            return Err(Error::Value(
                "Buffer shorter than required for Y stagger".into(),
            ));
        }
        let n = shifts.len();
        let out = Array3::from_shape_fn((out_h, width, 3), |(y, x, c)| {
            rgb[[y + shifts[x % n] as usize, x, c]]
        });
        debug!("applied y stagger shifts={shifts:?} out_h={out_h}");
        Ok(out)
    }

    /// Pull full-window holder chrome down to the film-window highlight peak (per channel).
    pub fn clamp_border_highlights(&self, rgb: Rgb16, inset: f64, peak_percentile: f64) -> Rgb16 {
        let (h, w, _) = rgb.dim();
        let Some(((y0, y1), (x0, x1))) = inset_slice(h, w, inset) else {
            return rgb;
        };
        let peaks = channel_percentiles(rgb.slice(s![y0..y1, x0..x1, ..]), peak_percentile);
        if peaks.iter().cloned().fold(f64::MIN, f64::max) <= 0.0 {
            return rgb;
        }
        let peaks32 = peaks.map(|p| (p as f32).max(1.0));
        let peaks_u16 = peaks32.map(|p| p.round_ties_even().clamp(0.0, 65535.0) as u16);
        let border = |y: usize, x: usize| !(y >= y0 && y < y1 && x >= x0 && x < x1);
        let mut out = rgb;
        let mut hot = 0usize;
        for y in 0..h {
            for x in 0..w {
                if !border(y, x) {
                    continue;
                }
                if (0..3).any(|c| out[[y, x, c]] > peaks_u16[c]) {
                    hot += 1;
                    for c in 0..3 {
                        out[[y, x, c]] = out[[y, x, c]].min(peaks_u16[c]);
                    }
                }
            }
        }
        if hot > 0 {
            info!(
                "border highlight clamp inset={inset:.2} peak_p{peak_percentile:.1}=({:.0},{:.0},{:.0}) pixels={hot}",
                peaks32[0], peaks32[1], peaks32[2]
            );
        }
        out
    }

    /// Lift the film window toward sensor white with one scalar gain.
    #[allow(clippy::too_many_arguments)]
    pub fn expose_film_base(
        &self,
        rgb: Rgb16,
        source: &str,
        peak_target: i64,
        peak_percentile: f64,
        preserve_headroom: bool,
        highlight_ceiling: i64,
        highlight_percentile: f64,
    ) -> Rgb16 {
        if peak_target <= 0 {
            return rgb;
        }
        let (h, w, _) = rgb.dim();
        let sample = match inset_slice(h, w, HOST_CALIB_BORDER_INSET) {
            Some(((y0, y1), (x0, x1))) => rgb.slice(s![y0..y1, x0..x1, ..]),
            None => rgb.view(),
        };
        let max3 = |a: [f64; 3]| a[0].max(a[1]).max(a[2]);
        let peak = max3(channel_percentiles(sample, peak_percentile));
        if peak <= 1.0 || peak >= peak_target as f64 * HOST_CALIB_PEAK_TRIGGER {
            return rgb;
        }
        let mut gain = peak_target as f64 / peak;
        if preserve_headroom && highlight_ceiling > 0 {
            let hi = max3(channel_percentiles(sample, highlight_percentile));
            if hi > 1.0 {
                gain = gain.min(highlight_ceiling as f64 / hi);
            }
        }
        if gain <= 1.0 + 1e-6 {
            return rgb;
        }
        info!(
            "{source} exposure makeup gain={gain:.3} peak_p{peak_percentile:.1}={peak:.0} → {peak_target}{}",
            if preserve_headroom { " (headroom)" } else { "" }
        );
        let g = gain as f32;
        rgb.mapv(|v| (v as f32 * g).clamp(0.0, 65535.0).round_ties_even() as u16)
    }

    /// Host-side shading: column flat-field, then optionally expose film base.
    pub fn apply_host_calib(
        &self,
        rgb: Rgb16,
        dark: &Array2<f32>,
        white: &Array2<f32>,
        peak_target: i64,
        peak_percentile: f64,
        expose_base: bool,
    ) -> Result<Rgb16> {
        if dark.dim() != white.dim() {
            return Err(Error::Value(format!(
                "dark/white shape mismatch: {:?} vs {:?}",
                dark.dim(),
                white.dim()
            )));
        }
        let (h, w, _) = rgb.dim();
        if dark.nrows() < w || dark.ncols() != 3 {
            return Err(Error::Value(format!(
                "calib width {:?} incompatible with image width {w}",
                dark.dim()
            )));
        }
        let mut bad = Array2::from_elem((w, 3), false);
        let mut offset = Array2::<f32>::zeros((w, 3));
        let mut mult = Array2::<f32>::zeros((w, 3));
        for x in 0..w {
            for c in 0..3 {
                let d = dark[[x, c]];
                let mut denom = white[[x, c]] - d;
                if denom <= 0.0 {
                    bad[[x, c]] = true;
                    denom = 1.0;
                }
                offset[[x, c]] = d / FS;
                mult[[x, c]] = FS / denom;
            }
        }
        let mut out = Array3::<u16>::zeros((h, w, 3));
        for y in 0..h {
            for x in 0..w {
                for c in 0..3 {
                    let v = rgb[[y, x, c]];
                    out[[y, x, c]] = if bad[[x, c]] {
                        v
                    } else {
                        let corrected = (v as f32 / FS - offset[[x, c]]) * mult[[x, c]];
                        (corrected * FS).clamp(0.0, 65535.0).round_ties_even() as u16
                    };
                }
            }
        }
        let _ = HOST_CALIB_CHUNK_ROWS;
        if !expose_base {
            return Ok(out);
        }
        let exposed = self.expose_film_base(
            out,
            "host calib",
            peak_target,
            peak_percentile,
            false,
            HOST_CALIB_HIGHLIGHT_CEILING,
            HOST_CALIB_HIGHLIGHT_PERCENTILE,
        );
        Ok(self.clamp_border_highlights(exposed, HOST_CALIB_BORDER_INSET, peak_percentile))
    }

    /// Decode USB RGB and apply calib / optional film-base makeup.
    pub fn assemble(
        &mut self,
        raw: &[u8],
        geometry: &ScanGeometry,
        dark: Option<&Array2<f32>>,
        white: Option<&Array2<f32>>,
        planar: Option<bool>,
        expose_base: bool,
        usb_end_drop: Option<i64>,
    ) -> Result<Rgb16> {
        let mut rgb = self.decode_rgb(raw, geometry, planar)?;
        rgb = self.reduce_y_oversample(rgb, geometry)?;
        rgb = self.apply_line_shifts(rgb, geometry)?;
        rgb = self.apply_y_stagger(rgb, geometry)?;
        let fl = geometry.frame_lines as usize;
        if fl > 0 && fl < rgb.dim().0 {
            rgb = rgb.slice(s![..fl, .., ..]).to_owned();
        }
        rgb = self.apply_host_downsample(rgb, geometry);
        rgb = trim_to_optical_span(rgb, geometry);
        let cap = if geometry.disable_buffer_full_move {
            0
        } else {
            geometry.usb_end_drop
        };
        let drop = match usb_end_drop {
            None => count_usb_end_dummy_columns(&rgb, cap, USB_END_DUMMY_MEAN_MAX),
            Some(d) => d.min(cap).min(0.max(rgb.dim().1 as i64 - 8)).max(0),
        };
        self.last_usb_end_drop = drop;
        if drop > 0 {
            let w = rgb.dim().1 - drop as usize;
            rgb = rgb.slice(s![.., ..w, ..]).to_owned();
            info!(
                "dropped {drop} USB-end columns (ENDPIXEL dummy, cap {cap}) → width {}",
                rgb.dim().1
            );
        }
        let host_calib = dark.is_some() && white.is_some();
        if let (Some(d), Some(wt)) = (dark, white) {
            rgb = self.apply_host_calib(
                rgb,
                d,
                wt,
                HOST_CALIB_PEAK_TARGET,
                HOST_CALIB_PEAK_PERCENTILE,
                expose_base,
            )?;
        } else if expose_base {
            rgb = self.expose_film_base(
                rgb,
                "asic shading",
                HOST_CALIB_PEAK_TARGET,
                HOST_CALIB_PEAK_PERCENTILE,
                false,
                HOST_CALIB_HIGHLIGHT_CEILING,
                HOST_CALIB_HIGHLIGHT_PERCENTILE,
            );
            rgb = self.clamp_border_highlights(
                rgb,
                HOST_CALIB_BORDER_INSET,
                HOST_CALIB_PEAK_PERCENTILE,
            );
        }
        let (pre_head, pre_tail) = edge_column_means(&rgb, 16);
        if self.model.mirror_x() {
            rgb.invert_axis(Axis(1));
            rgb = rgb.as_standard_layout().to_owned();
        }
        let (post_head, _) = edge_column_means(&rgb, 16);
        info!(
            "assemble {}dpi str={} end={} pixels={} optical={} offset={} dpiset={} usb_end_drop={cap} area={:?} host_calib={host_calib} \
             pre_usb[:,:8]={} pre_usb[:,-8:]={} post_mirror[:,:8]={}",
            geometry.resolution,
            geometry.pixel_startx,
            geometry.pixel_endx,
            geometry.pixels,
            geometry.optical_pixels,
            geometry.output_pixel_offset,
            geometry.register_dpiset,
            geometry.area,
            fmt_means(&pre_head),
            fmt_means(&pre_tail),
            fmt_means(&post_head)
        );
        Ok(rgb)
    }
}
