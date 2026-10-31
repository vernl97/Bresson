// SPDX-License-Identifier: GPL-3.0-or-later
//! Rust-only behaviour with no Python test: RAII cleanup on error and cancel
//! paths, and the `Scanner` contract the app relies on (`Send`, `close`).

mod common;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use opticfilm::asic::Asic;
use opticfilm::asic::registers::gl845 as r;
use opticfilm::device::model_by_name;
use opticfilm::error::Error;
use opticfilm::scan::request::{ScanControl, ScanRequest};
use opticfilm::usb::fake::MockScannerTransport;
use opticfilm::usb::transport::find_transport;
use opticfilm::{Scanner, clock};

const SE: &str = "plustek-opticfilm-8200i-se";

fn fake(name: &str, tag: &str) -> Scanner {
    clock::use_virtual(1000.0);
    let dir = common::tmp_path(&format!("guards-{tag}"));
    Scanner::open_fake(model_by_name(name).unwrap(), None, Some(dir.join("c.json"))).unwrap()
}

fn motor_armed(s: &mut Scanner) -> bool {
    let Asic::Gl128(a) = s.asic().unwrap() else { panic!("not GL128") };
    a.motor_moves_enabled
}

#[test]
fn scanner_is_send() {
    fn assert_send<T: Send>() {}
    assert_send::<Scanner>();
    assert_send::<ScanControl>();
}

/// GL128 calibration disarms the motor for its stationary strips. On the mock
/// the 1800 dpi strips never report data (as in pyopticfilm's `se_1800_crop_calib`
/// fixture), so calibration fails inside the guard: the motor must be re-armed.
/// (A follow-up scan is refused as "not at home" in both ports: the mock moves
/// the carriage on every START, stationary or not.)
#[test]
fn failed_stationary_calibration_rearms_the_motor() {
    let mut scanner = fake(SE, "stationary");
    assert!(motor_armed(&mut scanner));
    let req = ScanRequest { resolution: 1800, area: Some([0.2, 0.3, 0.45, 0.5]), ..Default::default() };
    let err = scanner.scan(&req, &mut ScanControl::default()).unwrap_err();
    assert!(!matches!(err, Error::ScanCancelled(_)), "{err:?}");
    assert!(motor_armed(&mut scanner), "motor left disarmed after {err}");
}

/// Cancelling during the GL128 image read stops the scan between chunks (no
/// transfer is in flight then, so nothing needs aborting) and clears SCAN, so
/// the next scan starts clean.
#[test]
fn gl128_cancel_mid_read_aborts_the_stream_and_stops_the_scan() {
    let mut scanner = fake(SE, "cancel");
    let cancel = Arc::new(AtomicBool::new(false));
    let c = cancel.clone();
    let mut ctl = ScanControl {
        cancel: Some(cancel),
        progress: Some(Box::new(move |p| {
            if p > 0.0 && p < 1.0 {
                c.store(true, Ordering::SeqCst);
            }
        })),
        ..Default::default()
    };
    let req = ScanRequest { resolution: 1200, area: Some([0.0, 0.0, 1.0, 0.3]), apply_calib: false, ..Default::default() };
    let err = scanner.scan(&req, &mut ctl).unwrap_err();
    assert!(matches!(err, Error::ScanCancelled(_)), "{err:?}");
    let core = &find_transport::<MockScannerTransport>(scanner.transport()).unwrap().core;
    assert_eq!(core.aborted, 0);
    assert_eq!(core.reg(0x01) & r::SCAN, 0, "SCAN left set");
    scanner.scan(&ScanRequest { area: Some([0.0, 0.0, 0.08, 0.08]), ..req }, &mut ScanControl::default()).unwrap();
}

#[test]
fn close_is_idempotent_and_closed_scanner_refuses_work() {
    let mut scanner = fake(SE, "close");
    scanner.close();
    scanner.close();
    let req = ScanRequest { resolution: 150, area: Some([0.0, 0.0, 0.08, 0.08]), apply_calib: false, ..Default::default() };
    let err = scanner.scan(&req, &mut ScanControl::default()).unwrap_err();
    assert!(matches!(err, Error::Usb(_)) && err.to_string().contains("closed"), "{err}");
    assert!(scanner.status().is_err());
}

/// Closing an open scanner turns the lamp off (`Drop` calls `close`).
#[test]
fn drop_turns_the_lamp_off() {
    let mut scanner = fake("plustek-opticfilm-8200i", "drop");
    scanner.asic().unwrap().init(false).unwrap();
    scanner.lamp_on(opticfilm::ScanMethod::Transparency).unwrap();
    assert!(scanner.status().unwrap().is_lamp_on);
    let lamp_reg = {
        let core = &find_transport::<MockScannerTransport>(scanner.transport()).unwrap().core;
        core.reg(0x03)
    };
    assert_ne!(lamp_reg & r::LAMPPWR, 0);
    scanner.close();
    let core = &find_transport::<MockScannerTransport>(scanner.transport()).unwrap().core;
    assert_eq!(core.reg(0x03) & r::LAMPPWR, 0, "lamp left on after close");
}
