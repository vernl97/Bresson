// SPDX-License-Identifier: GPL-3.0-or-later
//! Scan state machine: configure → acquire → assemble (GL845 / GL843 / GL842).
//!
//! The SANE `CommandSetGl846` OpticFilm path. GL843 / GL842 differ only in
//! dividing LPERIOD by `tgtime` (and the MAXWD quirk in `sensor_lookup`).

use log::{debug, info, warn};
use ndarray::{Array2, Array3, Axis, s};

use super::calibrate::{CalibEntry, Calibrator};
use super::exposure_merge::merge_exposures_result;
use super::geometry::{Area, MM_PER_INCH, ScanGeometry, compute_geometry};
use super::me_debug::MeScanDebug;
use super::me_exposure::{LongExposureParams, MeExposureDecision, select_long_exposure};
use super::pipeline::{
    HOST_CALIB_BORDER_INSET, HOST_CALIB_HIGHLIGHT_CEILING, HOST_CALIB_HIGHLIGHT_PERCENTILE,
    HOST_CALIB_PEAK_PERCENTILE, HOST_CALIB_PEAK_TARGET, ImagePipeline, inset_slice,
};
use super::request::{NoProgress, Progress, ScanControl, ScanMode, ScanRequest, SubProgress};
use crate::asic::gl845::{Gl84xVariant, Gl845};
use crate::asic::motor::{
    calculate_zmod, create_fast_slope_table, create_scan_slope_table, slope_table_ahb,
    slope_table_to_bytes,
};
use crate::asic::registers::gl845 as r;
use crate::clock::{monotonic, sleep};
use crate::device::model::{FilmModel, RegMap, ScanMethod};
use crate::device::sensor_lookup::{exposure_lperiod_for, maxwd_register_value, sensor_regs_for};
use crate::error::{Error, Result};
use crate::image::ScanImage;
use crate::pass_align::{align_ir_to_rgb_result, align_pass_to_reference};

pub const FEED_TIMEOUT_S: f64 = 60.0;
/// Feed fails early when the step counter stops advancing this long (stall).
pub const FEED_STALL_S: f64 = 10.0;
/// Pre-feed only when the image feed is this much slower than a low-res feed.
pub const PREFEED_MIN_SPEEDUP: f64 = 1.5;
/// Tiny normalized window for the pre-feed scan.
pub const PREFEED_AREA: Area = [0.0, 0.0, 0.05, 0.01];
pub const DATA_TIMEOUT_S: f64 = 30.0;
pub const BULK_CHUNK_LINES: i64 = 8;
pub const HOME_SCAN_PIXELS: i64 = 50;
pub const HOME_SCAN_LINES: i64 = 3;

/// Dark / white columns for the host flat-field (white may be exposure-scaled).
#[derive(Debug, Clone)]
pub struct HostCalib {
    pub dark: Array2<f32>,
    pub white: Array2<f32>,
}

impl HostCalib {
    pub fn from_entry(e: &CalibEntry) -> Self {
        Self {
            dark: e.dark.mapv(|v| v as f32),
            white: e.white.mapv(|v| v as f32),
        }
    }
}

pub fn set16(cache: &mut RegMap, addr: u16, value: i64) {
    let v = value & 0xFFFF;
    cache.insert(addr, ((v >> 8) & 0xFF) as u16);
    cache.insert(addr + 1, (v & 0xFF) as u16);
}

pub fn set24(cache: &mut RegMap, addr: u16, value: i64) {
    let v = value & 0xFF_FFFF;
    cache.insert(addr, ((v >> 16) & 0xFF) as u16);
    cache.insert(addr + 1, ((v >> 8) & 0xFF) as u16);
    cache.insert(addr + 2, (v & 0xFF) as u16);
}

pub fn device_model(model: &FilmModel) -> String {
    format!("{} {}", model.vendor, model.model)
}

/// The green CCD plane as an HxW sidecar (`rgb[:, :, 1]`).
pub fn green_plane(rgb: &Array3<u16>) -> Array2<u16> {
    rgb.index_axis(Axis(2), 1).to_owned()
}

/// Owns one colour/IR transparency scan from configure → TIFF-ready buffer.
pub struct ScanSession {
    pub model: FilmModel,
    pub pipeline: ImagePipeline,
    pub lperiod_divide_tgtime: bool,
    lamp_requested: bool,
    feedl: i64,
    lperiod: Option<i64>,
    pub last_align_shift_ir: Option<(f64, f64)>,
    pub last_me_debug: Option<MeScanDebug>,
}

impl ScanSession {
    pub fn new(model: FilmModel, variant: Gl84xVariant) -> Self {
        Self {
            pipeline: ImagePipeline::new(model.clone()),
            model,
            lperiod_divide_tgtime: variant != Gl84xVariant::Gl845,
            lamp_requested: true,
            feedl: 0,
            lperiod: None,
            last_align_shift_ir: None,
            last_me_debug: None,
        }
    }

    fn asic_name(&self) -> &'static str {
        self.model.asic
    }

    pub fn run(
        &mut self,
        asic: &mut Gl845,
        mut calibrator: Option<&mut Calibrator>,
        req: &ScanRequest,
        ctl: &mut ScanControl,
    ) -> Result<ScanImage> {
        let model = self.model.clone();
        if req.infrared && !model.supports_infrared {
            return Err(Error::Value(format!(
                "{} does not support infrared",
                model.model
            )));
        }
        let multi_pass_ok = self.asic_name() == "GL845";
        if (req.multi_exposure || req.infrared) && !multi_pass_ok {
            return Err(Error::NotImplemented(format!(
                "Multi-exposure / combined IR scans are not implemented for {}",
                self.asic_name()
            )));
        }
        if req.multi_exposure && !model.supports_multi_exposure.unwrap_or(false) {
            return Err(Error::NotImplemented(format!(
                "{} has no validated multi-exposure envelope",
                model.model
            )));
        }
        if (req.infrared || req.multi_exposure) && req.mode != ScanMode::Color {
            return Err(Error::Value(format!(
                "Combined infrared / multi-exposure scans require mode='color', got mode={:?}",
                req.mode.as_str()
            )));
        }
        if !multi_pass_ok && (req.me_short_exposure.is_some() || req.me_long_exposure.is_some()) {
            return Err(Error::NotImplemented(format!(
                "ME exposure overrides are only implemented for GL128 and GL845; {} does not support them.",
                self.asic_name()
            )));
        }
        if req.n_passes > 1 {
            return Err(Error::NotImplemented(format!(
                "Multi-Pass (n_passes > 1) is only implemented for GL128; {} does not support it.",
                self.asic_name()
            )));
        }
        if req.single_pass_exposure.is_some() && !multi_pass_ok {
            return Err(Error::NotImplemented(format!(
                "Manual exposure overrides are only implemented for GL128 and GL845; {} does not support single_pass_exposure.",
                self.asic_name()
            )));
        }
        if req.mode == ScanMode::Gray {
            return Err(Error::Value(
                "Grayscale is experimental / not implemented yet".into(),
            ));
        }
        if req.mode == ScanMode::Infrared && !model.supports_infrared {
            return Err(Error::Value(format!(
                "{} does not support infrared",
                model.model
            )));
        }
        if !asic.initialized {
            asic.init(false)?;
        }
        let method = req.mode.method();
        let mut geometry = match &req.geometry {
            Some(g) => g.clone(),
            None => {
                let mut g = compute_geometry(req.resolution, &model, req.area)?;
                // Re-bind exposure for IR when the model has method-keyed LPERIOD.
                let lp = exposure_lperiod_for(&model, g.resolution, method);
                if lp != g.exposure_lperiod {
                    g.exposure_lperiod = lp;
                }
                g
            }
        };
        info!(
            "scan {}dpi {} pixels={} lines={} starty={} bytes={} stagger={} combined_ir={}",
            geometry.resolution,
            req.mode.as_str(),
            geometry.pixels,
            geometry.lines,
            geometry.starty,
            geometry.total_bytes(),
            geometry.num_staggered_lines,
            req.infrared
        );
        let mut auto_exposure = false;
        if method == ScanMethod::Transparency && multi_pass_ok && !req.multi_exposure {
            match req.single_pass_exposure {
                Some(e) => geometry.exposure_lperiod = self.clamp_manual_exposure(e),
                None => auto_exposure = model.auto_exposure_target.is_some_and(|t| t != 0.0),
            }
        }
        if !req.infrared && !req.multi_exposure {
            let r = self.acquire_pass(
                asic,
                calibrator.as_deref_mut(),
                &geometry,
                method,
                req.apply_calib,
                ctl,
                true,
                None,
                auto_exposure,
            );
            let rgb = match r {
                Ok(v) => v,
                Err(e) => {
                    if multi_pass_ok {
                        self.park_quietly(asic);
                    }
                    return Err(e);
                }
            };
            let ir = (req.mode == ScanMode::Infrared).then(|| green_plane(&rgb));
            return Ok(ScanImage::new(
                rgb,
                geometry.resolution,
                device_model(&model),
                ir,
            ));
        }
        self.run_multi_pass(asic, calibrator, &geometry, req, ctl, auto_exposure)
    }

    /// Keep a manual LPERIOD inside the motor envelope validated for this model.
    fn clamp_manual_exposure(&self, exposure: i64) -> i64 {
        let base = self.model.exposure_lperiod;
        let hi = self.model.me_long_clamp_max.unwrap_or(base).min(0xFFFF);
        let clamped = exposure.min(hi).max(base);
        if clamped != exposure {
            warn!("single_pass_exposure {exposure} clamped to {clamped} (validated {base}..{hi})");
        }
        clamped
    }

    /// GL845 sequential passes: colour short → (IR) → (colour long), parking between.
    fn run_multi_pass(
        &mut self,
        asic: &mut Gl845,
        mut calibrator: Option<&mut Calibrator>,
        geometry: &ScanGeometry,
        req: &ScanRequest,
        ctl: &mut ScanControl,
        auto_exposure: bool,
    ) -> Result<ScanImage> {
        let model = self.model.clone();
        self.last_align_shift_ir = None;
        self.last_me_debug = None;
        let exp_short = req
            .me_short_exposure
            .filter(|&v| v != 0)
            .unwrap_or(geometry.exposure_lperiod);
        let short_geo = ScanGeometry {
            exposure_lperiod: exp_short,
            ..geometry.clone()
        };
        let n_pass = 1 + req.infrared as i64 + req.multi_exposure as i64;
        let mut pass_idx = 0i64;
        let mut rgb_long = None;
        let mut exp_long = None;
        let mut decision: Option<MeExposureDecision> = None;
        let mut ir_plane = None;

        let mut body =
            |this: &mut Self, asic: &mut Gl845, ctl: &mut ScanControl| -> Result<Array3<u16>> {
                let mut pass = |this: &mut Self,
                                asic: &mut Gl845,
                                ctl: &mut ScanControl,
                                geo: &ScanGeometry,
                                method: ScanMethod|
                 -> Result<Array3<u16>> {
                    let idx = pass_idx;
                    pass_idx += 1;
                    if idx > 0 {
                        ctl.check("Scan cancelled between passes")?;
                        asic.park(crate::asic::gl845::DEFAULT_HOME_TIMEOUT_S)?;
                        ctl.check("Scan cancelled between passes")?;
                    }
                    let mut sub = SubProgress {
                        ctl,
                        idx: idx as f64,
                        n: n_pass as f64,
                    };
                    this.acquire_pass(
                        asic,
                        calibrator.as_deref_mut(),
                        geo,
                        method,
                        req.apply_calib,
                        &mut sub,
                        // ME colour planes stay linear for the merge; IR keeps its makeup.
                        !(req.multi_exposure && method == ScanMethod::Transparency),
                        // Colour calib is measured at the model LPERIOD, never the long one.
                        Some(if method == ScanMethod::Transparency {
                            geometry
                        } else {
                            geo
                        }),
                        auto_exposure && method == ScanMethod::Transparency,
                    )
                };
                let rgb_short = pass(this, asic, ctl, &short_geo, ScanMethod::Transparency)?;
                if req.infrared {
                    let ir_lp =
                        exposure_lperiod_for(&model, geometry.resolution, ScanMethod::Infrared);
                    let ir_rgb = pass(
                        this,
                        asic,
                        ctl,
                        &ScanGeometry {
                            exposure_lperiod: ir_lp,
                            ..geometry.clone()
                        },
                        ScanMethod::Infrared,
                    )?;
                    ir_plane = Some(green_plane(&ir_rgb));
                }
                if req.multi_exposure {
                    let (el, d) = this.select_me_long(
                        &rgb_short,
                        exp_short,
                        geometry.resolution,
                        req.me_long_exposure,
                    );
                    exp_long = Some(el);
                    decision = d;
                    rgb_long = Some(pass(
                        this,
                        asic,
                        ctl,
                        &ScanGeometry {
                            exposure_lperiod: el,
                            ..geometry.clone()
                        },
                        ScanMethod::Transparency,
                    )?);
                }
                Ok(rgb_short)
            };
        let rgb_short = match body(self, asic, ctl) {
            Ok(v) => v,
            Err(e) => {
                self.park_quietly(asic);
                return Err(e);
            }
        };
        asic.park(crate::asic::gl845::DEFAULT_HOME_TIMEOUT_S)?;

        if let Some(ir) = ir_plane.take() {
            if req.align_passes {
                let (aligned, shift) = align_ir_to_rgb_result(&rgb_short, &ir)?;
                self.last_align_shift_ir = Some(shift);
                info!("IR pass shift (dx, dy)=({:.3}, {:.3})", shift.0, shift.1);
                ir_plane = Some(aligned);
            } else {
                ir_plane = Some(ir);
            }
        }
        let mut primary = rgb_short.clone();
        if let (Some(rgb_long), Some(exp_long)) = (rgb_long, exp_long) {
            let mut align_shift_long = None;
            if req.align_passes {
                let (_, s) = align_pass_to_reference(&rgb_short, &rgb_long, None)?;
                info!("ME long pass shift (dx, dy)=({:.3}, {:.3})", s.0, s.1);
                align_shift_long = Some(s);
            }
            // No model pins a fixed noise model: the merge fits it from the pair.
            let merged = merge_exposures_result(
                &rgb_short,
                &rgb_long,
                exp_short,
                exp_long,
                Some(align_shift_long.unwrap_or((0.0, 0.0))),
                None,
                None,
            )?;
            self.last_me_debug = Some(MeScanDebug {
                rgb_short: rgb_short.clone(),
                rgb_long: rgb_long.clone(),
                exposure_short: exp_short,
                exposure_long: exp_long,
                fusion_stats: merged.fusion_stats.clone(),
                align_shift_long,
                align_shift_ir: self.last_align_shift_ir,
                exposure_proposed: decision.as_ref().map(|d| d.proposed),
                exposure_reason: Some(
                    decision
                        .as_ref()
                        .map_or("manual-override".to_string(), |d| d.reason.clone()),
                ),
            });
            primary = self.pipeline.expose_film_base(
                merged.rgb,
                "me deliverable",
                HOST_CALIB_PEAK_TARGET,
                HOST_CALIB_PEAK_PERCENTILE,
                true,
                HOST_CALIB_HIGHLIGHT_CEILING,
                HOST_CALIB_HIGHLIGHT_PERCENTILE,
            );
            primary = self.pipeline.clamp_border_highlights(
                primary,
                HOST_CALIB_BORDER_INSET,
                HOST_CALIB_PEAK_PERCENTILE,
            );
        }
        let image = ScanImage::new(primary, geometry.resolution, device_model(&model), ir_plane);
        if image.ir_tracks_image() == Some(true) {
            warn!(
                "IR plane follows the picture (silver black-and-white film?); it is not a dust map — skip IR dust removal for this frame"
            );
        }
        Ok(image)
    }

    /// Pick the ME long LPERIOD, clamped to the model's validated envelope.
    fn select_me_long(
        &self,
        rgb_short: &Array3<u16>,
        exp_short: i64,
        resolution: i64,
        me_long_exposure: Option<i64>,
    ) -> (i64, Option<MeExposureDecision>) {
        let m = &self.model;
        let lo = m.me_long_clamp_min.unwrap_or(exp_short);
        let hi = m.me_long_clamp_max.unwrap_or(0xFFFF).min(0xFFFF);
        if let Some(v) = me_long_exposure {
            let el = v.min(hi).max(lo);
            if el != v {
                warn!("ME long exposure override {v} clamped to {el} (validated {lo}..{hi})");
            }
            return (el, None);
        }
        let default_long = m.exposure_long.unwrap_or(exp_short * 3).min(hi).max(lo);
        let (h, w, _) = rgb_short.dim();
        let view = match inset_slice(h, w, HOST_CALIB_BORDER_INSET) {
            Some(((y0, y1), (x0, x1))) => rgb_short.slice(s![y0..y1, x0..x1, ..]),
            None => rgb_short.view(),
        };
        let g = m.gl128.as_deref();
        let decision = select_long_exposure(
            view,
            exp_short,
            &LongExposureParams {
                black_level: g.map_or(0.0, |g| g.me_black_level),
                dense_percentile: g.map_or(5.0, |g| g.me_dense_percentile),
                target_dense_dn: g.map_or(10000.0, |g| g.me_target_dense_dn),
                adaptive_min: lo,
                adaptive_max: hi,
                hardware_max: hi,
                max_ratio: m.me_max_exposure_ratio.unwrap_or(3.0),
                default_long,
            },
        );
        let el = decision.selected.min(hi).max(lo);
        info!(
            "ME long exposure {resolution}dpi: short={exp_short} proposed={} selected={el} reason={}",
            decision.proposed, decision.reason
        );
        (el, Some(decision))
    }

    /// Return the head home after a failed pass without masking the original error.
    fn park_quietly(&self, asic: &mut Gl845) {
        if let Err(e) = asic.park(crate::asic::gl845::DEFAULT_HOME_TIMEOUT_S) {
            warn!("park after failed multi-pass scan: {e}");
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn acquire_pass(
        &mut self,
        asic: &mut Gl845,
        calibrator: Option<&mut Calibrator>,
        geometry: &ScanGeometry,
        method: ScanMethod,
        apply_calib: bool,
        progress: &mut dyn Progress,
        expose_base: bool,
        calib_geometry: Option<&ScanGeometry>,
        auto_exposure: bool,
    ) -> Result<Array3<u16>> {
        let calib_geo = calib_geometry.unwrap_or(geometry);
        let mut geometry = geometry.clone();
        let mut calib: Option<HostCalib> = None;
        let mut use_host_flag = false;
        if apply_calib && let Some(cal) = calibrator {
            // Genesys: host dark/white stretch.
            let mode = if method == ScanMethod::Infrared {
                ScanMode::Infrared
            } else {
                ScanMode::Color
            };
            let entry = cal.ensure_host_calib(asic, calib_geo, method, mode)?;
            let mut hc = HostCalib::from_entry(&entry);
            if method == ScanMethod::Transparency {
                if auto_exposure {
                    geometry.exposure_lperiod = self.auto_exposure(asic, &geometry, &entry)?;
                }
                if expose_base {
                    hc = self.calib_for_exposure(hc, &geometry, method);
                }
            }
            use_host_flag = cal.should_apply_host_calib(false);
            calib = Some(hc);
        }
        if self.asic_name() == "GL845" {
            geometry = self.prefeed(asic, &geometry, method, progress)?;
        }
        let raw = self.acquire_raw(asic, &geometry, method, true, true, progress)?;
        let use_host = calib.is_some() && use_host_flag;
        if calib.is_some() && !use_host {
            info!("Using ASIC shading; skipping host dark/white stretch");
        }
        let (dark, white) = match (&calib, use_host) {
            (Some(c), true) => (Some(&c.dark), Some(&c.white)),
            _ => (None, None),
        };
        let planar = self.model.gl128.as_ref().is_some_and(|g| g.usb_planar_rgb);
        self.pipeline.assemble(
            &raw,
            &geometry,
            dark,
            white,
            Some(planar),
            expose_base,
            None,
        )
    }

    /// Cover most of a slow feed with a tiny low-resolution scan first.
    fn prefeed(
        &mut self,
        asic: &mut Gl845,
        geometry: &ScanGeometry,
        method: ScanMethod,
        progress: &mut dyn Progress,
    ) -> Result<ScanGeometry> {
        let model = self.model.clone();
        let margin = model.prefeed_margin_steps.unwrap_or(0);
        if margin <= 0 {
            return Ok(geometry.clone());
        }
        let pre_res = *model.resolutions_dpi.iter().min().unwrap();
        let fast =
            (pre_res * exposure_lperiod_for(&model, pre_res, ScanMethod::Transparency)) as f64;
        let slow = (geometry.resolution * geometry.exposure_lperiod) as f64;
        let pre_steps = geometry.starty - margin;
        if slow <= PREFEED_MIN_SPEEDUP * fast || pre_steps <= margin {
            return Ok(geometry.clone());
        }
        let mut pre = compute_geometry(pre_res, &model, Some(PREFEED_AREA))?;
        pre.starty = pre_steps;
        let t0 = monotonic();
        struct CancelOnly<'a>(&'a dyn Progress);
        impl Progress for CancelOnly<'_> {
            fn cancelled(&self) -> bool {
                self.0.cancelled()
            }
            fn report(&mut self, _p: f64) {}
        }
        let _ = progress.cancelled();
        self.acquire_raw(asic, &pre, method, true, true, &mut CancelOnly(&*progress))?;
        // Line capture advances the motor by one line per scanned line.
        let moved = pre_steps
            + crate::num::floor_div(pre.lincnt_register() * model.motor_base_ydpi, pre_res)
            + model.prefeed_offset_steps.unwrap_or(0);
        info!(
            "prefeed {moved} steps at {pre_res} dpi in {:.1}s; image feed {} → {}",
            monotonic() - t0,
            geometry.starty,
            geometry.starty - moved
        );
        Ok(ScanGeometry {
            starty: geometry.starty - moved,
            ..geometry.clone()
        })
    }

    /// Scale white when the image LPERIOD differs from the calib LPERIOD.
    fn calib_for_exposure(
        &self,
        calib: HostCalib,
        geometry: &ScanGeometry,
        method: ScanMethod,
    ) -> HostCalib {
        let base = exposure_lperiod_for(&self.model, geometry.resolution, method);
        let ratio = geometry.exposure_lperiod as f64 / base as f64;
        if (ratio - 1.0).abs() < 1e-6 {
            return calib;
        }
        let r32 = ratio as f32;
        let white = Array2::from_shape_fn(calib.dark.dim(), |p| {
            calib.dark[p] + (calib.white[p] - calib.dark[p]) * r32
        });
        HostCalib {
            dark: calib.dark,
            white,
        }
    }

    /// Pick a single-pass LPERIOD that puts this frame's film peak near the target.
    fn auto_exposure(
        &mut self,
        asic: &mut Gl845,
        geometry: &ScanGeometry,
        calib: &CalibEntry,
    ) -> Result<i64> {
        let model = self.model.clone();
        let base = exposure_lperiod_for(&model, geometry.resolution, ScanMethod::Transparency);
        let target = model.auto_exposure_target.unwrap_or(0.0);
        let hi = model.me_long_clamp_max.unwrap_or(base).min(0xFFFF);
        if target <= 0.0 || hi <= base {
            return Ok(base);
        }
        let pre = compute_geometry(
            *model.resolutions_dpi.iter().min().unwrap(),
            &model,
            geometry.area,
        )?;
        let raw = self.acquire_raw(
            asic,
            &pre,
            ScanMethod::Transparency,
            true,
            true,
            &mut NoProgress,
        )?;
        asic.park(crate::asic::gl845::DEFAULT_HOME_TIMEOUT_S)?;
        let rgb = self
            .pipeline
            .assemble(&raw, &pre, None, None, None, false, None)?;
        let dark: Vec<f64> = (0..3)
            .map(|c| crate::num::mean(calib.dark.column(c).iter().map(|&v| v as f64)))
            .collect();
        let p = super::pipeline::channel_percentiles(rgb.view(), 99.5);
        let peak = (0..3).map(|c| p[c] - dark[c]).fold(f64::MIN, f64::max);
        let dark_max = dark.iter().cloned().fold(f64::MIN, f64::max);
        let headroom = (target - dark_max) / peak.max(1.0);
        let exposure = (hi as f64).min((base as f64).max(base as f64 * headroom)) as i64;
        info!("auto exposure: prescan peak={peak:.0} base={base} → {exposure} (cap {hi})");
        Ok(exposure)
    }

    /// Configure ASIC, run one capture, return raw optical bytes.
    pub fn acquire_raw(
        &mut self,
        asic: &mut Gl845,
        geometry: &ScanGeometry,
        method: ScanMethod,
        lamp_on: bool,
        start_motor: bool,
        progress: &mut dyn Progress,
    ) -> Result<Vec<u8>> {
        asic.set_scan_method(method);
        self.lamp_requested = lamp_on;
        if lamp_on {
            let was_lit = lamp_lit(asic, method);
            asic.lamp_on()?;
            let settle_s = self.model.lamp_settle_s.unwrap_or(0.0);
            if !was_lit && settle_s > 0.0 {
                info!(
                    "lamp settle {settle_s:.2}s before {} capture",
                    method.as_str()
                );
                sleep(settle_s);
            }
        } else {
            asic.lamp_off()?;
        }
        let r = (|| {
            self.configure(asic, geometry, false)?;
            self.begin_scan(asic, start_motor)?;
            let wait_feed = start_motor && self.feedl > 0;
            self.acquire(asic, geometry, progress, wait_feed)
        })();
        match r {
            Ok(raw) => {
                self.end_scan(asic)?;
                Ok(raw)
            }
            Err(e) => {
                if let Err(e2) = self.end_scan(asic) {
                    warn!("end_scan after failed capture: {e2}");
                }
                Err(e)
            }
        }
    }

    // --- configure ---

    /// Program the ASIC registers and slope tables for `geometry`.
    pub fn configure(
        &mut self,
        asic: &mut Gl845,
        geometry: &ScanGeometry,
        home_seek: bool,
    ) -> Result<()> {
        let model = self.model.clone();
        let mut cache = if !asic.reg_cache.is_empty() {
            asic.reg_cache.clone()
        } else {
            model.boot_register_map()
        };
        let method = asic.scan_method;
        for (&a, &v) in sensor_regs_for(&model, geometry.resolution).iter() {
            cache.insert(a, v);
        }
        asic.apply_frontend_for_scan(Some(geometry.resolution), Some(method))?;
        let g = |c: &RegMap, a: u16, d: u16| c.get_or(a, d) as i64;
        // Optical: SHDAREA on, DVDSET off (host-side calib), SCAN off
        cache.insert(
            0x01,
            ((g(&cache, 0x01, 0x22) | r::SHDAREA as i64) & !(r::DVDSET as i64) & !(r::SCAN as i64))
                as u16,
        );
        if home_seek {
            // keep the current LAMPPWR state
        } else if method == ScanMethod::Infrared || !self.lamp_requested {
            cache.insert(0x03, (g(&cache, 0x03, 0xBF) & !(r::LAMPPWR as i64)) as u16);
        } else {
            cache.insert(0x03, (g(&cache, 0x03, 0xBF) | r::LAMPPWR as i64) as u16);
        }
        let c03 = cache.get(&0x03).copied();
        if let Some(v) = c03 {
            cache.insert(0x03, v & !0x40); // clear AVEENB
        } else {
            // Python indexes cache[0x03], which the branch above always set except on home seeks.
            let v = g(&cache, 0x03, 0) as u16;
            cache.insert(0x03, v & !0x40);
        }
        if home_seek {
            // SANE's reverse-home mini-scan is 8-bit green-only.
            cache.insert(0x04, ((g(&cache, 0x04, 0x22) & !0xCC) | 0x28) as u16);
        } else {
            // 0x04: 16-bit color — BITSET, AFEMOD=2, FESET=2 (ADI)
            cache.insert(0x04, ((g(&cache, 0x04, 0x22) & !0x8C) | 0x40 | 0x20) as u16);
        }
        let dpihw_bits = match model.register_dpihw {
            600 => 0x00,
            1200 => 0x40,
            2400 => 0x80,
            4800 => 0xC0,
            _ => 0x40,
        };
        cache.insert(
            0x05,
            ((g(&cache, 0x05, 0x48) & !0xC0 & !0x08) | dpihw_bits) as u16,
        );
        cache.insert(0x2E, 0x7F);
        cache.insert(0x2F, 0x7F);

        let mut lperiod = geometry.exposure_lperiod;
        if lperiod <= 0 {
            lperiod = exposure_lperiod_for(&model, geometry.resolution, method);
        }
        let tgtime = 1i64 << (g(&cache, 0x1C, 0) & 0x07);
        if self.lperiod_divide_tgtime && tgtime > 1 {
            lperiod = (lperiod / tgtime).max(1);
        }
        set16(&mut cache, 0x2C, geometry.register_dpiset);
        set16(&mut cache, 0x30, geometry.pixel_startx);
        set16(&mut cache, 0x32, geometry.pixel_endx);
        set16(&mut cache, 0x38, lperiod);
        self.lperiod = Some(geometry.exposure_lperiod);
        cache.insert(0x34, geometry.dummy_pixel as u16);
        let maxwd = maxwd_register_value(&model, geometry.line_bytes, geometry.channels);
        set24(&mut cache, 0x35, maxwd);
        set24(&mut cache, 0x25, geometry.lincnt_register());

        let step_mult = 1;
        let mp = model.motor_profile;
        let scan_slope = create_scan_slope_table(
            geometry.resolution,
            geometry.exposure_lperiod,
            model.motor_base_ydpi,
            step_mult,
            Some(&mp),
        );
        let fast_slope = create_fast_slope_table(step_mult, Some(&mp));
        let mut c02 = r::MTRPWR;
        if home_seek {
            c02 |= r::AGOHOME | r::NOTHOME | r::MTRREV;
        }
        if geometry.disable_buffer_full_move {
            c02 |= r::ACDCDIS;
        }
        cache.insert(0x02, c02 as u16);
        let n_scan = scan_slope.table.len() as i64 / step_mult;
        let n_fast = fast_slope.table.len() as i64 / step_mult;
        for a in [0x21, 0x24, 0x69] {
            cache.insert(a, (n_scan & 0xFF) as u16);
        }
        cache.insert(0x6A, (n_fast & 0xFF) as u16);
        cache.insert(0x5F, (n_fast & 0xFF) as u16);
        let feed_steps = geometry.starty;
        let mut feedl = feed_steps << mp.step_type;
        let dist = scan_slope.table.len() as i64;
        feedl = if dist < feedl { feedl - dist } else { 0 };
        set24(&mut cache, 0x3D, feedl);
        let min_restep = 1.max(scan_slope.table.len() as i64 / step_mult / 2 - 1);
        cache.insert(0x22, (min_restep & 0xFF) as u16);
        cache.insert(0x23, (min_restep & 0xFF) as u16);
        let ccdlmt = (g(&cache, 0x0C, 0) & 0x0F) + 1;
        let exposure_mod = geometry.exposure_lperiod * ccdlmt * tgtime;
        let (z1, z2) = calculate_zmod(
            exposure_mod,
            &scan_slope.table,
            scan_slope.table.len() as i64,
            feedl,
            min_restep * step_mult,
        );
        let step_sel = mp.step_type << 5;
        set24(&mut cache, 0x60, (z1 & 0xFFFF) | (step_sel << 16));
        set24(&mut cache, 0x63, (z2 & 0xFFFF) | (step_sel << 16));
        cache.insert(0x1E, (g(&cache, 0x1E, 0xF0) & 0xF0) as u16);
        cache.insert(0x67, 0x7F);
        cache.insert(0x68, 0x7F);
        let vref = mp.vref | (mp.vref << 2) | (mp.vref << 4) | (mp.vref << 6);
        cache.insert(0x80, (vref & 0xFF) as u16);

        for (a, v) in cache.sorted_bytes() {
            if a != 0x0B {
                asic.protocol.write_register(a, v)?;
            }
        }
        asic.reg_cache = cache;
        let payload = slope_table_to_bytes(&scan_slope.table);
        let fast_payload = slope_table_to_bytes(&fast_slope.table);
        for nr in [0, 1, 2] {
            asic.protocol
                .write_ahb(slope_table_ahb(nr).unwrap(), &payload)?;
        }
        for nr in [3, 4] {
            asic.protocol
                .write_ahb(slope_table_ahb(nr).unwrap(), &fast_payload)?;
        }
        self.feedl = feedl;
        info!(
            "configured asic={} dpiset={} str={} end={} lperiod={lperiod} maxwd={maxwd} lincnt={} feedl={feedl} slopes={}",
            model.asic,
            geometry.register_dpiset,
            geometry.pixel_startx,
            geometry.pixel_endx,
            geometry.lincnt_register(),
            scan_slope.table.len()
        );
        Ok(())
    }

    // --- acquire ---

    fn begin_scan(&mut self, asic: &mut Gl845, start_motor: bool) -> Result<()> {
        asic.protocol.write_register(0x0D, 0x05)?; // CLRLNCNT | CLRMCNT
        let reg01 = asic.protocol.read_register(0x01)? | r::SCAN;
        asic.protocol.write_register(0x01, reg01)?;
        asic.reg_cache.insert(0x01, reg01 as u16);
        asic.protocol
            .write_register(0x0F, if start_motor { 0x01 } else { 0x00 })?;
        asic.update_home_sensor_gpio()?;
        info!(
            "scan started motor={}",
            if start_motor { "True" } else { "False" }
        );
        Ok(())
    }

    fn end_scan(&mut self, asic: &mut Gl845) -> Result<()> {
        asic.stop_motor(crate::asic::gl845::MOTOR_STOP_TIMEOUT_S)
    }

    fn read_feed_steps(asic: &mut Gl845) -> Result<i64> {
        let mut steps = asic.protocol.read_register(0x4A)? as i64;
        steps += asic.protocol.read_register(0x49)? as i64 * 256;
        steps += (asic.protocol.read_register(0x48)? & 0x1F) as i64 * 256 * 256;
        Ok(steps)
    }

    fn read_valid_words(asic: &mut Gl845) -> Result<i64> {
        let mut w = (asic.protocol.read_register(0x42)? & 0x02) as i64;
        w = w * 256 + asic.protocol.read_register(0x43)? as i64;
        w = w * 256 + asic.protocol.read_register(0x44)? as i64;
        w = w * 256 + asic.protocol.read_register(0x45)? as i64;
        Ok(w)
    }

    fn wait_feed(&mut self, asic: &mut Gl845, progress: &dyn Progress) -> Result<()> {
        let base = if self.model.exposure_lperiod != 0 {
            self.model.exposure_lperiod
        } else {
            14000
        };
        let lp = self.lperiod.unwrap_or(base);
        let timeout_s = FEED_TIMEOUT_S * (lp as f64 / base as f64).max(1.0);
        let start = monotonic();
        let deadline = start + timeout_s;
        let target = self.feedl;
        let (mut last_steps, mut last_move) = (-1i64, start);
        loop {
            let now = monotonic();
            if now >= deadline {
                break;
            }
            progress.check("cancelled during feed")?;
            let steps = Self::read_feed_steps(asic)?;
            if steps >= target {
                debug!("feed complete ({target}) in {:.1}s", now - start);
                return Ok(());
            }
            if steps != last_steps {
                last_steps = steps;
                last_move = now;
            } else if now - last_move > FEED_STALL_S {
                return Err(Error::MotorTimeout(format!(
                    "Feed stalled at {steps}/{target} steps for {FEED_STALL_S:.0}s"
                )));
            }
            sleep(0.05);
        }
        Err(Error::MotorTimeout(format!(
            "Feed did not complete within {timeout_s:.0}s ({last_steps}/{target} steps)"
        )))
    }

    fn wait_data(&mut self, asic: &mut Gl845, progress: &dyn Progress) -> Result<()> {
        let deadline = monotonic() + DATA_TIMEOUT_S;
        while monotonic() < deadline {
            progress.check("cancelled waiting for data")?;
            let st = asic.read_status()?;
            if !st.is_buffer_empty && Self::read_valid_words(asic)? >= 1 {
                return Ok(());
            }
            sleep(0.02);
        }
        Err(Error::Scan(format!(
            "No scan data within {DATA_TIMEOUT_S:.0}s"
        )))
    }

    fn acquire(
        &mut self,
        asic: &mut Gl845,
        geometry: &ScanGeometry,
        progress: &mut dyn Progress,
        wait_feed: bool,
    ) -> Result<Vec<u8>> {
        if wait_feed {
            self.wait_feed(asic, progress)?;
        }
        self.wait_data(asic, progress)?;
        let total = geometry.total_bytes() as usize;
        let chunk = (geometry.line_bytes * BULK_CHUNK_LINES) as usize;
        let mut buf = Vec::with_capacity(total);
        while buf.len() < total {
            progress.check("cancelled during bulk read")?;
            let need = chunk.min(total - buf.len());
            let data = asic.protocol.bulk_read_data(need)?;
            if data.is_empty() {
                return Err(Error::Scan("Empty bulk read during scan".into()));
            }
            buf.extend_from_slice(&data);
            progress.report((buf.len() as f64 / total as f64).min(1.0));
            debug!("acquired {} / {total}", buf.len());
        }
        progress.report(1.0);
        buf.truncate(total);
        Ok(buf)
    }
}

/// True when the light source for `method` is already on (cached regs).
fn lamp_lit(asic: &Gl845, method: ScanMethod) -> bool {
    let c = &asic.reg_cache;
    match method {
        ScanMethod::Infrared => c.get_or(0xA8, 0) as u8 & r::IR_LAMP_A8_MASK != 0,
        ScanMethod::Transparency => c.get_or(0x03, 0) as u8 & r::LAMPPWR != 0,
    }
}

/// Start the short reverse scan used by the GL845 home-sensor seek.
pub fn start_home_seek(asic: &mut Gl845, starty: i64) -> Result<()> {
    if !asic.initialized {
        return Err(Error::Asic("Call init()/warmup() before home()".into()));
    }
    let mut session = ScanSession::new(asic.model.clone(), asic.variant);
    let model = &session.model;
    let resolution = *model.resolutions_dpi.iter().min().unwrap();
    let width_mm = HOME_SCAN_PIXELS as f64 * MM_PER_INCH / resolution as f64 + 0.01;
    let height_mm = HOME_SCAN_LINES as f64 * MM_PER_INCH / resolution as f64 + 0.01;
    let area = [
        0.0,
        0.0,
        (width_mm / model.x_size_ta_mm).min(1.0),
        (height_mm / model.y_size_ta_mm).min(1.0),
    ];
    let mut geometry = compute_geometry(resolution, model, Some(area))?;
    if geometry.pixels < HOME_SCAN_PIXELS || geometry.lines < HOME_SCAN_LINES {
        return Err(Error::Scan(format!(
            "Home-scan geometry is too small: {}x{}",
            geometry.pixels, geometry.lines
        )));
    }
    geometry.startx = 0;
    geometry.starty = starty;
    geometry.channels = 1;
    geometry.depth = 8;
    geometry.line_bytes = geometry.pixels;
    geometry.optical_line_count = HOME_SCAN_LINES;
    geometry.shift_r = 0;
    geometry.shift_g = 0;
    geometry.shift_b = 0;
    geometry.stagger_y = Vec::new();
    geometry.num_staggered_lines = 0;
    geometry.register_lincnt = HOME_SCAN_LINES;
    // Leave the lamp alone: parking must not cool the LED.
    let prior = asic.scan_method;
    asic.set_scan_method(ScanMethod::Transparency);
    let r = session.configure(asic, &geometry, true);
    asic.set_scan_method(prior);
    r?;
    session.begin_scan(asic, true)?;
    info!(
        "started reverse home scan pixels={} lines={} starty={starty}",
        geometry.pixels, geometry.lines
    );
    Ok(())
}
