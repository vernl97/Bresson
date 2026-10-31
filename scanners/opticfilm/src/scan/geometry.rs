// SPDX-License-Identifier: GPL-3.0-or-later
//! Scan geometry helpers for OpticFilm models.

use crate::device::model::FilmModel;
use crate::device::sensor_lookup::{dummy_pixel_for, exposure_lperiod_for};
use crate::device::ScanMethod;
use crate::error::{Error, Result};
use crate::num::{floor_div, py_int, py_round};

pub const MM_PER_INCH: f64 = 25.4;

/// Normalized TA crop `(x1, y1, x2, y2)` in 0..1.
pub type Area = [f64; 4];

#[derive(Debug, Clone, PartialEq)]
pub struct ScanGeometry {
    pub resolution: i64,
    pub pixels: i64,
    pub lines: i64,
    pub startx: i64,
    pub starty: i64,
    pub pixel_startx: i64,
    pub pixel_endx: i64,
    pub optical_pixels: i64,
    pub register_dpiset: i64,
    pub output_pixel_offset: i64,
    pub shift_r: i64,
    pub shift_g: i64,
    pub shift_b: i64,
    pub optical_line_count: i64,
    pub line_bytes: i64,
    pub stagger_y: Vec<i64>,
    pub num_staggered_lines: i64,
    pub channels: i64,
    pub depth: i64,
    pub exposure_lperiod: i64,
    pub dummy_pixel: i64,
    pub disable_buffer_full_move: bool,
    /// Value programmed into `LINCNT` (may differ from `optical_line_count` on GL128).
    pub register_lincnt: i64,
    /// `LINCNT` register units per *output* line (1 on GL845, 4 on the GL128 image path).
    pub lincnt_per_line: i64,
    /// Native lines the ASIC covers per output line at `asic_dpi`.
    pub y_oversample: i64,
    /// Normalized TA crop top edge (0..1). Used by GL128 for the second feed.
    pub area_y1: f64,
    /// True when `compute_geometry` used the default full TA window.
    pub is_default_full_frame: bool,
    /// Original normalized area; `None` means full frame.
    pub area: Option<Area>,
    /// When > 1, the ASIC acquired at a higher PPI and the host downsamples.
    pub host_downsample: i64,
    /// Cap on content-aware trim of USB ENDPIXEL dummy columns after decode.
    pub usb_end_drop: i64,
    /// When > 0, assembled rows are cut to this height after alignment.
    pub frame_lines: i64,
}

impl ScanGeometry {
    pub fn max_color_shift(&self) -> i64 {
        self.shift_r.max(self.shift_g).max(self.shift_b)
    }

    pub fn total_bytes(&self) -> i64 {
        self.line_bytes * self.optical_line_count
    }

    /// ASIC `LINCNT` value (falls back to USB line count).
    pub fn lincnt_register(&self) -> i64 {
        if self.register_lincnt != 0 { self.register_lincnt } else { self.optical_line_count }
    }

    /// Resolution the ASIC is programmed at (>= `resolution`).
    pub fn asic_dpi(&self) -> i64 {
        self.resolution * self.host_downsample.max(1)
    }

    /// Physical Y distance the carriage covers during this pass.
    pub fn travel_mm(&self) -> f64 {
        let per_line = self.lincnt_per_line.max(1);
        self.lincnt_register() as f64 * MM_PER_INCH / (per_line * self.asic_dpi()) as f64
    }
}

fn stagger_for(resolution: i64, model: &FilmModel) -> (Vec<i64>, i64) {
    let shifts = model
        .stagger_y_for(resolution)
        .unwrap_or_else(|| model.stagger_y_by_dpi.get(&resolution).cloned().unwrap_or_default());
    if shifts.is_empty() {
        return (Vec::new(), 0);
    }
    let m = *shifts.iter().max().unwrap();
    (shifts, m)
}

/// Scale base-ydpi line-distance shift to `asic_dpi` (truncating, or the model's rounding).
fn scaled_ld_shift(ld: i64, asic_dpi: i64, model: &FilmModel) -> i64 {
    model.scaled_ld_shift(ld, asic_dpi).unwrap_or_else(|| floor_div(ld * asic_dpi, model.motor_base_ydpi))
}

fn lcm(a: i64, b: i64) -> i64 {
    fn gcd(a: i64, b: i64) -> i64 {
        if b == 0 { a.abs() } else { gcd(b, a % b) }
    }
    if a == 0 || b == 0 { 0 } else { (a / gcd(a, b) * b).abs() }
}

/// Session geometry for a color transparency scan.
///
/// `area` is optional normalized (x1,y1,x2,y2) in 0..1 over the TA window.
pub fn compute_geometry(resolution: i64, model: &FilmModel, area: Option<Area>) -> Result<ScanGeometry> {
    if !model.resolutions_dpi.contains(&resolution) {
        return Err(Error::Value(format!("Unsupported resolution {resolution}")));
    }
    let [x1, y1, x2, y2] = area.unwrap_or([0.0, 0.0, 1.0, 1.0]).map(|v| v.clamp(0.0, 1.0));
    if x2 <= x1 || y2 <= y1 {
        return Err(Error::Value("Invalid scan area".into()));
    }
    let full_frame = area.is_none();
    geometry_from_mm(
        resolution,
        model,
        GeomArgs {
            tl_x_mm: model.x_offset_ta_mm + x1 * model.x_size_ta_mm,
            tl_y_mm: model.y_offset_ta_mm + y1 * model.y_size_ta_mm,
            width_mm: (x2 - x1) * model.x_size_ta_mm,
            height_mm: (y2 - y1) * model.y_size_ta_mm,
            starty_override: None,
            disable_buffer_full_move: false,
            area_y1: y1,
            is_default_full_frame: full_frame,
            area: if full_frame { None } else { Some([x1, y1, x2, y2]) },
        },
    )
}

/// Geometry for dark/white shading strip (genesys `init_regs_for_shading`).
pub fn compute_calib_geometry(resolution: i64, model: &FilmModel, height_mm: Option<f64>) -> Result<ScanGeometry> {
    if !model.resolutions_dpi.contains(&resolution) {
        return Err(Error::Value(format!("Unsupported resolution {resolution}")));
    }
    let base = model.motor_base_ydpi as f64;
    let starty = match model.calib_white_home_y_mm {
        // Measured open light path between home and the holder.
        Some(home) => py_int(home * base / MM_PER_INCH),
        None => {
            let move_mm = model.y_offset_calib_white_ta_mm - model.y_offset_sensor_to_ta_mm;
            let s = py_int(move_mm * base / MM_PER_INCH);
            py_int(model.y_offset_ta_mm * base / MM_PER_INCH) + s
        }
    };
    geometry_from_mm(
        resolution,
        model,
        GeomArgs {
            tl_x_mm: 0.0,
            tl_y_mm: 0.0,
            width_mm: model.x_size_calib_mm,
            height_mm: height_mm.unwrap_or(model.y_size_calib_ta_mm),
            starty_override: Some(starty),
            disable_buffer_full_move: true,
            area_y1: 0.0,
            is_default_full_frame: false,
            area: None,
        },
    )
}

struct GeomArgs {
    tl_x_mm: f64,
    tl_y_mm: f64,
    width_mm: f64,
    height_mm: f64,
    starty_override: Option<i64>,
    disable_buffer_full_move: bool,
    area_y1: f64,
    is_default_full_frame: bool,
    area: Option<Area>,
}

fn geometry_from_mm(resolution: i64, model: &FilmModel, a: GeomArgs) -> Result<ScanGeometry> {
    let g = model.gl128.as_deref();
    let dbfm = a.disable_buffer_full_move;
    let asic_dpi = model.asic_dpi_for(resolution);
    let host_downsample = (asic_dpi / resolution).max(1);

    let mut lines = py_int(a.height_mm * asic_dpi as f64 / MM_PER_INCH);
    if lines < 1 {
        return Err(Error::Value("Scan height too small".into()));
    }
    let frame_inches = g.map_or(0.0, |g| g.full_frame_output_inches);
    let mut frame_lines = if a.is_default_full_frame && !dbfm && frame_inches > 0.0 { py_round(frame_inches * asic_dpi as f64) } else { 0 };

    let mut startx = py_int(a.tl_x_mm * asic_dpi as f64 / MM_PER_INCH);
    let starty = a.starty_override.unwrap_or_else(|| py_int(a.tl_y_mm * model.motor_base_ydpi as f64 / MM_PER_INCH));

    let (Some(&offset), Some(&register_dpiset)) =
        (model.output_pixel_offset_by_dpi.get(&resolution), model.register_dpiset_by_dpi.get(&resolution))
    else {
        return Err(Error::Value(format!("No sensor DPI maps for resolution {resolution}")));
    };

    let optical_res = model.optical_resolution;
    let span_align = g.map_or(0, |g| g.optical_span_alignment);
    let native_x = g.is_some_and(|g| g.strpixel_native_units);
    let (pixels, optical_pixels, pixel_startx, pixel_endx, usb_end_drop);
    if native_x {
        // Native 7200 clocks: session 03 full-width STR 242 / END 10610.
        let origin_native = offset * optical_res / asic_dpi;
        let end_inactive = g.map_or(0, |g| g.optical_end_inactive_native);
        let frame_shift = if dbfm || end_inactive <= 0 { 0 } else { end_inactive };
        let factor = (optical_res / asic_dpi).max(1);
        let frame_x_mm = model.x_offset_ta_mm;
        let (px, opx, psx);
        if !dbfm && span_align > 1 && a.tl_x_mm >= frame_x_mm {
            // SilverFast's image window: STR steps from the full-frame STR in
            // whole output pixels, width round(w × dpi) made even.
            let frame_str = origin_native + frame_shift + py_round(frame_x_mm * optical_res as f64 / MM_PER_INCH);
            psx = frame_str + py_round((a.tl_x_mm - frame_x_mm) * asic_dpi as f64 / MM_PER_INCH) * factor;
            let mut p = 2.max(floor_div(py_round(a.width_mm * asic_dpi as f64 / MM_PER_INCH), 2) * 2);
            // Keep the right edge inside the full frame's own window.
            let frame_end = frame_str + floor_div(py_round(model.x_size_ta_mm * asic_dpi as f64 / MM_PER_INCH), 2) * 2 * factor;
            while p > 2 && psx + p * factor > frame_end {
                p -= 2;
            }
            px = p;
            opx = p * factor;
        } else {
            psx = origin_native + frame_shift + py_round(a.tl_x_mm * optical_res as f64 / MM_PER_INCH);
            let mut o = 1.max(py_round(a.width_mm * optical_res as f64 / MM_PER_INCH));
            if span_align > 1 && o >= span_align {
                let align = lcm(span_align, 2 * factor);
                o = (o / align) * align;
            }
            px = 1.max(o * asic_dpi / optical_res);
            opx = px * optical_res / asic_dpi;
        }
        pixels = px;
        optical_pixels = opx;
        pixel_startx = psx;
        pixel_endx = psx + opx;
        startx = 0.max(floor_div(psx * asic_dpi, optical_res) - offset);
        usb_end_drop = if dbfm || end_inactive <= 0 { 0 } else { end_inactive * resolution / optical_res };
    } else {
        let mut px = py_int(a.width_mm * asic_dpi as f64 / MM_PER_INCH);
        // GL845: align to 16 when xres==yres and xres>1200.
        let alignment = g.map_or(16, |g| g.pixel_alignment);
        if alignment > 1 && asic_dpi > 1200 {
            px = (px / alignment) * alignment;
        }
        let mut opx = px * optical_res / asic_dpi;
        if span_align > 1 && opx >= span_align {
            let factor = (optical_res / asic_dpi).max(1);
            let align = lcm(span_align, 2 * factor);
            opx = (opx / align) * align;
            px = 1.max(opx * asic_dpi / optical_res);
            opx = px * optical_res / asic_dpi;
        }
        pixels = px;
        optical_pixels = opx;
        pixel_startx = floor_div((startx + offset) * optical_res, asic_dpi);
        pixel_endx = pixel_startx + opx;
        usb_end_drop = 0;
    }
    if pixels < 16 {
        return Err(Error::Value("Scan width too small".into()));
    }

    let shift_r = scaled_ld_shift(model.ld_shift_r, asic_dpi, model);
    let shift_g = scaled_ld_shift(model.ld_shift_g, asic_dpi, model);
    let shift_b = scaled_ld_shift(model.ld_shift_b, asic_dpi, model);
    let max_shift = shift_r.max(shift_g).max(shift_b);
    let (stagger_y, num_staggered) = stagger_for(asic_dpi, model);
    let y_oversample = if g.is_some_and(|g| g.y_oversampled) { (optical_res / asic_dpi).max(1) } else { 1 };
    let mut lincnt_per_line = g.map_or(0, |g| g.image_lincnt_per_line);
    if dbfm || lincnt_per_line <= 0 {
        lincnt_per_line = y_oversample;
    }
    let mut extra_lines = num_staggered;
    if g.is_none_or(|g| g.lincnt_includes_line_shift) {
        extra_lines += max_shift;
    }
    if g.is_some() && !dbfm {
        let out_lines = if frame_lines != 0 { frame_lines } else { lines };
        let mut total = out_lines + model.image_line_margin(resolution);
        if a.area.is_some() {
            let feed2 = model.feed_to_scan_steps_for_area(a.area);
            let room = model.max_safe_lincnt_for(feed2, resolution) / lincnt_per_line;
            total = total.min(room.max(out_lines + extra_lines));
        }
        lines = total - extra_lines;
        frame_lines = out_lines;
    }
    let register_lincnt = (lines + extra_lines) * lincnt_per_line;

    let channels = 3;
    let depth = match g {
        Some(g) if dbfm => g.usb_calib_depth,
        Some(g) => g.usb_image_depth,
        None => 16,
    };
    if depth != 8 && depth != 16 {
        return Err(Error::Value(format!("Unsupported USB depth {depth}")));
    }
    // Wire samples are 16-bit LE chunky: USB line count is LINCNT/2 on GL128 images.
    let mut optical_line_count = register_lincnt;
    if !dbfm && depth == 16 && g.is_some_and(|g| g.usb_image_lincnt_half_lines) {
        optical_line_count = register_lincnt / 2;
    }
    let line_bytes = pixels * channels * (depth / 8);
    let mut register_dpiset_out = register_dpiset;
    if dbfm && g.is_some_and(|g| g.calib_uses_native_dpiset) {
        register_dpiset_out = model.optical_resolution / 6;
    }
    let exposure_lperiod = if g.is_some() {
        model.line_period_for(resolution, Some(pixel_endx - pixel_startx))
    } else {
        exposure_lperiod_for(model, resolution, ScanMethod::Transparency)
    };

    Ok(ScanGeometry {
        resolution,
        pixels,
        lines,
        startx,
        starty,
        pixel_startx,
        pixel_endx,
        optical_pixels,
        register_dpiset: register_dpiset_out,
        output_pixel_offset: offset,
        shift_r,
        shift_g,
        shift_b,
        optical_line_count,
        line_bytes,
        stagger_y,
        num_staggered_lines: num_staggered,
        channels,
        depth,
        exposure_lperiod,
        dummy_pixel: dummy_pixel_for(model, resolution),
        disable_buffer_full_move: dbfm,
        register_lincnt,
        lincnt_per_line,
        y_oversample,
        area_y1: a.area_y1,
        is_default_full_frame: a.is_default_full_frame,
        area: a.area,
        host_downsample,
        usb_end_drop: usb_end_drop.min(0.max(pixels - 16)).max(0),
        frame_lines,
    })
}
