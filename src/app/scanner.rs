//! Film scanner worker: one thread owns the `opticfilm` scanner, takes
//! commands from the Scan panel and reports back through the job channel.
//!
//! Scans land in the roll folder as `scan_NNN.dng` (LinearRaw, see
//! `engine::scan_dng`), the IR plane as `NLP/scan_NNN.ir.tif`. Simulated
//! scanners (`NLP_FAKE_SCANNERS=1`, or debug builds) run the driver's mocks:
//! GL128 models on the AFE simulator, so calibration runs as on hardware, on
//! the driver's virtual clock, so their lamp and motor waits take no time.

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

/// A scanner to open: a USB device id, or a simulated model by name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScannerChoice {
    Device(String),
    Simulated(String),
}

pub enum ScanCmd {
    Open(ScannerChoice),
    Prescan,
    Scan { req: Box<ScanRequest>, path: PathBuf, ir_path: PathBuf },
    Close,
}

/// What the panel needs to know about the open scanner.
#[derive(Debug, Clone)]
pub struct ScannerInfo {
    pub choice: ScannerChoice,
    pub model: FilmModel,
    /// Validated for scanning (others open for status only).
    pub scan_ready: bool,
}

pub enum ScanEvent {
    Opened(Box<ScannerInfo>),
    Status(String),
    Progress(f64),
    /// The prescan rendered as a positive, and the scan area it covers.
    Prescan(egui::ColorImage),
    Saved { path: PathBuf, ir_path: Option<PathBuf> },
    /// The scan stopped on Cancel; nothing was written.
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
            .spawn(move || Worker { events, ctx, cancel: c, defaults, scanner: None, model: None }.run(rx))
            .expect("spawn scanner thread");
        Self { tx, cancel }
    }

    pub fn send(&self, cmd: ScanCmd) {
        if matches!(cmd, ScanCmd::Prescan | ScanCmd::Scan { .. }) {
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

/// Simulated scanners are offered in debug builds or with `NLP_FAKE_SCANNERS=1`.
pub fn simulated_scanners_enabled() -> bool {
    cfg!(debug_assertions) || std::env::var("NLP_FAKE_SCANNERS").is_ok_and(|v| v == "1")
}

/// Connected supported scanners: (device id, label).
pub fn list_scanners() -> Vec<(String, String)> {
    match opticfilm::usb::find_devices(true) {
        Ok(devs) => devs
            .into_iter()
            .map(|d| {
                let name = opticfilm::device::select::model_for_device(d.product_id, d.bcd_device)
                    .map_or_else(|_| format!("Plustek {:04x}", d.product_id), |m| format!("{} {}", m.vendor, m.model));
                (d.device_id(), name)
            })
            .collect(),
        Err(e) => {
            eprintln!("scanner listing failed: {e}");
            Vec::new()
        }
    }
}

/// `scan_NNN.dng` with the first number not used in `dir` (by any extension).
pub fn next_scan_path(dir: &Path) -> PathBuf {
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
    let n = (1..).find(|n| !used.contains(n)).unwrap();
    dir.join(format!("scan_{n:03}.dng"))
}

/// Where the IR plane of `scan` is kept: `NLP/<stem>.ir.tif`, out of the frame list.
pub fn ir_path_for(scan: &Path) -> PathBuf {
    let stem = scan.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
    sidecar::sidecar_path(scan).with_file_name(format!("{stem}.ir.tif"))
}

/// A scan as the engine's linear source: values over 65535, scanner RGB read
/// as sRGB primaries (as for its DNG's `ColorMatrix1`), no tone curve to undo.
pub fn scan_to_source(img: &ScanImage) -> SourceImage {
    let (h, w, _) = img.rgb.dim();
    let rgb = img.rgb.as_standard_layout();
    let s = rgb.as_slice().expect("standard layout");
    let data = s.chunks_exact(3).map(|p| [p[0] as f32 / 65535.0, p[1] as f32 / 65535.0, p[2] as f32 / 65535.0]).collect();
    SourceImage {
        width: w,
        height: h,
        data,
        to_melissa: color::to_f32(&color::rgb_to_rgb(&color::SRGB, &color::MELISSA)),
        as_shot_wb: [1.0; 3],
        is_raw: false,
        info: SourceInfo {
            full_width: w,
            full_height: h,
            software: opticfilm::image::SOFTWARE.into(),
            icc_trc: true,
            ..Default::default()
        },
    }
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

/// Plain words for driver errors.
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

struct Worker {
    events: Sender<JobResult>,
    ctx: egui::Context,
    cancel: Arc<AtomicBool>,
    defaults: Settings,
    scanner: Option<Scanner>,
    model: Option<FilmModel>,
}

impl Worker {
    fn send(&self, ev: ScanEvent) {
        let _ = self.events.send(JobResult::Scan(ev));
        self.ctx.request_repaint();
    }

    fn run(mut self, rx: Receiver<ScanCmd>) {
        while let Ok(cmd) = rx.recv() {
            match cmd {
                ScanCmd::Open(choice) => self.open(choice),
                ScanCmd::Prescan => self.prescan(),
                ScanCmd::Scan { req, path, ir_path } => self.scan(&req, &path, &ir_path),
                ScanCmd::Close => {
                    self.close();
                    break;
                }
            }
        }
        self.close();
    }

    fn close(&mut self) {
        if let Some(mut s) = self.scanner.take() {
            s.close();
            self.model = None;
            self.send(ScanEvent::Closed);
        }
    }

    fn open(&mut self, choice: ScannerChoice) {
        self.close();
        self.send(ScanEvent::Status("Opening the scanner…".into()));
        // The worker thread's clock: real for hardware, virtual for the mocks.
        match &choice {
            ScannerChoice::Device(_) => opticfilm::clock::use_real(),
            ScannerChoice::Simulated(_) => opticfilm::clock::use_virtual(1000.0),
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
        };
        let mut scanner = match opened {
            Ok(s) => s,
            Err(e) => return self.send(ScanEvent::Failed(describe(&e))),
        };
        let model = scanner.model().clone();
        let scan_ready = model.scan_ready || matches!(choice, ScannerChoice::Simulated(_));
        if scan_ready {
            self.send(ScanEvent::Status("Warming up…".into()));
            if let Err(e) = scanner.warmup(true, true) {
                return self.send(ScanEvent::Failed(describe(&e)));
            }
        }
        self.model = Some(model.clone());
        self.scanner = Some(scanner);
        self.send(ScanEvent::Opened(Box::new(ScannerInfo { choice, model, scan_ready })));
    }

    /// A control that reports progress and status to the panel.
    fn control(&self) -> ScanControl {
        let (tx, ctx) = (self.events.clone(), self.ctx.clone());
        let (tx2, ctx2) = (self.events.clone(), self.ctx.clone());
        // Progress arrives often; send only visible steps.
        let last = Arc::new(Mutex::new(-1.0f64));
        ScanControl {
            cancel: Some(self.cancel.clone()),
            progress: Some(Box::new(move |p| {
                let mut l = last.lock().unwrap();
                if p >= 1.0 || p - *l >= 0.005 {
                    *l = p;
                    let _ = tx.send(JobResult::Scan(ScanEvent::Progress(p)));
                    ctx.request_repaint();
                }
            })),
            on_status: Some(Box::new(move |s| {
                let _ = tx2.send(JobResult::Scan(ScanEvent::Status(s.to_string())));
                ctx2.request_repaint();
            })),
        }
    }

    fn run_scan(&mut self, req: &ScanRequest) -> Option<ScanImage> {
        let mut ctl = self.control();
        let scanner = self.scanner.as_mut()?;
        match scanner.scan(req, &mut ctl) {
            Ok(img) => Some(img),
            Err(ScanError::ScanCancelled(_)) => {
                self.send(ScanEvent::Cancelled);
                None
            }
            Err(e) => {
                self.send(ScanEvent::Failed(describe(&e)));
                None
            }
        }
    }

    fn prescan(&mut self) {
        let Some(model) = self.model.clone() else { return };
        let dpi = prescan_dpi(&model);
        self.send(ScanEvent::Status(format!("Prescan at {dpi} dpi…")));
        let req = ScanRequest { resolution: dpi, area: Some([0.0, 0.0, 1.0, 1.0]), ..Default::default() };
        if let Some(img) = self.run_scan(&req) {
            let preview = render_positive(&scan_to_source(&img), &self.defaults);
            self.send(ScanEvent::Prescan(preview));
        }
    }

    fn scan(&mut self, req: &ScanRequest, path: &Path, ir_path: &Path) {
        self.send(ScanEvent::Status(format!("Scanning at {} dpi…", req.resolution)));
        let Some(img) = self.run_scan(req) else { return };
        self.send(ScanEvent::Status("Saving…".into()));
        match self.save(&img, path, ir_path) {
            Ok(ir) => self.send(ScanEvent::Saved { path: path.to_path_buf(), ir_path: ir }),
            Err(e) => self.send(ScanEvent::Failed(format!("Saving {} failed: {e:#}", path.display()))),
        }
    }

    fn save(&self, img: &ScanImage, path: &Path, ir_path: &Path) -> anyhow::Result<Option<PathBuf>> {
        let model = self.model.as_ref().expect("open scanner");
        let (h, w, _) = img.rgb.dim();
        let rgb = img.rgb.as_standard_layout();
        let info = ScanDngInfo { make: model.vendor, model: model.model, software: opticfilm::image::SOFTWARE, dpi: img.dpi as u32 };
        // Written beside the target and renamed, so a half-written file never shows up as a frame.
        let tmp = path.with_extension("dng.part");
        scan_dng::write(&tmp, rgb.as_slice().expect("standard layout"), w, h, &info)?;
        std::fs::rename(&tmp, path)?;
        let Some(ir) = &img.ir else { return Ok(None) };
        if let Some(dir) = ir_path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        Ok(Some(opticfilm::image::save_gray16_tiff(ir, ir_path, img.dpi)?))
    }
}

/// The prescan resolution: the driver's GL128 prescan dpi, else the model's lowest.
pub fn prescan_dpi(model: &FilmModel) -> i64 {
    if model.resolutions_dpi.contains(&bringup::PRESCAN_DPI) {
        bringup::PRESCAN_DPI
    } else {
        model.resolutions_dpi.iter().copied().min().unwrap_or(bringup::PRESCAN_DPI)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_scan_path_skips_used_numbers() {
        let dir = std::env::temp_dir().join(format!("nlp-scan-next-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        assert_eq!(next_scan_path(&dir), dir.join("scan_001.dng"));
        for f in ["scan_001.dng", "scan_002.tif", "scan_004.dng", "other.dng"] {
            std::fs::write(dir.join(f), b"").unwrap();
        }
        assert_eq!(next_scan_path(&dir), dir.join("scan_003.dng"));
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

    #[test]
    fn worker_scans_a_simulated_frame_into_the_roll() {
        let dir = std::env::temp_dir().join(format!("nlp-scan-worker-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let (tx, rx) = channel();
        let worker = ScanWorker::spawn(tx, egui::Context::default(), Settings::default());
        worker.send(ScanCmd::Open(ScannerChoice::Simulated("plustek-opticfilm-8200i-se".into())));
        let ScanEvent::Opened(info) = next(&rx) else { panic!("not opened") };
        assert!(info.scan_ready);

        worker.send(ScanCmd::Prescan);
        let ScanEvent::Prescan(img) = next(&rx) else { panic!("no prescan") };
        assert!(img.width() > 0 && img.height() > 0);

        let req = ScanRequest { resolution: 1800, area: Some([0.2, 0.3, 0.3, 0.4]), infrared: true, ..Default::default() };
        let path = next_scan_path(&dir);
        worker.send(ScanCmd::Scan { req: Box::new(req.clone()), path: path.clone(), ir_path: ir_path_for(&path) });
        let ScanEvent::Saved { path: saved, ir_path } = next(&rx) else { panic!("not saved") };
        assert_eq!(saved, dir.join("scan_001.dng"));
        assert_eq!(ir_path.as_deref(), Some(dir.join("NLP").join("scan_001.ir.tif").as_path()));
        let src = decode::load(&saved, None).unwrap();
        assert_eq!(pipeline::detect_source(&src), Source::VueScanRaw);
        assert!(src.width > 10 && src.height > 10);

        // A cancelled scan writes nothing. (`send` clears the flag, so queue directly.)
        let path = next_scan_path(&dir);
        assert_eq!(path, dir.join("scan_002.dng"));
        worker.cancel();
        worker.tx.send(ScanCmd::Scan { req: Box::new(req), path: path.clone(), ir_path: ir_path_for(&path) }).unwrap();
        assert!(matches!(next(&rx), ScanEvent::Cancelled));
        assert!(!path.exists() && !path.with_extension("dng.part").exists());
        let _ = std::fs::remove_dir_all(&dir);
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
