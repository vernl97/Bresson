// SPDX-License-Identifier: GPL-3.0-or-later
//! Scan session for GL128 OpticFilm (8200i SE, 8100 V2, and 8300i SE).
//!
//! What differs from GL845: the register block comes from the model tables;
//! motor slope tables are replayed from the capture; feeding is a separate,
//! synchronous move before the scan starts; the image is streamed with
//! `VALUE_BUFFER` announces of at most `image_read_chunk_bytes`.

use std::io::Write;
use std::time::Instant;

use log::{info, warn};
use ndarray::{Array2, Array3, s};

use super::calib_gl128::{
    IR_FLATTEN_BORDER_INSET, IR_FLATTEN_MAX_GAIN, IR_SIDECAR_TARGET_LEVEL, flatten_ir_columns, flatten_ir_image_columns,
};
use super::calibrate::{CalibEntry, Calibrator};
use super::exposure_merge::{MergeStats, merge_exposures_result, merge_n_passes};
use super::exposure_override::{validate_manual_exposure, validate_n_passes};
use super::geometry::{ScanGeometry, compute_geometry};
use super::me_debug::{MeScanDebug, MultiPassDebug, SlotStackDebug};
use super::me_exposure::{LongExposureParams, MeExposureDecision, MeLongMode, select_long_exposure};
use super::pipeline::{
    HOST_CALIB_BORDER_INSET, HOST_CALIB_HIGHLIGHT_CEILING, HOST_CALIB_HIGHLIGHT_PERCENTILE, HOST_CALIB_PEAK_PERCENTILE, HOST_CALIB_PEAK_TARGET, ImagePipeline,
    inset_slice,
};
use super::request::{Progress, ScanControl, ScanMode, ScanRequest, SubProgress};
use super::session::{DATA_TIMEOUT_S, HostCalib, device_model, green_plane, set16, set24};
use crate::asic::Gl128;
use crate::asic::gl128::MOTOR_GATED_HINT;
use crate::asic::registers::gl128 as r;
use crate::clock::{monotonic, sleep};
use crate::device::model::{FilmModel, RegMap, ScanMethod};
use crate::device::tables::exposure_table;
use crate::error::{Error, Result};
use crate::image::ScanImage;
use crate::pass_align::{align_pass_to_reference, align_pass_to_reference_banded, estimate_pass_shift};
use crate::usb::device::BULK_MAX_SIZE;

/// Default max bytes announced per `bulk_read_begin`.
pub const IMAGE_CHUNK_BYTES: usize = BULK_MAX_SIZE;
/// Environment variable naming a CSV that each image read appends per-chunk timing to.
pub const ACQUIRE_TIMING_ENV: &str = "PYOPTICFILM_ACQUIRE_TIMING";
/// Low bits of `0x1D` for an image pass, by ASIC dpi.
const IMAGE_REG_1D_LOW_BY_DPI: [(i64, u16); 1] = [(7200, 0x01)];
const REG_1D: u16 = 0x1D;
const REG_1D_LOW_MASK: u16 = 0x1F;
/// Margin under the 16-bit AHB scan motor table wrap (issue #66).
const ME_CHANNEL_EXPOSURE_MAX: i64 = 64_000;
const INSTANT_READ_BPS: f64 = 20_000_000.0;
const INSTANT_READ_SLACK_S: f64 = 0.002;

/// Clamp ME colour-long exposure to the shared range and the AHB table.
pub fn clamp_me_long(model: &FilmModel, resolution: i64, exp_long: i64) -> i64 {
    let lo = model.me_long_clamp_min.unwrap_or(14000);
    let hi = model.me_long_clamp_max.unwrap_or(85000);
    let oversample = model.oversample_for(resolution).max(1);
    let hi = hi.min(ME_CHANNEL_EXPOSURE_MAX * oversample);
    exp_long.max(lo).min(hi)
}

/// Lowest scan motor table word for a driver-derived image pass at oversample 1.
pub fn native_line_floor(model: &FilmModel) -> i64 {
    model.g().exposure_short * model.g().native_image_exposure_factor.max(1)
}

/// Raise for multi-exposure / Multi-Pass where they are disabled.
pub fn check_gl128_multi_pass(model: &FilmModel, resolution: i64, multi_exposure: bool, n_passes: i64) -> Result<()> {
    if !(multi_exposure || n_passes > 1) || !model.is_gl128() || model.multi_pass_allowed_at(resolution) {
        return Ok(());
    }
    let mode = if multi_exposure { "multi_exposure=True".to_string() } else { format!("n_passes={n_passes}") };
    Err(Error::Scan(format!(
        "{mode} is not supported at {resolution} dpi on {}: multi-exposure and Multi-Pass are disabled at native resolution. \
         Scan a single pass, or pick a lower resolution.",
        model.model
    )))
}

/// Second-feed steps for an image pass.
pub fn image_feed2_steps(model: &FilmModel, geometry: &ScanGeometry, long_exposure: bool) -> i64 {
    let mut area = geometry.area;
    if area.is_none() && geometry.area_y1 > 1e-9 {
        area = Some([0.0, geometry.area_y1, 1.0, 1.0]);
    }
    match area {
        None => model.feed_to_scan_steps_for_dpi(geometry.resolution, long_exposure),
        Some(a) => model.feed_to_scan_steps_for_area(Some(a)),
    }
}

/// GL128 scan state machine.
pub struct Gl128ScanSession {
    pub model: FilmModel,
    pub pipeline: ImagePipeline,
    lamp_requested: bool,
    bulk_stream_active: bool,
    pass_exposure: Option<i64>,
    pass_long_exposure: bool,
    pass_manual: bool,
    pub last_me_debug: Option<MeScanDebug>,
    pub last_multi_pass_debug: Option<MultiPassDebug>,
    pub last_align_shift_ir: Option<(f64, f64)>,
    /// True when this scan runs colour calibration (otherwise DVDSET stays off).
    use_asic_shading: bool,
}

type Shift = (f64, f64);

impl Gl128ScanSession {
    pub fn new(model: FilmModel) -> Self {
        Self {
            pipeline: ImagePipeline::new(model.clone()),
            model,
            lamp_requested: true,
            bulk_stream_active: false,
            pass_exposure: None,
            pass_long_exposure: false,
            pass_manual: false,
            last_me_debug: None,
            last_multi_pass_debug: None,
            last_align_shift_ir: None,
            use_asic_shading: false,
        }
    }

    /// Exposure state for the next [`Gl128ScanSession::configure`] (multi-pass internals, tests).
    pub fn set_pass(&mut self, exposure: Option<i64>, long_exposure: bool, manual: bool) {
        self.pass_exposure = exposure;
        self.pass_long_exposure = long_exposure;
        self.pass_manual = manual;
    }

    /// Refuse unless the ASIC explicitly arms motor moves.
    pub fn run(&mut self, asic: &mut Gl128, calibrator: Option<&mut Calibrator>, req: &ScanRequest, ctl: &mut ScanControl) -> Result<ScanImage> {
        if !asic.motor_moves_enabled {
            return Err(Error::Asic(MOTOR_GATED_HINT.into()));
        }
        validate_manual_exposure(req.single_pass_exposure, "single_pass_exposure")?;
        validate_manual_exposure(req.me_short_exposure, "me_short_exposure")?;
        validate_manual_exposure(req.me_long_exposure, "me_long_exposure")?;
        validate_n_passes(req.n_passes)?;
        self.use_asic_shading = req.apply_calib && calibrator.is_some();
        check_gl128_multi_pass(&self.model, req.resolution(), req.multi_exposure, req.n_passes)?;
        if req.multi_exposure || req.n_passes > 1 || (req.infrared && req.mode == ScanMode::Color) {
            if req.infrared && !self.model.supports_infrared {
                return Err(Error::Scan(format!(
                    "{} has no infrared channel: IR scans are unavailable on this model. Scan with mode='color', \
                     infrared=False (optionally multi_exposure=True).",
                    self.model.model
                )));
            }
            return self.run_multi_pass(asic, calibrator, req, ctl);
        }
        asic.enter_scan_clock()?;
        if let Some(e) = req.single_pass_exposure {
            self.pass_exposure = Some(e);
            self.pass_manual = true;
        }
        // Python forwards only the base keywords to `ScanSession.run`.
        let base = ScanRequest {
            multi_exposure: false,
            infrared: false,
            align_passes: true,
            single_pass_exposure: None,
            me_short_exposure: None,
            me_long_exposure: None,
            me_long_mode: MeLongMode::Adaptive,
            n_passes: 1,
            ..req.clone()
        };
        let r = self.run_single(asic, calibrator, &base, ctl);
        self.pass_exposure = None;
        self.pass_manual = false;
        r
    }

    /// The base `ScanSession.run` single-pass path with GL128 hooks.
    fn run_single(&mut self, asic: &mut Gl128, calibrator: Option<&mut Calibrator>, req: &ScanRequest, ctl: &mut ScanControl) -> Result<ScanImage> {
        let model = self.model.clone();
        if req.infrared && !model.supports_infrared {
            return Err(Error::Value(format!("{} does not support infrared", model.model)));
        }
        if req.multi_exposure || req.infrared {
            return Err(Error::NotImplemented("Multi-exposure / combined IR scans are not implemented for GL128".into()));
        }
        if req.me_short_exposure.is_some() || req.me_long_exposure.is_some() {
            return Err(Error::NotImplemented("ME exposure overrides are only implemented for GL128 and GL845; GL128 does not support them.".into()));
        }
        if req.single_pass_exposure.is_some() {
            return Err(Error::NotImplemented("Manual exposure overrides are only implemented for GL128 and GL845; GL128 does not support single_pass_exposure.".into()));
        }
        if req.mode == ScanMode::Gray {
            return Err(Error::Value("Grayscale is experimental / not implemented yet".into()));
        }
        if req.mode == ScanMode::Infrared && !model.supports_infrared {
            return Err(Error::Value(format!("{} does not support infrared", model.model)));
        }
        if !asic.initialized {
            asic.init(false)?;
        }
        let method = req.mode.method();
        let geometry = match &req.geometry {
            Some(g) => g.clone(),
            None => {
                let mut g = compute_geometry(req.resolution, &model, req.area)?;
                let lp = crate::device::sensor_lookup::exposure_lperiod_for(&model, g.resolution, method);
                if lp != g.exposure_lperiod {
                    g.exposure_lperiod = lp;
                }
                g
            }
        };
        info!(
            "scan {}dpi {} pixels={} lines={} starty={} bytes={} stagger={} combined_ir=False",
            geometry.resolution,
            req.mode.as_str(),
            geometry.pixels,
            geometry.lines,
            geometry.starty,
            geometry.total_bytes(),
            geometry.num_staggered_lines
        );
        let rgb = self.acquire_pass_single(asic, calibrator, &geometry, method, req.apply_calib, ctl)?;
        let ir = if req.mode == ScanMode::Infrared { Some(self.infrared_plane(asic, &rgb)?) } else { None };
        Ok(ScanImage::new(rgb, geometry.resolution, device_model(&model), ir))
    }

    /// Base `_acquire_pass` with the GL128 calibration branch.
    fn acquire_pass_single(&mut self, asic: &mut Gl128, calibrator: Option<&mut Calibrator>, geometry: &ScanGeometry, method: ScanMethod, apply_calib: bool, ctl: &mut ScanControl) -> Result<Array3<u16>> {
        let mut calib: Option<CalibEntry> = None;
        let mut use_host = false;
        if apply_calib && let Some(cal) = calibrator {
            if method == ScanMethod::Transparency {
                // SilverFast / SE: ASIC shading before any image feed.
                calib = Some(cal.ensure_colour_asic_shading(asic, geometry)?);
            } else {
                cal.measure_ir_host_flatten(asic, geometry)?;
            }
            use_host = calib.is_some() && cal.should_apply_host_calib(asic.asic_shading_ready);
        }
        let raw = self.acquire_raw(asic, geometry, method, true, true, ctl)?;
        if calib.is_some() && !use_host {
            info!("Using ASIC shading; skipping host dark/white stretch");
        }
        let hc = calib.as_ref().filter(|_| use_host).map(HostCalib::from_entry);
        let planar = asic.usb_planar_rgb;
        self.pipeline.assemble(&raw, geometry, hc.as_ref().map(|h| &h.dark), hc.as_ref().map(|h| &h.white), Some(planar), true, None)
    }

    /// Build the HxW IR sidecar: green CCD plane + host flatten.
    fn infrared_plane(&self, asic: &Gl128, rgb: &Array3<u16>) -> Result<Array2<u16>> {
        let mut plane = green_plane(rgb);
        if asic.ir_host_flatten_ready
            && let Some(white) = asic.last_ir_host_white.as_ref().filter(|w| !w.is_empty())
        {
            // The strip profile is in USB column order; `rgb` is already mirrored.
            let mut white = white.clone();
            if self.model.mirror_x() {
                white.truncate(plane.ncols());
                white.reverse();
            }
            match flatten_ir_columns(plane.view(), &white, None, None) {
                Ok(p) => plane = p,
                Err(e) => warn!("IR host-white flatten skipped: {e}"),
            }
        }
        flatten_ir_image_columns(plane.view(), 90.0, Some(IR_SIDECAR_TARGET_LEVEL), None, IR_FLATTEN_BORDER_INSET, IR_FLATTEN_MAX_GAIN)
    }

    /// Configure ASIC, run one capture, return raw optical bytes.
    pub fn acquire_raw(&mut self, asic: &mut Gl128, geometry: &ScanGeometry, method: ScanMethod, lamp_on: bool, start_motor: bool, progress: &mut dyn Progress) -> Result<Vec<u8>> {
        asic.set_scan_method(method);
        self.lamp_requested = lamp_on;
        if lamp_on {
            asic.lamp_on()?;
        } else {
            asic.lamp_off()?;
        }
        let r = (|| {
            self.configure(asic, geometry)?;
            self.begin_scan(asic, start_motor)?;
            self.acquire(asic, geometry, progress)
        })();
        let end = self.end_scan(asic);
        match r {
            Ok(raw) => {
                end?;
                Ok(raw)
            }
            Err(e) => {
                if let Err(e2) = end {
                    warn!("end_scan after failed capture: {e2}");
                }
                Err(e)
            }
        }
    }

    // --- configure ---

    /// Program the ASIC for `geometry` (feeds only when motor moves are armed).
    pub fn configure(&mut self, asic: &mut Gl128, geometry: &ScanGeometry) -> Result<()> {
        let model = self.model.clone();
        let g = model.g();
        let dpi = geometry.resolution;
        let shading = geometry.disable_buffer_full_move;
        let mut cache = model.boot_register_map();
        cache.update(&model.gpo_regs);
        cache.update(&model.sensor_custom_regs);
        let asic_dpi = model.asic_dpi_for(dpi);
        let no_tables = || {
            let mut res = model.resolutions_dpi.clone();
            res.sort_unstable();
            Error::Scan(format!("No capture-derived register values for {dpi} dpi on {}; supported: {res:?}", model.model))
        };
        cache.insert(0x2B, *g.dummy_by_dpi.get(&asic_dpi).ok_or_else(no_tables)? as u16);
        let clk = model.pixel_clock_for_image(dpi, self.pass_long_exposure);
        cache.insert(0xA5, clk as u16);
        cache.insert(0xAB, clk as u16);
        // Image: DEPTH8 registers but 16-bit LE samples on the wire.
        if shading {
            cache.insert(r::REG_DEPTH_A, r::DEPTH16_A as u16);
            cache.insert(r::REG_DEPTH_B, r::DEPTH16_B as u16);
        } else {
            cache.insert(r::REG_DEPTH_A, r::DEPTH8_A as u16);
            cache.insert(r::REG_DEPTH_B, r::DEPTH8_B as u16);
        }
        // DVDSET only with a measured ASIC table for this scan, never on IR or calib.
        let mut reg01 = (cache.get_or(r::REG_0X01, 0x22) | r::SHDAREA as u16) & !(r::SCAN as u16) & !(r::STAGGER as u16);
        let infrared = asic.scan_method == ScanMethod::Infrared;
        let ready = self.use_asic_shading && asic.asic_shading_ready;
        if shading || infrared || !ready {
            reg01 &= !(r::DVDSET as u16);
        }
        cache.insert(r::REG_0X01, reg01);
        // Leave lamp / IR LED and the AFE ports alone.
        for a in [r::REG_0X03, r::REG_IR, r::REG_FE_INDEX, r::REG_FE_HIGH, r::REG_FE_LOW] {
            cache.remove(&a);
        }
        if !shading {
            let low = crate::num::lookup(&IMAGE_REG_1D_LOW_BY_DPI, asic_dpi).unwrap_or(0);
            cache.insert(REG_1D, (cache.get_or(REG_1D, 0x80) & !REG_1D_LOW_MASK) | low);
        }
        let mut motor = r::MTRPWR;
        if !shading {
            // Image pass: AGOHOME parks the carriage when the scan ends.
            motor |= r::AGOHOME;
        }
        cache.insert(r::REG_0X02, motor as u16);
        set24(&mut cache, r::REG_LINCNT, geometry.lincnt_register());
        let span = geometry.pixel_endx - geometry.pixel_startx;
        let lperiod = model.line_period_for(dpi, Some(span));
        set24(&mut cache, r::REG_LPERIOD, lperiod);
        let dpiset = if shading { model.optical_resolution / 6 } else { geometry.register_dpiset };
        set16(&mut cache, r::REG_DPISET, dpiset);
        set24(&mut cache, r::REG_STRPIXEL, geometry.pixel_startx);
        set24(&mut cache, r::REG_ENDPIXEL, geometry.pixel_endx);
        verify_geometry_usb_span(geometry)?;
        let mut exposure_reg = self.pass_exposure.unwrap_or_else(|| model.image_exposure(false));
        if self.pass_manual {
            info!("REG_EXPOSURE {exposure_reg} — manual-override, hardware-max clamp bypassed");
        } else if exposure_reg > g.me_hardware_max_exposure {
            warn!("REG_EXPOSURE {exposure_reg} above hardware max {} — clamping", g.me_hardware_max_exposure);
            exposure_reg = g.me_hardware_max_exposure;
        }
        set24(&mut cache, r::REG_EXPOSURE, exposure_reg);
        // Image/calib acquire with FEEDL=1; positioning is a separate feed pair.
        set24(&mut cache, r::REG_FEEDL, 1);
        cache.remove(&r::REG_CLRCNT);
        cache.remove(&r::REG_START);

        let mut channel_exp = model.channel_exposure_for(dpi, Some(exposure_reg));
        if !shading && !self.pass_manual && model.oversample_for(dpi) == 1 {
            channel_exp = channel_exp.max(native_line_floor(&model));
        }
        // Validate before any motor move below.
        exposure_table(channel_exp)?;

        if !shading && asic.motor_moves_enabled {
            let scan_steps = image_feed2_steps(&model, geometry, self.pass_long_exposure);
            let max_lc = model.max_safe_lincnt_for(scan_steps, dpi);
            if geometry.lincnt_register() > max_lc {
                let start_mm = scan_steps as f64 * 25.4 / g.feed_steps_per_inch as f64;
                return Err(Error::Scan(format!(
                    "Image LINCNT {} at {dpi} dpi is {:.1} mm of travel from feed2={scan_steps} ({start_mm:.1} mm), past the {:.1} mm \
                     travel limit. Max LINCNT here is {max_lc} (see docs/gl128-model-comparison.md, section 4).",
                    geometry.lincnt_register(),
                    geometry.travel_mm(),
                    g.max_travel_end_steps as f64 * 25.4 / g.feed_steps_per_inch as f64
                )));
            }
            // Feeds run under the image registers, with the white shading strip's dummy.
            let mut feed_regs = cache.clone();
            feed_regs.insert(0x2B, model.shading_strip_clocks(dpi, true, None).0 as u16);
            asic.position_for_full_frame_scan(Some(scan_steps), 60.0, Some(&feed_regs))?;
        }
        asic.upload_tables(dpi, shading, true, Some(channel_exp))?;
        // Re-apply the last AFE search (never set_frontend_init here).
        if let Some(afe) = asic.last_afe {
            asic.apply_frontend(afe)?;
        }
        let pairs = cache.sorted_bytes();
        asic.protocol.write_registers_batched(&pairs, 32)?;
        for (&a, &v) in cache.iter() {
            asic.reg_cache.insert(a, v);
        }
        if !shading {
            // The next stationary strip block re-initialises the ASIC first (B8).
            asic.image_pass_since_init = true;
        }
        if self.lamp_requested {
            asic.lamp_on()?;
        } else {
            asic.lamp_off()?;
        }
        info!(
            "GL128 configured {dpi}dpi dpiset={dpiset} lincnt={} str={} end={} lperiod={lperiod} shading={} exposure={exposure_reg}",
            geometry.lincnt_register(),
            geometry.pixel_startx,
            geometry.pixel_endx,
            if shading { "True" } else { "False" }
        );
        Ok(())
    }

    // --- multi-pass (ME / IR) ---

    fn run_multi_pass(&mut self, asic: &mut Gl128, mut calibrator: Option<&mut Calibrator>, req: &ScanRequest, ctl: &mut ScanControl) -> Result<ScanImage> {
        if req.mode == ScanMode::Infrared {
            return Err(Error::Value("Use mode='color' with infrared=True for colour+IR scans".into()));
        }
        let n_passes = req.n_passes;
        if req.infrared && n_passes > 1 {
            return Err(Error::Scan(
                "n_passes > 1 is not supported together with infrared=True yet (each repeat is its own motor cycle and IR stacking \
                 is unvalidated) — scan infrared separately, or use n_passes=1."
                    .into(),
            ));
        }
        let model = self.model.clone();
        let g = model.g().clone();
        let (short_manual, exp_short) = match (req.me_short_exposure, req.single_pass_exposure) {
            (Some(e), _) => (true, e),
            (None, Some(e)) if !req.multi_exposure => (true, e),
            _ => (false, g.exposure_short),
        };
        let mut exp_long = model.exposure_long.unwrap_or(exp_short * 3);
        if !asic.initialized {
            asic.init(false)?;
        }
        self.last_me_debug = None;
        self.last_multi_pass_debug = None;
        self.last_align_shift_ir = None;
        let geometry = match &req.geometry {
            Some(gm) => gm.clone(),
            None => compute_geometry(req.resolution, &model, req.area)?,
        };
        let n_pass = n_passes + req.infrared as i64 + if req.multi_exposure { n_passes } else { 0 };
        info!(
            "GL128 multi-pass {}dpi passes={n_pass} n_passes={n_passes} me={} ir={}",
            geometry.resolution,
            if req.multi_exposure { "True" } else { "False" },
            if req.infrared { "True" } else { "False" }
        );
        let mut st = PassState { locked_usb_end_drop: None, pass_idx: 0, n_pass };

        let (rgb_short, short_shifts, short_stats) =
            self.capture_stacked_slot(asic, calibrator.as_deref_mut(), req, ctl, &mut st, &geometry, "color_short", ScanMethod::Transparency, exp_short, false, short_manual, false)?;
        let mut ir_plane = None;
        if req.infrared {
            let ir_raw = self.mp_acquire_pass(asic, calibrator.as_deref_mut(), req, ctl, &mut st, &geometry, "ir", ScanMethod::Infrared, exp_short, false, false, short_manual)?;
            ir_plane = Some(self.infrared_plane(asic, &ir_raw)?);
        }
        let long_manual = req.me_long_exposure.is_some();
        let mut rgb_long = None;
        let mut long_shifts = Vec::new();
        let mut long_stats = None;
        let mut decision: Option<MeExposureDecision> = None;
        let mut fixed_reason: Option<String> = None;
        if req.multi_exposure {
            if let Some(v) = req.me_long_exposure {
                exp_long = v;
                info!("ME long exposure: manual-override selected={exp_long}");
            } else if req.me_long_mode == MeLongMode::Fixed {
                let requested = g.me_fixed_long_exposure;
                exp_long = clamp_me_long(&model, geometry.resolution, requested);
                fixed_reason = Some(if exp_long == requested { "fixed".into() } else { "fixed (clamped)".into() });
                info!("ME long exposure: fixed requested={requested} selected={exp_long}");
            } else {
                let adaptive_max = clamp_me_long(&model, geometry.resolution, g.me_adaptive_max_exposure);
                let hardware_max = clamp_me_long(&model, geometry.resolution, g.me_hardware_max_exposure);
                let (h, w, _) = rgb_short.dim();
                let metered = match inset_slice(h, w, HOST_CALIB_BORDER_INSET) {
                    Some(((y0, y1), (x0, x1))) => rgb_short.slice(s![y0..y1, x0..x1, ..]),
                    None => rgb_short.view(),
                };
                let d = select_long_exposure(
                    metered,
                    exp_short,
                    &LongExposureParams {
                        black_level: g.me_black_level,
                        dense_percentile: g.me_dense_percentile,
                        target_dense_dn: g.me_target_dense_dn,
                        adaptive_min: g.me_adaptive_min_exposure,
                        adaptive_max,
                        hardware_max,
                        max_ratio: model.me_max_exposure_ratio.unwrap_or(5.0),
                        default_long: clamp_me_long(&model, geometry.resolution, exp_long),
                    },
                );
                exp_long = d.selected;
                let clamped = clamp_me_long(&model, geometry.resolution, exp_long);
                if clamped != exp_long {
                    warn!("ME colour-long exposure clamped at {} dpi: {exp_long} → {clamped}", geometry.resolution);
                    exp_long = clamped;
                }
                let (p, c) = (d.dense_p05, d.predicted_clip);
                info!(
                    "ME long exposure: short={exp_short}  R/G/B p05=({:.0}, {:.0}, {:.0})  proposed={}  predicted clip R/G/B=({:.2}%, {:.2}%, {:.2}%)  \
                     safety_max={hardware_max}  selected={exp_long}  reason={}",
                    p[0],
                    p[1],
                    p[2],
                    d.proposed,
                    c[0] * 100.0,
                    c[1] * 100.0,
                    c[2] * 100.0,
                    d.reason
                );
                decision = Some(d);
            }
            let (rl, ls, lstats) =
                self.capture_stacked_slot(asic, calibrator.as_deref_mut(), req, ctl, &mut st, &geometry, "color_long", ScanMethod::Transparency, exp_long, true, long_manual, true)?;
            rgb_long = Some(rl);
            long_shifts = ls;
            long_stats = lstats;
        }
        if n_passes > 1 {
            self.last_multi_pass_debug = Some(MultiPassDebug {
                short: SlotStackDebug { n_passes, align_shifts: short_shifts, stack_stats: short_stats },
                long: req.multi_exposure.then(|| SlotStackDebug { n_passes, align_shifts: long_shifts, stack_stats: long_stats }),
            });
        }
        let mut align_shift_long: Option<Shift> = None;
        let mut align_shift_ir: Option<Shift> = None;
        if req.align_passes
            && let Some(rl) = &rgb_long
        {
            let s = estimate_pass_shift(&rgb_short, rl);
            info!("ME long pass shift (dx, dy)=({:.3}, {:.3})", s.0, s.1);
            align_shift_long = Some(s);
        }
        if req.align_passes
            && let Some(ir) = ir_plane.take()
        {
            let ir3 = ir.insert_axis(ndarray::Axis(2));
            let (aligned, s) = align_pass_to_reference(&rgb_short, &ir3, None)?;
            ir_plane = Some(aligned.index_axis_move(ndarray::Axis(2), 0));
            self.last_align_shift_ir = Some(s);
            align_shift_ir = Some(s);
            info!("IR pass shift (dx, dy)=({:.3}, {:.3})", s.0, s.1);
        }
        let mut primary = rgb_short.clone();
        if req.multi_exposure
            && let Some(rl) = &rgb_long
        {
            let shift = if req.align_passes { align_shift_long } else { Some((0.0, 0.0)) };
            let merged = merge_exposures_result(&rgb_short, rl, exp_short, exp_long, shift, None, None)?;
            primary = merged.rgb;
            self.last_me_debug = Some(MeScanDebug {
                rgb_short: rgb_short.clone(),
                rgb_long: rl.clone(),
                exposure_short: exp_short,
                exposure_long: exp_long,
                fusion_stats: merged.fusion_stats,
                align_shift_long,
                align_shift_ir,
                exposure_proposed: decision.as_ref().map(|d| d.proposed),
                exposure_reason: if long_manual { Some("manual-override".into()) } else { fixed_reason.or_else(|| decision.as_ref().map(|d| d.reason.clone())) },
            });
        }
        // Single film-base makeup on the deliverable only.
        let preserve_headroom = (req.multi_exposure && rgb_long.is_some()) || n_passes == 1;
        primary = self.pipeline.expose_film_base(primary, "me deliverable", HOST_CALIB_PEAK_TARGET, HOST_CALIB_PEAK_PERCENTILE, preserve_headroom, HOST_CALIB_HIGHLIGHT_CEILING, HOST_CALIB_HIGHLIGHT_PERCENTILE);
        primary = self.pipeline.clamp_border_highlights(primary, HOST_CALIB_BORDER_INSET, HOST_CALIB_PEAK_PERCENTILE);
        Ok(ScanImage::new(primary, geometry.resolution, device_model(&model), ir_plane))
    }

    #[allow(clippy::too_many_arguments)]
    fn mp_acquire_pass(
        &mut self,
        asic: &mut Gl128,
        calibrator: Option<&mut Calibrator>,
        req: &ScanRequest,
        ctl: &mut ScanControl,
        st: &mut PassState,
        geometry: &ScanGeometry,
        key: &str,
        method: ScanMethod,
        exposure: i64,
        remeasure: bool,
        long_pass: bool,
        manual: bool,
    ) -> Result<Array3<u16>> {
        let idx = st.pass_idx;
        st.pass_idx += 1;
        asic.enter_scan_clock()?;
        let mut calib = None;
        let mut use_host = false;
        if req.apply_calib && let Some(cal) = calibrator {
            if method == ScanMethod::Transparency {
                if remeasure {
                    asic.asic_shading_ready = false;
                    calib = Some(cal.measure_colour_asic_shading(asic, geometry, false)?);
                } else {
                    calib = Some(cal.ensure_colour_asic_shading(asic, geometry)?);
                }
            } else {
                cal.measure_ir_host_flatten(asic, geometry)?;
            }
            use_host = calib.is_some() && cal.should_apply_host_calib(asic.asic_shading_ready);
        }
        info!(
            "GL128 pass {key} dpi={} area={:?} feed2={} lincnt={} str={} end={} {}x{}",
            geometry.resolution,
            geometry.area,
            image_feed2_steps(&self.model, geometry, long_pass),
            geometry.lincnt_register(),
            geometry.pixel_startx,
            geometry.pixel_endx,
            geometry.pixels,
            geometry.lines
        );
        self.pass_exposure = Some(exposure);
        self.pass_long_exposure = long_pass;
        self.pass_manual = manual;
        let raw = {
            let mut sub = SubProgress { ctl, idx: idx as f64, n: st.n_pass as f64 };
            self.acquire_raw(asic, geometry, method, true, true, &mut sub)
        };
        self.pass_exposure = None;
        self.pass_long_exposure = false;
        self.pass_manual = false;
        let raw = raw?;
        let hc = calib.as_ref().filter(|_| use_host).map(HostCalib::from_entry);
        let planar = asic.usb_planar_rgb;
        // Keep ME colour (and IR) planes linear; makeup runs once on the deliverable.
        let rgb = self.pipeline.assemble(&raw, geometry, hc.as_ref().map(|h| &h.dark), hc.as_ref().map(|h| &h.white), Some(planar), false, st.locked_usb_end_drop)?;
        if st.locked_usb_end_drop.is_none() {
            st.locked_usb_end_drop = Some(self.pipeline.last_usb_end_drop);
        }
        Ok(rgb)
    }

    /// Capture `key` `n_passes` times, align repeats 2..N to repeat 1 and stack.
    #[allow(clippy::too_many_arguments)]
    fn capture_stacked_slot(
        &mut self,
        asic: &mut Gl128,
        mut calibrator: Option<&mut Calibrator>,
        req: &ScanRequest,
        ctl: &mut ScanControl,
        st: &mut PassState,
        geometry: &ScanGeometry,
        key: &str,
        method: ScanMethod,
        exposure: i64,
        long_pass: bool,
        manual: bool,
        remeasure_first: bool,
    ) -> Result<(Array3<u16>, Vec<Shift>, Option<MergeStats>)> {
        let first = self.mp_acquire_pass(asic, calibrator.as_deref_mut(), req, ctl, st, geometry, key, method, exposure, remeasure_first, long_pass, manual)?;
        if req.n_passes == 1 {
            return Ok((first, Vec::new(), None));
        }
        let mut shifts = Vec::new();
        let mut frames = vec![first];
        for _ in 0..req.n_passes - 1 {
            let frame = self.mp_acquire_pass(asic, calibrator.as_deref_mut(), req, ctl, st, geometry, key, method, exposure, false, long_pass, manual)?;
            if req.align_passes {
                let (aligned, shift) = align_pass_to_reference_banded(&frames[0], &frame)?;
                frames.push(aligned);
                shifts.push(shift);
            } else {
                frames.push(frame);
            }
        }
        let stacked = merge_n_passes(&frames, None, None)?;
        Ok((stacked.rgb, shifts, stacked.fusion_stats))
    }

    // --- acquire ---

    /// Capture order: `0x0d=0x07` → set SCAN → `0x0f` (session 03).
    fn begin_scan(&mut self, asic: &mut Gl128, start_motor: bool) -> Result<()> {
        if start_motor {
            asic.wait_motor_idle("before the image pass START")?;
        }
        asic.protocol.write_register(r::REG_CLRCNT, r::CLRCNT_ALL)?;
        let reg01 = asic.protocol.read_register(r::REG_0X01)? | r::SCAN;
        asic.protocol.write_register(r::REG_0X01, reg01)?;
        asic.reg_cache.insert(r::REG_0X01, reg01 as u16);
        asic.protocol.write_register(r::REG_START, if start_motor { r::START_GO } else { 0x00 })?;
        info!("GL128 scan started motor={}", if start_motor { "True" } else { "False" });
        Ok(())
    }

    /// Wait until the ASIC reports data in its buffer.
    fn wait_data(&mut self, asic: &mut Gl128, progress: &dyn Progress) -> Result<()> {
        let deadline = monotonic() + DATA_TIMEOUT_S;
        while monotonic() < deadline {
            progress.check("cancelled waiting for data")?;
            if !asic.read_status()?.is_buffer_empty {
                return Ok(());
            }
            sleep(0.02);
        }
        Err(Error::Scan(format!("No scan data within {DATA_TIMEOUT_S:.0}s")))
    }

    #[doc(hidden)]
    pub fn acquire(&mut self, asic: &mut Gl128, geometry: &ScanGeometry, progress: &mut dyn Progress) -> Result<Vec<u8>> {
        self.wait_data(asic, progress)?;
        let total = geometry.total_bytes() as usize;
        let index = if geometry.disable_buffer_full_move { r::BULK_INDEX_RAM } else { r::BULK_INDEX_IMAGE };
        let chunk_max = (self.model.g().image_read_chunk_bytes as usize).max(1);
        let step = if chunk_max <= BULK_MAX_SIZE { BULK_MAX_SIZE } else { chunk_max };
        let mut buf = vec![0u8; total];
        let mut got = 0usize;
        let mut timing: Vec<(usize, usize, f64, f64)> = Vec::new();
        let t_start = Instant::now();
        let mut t_prev_end = Instant::now();
        let r = (|| -> Result<()> {
            while got < total {
                progress.check("cancelled during bulk read")?;
                let remaining = total - got;
                let want = chunk_bytes(geometry, remaining, chunk_max);
                let p0 = Instant::now();
                asic.protocol.bulk_read_begin(want, index, 0x1000_0000)?;
                self.bulk_stream_active = true;
                let chunk = asic.protocol.bulk_read_exact(want, step);
                self.bulk_stream_active = false;
                let chunk = chunk?;
                if chunk.is_empty() {
                    return Err(Error::Scan(format!("Bulk stream ended after {got} of {total} bytes")));
                }
                let n = chunk.len().min(remaining);
                buf[got..got + n].copy_from_slice(&chunk[..n]);
                got += n;
                progress.report((got as f64 / total as f64).min(1.0));
                timing.push((got - n, n, p0.elapsed().as_secs_f64(), (p0 - t_prev_end).as_secs_f64()));
                t_prev_end = Instant::now();
            }
            Ok(())
        })();
        report_acquire_timing(geometry, &timing, t_start.elapsed().as_secs_f64());
        r?;
        progress.report(1.0);
        Ok(buf)
    }

    fn end_scan(&mut self, asic: &mut Gl128) -> Result<()> {
        // Capture end/cancel recipe lives in Gl128::stop_motor; then abort the
        // host bulk IN so the pipe is not left half-open across close/reopen.
        asic.stop_motor();
        if self.bulk_stream_active {
            self.bulk_stream_active = false;
            let drained = asic.protocol.abort_bulk_stream();
            info!("GL128 bulk abort after end_scan drained={drained}");
        }
        Ok(())
    }
}

struct PassState {
    locked_usb_end_drop: Option<i64>,
    pass_idx: i64,
    n_pass: i64,
}

/// Fail loud when STR/END span disagrees with USB line width (diamond shear).
fn verify_geometry_usb_span(geometry: &ScanGeometry) -> Result<()> {
    let span = geometry.pixel_endx - geometry.pixel_startx;
    if span != geometry.optical_pixels {
        return Err(Error::Scan(format!("STR/END span {span} != optical_pixels {} at {} dpi", geometry.optical_pixels, geometry.resolution)));
    }
    let sample = if geometry.depth == 16 { 2 } else { 1 };
    let expected = geometry.pixels * geometry.channels * sample;
    if geometry.line_bytes != expected {
        return Err(Error::Scan(format!("USB line_bytes {} != pixels×channels×sample ({expected}) at {} dpi", geometry.line_bytes, geometry.resolution)));
    }
    Ok(())
}

/// Bytes for the next announce: whole lines up to `max_bytes`.
pub fn chunk_bytes(geometry: &ScanGeometry, remaining: usize, max_bytes: usize) -> usize {
    let line = (geometry.line_bytes.max(1)) as usize;
    if remaining <= line {
        return remaining;
    }
    if line > max_bytes {
        return line.min(remaining);
    }
    let full = (remaining / line).min(max_bytes / line);
    if full >= 1 { full * line } else { line.min(remaining) }
}

fn read_was_instant(nbytes: usize, seconds: f64) -> bool {
    seconds <= nbytes as f64 / INSTANT_READ_BPS + INSTANT_READ_SLACK_S
}

/// Log one summary line per image read; optionally dump per-chunk CSV.
fn report_acquire_timing(geometry: &ScanGeometry, timing: &[(usize, usize, f64, f64)], wall_s: f64) {
    if timing.is_empty() {
        return;
    }
    let nbytes: usize = timing.iter().map(|t| t.1).sum();
    let instant = timing.iter().filter(|t| read_was_instant(t.1, t.2)).count();
    let longest_read = timing.iter().map(|t| t.2).fold(0.0, f64::max);
    let longest_gap = timing.iter().map(|t| t.3).fold(0.0, f64::max);
    info!(
        "GL128 image read: {} dpi {:.1} MB in {wall_s:.1} s ({:.2} MB/s) chunks={} instant_reads={instant} longest_read={longest_read:.3} s longest_gap={longest_gap:.3} s",
        geometry.resolution,
        nbytes as f64 / 1e6,
        if wall_s > 0.0 { nbytes as f64 / 1e6 / wall_s } else { 0.0 },
        timing.len()
    );
    let Ok(path) = std::env::var(ACQUIRE_TIMING_ENV) else { return };
    if path.is_empty() {
        return;
    }
    let out = std::path::Path::new(&path);
    let new = !out.exists();
    let r = std::fs::OpenOptions::new().create(true).append(true).open(out).and_then(|mut fh| {
        if new {
            writeln!(fh, "dpi,offset,bytes,read_s,gap_s")?;
        }
        for (off, n, read_s, gap) in timing {
            writeln!(fh, "{},{off},{n},{read_s:.6},{gap:.6}", geometry.resolution)?;
        }
        Ok(())
    });
    match r {
        Ok(()) => info!("GL128 image read timing appended to {}", out.display()),
        Err(e) => warn!("Could not write {ACQUIRE_TIMING_ENV}={path}: {e}"),
    }
}

/// Keep the `RegMap` type referenced for docs.
#[allow(dead_code)]
fn _regmap(_: &RegMap) {}
