// SPDX-License-Identifier: GPL-3.0-or-later
//! GL128 full-window / PPI-ladder scan geometry (Lab `preview_safe` parity).
//!
//! Both profiles pin `LINCNT` to a value the captures prove safe for the
//! second feed they use.

use super::calib_gl128::{shading_acquire_width, shading_width_for_resolution};
use super::geometry::{Area, ScanGeometry, compute_geometry};
use crate::device::model::FilmModel;
use crate::error::Result;
use crate::num::py_round_to;

pub const NATIVE_DPI: i64 = 7200;
pub const MM_PER_INCH: f64 = 25.4;
/// Fixed Prescan DPI (full window).
pub const PRESCAN_DPI: i64 = 1200;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeometryProfile {
    PreviewSafe,
    PpiLadder,
}

/// What a bring-up geometry was built from (Python `meta` dict).
#[derive(Debug, Clone, PartialEq)]
pub struct GeometryMeta {
    pub profile: &'static str,
    pub feed2: i64,
    pub max_lincnt: i64,
    pub target_lincnt: i64,
    pub oversample: i64,
    pub y2: f64,
    pub travel_mm: f64,
    pub dpi: i64,
    pub geometry_lincnt: Option<i64>,
    pub optical_line_count: Option<i64>,
    pub area: Option<Area>,
    pub pixels: Option<i64>,
    pub requested_area: Option<Area>,
    pub effective_area: Option<Area>,
    pub lincnt_clamped: Option<bool>,
}

/// True for GL128 OpticFilm models (8200i SE, 8100 V2, 8300i SE).
pub fn is_gl128_opticfilm(model: &FilmModel) -> bool {
    model.asic == "GL128" || [0x1825, 0x1824, 0x181F].contains(&model.usb_product_id)
}

fn window_end_steps(model: &FilmModel) -> i64 {
    let g = model.g();
    if g.scan_window_end_steps != 0 { g.scan_window_end_steps } else { g.feed_to_scan_bottom_steps }
}

/// Invert `feed_to_scan_steps_for_area` so a feed2 maps back to `y1`.
pub fn y1_for_feed2(model: &FilmModel, feed2: i64) -> f64 {
    let top = model.g().feed_to_scan_top_steps;
    let end = window_end_steps(model);
    if end == top {
        return 0.0;
    }
    ((feed2 - top) as f64 / (end - top) as f64).clamp(0.0, 1.0)
}

fn feed2_for(model: &FilmModel, y1: f64) -> i64 {
    model.feed_to_scan_steps_for_area(Some([0.0, y1, 1.0, 1.0]))
}

fn travel_mm(model: &FilmModel, lincnt: i64, dpi: i64) -> f64 {
    model.travel_mm_for_lincnt(lincnt, dpi)
}

/// Full-width crop from `y1` to the end of the scan window (Lab Full window).
pub fn preview_safe_scan_area(model: &FilmModel, dpi: i64, y1: f64) -> (Area, GeometryMeta) {
    let y1 = y1.clamp(0.0, 1.0);
    let feed2 = feed2_for(model, y1);
    let mut max_lincnt = model.max_lincnt_for(feed2, dpi);
    if max_lincnt <= 0 {
        max_lincnt = 4836;
    }
    let t = travel_mm(model, max_lincnt, dpi);
    let y2 = (y1 + t / model.y_size_ta_mm).max(y1 + 1e-6).min(1.0);
    let meta = GeometryMeta {
        profile: "preview_safe",
        feed2,
        max_lincnt,
        target_lincnt: max_lincnt,
        oversample: model.oversample_for(dpi),
        y2,
        travel_mm: py_round_to(t, 3),
        dpi,
        geometry_lincnt: None,
        optical_line_count: None,
        area: None,
        pixels: None,
        requested_area: None,
        effective_area: None,
        lincnt_clamped: None,
    };
    ([0.0, y1, 1.0, y2], meta)
}

/// Session-13 PPI-ladder crop.
pub fn ladder_scan_area(model: &FilmModel, dpi: i64) -> (Area, GeometryMeta) {
    let feed2 = model.g().ladder_feed2_steps;
    let target_lincnt = model.ladder_lincnt_for(dpi);
    let y1 = y1_for_feed2(model, feed2);
    let t = travel_mm(model, target_lincnt, dpi);
    let y2 = (y1 + t / model.y_size_ta_mm).max(y1 + 1e-6).min(1.0);
    let area = [0.0, y1, 1.0, y2];
    let actual_feed2 = model.feed_to_scan_steps_for_area(Some(area));
    let max_lincnt = model.max_lincnt_for(actual_feed2, dpi);
    let meta = GeometryMeta {
        profile: "ppi_ladder",
        feed2: actual_feed2,
        max_lincnt: if max_lincnt != 0 { max_lincnt } else { target_lincnt },
        target_lincnt,
        oversample: model.oversample_for(dpi),
        y2,
        travel_mm: py_round_to(t, 3),
        dpi,
        geometry_lincnt: None,
        optical_line_count: None,
        area: None,
        pixels: None,
        requested_area: None,
        effective_area: None,
        lincnt_clamped: None,
    };
    (area, meta)
}

/// Force image `LINCNT` / USB line count to a capture table value.
pub fn apply_target_lincnt(geometry: &ScanGeometry, target_lincnt: i64) -> ScanGeometry {
    let per_line = geometry.lincnt_per_line.max(1);
    let halved = geometry.register_lincnt != 0 && geometry.optical_line_count * 2 == geometry.register_lincnt;
    let lines = (target_lincnt / per_line - geometry.num_staggered_lines).max(1);
    ScanGeometry {
        register_lincnt: target_lincnt,
        optical_line_count: if halved { target_lincnt / 2 } else { target_lincnt },
        lines,
        ..geometry.clone()
    }
}

/// Build SE geometry for Full window (`preview_safe`) or PPI ladder.
pub fn bringup_scan_geometry(model: &FilmModel, dpi: i64, profile: GeometryProfile) -> Result<(ScanGeometry, GeometryMeta)> {
    let (area, mut meta) = match profile {
        GeometryProfile::PpiLadder => ladder_scan_area(model, dpi),
        GeometryProfile::PreviewSafe => preview_safe_scan_area(model, dpi, 0.0),
    };
    let geometry = apply_target_lincnt(&compute_geometry(dpi, model, Some(area))?, meta.target_lincnt);
    meta.geometry_lincnt = Some(geometry.lincnt_register());
    meta.optical_line_count = Some(geometry.optical_line_count);
    meta.area = geometry.area;
    meta.pixels = Some(geometry.pixels);
    Ok((geometry, meta))
}

/// Deprecated: shrinking the image window to the AHB table shears USB lines.
pub fn clamp_area_to_shading_table(model: &FilmModel, dpi: i64, area: Area) -> Result<Area> {
    let probe = compute_geometry(dpi, model, Some(area))?;
    let table_n = shading_width_for_resolution(dpi);
    let n = shading_acquire_width(probe.pixel_startx, probe.pixel_endx, probe.register_dpiset, model.optical_resolution)?;
    if n <= table_n {
        return Ok(area);
    }
    let [x1, y1, x2, y2] = area;
    let width = (x2 - x1).max(1e-9);
    Ok([x1, y1, x1 + width * (table_n as f64 / n as f64), y2])
}

/// Clamp a normalized TA rect to `0..1` with a tiny positive size.
pub fn clamp_area(area: Area) -> Area {
    let [x1, y1, mut x2, mut y2] = area.map(|v| v.clamp(0.0, 1.0));
    if x2 <= x1 {
        x2 = (x1 + 1e-3).min(1.0);
    }
    if y2 <= y1 {
        y2 = (y1 + 1e-3).min(1.0);
    }
    [x1, y1, x2, y2]
}

/// Map Prescan crop widget coords (display space, after `mirror_x`) → TA `area`.
pub fn image_crop_to_scan_area(model: &FilmModel, crop_norm: Area) -> Area {
    let [mut x1, y1, mut x2, y2] = clamp_area(crop_norm);
    if model.mirror_x() {
        (x1, x2) = (1.0 - x2, 1.0 - x1);
    }
    clamp_area([x1, y1, x2, y2])
}

/// Map TA `area` → Prescan crop widget coords (the same involution).
pub fn scan_area_to_image_crop(model: &FilmModel, area: Area) -> Area {
    image_crop_to_scan_area(model, area)
}

/// Centered ~35 mm frame crop in TA space (the whole area off GL128).
pub fn default_frame_crop_norm(model: &FilmModel) -> Area {
    if !is_gl128_opticfilm(model) {
        return [0.0, 0.0, 1.0, 1.0];
    }
    clamp_area(ladder_scan_area(model, PRESCAN_DPI).0)
}

/// Normalized TA rect actually covered by `geometry`.
pub fn effective_scan_area(model: &FilmModel, geometry: &ScanGeometry, requested: Area) -> Area {
    let [x1, y1, _, _] = clamp_area(requested);
    let width_px = (geometry.pixels - geometry.usb_end_drop).max(1);
    let width_mm = width_px as f64 * MM_PER_INCH / geometry.asic_dpi().max(1) as f64;
    let eff_x2 = (x1 + width_mm / model.x_size_ta_mm).max(x1 + 1e-6).min(1.0);
    let mut out_mm = geometry.travel_mm();
    if geometry.frame_lines > 0 {
        out_mm = out_mm.min(geometry.frame_lines as f64 * MM_PER_INCH / geometry.asic_dpi().max(1) as f64);
    }
    let eff_y2 = (y1 + out_mm / model.y_size_ta_mm).max(y1 + 1e-6).min(1.0);
    clamp_area([x1, y1, eff_x2, eff_y2])
}

/// Human-readable note when optical alignment narrowed the crop.
pub fn crop_adjustment_message(requested: Area, effective: Area) -> Option<String> {
    let (req, eff) = (clamp_area(requested), clamp_area(effective));
    let mut parts = Vec::new();
    if (req[2] - eff[2]).abs() > 1e-4 {
        parts.push(format!("x2 {:.3}→{:.3}", req[2], eff[2]));
    }
    if (req[3] - eff[3]).abs() > 1e-4 {
        parts.push(format!("y2 {:.3}→{:.3}", req[3], eff[3]));
    }
    (!parts.is_empty()).then(|| format!("Crop adjusted ({})", parts.join(", ")))
}

/// Geometry for an arbitrary normalized crop inside the TA window (LINCNT clamped).
pub fn crop_scan_geometry(model: &FilmModel, dpi: i64, area: Area) -> Result<(ScanGeometry, GeometryMeta)> {
    let area = clamp_area(area);
    let feed2 = feed2_for(model, area[1]);
    let max_lincnt = model.max_lincnt_for(feed2, dpi);
    let mut geometry = compute_geometry(dpi, model, Some(area))?;
    let before = geometry.lincnt_register();
    let mut target = before;
    if max_lincnt > 0 && target > max_lincnt {
        geometry = apply_target_lincnt(&geometry, max_lincnt);
        target = max_lincnt;
    }
    let effective = effective_scan_area(model, &geometry, area);
    let meta = GeometryMeta {
        profile: "crop",
        feed2,
        max_lincnt: if max_lincnt != 0 { max_lincnt } else { target },
        target_lincnt: target,
        oversample: model.oversample_for(dpi),
        y2: area[3],
        travel_mm: py_round_to(travel_mm(model, target, dpi), 3),
        dpi,
        geometry_lincnt: Some(geometry.lincnt_register()),
        optical_line_count: Some(geometry.optical_line_count),
        area: Some(geometry.area.unwrap_or(area)),
        pixels: Some(geometry.pixels),
        requested_area: Some(area),
        effective_area: Some(effective),
        lincnt_clamped: Some(target < before),
    };
    Ok((geometry, meta))
}
