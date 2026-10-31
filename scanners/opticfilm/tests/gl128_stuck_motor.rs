// SPDX-License-Identifier: GPL-3.0-or-later
//! Port of `test_gl128_stuck_motor.py`: a scanner stuck with MOTORENB set and
//! motor power off must stop the scan before any feed.
//!
//! Seen on an 8200i SE after a colour pass: every stationary strip START hangs
//! with status `0xc9` (PWR | BUFEMPTY | HOME | MOTORENB) and `0x02=0x00` until
//! a power-cycle.

mod common;

use common::caplog;
use opticfilm::asic::registers::{gl128 as se, gl845 as r};
use opticfilm::asic::{Asic, Gl128};
use opticfilm::clock;
use opticfilm::device::{FilmModel, model_by_name};
use opticfilm::error::Error;
use opticfilm::scan::calib_gl128::AfeFrontend;
use opticfilm::scan::calibrate::Calibrator;
use opticfilm::scan::geometry::compute_geometry;
use opticfilm::scan::request::{ScanControl, ScanRequest};
use opticfilm::scan::session_gl128::Gl128ScanSession;
use opticfilm::usb::fake::{FakeCore, FakeDevice, FakeUsbTransport, MockScanner, MockScannerTransport};
use opticfilm::usb::trace::UsbOp;
use opticfilm::usb::transport::find_transport;
use opticfilm::Scanner;

const TINY: [f64; 4] = [0.0, 0.0, 0.08, 0.08];

/// Mock whose motor flag can stick on, as on the hardware.
struct StuckMock {
    inner: MockScanner,
    stuck: bool,
    /// Keep `0x02` reading back with AGOHOME armed (the older `0xcd` case).
    reg02_armed: bool,
    /// Report the carriage away from home while stuck.
    away: bool,
}

type StuckTransport = FakeUsbTransport<StuckMock>;

impl StuckMock {
    fn transport(stuck: bool) -> StuckTransport {
        let MockScannerTransport { core, dev } = MockScannerTransport::new();
        FakeUsbTransport::with_device(core, StuckMock { inner: dev, stuck, reg02_armed: false, away: false })
    }

    fn stuck_status(&self, core: &mut FakeCore) {
        if !self.stuck {
            return;
        }
        let mut v = r::STATUS_PWRBIT | r::STATUS_BUFEMPTY | r::STATUS_MOTORENB;
        if !self.away {
            v |= r::STATUS_HOMESNR;
        }
        core.registers.insert(r::REG_0X41, v);
        core.registers.insert(se::REG_STATUS, v);
    }
}

impl FakeDevice for StuckMock {
    fn before_register_read(&mut self, core: &mut FakeCore, addr: u16) {
        // Stuck: no motion ticks, the status byte stays put.
        if !(self.stuck && (addr == r::REG_0X41 || addr == se::REG_STATUS)) {
            self.inner.before_register_read(core, addr);
        }
        self.stuck_status(core);
        if addr == 0x02 && self.reg02_armed {
            core.registers.insert(0x02, 0x30); // MTRPWR | AGOHOME
        }
    }
    fn before_probe_read(&mut self, core: &mut FakeCore, index: u16) {
        if !self.stuck {
            self.inner.before_probe_read(core, index);
        }
    }
    fn after_register_write(&mut self, core: &mut FakeCore, addr: u16, value: u8) {
        self.inner.after_register_write(core, addr, value);
        self.stuck_status(core);
    }
    fn bulk_in_bytes(&mut self, core: &mut FakeCore, size: usize) -> Vec<u8> {
        self.inner.bulk_in_bytes(core, size)
    }
}

fn model(name: &str) -> FilmModel {
    model_by_name(name).unwrap()
}

fn se_model() -> FilmModel {
    model("plustek-opticfilm-8200i-se")
}

fn asic_on(usb: StuckTransport, m: FilmModel) -> Gl128 {
    clock::use_virtual(1000.0);
    let Asic::Gl128(mut a) = Asic::create(Box::new(usb), m).unwrap() else { panic!("not GL128") };
    a.init(false).unwrap();
    a
}

fn mock(a: &mut Gl128) -> &mut StuckTransport {
    find_transport::<StuckTransport>(a.protocol.transport()).unwrap()
}

/// Boot-blob uploads (`write_ahb` to `0x000fff00`) seen so far.
fn boots(a: &mut Gl128) -> usize {
    mock(a)
        .core
        .transactions
        .iter()
        .filter(|t| {
            t.operation == UsbOp::ControlWrite
                && t.value == Some(0x82)
                && t.index == Some(0x01)
                && t.data.as_ref().is_some_and(|d| d.len() >= 4 && u32::from_le_bytes([d[0], d[1], d[2], d[3]]) == 0x000F_FF00)
        })
        .count()
}

#[test]
fn reset_raises_when_motor_stuck_with_power_off() {
    let mut a = asic_on(StuckMock::transport(false), se_model());
    mock(&mut a).dev.stuck = true;
    let err = a.reset_stationary_scan_engine("AFE strip").unwrap_err();
    let msg = err.to_string();
    assert!(matches!(err, Error::ScannerStuck(_)), "{err:?}");
    assert!(msg.contains("status 0xc9") && msg.contains("Switch the scanner off and on"), "{msg}");
}

#[test]
fn reset_is_quiet_on_a_healthy_scanner() {
    let mut a = asic_on(StuckMock::transport(false), se_model());
    a.reset_stationary_scan_engine("AFE strip").unwrap();
}

/// The older 0xcd case (AGOHOME left armed) is not the stuck signature.
#[test]
fn reset_only_warns_when_agohome_is_still_armed() {
    let mut a = asic_on(StuckMock::transport(false), se_model());
    let dev = &mut mock(&mut a).dev;
    dev.stuck = true;
    dev.reg02_armed = true;
    caplog::start();
    a.reset_stationary_scan_engine("AFE strip").unwrap();
    assert!(caplog::contains(log::Level::Warn, "scan engine not idle"));
}

#[test]
fn reset_only_warns_when_carriage_is_not_at_home() {
    let mut a = asic_on(StuckMock::transport(false), se_model());
    let dev = &mut mock(&mut a).dev;
    dev.stuck = true;
    dev.away = true;
    caplog::start();
    a.reset_stationary_scan_engine("AFE strip").unwrap();
    assert!(caplog::contains(log::Level::Warn, "scan engine not idle"));
}

/// Motor feeds started (`0x0F = 0x01` register writes) on the mock so far.
fn feeds(scanner: &mut Scanner) -> usize {
    let Asic::Gl128(a) = scanner.asic().unwrap() else { panic!() };
    mock(a)
        .core
        .transactions
        .iter()
        .filter(|t| t.operation == UsbOp::ControlWrite && t.data.as_deref().is_some_and(|d| d.chunks_exact(2).any(|w| w == [0x0F, 0x01])))
        .count()
}

fn scan_on(stuck: bool, apply_calib: bool) -> (opticfilm::Result<opticfilm::ScanImage>, usize) {
    clock::use_virtual(1000.0);
    let dir = common::tmp_path(&format!("stuck-scan-{stuck}"));
    let mut scanner = Scanner::open_fake(se_model(), Some(Box::new(StuckMock::transport(stuck))), Some(dir.join("c.json"))).unwrap();
    let req = ScanRequest { resolution: 600, area: Some(TINY), apply_calib, ..Default::default() };
    let res = scanner.scan(&req, &mut ScanControl::default());
    let n = feeds(&mut scanner);
    scanner.close();
    (res, n)
}

#[test]
fn scan_stops_before_any_feed_when_stuck() {
    // A healthy scanner does feed (so the count below can't pass vacuously).
    let (res, n) = scan_on(false, false);
    res.unwrap();
    assert!(n > 0);
    // The Python test patches position_for_full_frame_scan; here no feed is started.
    let (res, n) = scan_on(true, true);
    let err = res.unwrap_err();
    assert!(matches!(err, Error::ScannerStuck(_)), "{err:?}");
    assert_eq!(n, 0);
}

/// It used to log a warning and return false, and the scan then started feeding.
#[test]
fn ir_calibration_does_not_swallow_a_stuck_scanner() {
    let mut a = asic_on(StuckMock::transport(false), se_model());
    a.motor_moves_enabled = true;
    let dir = common::tmp_path("stuck-ir");
    let mut cal = Calibrator::new(se_model(), Some(dir.join("c.json")), true);
    mock(&mut a).dev.stuck = true;
    let g = compute_geometry(600, &se_model(), Some(TINY)).unwrap();
    let err = cal.measure_ir_host_flatten(&mut a, &g).unwrap_err();
    assert!(matches!(err, Error::ScannerStuck(_)), "{err:?}");
}

/// Callers that already handle scan errors keep working.
#[test]
fn stuck_error_is_a_scan_error() {
    assert!(Error::ScannerStuck("x".into()).is_scan_error());
}

// --- root cause: re-initialise before stationary strips after an image pass ---

#[test]
fn reinit_is_a_no_op_without_an_image_pass() {
    let mut a = asic_on(StuckMock::transport(false), se_model());
    let before = boots(&mut a);
    a.reinit_before_stationary_strips().unwrap();
    assert_eq!(boots(&mut a), before);
}

#[test]
fn reinit_after_image_pass_boots_once_and_keeps_host_state() {
    let m83 = model("plustek-opticfilm-8300i-se");
    let mut a = asic_on(StuckMock::transport(false), m83.clone());
    let afe = AfeFrontend { gains: [30, 25, 30], offsets: [16, 22, 20] };
    a.last_afe = Some(afe);
    a.asic_shading_ready = true;
    a.image_pass_since_init = true;
    let before = boots(&mut a);

    a.reinit_before_stationary_strips().unwrap();
    a.reinit_before_stationary_strips().unwrap(); // second call: nothing left to do

    assert_eq!(boots(&mut a), before + 1);
    assert!(!a.image_pass_since_init);
    assert_eq!(a.last_afe, Some(afe)); // AFE codes still valid; re-applied by the strips
    assert!(!a.asic_shading_ready); // boot may wipe the table RAM
    let clock: Vec<u8> = mock(&mut a)
        .core
        .transactions
        .iter()
        .filter(|t| t.operation == UsbOp::ControlWrite && t.request == Some(0x0C) && t.value == Some(0x8C))
        .map(|t| t.data.as_ref().unwrap()[0])
        .collect();
    // Boot leaves the boot clock (0x0E), which runs the 8300i SE motor too fast.
    assert_eq!(*clock.last().unwrap() as i64, m83.gl128.as_ref().unwrap().asic_clock_scan_0x8c);
}

#[test]
fn image_configure_flags_the_asic_and_shading_configure_does_not() {
    let mut a = asic_on(StuckMock::transport(false), se_model());
    a.motor_moves_enabled = false; // configure only, no feeds
    let mut session = Gl128ScanSession::new(se_model());
    let g = compute_geometry(600, &se_model(), Some(TINY)).unwrap();
    let shading = opticfilm::scan::geometry::ScanGeometry { disable_buffer_full_move: true, ..g.clone() };
    session.configure(&mut a, &shading).unwrap();
    assert!(!a.image_pass_since_init);
    session.configure(&mut a, &g).unwrap();
    assert!(a.image_pass_since_init);
}

/// `search_afe` and `run_asic_shading` re-initialise before their first strip.
/// (Python patches the strips to fail; here the boot count shows the reinit.)
#[test]
fn stationary_entry_points_reinit_first() {
    for entry in ["search_afe", "run_asic_shading"] {
        let mut a = asic_on(StuckMock::transport(false), se_model());
        a.motor_moves_enabled = false;
        a.image_pass_since_init = true;
        let before = boots(&mut a);
        let _ = match entry {
            "search_afe" => a.search_afe(None, Some(600)).map(|_| ()),
            _ => a.run_asic_shading(600, None, None, None, None).map(|_| ()),
        };
        assert_eq!(boots(&mut a), before + 1, "{entry}");
        assert!(!a.image_pass_since_init, "{entry}");
    }
}
