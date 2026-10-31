// SPDX-License-Identifier: GPL-3.0-or-later
//! Port of `test_mock_scan.py`: headless mock-scanner scans through the
//! `Scanner` façade (lock-out, progress, cancel, IR/ME pass rules, init
//! recovery). Tests that monkeypatch session internals are not ported; their
//! paths are covered by the byte-level parity scans.

mod common;

use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

use common::{noise, tmp_path};
use ndarray::{Array2, Array3};
use opticfilm::asic::registers::gl845 as r;
use opticfilm::device::select::model_is_scan_ready;
use opticfilm::device::{FilmModel, ScanMethod, model_by_name};
use opticfilm::error::Error;
use opticfilm::image::{ScanImage, infrared_tracks_image};
use opticfilm::scan::calibrate::CalibCache;
use opticfilm::scan::request::{ScanControl, ScanMode, ScanRequest};
use opticfilm::usb::device::UsbDeviceInfo;
use opticfilm::usb::fake::{FakeCore, FakeDevice, FakeUsbTransport, MockScanner, MockScannerTransport};
use opticfilm::usb::protocol::GenesysUsbProtocol;
use opticfilm::usb::trace::{RecordingTransport, UsbOp, format_usb_log_line};
use opticfilm::usb::transport::find_transport;
use opticfilm::{Scanner, clock};

const TINY: [f64; 4] = [0.0, 0.0, 0.08, 0.08];
const S8200I: &str = "plustek-opticfilm-8200i";
const S7600V2: &str = "plustek-opticfilm-7600i-v2";

fn m(name: &str) -> FilmModel {
    model_by_name(name).unwrap()
}

fn fake(name: &str) -> Scanner {
    clock::use_virtual(1000.0);
    let dir = tmp_path(&format!("mock-scan-{name}-{:?}", std::thread::current().id()));
    Scanner::open_fake(m(name), None, Some(dir.join("calib.json"))).unwrap()
}

fn tiny(resolution: i64) -> ScanRequest {
    ScanRequest { resolution, area: Some(TINY), apply_calib: false, ..Default::default() }
}

/// A [`ScanControl`] whose progress values land in the returned list.
fn recording() -> (ScanControl, Arc<Mutex<Vec<f64>>>) {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let s = seen.clone();
    let ctl = ScanControl { progress: Some(Box::new(move |p| s.lock().unwrap().push(p))), ..Default::default() };
    (ctl, seen)
}

fn mock(scanner: &mut Scanner) -> &mut MockScannerTransport {
    find_transport::<MockScannerTransport>(scanner.transport()).unwrap()
}

#[test]
fn scan_ready_unchanged_by_open_fake() {
    assert!(!model_is_scan_ready(&m(S8200I)));
    let mut scanner = fake(S8200I);
    assert!(!model_is_scan_ready(scanner.model()));
    scanner.scan(&tiny(900), &mut ScanControl::default()).unwrap(); // open_fake lifts the gate
    scanner.close();
}

#[test]
fn real_path_still_locks_non_se_without_open_fake() {
    let model = m(S8200I);
    let info = UsbDeviceInfo { vendor_id: 0x07B3, product_id: model.usb_product_id as u16, ..Default::default() };
    let dir = tmp_path("mock-scan-locked");
    let mut scanner = Scanner::new(Box::new(MockScannerTransport::new()), info, Some(model), Some(dir.join("c.json"))).unwrap();
    let err = scanner.scan(&tiny(900), &mut ScanControl::default()).unwrap_err();
    assert!(matches!(err, Error::Asic(_)) && err.to_string().contains("locked out"), "{err}");
    scanner.close();
}

#[test]
fn open_fake_8200i_scan_pattern_and_progress() {
    let mut scanner = fake(S8200I);
    let (mut ctl, seen) = recording();
    let image = scanner.scan(&tiny(900), &mut ctl).unwrap();
    scanner.close();
    assert_eq!(image.rgb.dim().2, 3);
    assert!(image.rgb.iter().any(|&v| v > 0));
    assert_eq!(seen.lock().unwrap().last().copied(), Some(1.0));
}

#[test]
fn open_fake_cancel() {
    let mut scanner = fake(S8200I);
    let mut ctl = ScanControl { cancel: Some(Arc::new(AtomicBool::new(true))), ..Default::default() };
    let err = scanner.scan(&tiny(900), &mut ctl).unwrap_err();
    assert!(matches!(err, Error::ScanCancelled(_)), "{err:?}");
    scanner.close();
}

#[test]
fn gl128_session_mock_scan() {
    let mut scanner = fake("plustek-opticfilm-8200i-se");
    let image = scanner.scan(&tiny(150), &mut ScanControl::default()).unwrap();
    assert!(image.rgb.iter().any(|&v| v > 0));
    assert!(mock(&mut scanner).core.transactions.iter().any(|t| t.operation == UsbOp::BulkRead));
    scanner.close();
}

#[test]
fn recording_transport_listener() {
    let lines = Arc::new(Mutex::new(Vec::<String>::new()));
    let l = lines.clone();
    let mut rec = RecordingTransport::new(Box::new(MockScannerTransport::new()));
    rec.listener = Some(Box::new(move |t| l.lock().unwrap().push(format_usb_log_line(t, 64))));
    let mut proto = GenesysUsbProtocol::new(Box::new(rec));
    proto.write_register(0x01, 0x22).unwrap();
    assert!(!find_transport::<RecordingTransport>(proto.transport()).unwrap().transactions.is_empty());
    let lines = lines.lock().unwrap();
    assert!(lines.first().is_some_and(|l| l.contains("control_write")), "{lines:?}");
}

#[test]
fn gl845_home_uses_reverse_mini_scan_and_stops() {
    let mut scanner = fake(S7600V2);
    scanner.asic().unwrap().init(false).unwrap();
    scanner.lamp_on(ScanMethod::Transparency).unwrap();
    {
        let t = mock(&mut scanner);
        t.dev.at_home = false;
        t.dev.sync_status(&mut t.core);
    }
    scanner.home(1.0).unwrap();

    let status = scanner.status().unwrap();
    assert!(status.is_at_home);
    assert!(!status.is_motor_enabled);
    // Parking between calibration and image passes must not cool the lamp.
    assert!(status.is_lamp_on);
    let core = &mock(&mut scanner).core;
    let dir = r::AGOHOME | r::NOTHOME | r::MTRREV;
    assert_eq!(core.reg(0x02) & dir, dir);
    assert_eq!((core.reg(0x25), core.reg(0x26), core.reg(0x27)), (0, 0, 3));
    assert_eq!(core.reg(0x01) & r::SCAN, 0);
    assert_eq!(core.reg(0x40) & (r::DATAENB | r::MOTMFLG), 0);
    scanner.close();
}

/// The regular mock; once `stuck`, the status register always reports MOTORENB.
struct MotorStuckOn {
    inner: MockScanner,
    stuck: bool,
}

impl FakeDevice for MotorStuckOn {
    fn before_register_read(&mut self, core: &mut FakeCore, addr: u16) {
        self.inner.before_register_read(core, addr);
        if self.stuck && addr == r::REG_0X41 {
            core.registers.insert(r::REG_0X41, r::STATUS_MOTORENB);
        }
    }
    fn before_probe_read(&mut self, core: &mut FakeCore, index: u16) {
        self.inner.before_probe_read(core, index);
    }
    fn after_register_write(&mut self, core: &mut FakeCore, addr: u16, value: u8) {
        self.inner.after_register_write(core, addr, value);
    }
    fn bulk_in_bytes(&mut self, core: &mut FakeCore, size: usize) -> Vec<u8> {
        self.inner.bulk_in_bytes(core, size)
    }
}

#[test]
fn gl845_stop_motor_reports_motor_still_enabled() {
    clock::use_virtual(1000.0);
    let MockScannerTransport { core, dev } = MockScannerTransport::new();
    let usb = FakeUsbTransport::with_device(core, MotorStuckOn { inner: dev, stuck: false });
    let mut scanner = Scanner::open_fake(m(S7600V2), Some(Box::new(usb)), Some(tmp_path("stop-motor").join("c.json"))).unwrap();
    scanner.asic().unwrap().init(false).unwrap();
    // Stuck only after init (Python patches read_status the same way).
    find_transport::<FakeUsbTransport<MotorStuckOn>>(scanner.transport()).unwrap().dev.stuck = true;
    let opticfilm::asic::Asic::Gl845(a) = scanner.asic().unwrap() else { panic!() };
    let err = a.stop_motor(0.0).unwrap_err();
    assert!(matches!(err, Error::MotorTimeout(_)) && err.to_string().contains("Could not stop GL845 motor"), "{err}");
}

#[test]
fn gl845_combined_infrared_scan() {
    for name in [S8200I, S7600V2] {
        let mut scanner = fake(name);
        let (mut ctl, seen) = recording();
        let image = scanner.scan(&ScanRequest { infrared: true, ..tiny(900) }, &mut ctl).unwrap();
        assert_eq!(image.rgb.dim().2, 3);
        let ir = image.ir.as_ref().unwrap();
        assert_eq!(ir.dim(), (image.rgb.dim().0, image.rgb.dim().1));
        assert_eq!(seen.lock().unwrap().last().copied(), Some(1.0), "{name}");
        scanner.close();
    }
}

#[test]
fn gl845_combined_infrared_unsupported_model_raises() {
    let mut scanner = fake("plustek-opticfilm-7200");
    let err = scanner.scan(&ScanRequest { infrared: true, ..tiny(900) }, &mut ScanControl::default()).unwrap_err();
    assert!(matches!(err, Error::Value(_)) && err.to_string().contains("does not support infrared"), "{err}");
    scanner.close();
}

#[test]
fn gl845_combined_infrared_requires_color_mode() {
    let mut scanner = fake(S8200I);
    let req = ScanRequest { infrared: true, mode: ScanMode::Infrared, ..tiny(900) };
    let err = scanner.scan(&req, &mut ScanControl::default()).unwrap_err();
    assert!(matches!(err, Error::Value(_)) && err.to_string().contains("require mode='color'"), "{err}");
    scanner.close();
}

fn mean(a: &Array3<u16>) -> f64 {
    a.iter().map(|&v| v as f64).sum::<f64>() / a.len() as f64
}

#[test]
fn gl845_multi_exposure_scan_merges_long_pass() {
    let model = m(S7600V2);
    let mut scanner = fake(S7600V2);
    let image = scanner.scan(&ScanRequest { multi_exposure: true, ..tiny(900) }, &mut ScanControl::default()).unwrap();
    let debug = scanner.last_me_debug().unwrap();
    assert!(model.me_long_clamp_min.unwrap() <= debug.exposure_long && debug.exposure_long <= model.me_long_clamp_max.unwrap());
    assert_eq!(debug.exposure_short, model.exposure_lperiod);
    assert!(mean(&debug.rgb_long) > mean(&debug.rgb_short));
    assert_eq!(image.rgb.dim(), debug.rgb_short.dim());
    assert!(image.ir.is_none());
}

#[test]
fn gl845_multi_exposure_with_infrared_runs_three_passes() {
    let mut scanner = fake(S7600V2);
    let (mut ctl, seen) = recording();
    let image = scanner.scan(&ScanRequest { multi_exposure: true, infrared: true, ..tiny(900) }, &mut ctl).unwrap();
    let ir = image.ir.as_ref().unwrap();
    assert_eq!(ir.dim(), (image.rgb.dim().0, image.rgb.dim().1));
    assert!(scanner.last_me_debug().is_some());
    let seen = seen.lock().unwrap();
    assert_eq!(seen.last().copied(), Some(1.0));
    assert!(seen.windows(2).all(|w| w[0] <= w[1]), "progress went backwards: {seen:?}");
    // Parked after the last pass.
    assert!(mock(&mut scanner).dev.at_home);
}

#[test]
fn gl845_me_long_override_is_clamped_to_validated_envelope() {
    let mut scanner = fake(S8200I);
    scanner.scan(&ScanRequest { multi_exposure: true, me_long_exposure: Some(60000), ..tiny(900) }, &mut ScanControl::default()).unwrap();
    let debug = scanner.last_me_debug().unwrap();
    assert_eq!(Some(debug.exposure_long), m(S8200I).me_long_clamp_max);
    assert_eq!(debug.exposure_reason.as_deref(), Some("manual-override"));
}

#[test]
fn gl845_cancel_between_passes_parks_head() {
    let mut scanner = fake(S7600V2);
    let cancel = Arc::new(AtomicBool::new(false));
    let c = cancel.clone();
    let mut ctl = ScanControl {
        cancel: Some(cancel),
        progress: Some(Box::new(move |p| {
            if p >= 0.5 {
                c.store(true, std::sync::atomic::Ordering::SeqCst);
            }
        })),
        ..Default::default()
    };
    let err = scanner.scan(&ScanRequest { infrared: true, ..tiny(900) }, &mut ctl).unwrap_err();
    assert!(matches!(err, Error::ScanCancelled(_)), "{err:?}");
    assert!(mock(&mut scanner).dev.at_home, "head not parked after cancel");
}

/// Python records `time.sleep` calls; on the virtual clock the settle shows
/// up as scan time.
#[test]
fn lamp_settles_only_when_relit() {
    let settle = m(S8200I).lamp_settle_s.unwrap();
    let mut scanner = fake(S8200I);
    scanner.asic().unwrap().init(false).unwrap();
    scanner.lamp_off().unwrap();
    let t0 = clock::monotonic();
    scanner.scan(&tiny(900), &mut ScanControl::default()).unwrap();
    let relit = clock::monotonic() - t0;
    let t1 = clock::monotonic();
    scanner.scan(&tiny(900), &mut ScanControl::default()).unwrap();
    let lit = clock::monotonic() - t1;
    assert!(relit - lit >= settle - 1e-6, "relit {relit:.2}s vs lit {lit:.2}s, settle {settle}s");
    assert!(lit < settle, "already-lit scan waited {lit:.2}s");
}

#[test]
fn gl843_keeps_multi_pass_unimplemented() {
    for (ir, me) in [(true, false), (false, true)] {
        let mut scanner = fake("plustek-opticfilm-7200i");
        let err = scanner.scan(&ScanRequest { infrared: ir, multi_exposure: me, ..tiny(900) }, &mut ScanControl::default()).unwrap_err();
        assert!(matches!(err, Error::NotImplemented(_)), "{err:?}");
    }
}

#[test]
fn gl845_without_me_envelope_rejects_multi_exposure() {
    let name = model_by_name("plustek-opticfilm-8100").map(|_| "plustek-opticfilm-8100").unwrap();
    let mut scanner = fake(name);
    let err = scanner.scan(&ScanRequest { multi_exposure: true, ..tiny(900) }, &mut ScanControl::default()).unwrap_err();
    assert!(matches!(err, Error::NotImplemented(_)) && err.to_string().contains("multi-exposure envelope"), "{err}");
}

fn mean2(a: &Array2<u16>) -> f64 {
    a.iter().map(|&v| v as f64).sum::<f64>() / a.len() as f64
}

#[test]
fn gl845_dark_calib_capture_keeps_lamp_off() {
    clock::use_virtual(1000.0);
    let path = tmp_path("dark-calib").join("calib.json");
    let mut scanner = Scanner::open_fake(m(S7600V2), None, Some(path.clone())).unwrap();
    scanner.scan(&ScanRequest { apply_calib: true, ..tiny(900) }, &mut ScanControl::default()).unwrap();
    scanner.close();
    let mut cache = CalibCache::new(Some(path), None, true);
    assert!(cache.load());
    let e = cache.entries.iter().find(|e| e.method == ScanMethod::Transparency).unwrap();
    // Before the fix configure re-lit the lamp, so dark ≈ white.
    assert!(mean2(&e.dark) < 0.5 * mean2(&e.white));
}

#[test]
fn gl845_infrared_calib_measures_dark() {
    clock::use_virtual(1000.0);
    let path = tmp_path("ir-calib").join("calib.json");
    let mut scanner = Scanner::open_fake(m(S7600V2), None, Some(path.clone())).unwrap();
    scanner.scan(&ScanRequest { mode: ScanMode::Infrared, apply_calib: true, ..tiny(900) }, &mut ScanControl::default()).unwrap();
    scanner.close();
    let mut cache = CalibCache::new(Some(path), None, true);
    assert!(cache.load());
    let e = cache.entries.iter().find(|e| e.method == ScanMethod::Infrared).unwrap();
    // IR LED and lamp off: a real (dim, non-zero) offset, not zeros.
    let (d, w) = (mean2(&e.dark), mean2(&e.white));
    assert!(0.0 < d && d < 0.5 * w, "dark {d} white {w}");
}

#[test]
fn ir_tracks_image_flags_silver_film_only() {
    let (h, w) = (400usize, 600usize);
    let wave = |amp: f64| Array2::from_shape_fn((h, w), |(y, x)| amp * ((x as f64 / 40.0).sin() * (y as f64 / 55.0).cos()));
    let scene = wave(15000.0).mapv(|v| 35000.0 + v);
    let to_rgb = |s: &Array2<f64>| Array3::from_shape_fn((h, w, 3), |(y, x, _)| s[(y, x)] as u16);
    let rgb = to_rgb(&scene);
    let n = noise(h * w, 7);
    let jitter = |sigma: f64| Array2::from_shape_fn((h, w), |(y, x)| sigma * 1.7 * n[y * w + x]);
    let u16s = |a: Array2<f64>| a.mapv(|v| v.clamp(0.0, 65535.0) as u16);

    // Colour film: IR flat apart from dust specks and a faint image echo.
    let smean = scene.mean().unwrap();
    let mut colour = scene.mapv(|v| 56000.0 + 0.01 * (v - smean));
    for i in 0..40 {
        colour[((i * 97) % h, (i * 131) % w)] = 20000.0;
    }
    assert!(!infrared_tracks_image(&rgb, &u16s(colour), 0.06, 8).unwrap());

    // Silver film: IR is the picture.
    let silver = u16s(scene.mapv(|v| 0.9 * v) + jitter(800.0));
    assert!(infrared_tracks_image(&rgb, &silver, 0.06, 8).unwrap());

    // Thin, low-contrast silver frame: IR spread under 0.2, but it matches the visible.
    let flat = wave(3000.0).mapv(|v| 40000.0 + v);
    let flat_ir = u16s(flat.mapv(|v| 0.9 * v) + jitter(150.0));
    assert!(infrared_tracks_image(&to_rgb(&flat), &flat_ir, 0.06, 8).unwrap());

    assert_eq!(ScanImage::new(rgb.clone(), 900, "x", Some(silver)).ir_tracks_image(), Some(true));
    assert_eq!(ScanImage::new(rgb, 900, "x", None).ir_tracks_image(), None);
}

#[test]
fn gl845_init_stops_scan_left_running_by_dead_session() {
    let mut scanner = fake(S7600V2);
    {
        // A previous process died mid-scan: SCAN set, data still buffered.
        let t = mock(&mut scanner);
        t.core.registers.insert(0x01, 0x22 | r::SCAN);
        t.dev.buffer_empty = false;
        t.dev.sync_status(&mut t.core);
    }
    scanner.asic().unwrap().init(false).unwrap();
    let core = &mock(&mut scanner).core;
    assert_eq!(core.aborted, 1);
    assert_eq!(core.reg(0x01) & r::SCAN, 0);
}

#[test]
fn gl845_init_leaves_idle_scanner_alone() {
    let mut scanner = fake(S7600V2);
    scanner.asic().unwrap().init(false).unwrap();
    assert_eq!(mock(&mut scanner).core.aborted, 0);
}
