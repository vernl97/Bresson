// SPDX-License-Identifier: GPL-3.0-or-later
//! OpticFilm 7600i v1 scan session: replay the vendor job, then assemble square-pixel RGB16.

use std::collections::HashMap;

use log::warn;
use ndarray::{Array2, Array3, Axis, s};
use serde_json::Value;

use super::geometry::Area;
use super::replay_gl843_v1::{ReplayHooks, prepare_profile, run_profile, widen_window};
use super::request::{ScanControl, ScanMode, ScanRequest};
use crate::asic::Gl843V1;
use crate::clock::{monotonic, sleep};
use crate::device::model::{FilmModel, shifts_for_7600i_v1};
use crate::device::replay_data::replay_profile;
use crate::error::{Error, Result};
use crate::image::ScanImage;
use crate::num::py_round;
use crate::pass_align::align_ir_to_rgb;

/// Calibration frames vs the vendor reference: white level %, G/R and B/R balance %, white ripple %.
pub const LAMP_LIMIT_LEVEL: f64 = 10.0;
pub const LAMP_LIMIT_BALANCE: f64 = 5.0;
pub const LAMP_LIMIT_RIPPLE: f64 = 0.6;
pub const RETURN_HOME_TIMEOUT_S: f64 = 60.0;

fn vi(v: &Value) -> i64 {
    v.as_i64()
        .or_else(|| v.as_f64().map(|f| f as i64))
        .unwrap_or(0)
}

/// Per-channel mean of the central 80 % of columns, and the worst line-to-line variation (%).
pub fn frame_stats(data: &[u8], pixels: usize, lines: usize) -> ([f64; 3], f64) {
    let px = |y: usize, x: usize, c: usize| {
        let i = ((y * pixels + x) * 3 + c) * 2;
        u16::from_le_bytes([data[i], data[i + 1]]) as f64
    };
    let (x0, x1) = (pixels / 10, pixels - pixels / 10);
    let rows: Vec<[f64; 3]> = (0..lines)
        .map(|y| [0, 1, 2].map(|c| (x0..x1).map(|x| px(y, x, c)).sum::<f64>() / (x1 - x0) as f64))
        .collect();
    let mean = [0, 1, 2].map(|c| rows.iter().map(|r| r[c]).sum::<f64>() / lines as f64);
    if lines <= 1 {
        return (mean, 0.0);
    }
    let ripple = (0..3)
        .map(|c| {
            let var = rows.iter().map(|r| (r[c] - mean[c]).powi(2)).sum::<f64>() / lines as f64;
            var.sqrt() / mean[c].max(1e-9)
        })
        .fold(f64::MIN, f64::max)
        * 100.0;
    (mean, ripple)
}

pub fn lamp_issues(mean: [f64; 3], ripple: f64, reference: &Value) -> Vec<String> {
    let r: Vec<f64> = reference["mean"]
        .as_array()
        .map(|a| a.iter().map(|v| v.as_f64().unwrap_or(0.0)).collect())
        .unwrap_or_default();
    let level = (mean.iter().sum::<f64>() / r.iter().sum::<f64>() - 1.0) * 100.0;
    let gr = (mean[1] / mean[0]) / (r[1] / r[0]) * 100.0 - 100.0;
    let br = (mean[2] / mean[0]) / (r[2] / r[0]) * 100.0 - 100.0;
    let mut issues = Vec::new();
    if level.abs() > LAMP_LIMIT_LEVEL {
        issues.push(format!("level {level:+.1} %"));
    }
    if gr.abs().max(br.abs()) > LAMP_LIMIT_BALANCE {
        issues.push(format!("G/R {gr:+.1} %, B/R {br:+.1} %"));
    }
    if ripple > LAMP_LIMIT_RIPPLE {
        issues.push(format!("ripple {ripple:.2} %"));
    }
    issues
}

/// Interleaved RGB16 lines → aligned square pixels: fractional R/G/B delays,
/// column stagger, line pairs averaged, black offsets subtracted, mirrored.
#[allow(clippy::too_many_arguments)]
pub fn assemble(
    raw: &[u8],
    pixels: usize,
    lines: usize,
    dpi: i64,
    yres: i64,
    shifts: [f64; 3],
    stagger: &[i64],
    offsets: Option<[f64; 3]>,
    mirror: bool,
) -> Result<Array3<u16>> {
    let src = |y: usize, x: usize, c: usize| {
        let i = ((y * pixels + x) * 3 + c) * 2;
        u16::from_le_bytes([raw[i], raw[i + 1]]) as f64
    };
    let k = py_round(yres as f64 / dpi as f64).max(1) as usize;
    let max_shift = shifts.iter().cloned().fold(f64::MIN, f64::max);
    let max_stagger = stagger.iter().copied().max().unwrap_or(0);
    let height =
        (lines as i64 - (max_shift - 1e-6).ceil() as i64 - max_stagger).div_euclid(k as i64);
    if height <= 0 {
        return Err(Error::Scan("frame too short to align".into()));
    }
    let height = height as usize;
    let mut out = Array3::<u16>::zeros((height, pixels, 3));
    let parities: Vec<(usize, usize, i64)> = if stagger.len() >= 2 {
        vec![(0, 2, stagger[0]), (1, 2, stagger[1])]
    } else {
        vec![(0, 1, 0)]
    };
    for c in 0..3 {
        let i0 = (shifts[c] + 1e-6).floor() as i64;
        let fr = if shifts[c] - i0 as f64 >= 1e-3 {
            shifts[c] - i0 as f64
        } else {
            0.0
        };
        for &(start, step, st) in &parities {
            for y in 0..height {
                let row = (y * k) as i64 + i0 + st;
                for x in (start..pixels).step_by(step) {
                    let mut acc = 0.0f64;
                    for j in 0..k {
                        acc += (1.0 - fr) / k as f64 * src((row + j as i64) as usize, x, c);
                        if fr != 0.0 {
                            acc += fr / k as f64 * src((row + j as i64 + 1) as usize, x, c);
                        }
                    }
                    if let Some(o) = offsets {
                        acc -= o[c];
                    }
                    // Round half up.
                    out[[y, x, c]] = (acc + 0.5).floor().clamp(0.0, 65535.0) as u16;
                }
            }
        }
    }
    if mirror {
        out.invert_axis(Axis(1));
        out = out.as_standard_layout().to_owned();
    }
    Ok(out)
}

fn crop_bounds(
    h: usize,
    w: usize,
    area: Option<Area>,
) -> Result<Option<(usize, usize, usize, usize)>> {
    let Some([x1, y1, x2, y2]) = area else {
        return Ok(None);
    };
    if !(0.0 <= x1 && x1 < x2 && x2 <= 1.0 && 0.0 <= y1 && y1 < y2 && y2 <= 1.0) {
        return Err(Error::Value(
            "area must be (x1, y1, x2, y2) within 0..1".into(),
        ));
    }
    let ya = (y1 * h as f64) as usize;
    let xa = (x1 * w as f64) as usize;
    let yb = (ya + 1).max(py_round(y2 * h as f64) as usize);
    let xb = (xa + 1).max(py_round(x2 * w as f64) as usize);
    Ok(Some((ya, yb.min(h), xa, xb.min(w))))
}

pub fn crop(rgb: Array3<u16>, area: Option<Area>) -> Result<Array3<u16>> {
    let (h, w, _) = rgb.dim();
    Ok(match crop_bounds(h, w, area)? {
        None => rgb,
        Some((ya, yb, xa, xb)) => rgb.slice(s![ya..yb, xa..xb, ..]).to_owned(),
    })
}

fn crop2(p: Array2<u16>, area: Option<Area>) -> Result<Array2<u16>> {
    let (h, w) = p.dim();
    Ok(match crop_bounds(h, w, area)? {
        None => p,
        Some((ya, yb, xa, xb)) => p.slice(s![ya..yb, xa..xb]).to_owned(),
    })
}

/// Per-job diagnostics (`last_scan_info`).
#[derive(Debug, Clone, Default)]
pub struct JobInfo {
    pub issues: Vec<String>,
    pub dark_delta: Option<[f64; 3]>,
    pub calibration: String,
    pub positioning_stop_ms: Option<f64>,
}

/// Colour (1440 / 3600 / 7200 dpi), infrared and colour + infrared (3600 / 7200 dpi) scans.
pub struct Gl843V1ScanSession {
    pub model: FilmModel,
    pub last_scan_info: HashMap<String, JobInfo>,
}

impl Gl843V1ScanSession {
    pub fn new(model: FilmModel) -> Self {
        Self {
            model,
            last_scan_info: HashMap::new(),
        }
    }

    pub fn run(
        &mut self,
        asic: &mut Gl843V1,
        req: &ScanRequest,
        ctl: &mut ScanControl,
        dummy_lines: Option<&str>,
    ) -> Result<ScanImage> {
        let m = &self.model;
        if req.multi_exposure || req.n_passes > 1 || req.single_pass_exposure.is_some() {
            return Err(Error::NotImplemented(format!(
                "Multi-exposure, Multi-Pass and manual exposure: not for {}",
                m.model
            )));
        }
        if req.geometry.is_some() || req.mode == ScanMode::Gray {
            return Err(Error::Value(
                "Unsupported geometry or mode (use area= to crop; mode color|infrared)".into(),
            ));
        }
        let res = req.resolution;
        let sorted = |v: &[i64]| {
            let mut v = v.to_vec();
            v.sort_unstable();
            v.iter()
                .map(|d| d.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        };
        if !m.resolutions_dpi.contains(&res) {
            return Err(Error::Value(format!(
                "{} scans at {} dpi",
                m.model,
                sorted(&m.resolutions_dpi)
            )));
        }
        let ir_dpis = m.infrared_resolutions();
        if (req.mode == ScanMode::Infrared || req.infrared) && !ir_dpis.contains(&res) {
            return Err(Error::Value(format!(
                "{} scans infrared at {} dpi",
                m.model,
                sorted(&ir_dpis)
            )));
        }
        if !asic.initialized {
            asic.init(false)?;
        }
        let jobs: Vec<&str> = if req.mode == ScanMode::Infrared {
            vec!["infrared"]
        } else if req.infrared {
            vec!["color", "infrared"]
        } else {
            vec!["color"]
        };
        self.last_scan_info.clear();
        let mut planes: HashMap<&str, Array3<u16>> = HashMap::new();
        let nj = jobs.len() as f64;
        for (n, job) in jobs.iter().enumerate() {
            let plane = self.scan_job(
                asic,
                job,
                res,
                ctl,
                n as f64,
                nj,
                req.apply_calib,
                dummy_lines,
            )?;
            planes.insert(job, plane);
        }
        let device = format!("{} {}", self.model.vendor, self.model.model);
        // IR is read by the red row.
        let mut ir: Option<Array2<u16>> = planes
            .get("infrared")
            .map(|p| p.index_axis(Axis(2), 0).to_owned());
        let mut rgb = planes.remove(jobs[0]).unwrap();
        if req.mode == ScanMode::Color
            && let Some(irp) = ir.take()
        {
            let h = rgb.dim().0.min(irp.nrows());
            rgb = rgb.slice(s![..h, .., ..]).to_owned();
            let mut irp = irp.slice(s![..h, ..]).to_owned();
            if req.align_passes {
                irp = align_ir_to_rgb(&rgb, &irp)?;
            }
            ir = Some(irp);
        }
        let ir = match ir {
            Some(p) => Some(crop2(p, req.area)?),
            None => None,
        };
        Ok(ScanImage::new(crop(rgb, req.area)?, res, device, ir))
    }

    #[allow(clippy::too_many_arguments)]
    fn scan_job(
        &mut self,
        asic: &mut Gl843V1,
        job: &str,
        resolution: i64,
        ctl: &mut ScanControl,
        n: f64,
        nj: f64,
        apply_calib: bool,
        dummy_lines: Option<&str>,
    ) -> Result<Array3<u16>> {
        let model = self.model.clone();
        let mut profile = replay_profile(model.model, job, resolution)?;
        if let Some(ws) = model.window_start {
            profile = widen_window(&profile, ws)?;
        }
        profile = prepare_profile(
            &profile,
            dummy_lines.or(model.dummy_lines).unwrap_or("recorded"),
            1,
            model.single_sample.unwrap_or(false),
        )?;
        if !asic.position_known || !asic.is_at_home()? {
            asic.home(30.0)?;
        }
        asic.position_known = false;
        if asic.lamp_on_at.is_none() || asic.usb.shadow.get(&0x03).copied().unwrap_or(0) & 0x10 == 0
        {
            asic.white_led(true)?;
        }
        let left = model.lamp_warmup_s.unwrap_or(0.0)
            - (monotonic() - asic.lamp_on_at.unwrap_or(monotonic()));
        if left > 0.0 {
            sleep(left);
        }
        let main = vi(&profile["mainFrame"]);
        let lamp = profile["lamp"].clone();
        let roles: HashMap<i64, &str> = ["line", "shading", "dark"]
            .iter()
            .map(|k| (vi(&lamp[*k]["frame"]), *k))
            .collect();
        let mut info = JobInfo::default();
        let frames = profile["frames"].clone();
        let job_s = job.to_string();
        let mut frame_done = |i: i64, data: &[u8]| {
            let kind = roles[&i];
            let f = &frames[i as usize];
            let (mean, ripple) =
                frame_stats(data, vi(&f["pixels"]) as usize, vi(&f["lines"]) as usize);
            if kind == "dark" {
                let refm: Vec<f64> = lamp["dark"]["mean"]
                    .as_array()
                    .map(|a| a.iter().map(|v| v.as_f64().unwrap_or(0.0)).collect())
                    .unwrap_or_default();
                info.dark_delta =
                    Some([0, 1, 2].map(|c| crate::num::py_round_to(mean[c] - refm[c], 1)));
                return;
            }
            for issue in lamp_issues(mean, ripple, &lamp[kind]) {
                warn!("{job_s} calibration {kind} from the vendor reference: {issue}");
                info.issues.push(format!("{kind}: {issue}"));
            }
        };
        let cancel = ctl.cancel.clone();
        let check = move || -> Result<()> {
            if cancel
                .as_ref()
                .is_some_and(|c| c.load(std::sync::atomic::Ordering::SeqCst))
            {
                return Err(Error::ScanCancelled("cancelled".into()));
            }
            Ok(())
        };
        let mut progress = |i: i64, got: usize, total: usize| {
            if i == main {
                ctl.report((n + got as f64 / total as f64) / nj);
            }
        };
        let r = (|| -> Result<_> {
            let mut hooks = ReplayHooks::new(&check);
            hooks.frame_done = Some(&mut frame_done);
            hooks.progress = Some(&mut progress);
            let result = run_profile(&profile, &mut asic.usb, hooks, apply_calib)?;
            asic.forget_motor_tables();
            if job == "infrared" {
                asic.usb.write_register(0xA8, 0x20)?; // the job ends with the infrared LED on
            }
            let deadline = monotonic() + RETURN_HOME_TIMEOUT_S;
            loop {
                // The chip returns the carriage after the scan.
                check()?;
                if asic.is_at_home()? {
                    break;
                }
                if monotonic() > deadline {
                    return Err(Error::Scan("carriage did not return home".into()));
                }
                sleep(0.1);
            }
            Ok(result)
        })();
        let result = match r {
            Ok(v) => v,
            Err(e) => {
                asic.position_known = false;
                asic.forget_motor_tables();
                if let Err(e2) = asic.stop_motor() {
                    warn!("cleanup after failed scan: {e2}");
                }
                let _ = asic.usb.transport.abort_bulk_in();
                if let Err(e2) = asic.lamp_off() {
                    warn!("cleanup after failed scan: {e2}");
                }
                return Err(e);
            }
        };
        asic.position_known = true;
        asic.lamp_on_at = if asic.usb.shadow.get(&0x03).copied().unwrap_or(0) & 0x10 != 0 {
            Some(asic.lamp_on_at.unwrap_or_else(monotonic))
        } else {
            None
        };
        info.calibration = if apply_calib {
            "live".into()
        } else {
            "recorded".into()
        };
        info.positioning_stop_ms = result.positioning_stop_ms;
        self.last_scan_info.insert(job.to_string(), info);
        let yres = vi(&profile["yres"]);
        let frame = &profile["frames"][main as usize];
        let stagger: Vec<i64> = model
            .stagger_y_by_dpi
            .get(&resolution)
            .cloned()
            .unwrap_or_default()
            .iter()
            .map(|&v| py_round(v as f64 * yres as f64 / 7200.0))
            .collect();
        assemble(
            &result.main,
            vi(&frame["pixels"]) as usize,
            vi(&frame["lines"]) as usize,
            resolution,
            yres,
            shifts_for_7600i_v1(yres),
            &stagger,
            None,
            model.mirror_x(),
        )
    }
}
