//! Film scanner worker: one thread owns the open scanner (an OpticFilm
//! through the `opticfilm` driver, or a SANE device such as a flatbed with a
//! transparency unit), takes commands from the Scan panel and reports back
//! through the job channel. SANE isn't thread-safe; all its calls stay here.
//!
//! Scans land in the roll folder as `scan_NNN.dng` (LinearRaw, see
//! `engine::scan_dng`), the IR plane as `NLP/scan_NNN.ir.tif`. SANE devices
//! marked gamma-encoded save a 16-bit TIFF instead. Simulated scanners
//! (`NLP_FAKE_SCANNERS=1`, or debug builds) run the drivers' mocks: OpticFilm
//! GL128 models on the AFE simulator, on the driver's virtual clock, and a
//! simulated SANE flatbed.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};

use eframe::egui;
use opticfilm::scan::bringup;
use opticfilm::usb::fake_gl128_sim::Gl128Sim;
use opticfilm::usb::transport::UsbTransport;
use opticfilm::{Error as ScanError, FilmModel, ScanControl, ScanImage, ScanRequest, Scanner};

use crate::engine::decode::{self, SourceImage, SourceInfo};
use crate::engine::pipeline::{self, Renderer};
use crate::engine::scan_dng::{self, ScanDngInfo};
use crate::engine::settings::{FilmBase, OutputSpace, Settings, Source};
use crate::engine::{color, sidecar};

use super::jobs::JobResult;

/// Long edge of the rendered prescan.
const PRESCAN_PREVIEW: usize = 1024;
/// Lowest resolution a flatbed prescan uses.
const SANE_PRESCAN_DPI: u32 = 100;
/// Software tag of SANE scans.
const SANE_SOFTWARE: &str = "NLP Studio (SANE)";

/// A scanner to open.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ScannerChoice {
    /// An OpticFilm on USB, by `opticfilm` device id.
    Device(String),
    /// A simulated OpticFilm model, by name.
    Simulated(String),
    /// A SANE device, by SANE name.
    Sane(String),
    /// The simulated SANE flatbed.
    SimulatedFlatbed,
}

impl ScannerChoice {
    /// Key for per-scanner settings (`Defaults.scan`).
    pub fn key(&self) -> String {
        match self {
            Self::Device(id) => format!("opticfilm:{id}"),
            Self::Simulated(name) => format!("simulated:{name}"),
            Self::Sane(name) => format!("sane:{name}"),
            Self::SimulatedFlatbed => "simulated:flatbed".into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScannerKind {
    /// A dedicated film scanner: one frame per holder position.
    Film,
    /// A flatbed: many frames on the bed, one box each.
    Flatbed,
}

/// What the panel needs to know about the open scanner.
#[derive(Debug, Clone)]
pub struct ScannerInfo {
    pub choice: ScannerChoice,
    pub label: String,
    pub kind: ScannerKind,
    /// Validated for scanning (locked OpticFilm models open for status only).
    pub scan_ready: bool,
    pub resolutions: Vec<i64>,
    pub sources: Vec<String>,
    pub source: Option<String>,
    pub supports_ir: bool,
    pub supports_me: bool,
    pub supports_passes: bool,
    /// The initial crop on the prescan, normalised.
    pub default_crop: Option<[f32; 4]>,
}

/// A scanner the picker can offer.
#[derive(Debug, Clone)]
pub struct DeviceEntry {
    pub choice: ScannerChoice,
    pub label: String,
}

/// One press of Scan: every framed area, in order.
#[derive(Debug, Clone)]
pub struct ScanJob {
    pub dpi: i64,
    /// Areas normalised to the prescan.
    pub areas: Vec<[f32; 4]>,
    pub infrared: bool,
    pub multi_exposure: bool,
    pub passes: i64,
    pub source: Option<String>,
    /// SANE data with a tone curve: saved as TIFF, not linear DNG.
    pub gamma_encoded: bool,
    /// One output file per area.
    pub paths: Vec<PathBuf>,
}

pub enum ScanCmd {
    /// List OpticFilm and SANE scanners.
    Refresh,
    Open(ScannerChoice),
    /// Select a flatbed source (resolutions and area may change).
    SetSource(String),
    Prescan,
    Scan(Box<ScanJob>),
    Close,
}

pub enum ScanEvent {
    /// Scanners found, and why SANE isn't available if it isn't.
    Devices { devices: Vec<DeviceEntry>, sane_note: Option<String> },
    Opened(Box<ScannerInfo>),
    Status(String),
    /// Progress of the whole job, 0..1.
    Progress(f64),
    /// The prescan rendered as a positive.
    Prescan(egui::ColorImage),
    Saved { path: PathBuf, ir_path: Option<PathBuf> },
    /// The scan stopped on Cancel; frames already saved stay.
    Cancelled,
    Failed(String),
    Closed,
}

/// Handle to the worker thread; dropping it closes the scanner.
pub struct ScanWorker {
    tx: Sender<ScanCmd>,
    pub cancel: Arc<AtomicBool>,
}

impl ScanWorker {
    pub fn spawn(events: Sender<JobResult>, ctx: egui::Context, defaults: Settings) -> Self {
        let (tx, rx) = channel();
        let cancel = Arc::new(AtomicBool::new(false));
        let c = cancel.clone();
        std::thread::Builder::new()
            .name("scanner".into())
            .spawn(move || Worker { events, ctx, cancel: c, defaults, sane: None, open: None }.run(rx))
            .expect("spawn scanner thread");
        Self { tx, cancel }
    }

    pub fn send(&self, cmd: ScanCmd) {
        if matches!(cmd, ScanCmd::Prescan | ScanCmd::Scan(_)) {
            self.cancel.store(false, Ordering::SeqCst);
        }
        let _ = self.tx.send(cmd);
    }

    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::SeqCst);
    }
}

impl Drop for ScanWorker {
    fn drop(&mut self) {
        self.cancel();
        let _ = self.tx.send(ScanCmd::Close);
    }
}

/// Simulated scanners are offered in debug and test builds or with `NLP_FAKE_SCANNERS=1`.
pub fn simulated_scanners_enabled() -> bool {
    cfg!(debug_assertions) || cfg!(test) || std::env::var("NLP_FAKE_SCANNERS").is_ok_and(|v| v == "1")
}

/// Connected OpticFilm scanners.
fn opticfilm_devices() -> Vec<DeviceEntry> {
    match opticfilm::usb::find_devices(true) {
        Ok(devs) => devs
            .into_iter()
            .map(|d| {
                let label = opticfilm::device::select::model_for_device(d.product_id, d.bcd_device)
                    .map_or_else(|_| format!("Plustek {:04x}", d.product_id), |m| format!("{} {}", m.vendor, m.model));
                DeviceEntry { choice: ScannerChoice::Device(d.device_id()), label }
            })
            .collect(),
        Err(e) => {
            eprintln!("scanner listing failed: {e}");
            Vec::new()
        }
    }
}

/// `n` unused `scan_NNN.dng` names in `dir` (numbers taken by any extension count), in order.
pub fn next_scan_paths(dir: &Path, n: usize) -> Vec<PathBuf> {
    let used: Vec<u32> = std::fs::read_dir(dir)
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .filter_map(|e| {
                    let name = e.file_name().to_string_lossy().to_string();
                    let stem = name.split('.').next()?.strip_prefix("scan_")?.to_string();
                    stem.parse().ok()
                })
                .collect()
        })
        .unwrap_or_default();
    (1..).filter(|k| !used.contains(k)).take(n).map(|k| dir.join(format!("scan_{k:03}.dng"))).collect()
}

/// Where the IR plane of `scan` is kept: `NLP/<stem>.ir.tif`, out of the frame list.
pub fn ir_path_for(scan: &Path) -> PathBuf {
    let stem = scan.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
    sidecar::sidecar_path(scan).with_file_name(format!("{stem}.ir.tif"))
}

/// Row-major RGB16 as the engine's linear source: values over 65535, scanner
/// RGB read as sRGB primaries (as for its DNG's `ColorMatrix1`), no tone
/// curve to undo.
pub fn rgb16_to_source(rgb: &[u16], width: usize, height: usize) -> SourceImage {
    let data = rgb.chunks_exact(3).map(|p| [p[0] as f32 / 65535.0, p[1] as f32 / 65535.0, p[2] as f32 / 65535.0]).collect();
    SourceImage {
        width,
        height,
        data,
        to_melissa: color::to_f32(&color::rgb_to_rgb(&color::SRGB, &color::MELISSA)),
        as_shot_wb: [1.0; 3],
        is_raw: false,
        info: SourceInfo {
            full_width: width,
            full_height: height,
            software: opticfilm::image::SOFTWARE.into(),
            icc_trc: true,
            ..Default::default()
        },
    }
}

/// An OpticFilm scan as the engine's linear source.
pub fn scan_to_source(img: &ScanImage) -> SourceImage {
    let (h, w, _) = img.rgb.dim();
    let rgb = img.rgb.as_standard_layout();
    rgb16_to_source(rgb.as_slice().expect("standard layout"), w, h)
}

/// The negative rendered as a positive with the app's default conversion.
pub fn render_positive(src: &SourceImage, defaults: &Settings) -> egui::ColorImage {
    let (data, width, height) = decode::downscale(&src.data, src.width, src.height, PRESCAN_PREVIEW);
    let small = SourceImage { data, width, height, info: src.info.clone(), ..*src };
    let mut s = defaults.clone();
    s.source = Source::LinearScan;
    s.film_base = FilmBase::Off;
    s.crop = [0.0; 4];
    s.rotation = 0;
    s.flip_h = false;
    s.straighten = 0.0;
    let base = pipeline::resolve_base(&small, &s, None);
    let neg = pipeline::prepare_negative(&small, &s, base);
    let analysis = pipeline::analyze(&neg, &s, base, None);
    let levels = pipeline::levels_for(&s, Some(&analysis), None);
    let px = Renderer::new(&s, levels.as_ref(), OutputSpace::Srgb).render(&neg, 0.0);
    super::to_color_image(&px, neg.width, neg.height)
}

/// Plain words for OpticFilm driver errors.
pub fn describe(e: &ScanError) -> String {
    match e {
        ScanError::DeviceNotFound(_) => "No OpticFilm scanner found. Check the cable and power.".into(),
        ScanError::DriverBinding(m) => format!("The scanner can't be opened: {m}"),
        ScanError::ScannerStuck(_) => "The scanner reports its motor stuck. Switch it off and on, then scan again.".into(),
        ScanError::MotorTimeout(m) => format!("The carriage didn't move as expected: {m}"),
        ScanError::NotImplemented(m) | ScanError::Value(m) => m.clone(),
        e => e.to_string(),
    }
}

/// Plain words for SANE errors.
pub fn describe_sane(e: &sane::Error) -> String {
    use sane::Status;
    match e {
        sane::Error::Status { status: Status::DeviceBusy, .. } => "The scanner is busy. Close other scanning software and try again.".into(),
        sane::Error::Status { status: Status::CoverOpen, .. } => "The scanner lid is open.".into(),
        sane::Error::Status { status: Status::AccessDenied, .. } => "Access to the scanner was denied.".into(),
        sane::Error::Status { status: Status::IoError, .. } => "The scanner stopped responding. Check the cable and power.".into(),
        e => e.to_string(),
    }
}

/// How to get SANE when it's missing.
fn sane_install_hint() -> &'static str {
    if cfg!(target_os = "macos") {
        "Flatbed scanners need SANE: brew install sane-backends"
    } else if cfg!(target_os = "linux") {
        "Flatbed scanners need SANE: install your distribution's sane-backends (libsane) package"
    } else {
        "Flatbed scanners need SANE, which is available on macOS and Linux"
    }
}

/// The prescan resolution of an OpticFilm: the driver's GL128 prescan dpi, else its lowest.
pub fn prescan_dpi(model: &FilmModel) -> i64 {
    if model.resolutions_dpi.contains(&bringup::PRESCAN_DPI) {
        bringup::PRESCAN_DPI
    } else {
        model.resolutions_dpi.iter().copied().min().unwrap_or(bringup::PRESCAN_DPI)
    }
}

/// A flatbed prescan resolution: the lowest offered at or above 100 dpi.
fn sane_prescan_dpi(resolutions: &[u32]) -> u32 {
    resolutions.iter().copied().find(|&d| d >= SANE_PRESCAN_DPI).or_else(|| resolutions.last().copied()).unwrap_or(SANE_PRESCAN_DPI)
}

enum Open {
    Film { scanner: Box<Scanner>, model: Box<FilmModel> },
    Sane { device: Box<sane::Device>, vendor: String, model: String },
}

struct Worker {
    events: Sender<JobResult>,
    ctx: egui::Context,
    cancel: Arc<AtomicBool>,
    defaults: Settings,
    /// libsane, once loaded; `Err` holds why it isn't available.
    sane: Option<Result<sane::Sane, String>>,
    open: Option<(ScannerInfo, Open)>,
}

impl Worker {
    fn send(&self, ev: ScanEvent) {
        let _ = self.events.send(JobResult::Scan(ev));
        self.ctx.request_repaint();
    }

    fn status(&self, s: impl Into<String>) {
        self.send(ScanEvent::Status(s.into()));
    }

    fn run(mut self, rx: Receiver<ScanCmd>) {
        while let Ok(cmd) = rx.recv() {
            match cmd {
                ScanCmd::Refresh => self.refresh(),
                ScanCmd::Open(choice) => self.open(choice),
                ScanCmd::SetSource(s) => self.set_source(&s),
                ScanCmd::Prescan => self.prescan(),
                ScanCmd::Scan(job) => self.scan(&job),
                ScanCmd::Close => {
                    self.close();
                    break;
                }
            }
        }
        self.close();
    }

    fn close(&mut self) {
        if let Some((_, open)) = self.open.take() {
            if let Open::Film { mut scanner, .. } = open {
                scanner.close();
            }
            self.send(ScanEvent::Closed);
        }
    }

    /// libsane, loaded on first use.
    fn libsane(&mut self) -> Result<&mut sane::Sane, String> {
        let s = self.sane.get_or_insert_with(|| sane::Sane::load().map_err(|e| {
            eprintln!("SANE unavailable: {e}");
            sane_install_hint().to_string()
        }));
        s.as_mut().map_err(|e| e.clone())
    }

    fn refresh(&mut self) {
        let mut devices = opticfilm_devices();
        let mut sane_note = None;
        match self.libsane() {
            Ok(sane) => match sane.devices() {
                // OpticFilms are driven natively; SANE's genesys backend would compete for them.
                Ok(list) => devices.extend(
                    list.into_iter()
                        .filter(|d| !(d.vendor.eq_ignore_ascii_case("plustek") && d.model.to_ascii_lowercase().contains("opticfilm")))
                        .map(|d| DeviceEntry { label: format!("{} {}", d.vendor, d.model).trim().to_string(), choice: ScannerChoice::Sane(d.name) }),
                ),
                Err(e) => sane_note = Some(format!("SANE couldn't list scanners: {}", describe_sane(&e))),
            },
            Err(note) => sane_note = Some(note),
        }
        if simulated_scanners_enabled() {
            devices.push(DeviceEntry { choice: ScannerChoice::SimulatedFlatbed, label: "Simulated flatbed (SANE)".into() });
            devices.extend(
                opticfilm::known_models()
                    .iter()
                    .map(|m| DeviceEntry { choice: ScannerChoice::Simulated(m.name.to_string()), label: format!("Simulated {}", m.model) }),
            );
        }
        self.send(ScanEvent::Devices { devices, sane_note });
    }

    fn open(&mut self, choice: ScannerChoice) {
        self.close();
        self.status("Opening the scanner…");
        let r = match &choice {
            ScannerChoice::Device(_) | ScannerChoice::Simulated(_) => self.open_film(choice.clone()),
            ScannerChoice::Sane(_) | ScannerChoice::SimulatedFlatbed => self.open_sane(choice.clone()),
        };
        match r {
            Ok((info, open)) => {
                self.send(ScanEvent::Opened(Box::new(info.clone())));
                self.open = Some((info, open));
            }
            Err(m) => self.send(ScanEvent::Failed(m)),
        }
    }

    fn open_film(&mut self, choice: ScannerChoice) -> Result<(ScannerInfo, Open), String> {
        // The worker thread's clock: real for hardware, virtual for the mocks.
        let simulated = matches!(choice, ScannerChoice::Simulated(_));
        if simulated {
            opticfilm::clock::use_virtual(1000.0);
        } else {
            opticfilm::clock::use_real();
        }
        let opened = match &choice {
            ScannerChoice::Device(id) => Scanner::open(Some(id), None),
            ScannerChoice::Simulated(name) => match opticfilm::model_by_name(name) {
                Some(m) => {
                    let transport: Option<Box<dyn UsbTransport>> =
                        bringup::is_gl128_opticfilm(&m).then(|| Box::new(Gl128Sim::transport()) as Box<dyn UsbTransport>);
                    Scanner::open_fake(m, transport, None)
                }
                None => Err(ScanError::Value(format!("unknown scanner model {name}"))),
            },
            _ => unreachable!(),
        };
        let mut scanner = opened.map_err(|e| describe(&e))?;
        let model = scanner.model().clone();
        let scan_ready = model.scan_ready || simulated;
        if scan_ready {
            self.status("Warming up…");
            scanner.warmup(true, true).map_err(|e| describe(&e))?;
        }
        let mut resolutions = model.resolutions_dpi.clone();
        resolutions.sort_unstable();
        let crop = bringup::scan_area_to_image_crop(&model, bringup::default_frame_crop_norm(&model)).map(|v| v as f32);
        let info = ScannerInfo {
            label: format!("{} {}", model.vendor, model.model),
            choice,
            kind: ScannerKind::Film,
            scan_ready,
            resolutions,
            sources: Vec::new(),
            source: None,
            supports_ir: model.supports_infrared,
            supports_me: bringup::is_gl128_opticfilm(&model) || model.me_long_clamp_max.is_some(),
            supports_passes: bringup::is_gl128_opticfilm(&model),
            default_crop: Some(crop),
        };
        Ok((info, Open::Film { scanner: Box::new(scanner), model: Box::new(model) }))
    }

    fn open_sane(&mut self, choice: ScannerChoice) -> Result<(ScannerInfo, Open), String> {
        let (mut device, vendor, model) = match &choice {
            ScannerChoice::SimulatedFlatbed => {
                let mut sim = sane::Sane::simulated(sane::FakeConfig::default());
                let d = sim.open(sane::fake::DEVICE_NAME).map_err(|e| describe_sane(&e))?;
                (d, "NLP Studio".to_string(), "Simulated flatbed".to_string())
            }
            ScannerChoice::Sane(name) => {
                let sane = self.libsane()?;
                let info = sane.devices().ok().and_then(|l| l.into_iter().find(|d| &d.name == name));
                let d = sane.open(name).map_err(|e| describe_sane(&e))?;
                let (v, m) = info.map_or_else(|| ("SANE".to_string(), name.clone()), |i| (i.vendor, i.model));
                (d, v, m)
            }
            _ => unreachable!(),
        };
        device.prepare_film(None).map_err(|e| describe_sane(&e))?;
        let info = sane_info(choice, &mut device, &vendor, &model);
        Ok((info, Open::Sane { device: Box::new(device), vendor, model }))
    }

    fn set_source(&mut self, source: &str) {
        let Some((info, Open::Sane { device, .. })) = self.open.as_mut() else { return };
        if let Err(e) = device.prepare_film(Some(source)) {
            return self.send(ScanEvent::Failed(describe_sane(&e)));
        }
        let (choice, label) = (info.choice.clone(), info.label.clone());
        let new = ScannerInfo { label, ..sane_info(choice, device, "", "") };
        *info = new.clone();
        self.send(ScanEvent::Opened(Box::new(new)));
    }

    /// A control that reports progress (scaled into `part` of the job) and status.
    fn control(&self, part: (f64, f64)) -> ScanControl {
        let (tx, ctx) = (self.events.clone(), self.ctx.clone());
        let (tx2, ctx2) = (self.events.clone(), self.ctx.clone());
        ScanControl {
            cancel: Some(self.cancel.clone()),
            progress: Some(Box::new(progress_sender(tx, ctx, part))),
            on_status: Some(Box::new(move |s| {
                let _ = tx2.send(JobResult::Scan(ScanEvent::Status(s.to_string())));
                ctx2.request_repaint();
            })),
        }
    }

    fn prescan(&mut self) {
        let defaults = self.defaults.clone();
        let ctl = self.control((0.0, 1.0));
        let Some((_, open)) = self.open.as_mut() else { return };
        let r: Result<SourceImage, Outcome> = match open {
            Open::Film { scanner, model } => {
                let dpi = prescan_dpi(model);
                let _ = self.events.send(JobResult::Scan(ScanEvent::Status(format!("Prescan at {dpi} dpi…"))));
                let req = ScanRequest { resolution: dpi, area: Some([0.0, 0.0, 1.0, 1.0]), ..Default::default() };
                let mut ctl = ctl;
                scanner.scan(&req, &mut ctl).map(|img| scan_to_source(&img)).map_err(Outcome::from_film)
            }
            Open::Sane { device, .. } => {
                let dpi = sane_prescan_dpi(&device.resolutions());
                let _ = self.events.send(JobResult::Scan(ScanEvent::Status(format!("Prescan at {dpi} dpi…"))));
                let mut progress = progress_sender(self.events.clone(), self.ctx.clone(), (0.0, 1.0));
                device
                    .set_window(dpi, [0.0, 0.0, 1.0, 1.0])
                    .and_then(|()| device.scan(&self.cancel, &mut progress))
                    .map(|img| rgb16_to_source(&img.rgb, img.width, img.height))
                    .map_err(Outcome::from_sane)
            }
        };
        match r {
            Ok(src) => self.send(ScanEvent::Prescan(render_positive(&src, &defaults))),
            Err(o) => self.send(o.event()),
        }
    }

    fn scan(&mut self, job: &ScanJob) {
        let n = job.areas.len().min(job.paths.len());
        for k in 0..n {
            if self.cancel.load(Ordering::SeqCst) {
                return self.send(ScanEvent::Cancelled);
            }
            let frame = if n > 1 { format!("Frame {} of {n}: ", k + 1) } else { String::new() };
            self.status(format!("{frame}scanning at {} dpi…", job.dpi));
            let part = (k as f64 / n as f64, (k + 1) as f64 / n as f64);
            let (path, ir_path) = (&job.paths[k], ir_path_for(&job.paths[k]));
            match self.scan_area(job, job.areas[k], part, path, &ir_path) {
                Ok((path, ir_path)) => self.send(ScanEvent::Saved { path, ir_path }),
                Err(o) => return self.send(o.event()),
            }
        }
    }

    /// Scan one area and save it; returns the file saved and the IR plane's, if any.
    fn scan_area(&mut self, job: &ScanJob, area: [f32; 4], part: (f64, f64), path: &Path, ir_path: &Path) -> Result<(PathBuf, Option<PathBuf>), Outcome> {
        let ctl = self.control(part);
        let save_err = |e: anyhow::Error| Outcome::Failed(format!("Saving {} failed: {e:#}", path.display()));
        let Some((_, open)) = self.open.as_mut() else { return Err(Outcome::Failed("No scanner open".into())) };
        match open {
            Open::Film { scanner, model } => {
                let req = ScanRequest {
                    resolution: job.dpi,
                    area: Some(bringup::image_crop_to_scan_area(model, area.map(f64::from))),
                    infrared: job.infrared && model.supports_infrared,
                    multi_exposure: job.multi_exposure,
                    n_passes: job.passes.max(1),
                    ..Default::default()
                };
                let mut ctl = ctl;
                let img = scanner.scan(&req, &mut ctl).map_err(Outcome::from_film)?;
                let (h, w, _) = img.rgb.dim();
                let rgb = img.rgb.as_standard_layout();
                let info = ScanDngInfo { make: model.vendor, model: model.model, software: opticfilm::image::SOFTWARE, dpi: img.dpi as u32 };
                write_atomic(path, |tmp| scan_dng::write(tmp, rgb.as_slice().expect("standard layout"), w, h, &info)).map_err(save_err)?;
                let Some(ir) = &img.ir else { return Ok((path.to_path_buf(), None)) };
                if let Some(dir) = ir_path.parent() {
                    std::fs::create_dir_all(dir).map_err(|e| save_err(e.into()))?;
                }
                let ir = opticfilm::image::save_gray16_tiff(ir, ir_path, img.dpi).map_err(|e| save_err(e.into()))?;
                Ok((path.to_path_buf(), Some(ir)))
            }
            Open::Sane { device, vendor, model } => {
                let scan = sane::FilmScan { source: job.source.clone(), dpi: job.dpi.max(1) as u32, area: area.map(f64::from) };
                let mut progress = progress_sender(self.events.clone(), self.ctx.clone(), part);
                let img = device.scan_film(&scan, &self.cancel, &mut progress).map_err(Outcome::from_sane)?;
                if job.gamma_encoded {
                    // A tone curve was applied: a TIFF, opened as a TIF Scan.
                    let tiff = path.with_extension("tif");
                    write_atomic(&tiff, |tmp| scan_dng::write_tiff16(tmp, &img.rgb, img.width, img.height, img.dpi, "NLP Studio")).map_err(save_err)?;
                    return Ok((tiff, None));
                }
                let info = ScanDngInfo { make: vendor, model, software: SANE_SOFTWARE, dpi: img.dpi };
                write_atomic(path, |tmp| scan_dng::write(tmp, &img.rgb, img.width, img.height, &info)).map_err(save_err)?;
                Ok((path.to_path_buf(), None))
            }
        }
    }
}

/// Why a scan step ended without a result.
enum Outcome {
    Cancelled,
    Failed(String),
}

impl Outcome {
    fn from_film(e: ScanError) -> Self {
        match e {
            ScanError::ScanCancelled(_) => Self::Cancelled,
            e => Self::Failed(describe(&e)),
        }
    }
    fn from_sane(e: sane::Error) -> Self {
        match e {
            sane::Error::Cancelled => Self::Cancelled,
            e => Self::Failed(describe_sane(&e)),
        }
    }
    fn event(self) -> ScanEvent {
        match self {
            Self::Cancelled => ScanEvent::Cancelled,
            Self::Failed(m) => ScanEvent::Failed(m),
        }
    }
}

/// A SANE device's capabilities for the panel.
fn sane_info(choice: ScannerChoice, device: &mut sane::Device, vendor: &str, model: &str) -> ScannerInfo {
    let source = match device.get("source") {
        Ok(sane::Value::Str(s)) => Some(s),
        _ => None,
    };
    ScannerInfo {
        label: format!("{vendor} {model}").trim().to_string(),
        choice,
        kind: ScannerKind::Flatbed,
        scan_ready: true,
        resolutions: device.resolutions().into_iter().map(i64::from).collect(),
        sources: device.sources(),
        source,
        supports_ir: false,
        supports_me: false,
        supports_passes: false,
        default_crop: None,
    }
}

/// A progress callback that reports `fraction` of `part` of the job, in
/// visible steps.
fn progress_sender(tx: Sender<JobResult>, ctx: egui::Context, part: (f64, f64)) -> impl FnMut(f64) + Send + 'static {
    let last = Arc::new(Mutex::new(-1.0f64));
    move |p: f64| {
        let v = part.0 + (part.1 - part.0) * p.clamp(0.0, 1.0);
        let mut l = last.lock().unwrap();
        if p >= 1.0 || v - *l >= 0.005 {
            *l = v;
            let _ = tx.send(JobResult::Scan(ScanEvent::Progress(v)));
            ctx.request_repaint();
        }
    }
}

/// Write beside `path` and rename, so a half-written file never shows up as a frame.
fn write_atomic(path: &Path, write: impl FnOnce(&Path) -> anyhow::Result<()>) -> anyhow::Result<()> {
    let tmp = path.with_extension(format!("{}.part", path.extension().and_then(|e| e.to_str()).unwrap_or("")));
    write(&tmp)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_scan_path_skips_used_numbers() {
        let dir = std::env::temp_dir().join(format!("nlp-scan-next-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        assert_eq!(next_scan_paths(&dir, 1), [dir.join("scan_001.dng")]);
        for f in ["scan_001.dng", "scan_002.tif", "scan_004.dng", "other.dng"] {
            std::fs::write(dir.join(f), b"").unwrap();
        }
        assert_eq!(next_scan_paths(&dir, 3), ["scan_003.dng", "scan_005.dng", "scan_006.dng"].map(|f| dir.join(f)));
        assert_eq!(ir_path_for(&dir.join("scan_003.dng")), dir.join("NLP").join("scan_003.ir.tif"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Wait for the next scan event, skipping status and progress.
    fn next(rx: &std::sync::mpsc::Receiver<JobResult>) -> ScanEvent {
        loop {
            match rx.recv_timeout(std::time::Duration::from_secs(60)).expect("scanner event") {
                JobResult::Scan(ScanEvent::Status(_) | ScanEvent::Progress(_)) => {}
                JobResult::Scan(ev) => return ev,
                _ => {}
            }
        }
    }

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("nlp-scan-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn job(areas: Vec<[f32; 4]>, paths: Vec<PathBuf>) -> Box<ScanJob> {
        Box::new(ScanJob {
            dpi: 1800,
            areas,
            infrared: true,
            multi_exposure: false,
            passes: 1,
            source: None,
            gamma_encoded: false,
            paths,
        })
    }

    #[test]
    fn worker_scans_a_simulated_frame_into_the_roll() {
        let dir = temp_dir("worker");
        let (tx, rx) = channel();
        let worker = ScanWorker::spawn(tx, egui::Context::default(), Settings::default());
        worker.send(ScanCmd::Open(ScannerChoice::Simulated("plustek-opticfilm-8200i-se".into())));
        let ScanEvent::Opened(info) = next(&rx) else { panic!("not opened") };
        assert!(info.scan_ready && info.kind == ScannerKind::Film);

        worker.send(ScanCmd::Prescan);
        let ScanEvent::Prescan(img) = next(&rx) else { panic!("no prescan") };
        assert!(img.width() > 0 && img.height() > 0);

        let paths = next_scan_paths(&dir, 1);
        worker.send(ScanCmd::Scan(job(vec![[0.2, 0.3, 0.3, 0.4]], paths.clone())));
        let ScanEvent::Saved { path: saved, ir_path } = next(&rx) else { panic!("not saved") };
        assert_eq!(saved, dir.join("scan_001.dng"));
        assert_eq!(ir_path.as_deref(), Some(dir.join("NLP").join("scan_001.ir.tif").as_path()));
        let src = decode::load(&saved, None).unwrap();
        assert_eq!(pipeline::detect_source(&src), Source::VueScanRaw);
        assert!(src.width > 10 && src.height > 10);

        // A cancelled scan writes nothing. (`send` clears the flag, so queue directly.)
        let paths = next_scan_paths(&dir, 1);
        assert_eq!(paths[0], dir.join("scan_002.dng"));
        worker.cancel();
        worker.tx.send(ScanCmd::Scan(job(vec![[0.2, 0.3, 0.3, 0.4]], paths.clone()))).unwrap();
        assert!(matches!(next(&rx), ScanEvent::Cancelled));
        assert!(!paths[0].exists() && !paths[0].with_extension("dng.part").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Three boxes on the simulated flatbed's transparency unit: three DNGs,
    /// opened as scanner DNGs; the gamma-encoded switch gives a TIF Scan TIFF.
    #[test]
    fn worker_scans_flatbed_boxes() {
        let dir = temp_dir("flatbed");
        let (tx, rx) = channel();
        let worker = ScanWorker::spawn(tx, egui::Context::default(), Settings::default());
        worker.send(ScanCmd::Refresh);
        let ScanEvent::Devices { devices, .. } = next(&rx) else { panic!("no device list") };
        assert!(devices.iter().any(|d| d.choice == ScannerChoice::SimulatedFlatbed));
        worker.send(ScanCmd::Open(ScannerChoice::SimulatedFlatbed));
        let ScanEvent::Opened(info) = next(&rx) else { panic!("not opened") };
        assert_eq!(info.kind, ScannerKind::Flatbed);
        assert_eq!(info.source.as_deref(), Some("Transparency Unit"));
        assert_eq!(info.sources, ["Flatbed", "Transparency Unit"]);
        assert!(info.resolutions.contains(&2400));

        worker.send(ScanCmd::Prescan);
        let ScanEvent::Prescan(img) = next(&rx) else { panic!("no prescan") };
        assert!(img.height() > img.width(), "the unit's strip area is tall: {:?}", img.size);

        // Three frames down the strip (24 × 36 mm at 38 mm pitch from 10 mm).
        let frame = |k: f32| [19.75 / 63.5, (10.0 + 38.0 * k) / 228.6, 43.75 / 63.5, (46.0 + 38.0 * k) / 228.6];
        let paths = next_scan_paths(&dir, 3);
        worker.send(ScanCmd::Scan(Box::new(ScanJob { dpi: 300, infrared: false, ..*job((0..3).map(|k| frame(k as f32)).collect(), paths.clone()) })));
        for p in &paths {
            let ScanEvent::Saved { path, ir_path } = next(&rx) else { panic!("not saved") };
            assert_eq!(&path, p);
            assert!(ir_path.is_none());
            let src = decode::load(&path, None).unwrap();
            assert_eq!(pipeline::detect_source(&src), Source::VueScanRaw);
            assert_eq!((src.width, src.height), (283, 425));
            assert_eq!(src.info.make, "NLP Studio");
        }

        // Gamma-encoded: a TIFF, read as a TIF Scan.
        let paths = next_scan_paths(&dir, 1);
        worker.send(ScanCmd::Scan(Box::new(ScanJob { dpi: 150, infrared: false, gamma_encoded: true, ..*job(vec![frame(0.0)], paths.clone()) })));
        let ScanEvent::Saved { path, .. } = next(&rx) else { panic!("not saved") };
        assert_eq!(path, dir.join("scan_004.tif"));
        let src = decode::load(&path, None).unwrap();
        assert_eq!(pipeline::detect_source(&src), Source::TifScan);

        // Switching the source reports the new capabilities.
        worker.send(ScanCmd::SetSource("Flatbed".into()));
        let ScanEvent::Opened(info) = next(&rx) else { panic!("no update") };
        assert_eq!(info.source.as_deref(), Some("Flatbed"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The worker against the real libsane's `test` backend (needs sane-backends):
    /// list, open, scan into a DNG, then close and let the worker thread exit.
    /// (The thread-exit crash with the full backend list is covered by the
    /// `sane` crate's `libsane_system_config_survives_thread_exit`.) Run with
    /// `cargo test --release worker_scans_from_real_libsane -- --ignored`.
    #[test]
    #[ignore]
    fn worker_scans_from_real_libsane() {
        let conf = temp_dir("sane-conf");
        std::fs::write(conf.join("dll.conf"), "test\n").unwrap();
        // SAFETY: set before libsane is loaded; run this ignored test on its own.
        unsafe { std::env::set_var("SANE_CONFIG_DIR", &conf) };
        let dir = temp_dir("libsane");
        let (tx, rx) = channel();
        let worker = ScanWorker::spawn(tx, egui::Context::default(), Settings::default());
        worker.send(ScanCmd::Refresh);
        let ScanEvent::Devices { devices, sane_note } = next(&rx) else { panic!("no device list") };
        assert!(sane_note.is_none(), "{sane_note:?}");
        let test = devices.iter().find(|d| matches!(&d.choice, ScannerChoice::Sane(n) if n.starts_with("test"))).expect("test device");
        worker.send(ScanCmd::Open(test.choice.clone()));
        let ScanEvent::Opened(info) = next(&rx) else { panic!("not opened") };
        assert_eq!(info.kind, ScannerKind::Flatbed);
        let paths = next_scan_paths(&dir, 1);
        worker.send(ScanCmd::Scan(Box::new(ScanJob { dpi: 75, infrared: false, ..*job(vec![[0.0, 0.0, 0.3, 0.3]], paths.clone()) })));
        let ScanEvent::Saved { path, .. } = next(&rx) else { panic!("not saved") };
        let src = decode::load(&path, None).unwrap();
        assert_eq!(pipeline::detect_source(&src), Source::VueScanRaw);
        // Close the scanner and let the worker thread exit while the process lives on.
        drop(worker);
        assert!(matches!(next(&rx), ScanEvent::Closed));
        std::thread::sleep(std::time::Duration::from_millis(300));
        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::remove_dir_all(&conf);
    }

    #[test]
    fn scan_to_source_keeps_linear_values() {
        let rgb = ndarray::Array3::from_shape_fn((2, 3, 3), |(y, x, c)| (y * 30000 + x * 1000 + c * 7) as u16);
        let src = scan_to_source(&ScanImage::new(rgb.clone(), 900, "x", None));
        assert_eq!((src.width, src.height), (3, 2));
        assert!(src.info.icc_trc && !src.is_raw);
        assert_eq!(src.data[3 + 2][1], rgb[[1, 2, 1]] as f32 / 65535.0);
    }
}
