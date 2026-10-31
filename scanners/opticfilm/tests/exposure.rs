// SPDX-License-Identifier: GPL-3.0-or-later
//! Port of `test_me_exposure.py` and `test_manual_exposure.py`: adaptive ME
//! long-pass selection, and manual exposure overrides (validation, GL128
//! single pass, GL128 ME short/long). An explicit value reaches
//! `REG_EXPOSURE` verbatim; derived values keep the envelope and hardware clamp.
//!
//! Not ported: the float64/NaN and numpy-integer/bool input tests (the Rust
//! API is typed `u16` / `i64`), and the tests that patch `_configure` or
//! `pipeline.assemble` to spy on passes.

mod common;

use ndarray::Array3;
use opticfilm::asic::{Asic, Gl128};
use opticfilm::device::{FilmModel, model_by_name};
use opticfilm::error::Error;
use opticfilm::scan::exposure_override::{MAX_EXPOSURE_REGISTER, validate_manual_exposure, validate_n_passes};
use opticfilm::scan::geometry::compute_geometry;
use opticfilm::scan::me_exposure::{LongExposureParams, choose_long_exposure, clamp_long_exposure, select_long_exposure};
use opticfilm::scan::request::{ScanControl, ScanRequest};
use opticfilm::scan::session_gl128::{Gl128ScanSession, clamp_me_long};
use opticfilm::usb::fake::MockScannerTransport;
use opticfilm::usb::transport::find_transport;
use opticfilm::{Scanner, clock};

const TINY: [f64; 4] = [0.0, 0.0, 0.08, 0.08];

fn se() -> FilmModel {
    model_by_name("plustek-opticfilm-8200i-se").unwrap()
}

fn rgb(value: u16) -> Array3<u16> {
    Array3::from_elem((32, 32, 3), value)
}

// --- test_me_exposure.py ---

#[test]
fn choose_long_dense_frame_raises_exposure() {
    // p05 ≈ 1765: needs ~5.7× to reach the 10000 target.
    let proposed = choose_long_exposure(rgb(1765).view(), 14000, 0.0, 5.0, 10000.0).unwrap();
    assert!(proposed > 42000.0);
    assert!((proposed / (14000.0 * 10000.0 / 1765.0) - 1.0).abs() < 1e-6);
}

#[test]
fn choose_long_thin_frame_low_ratio() {
    let proposed = choose_long_exposure(rgb(12000).view(), 14000, 0.0, 5.0, 10000.0).unwrap();
    assert!(proposed < 42000.0);
    assert!((proposed / (14000.0 * 10000.0 / 12000.0) - 1.0).abs() < 1e-6);
}

#[test]
fn clamp_long_exposure_limits() {
    let (sel, reason) = clamp_long_exposure(100000.0, 14000, 42000, 85000, 85000, 7.0);
    assert_eq!(sel, 85000);
    assert!(reason.contains("hardware"), "{reason}");
    let (sel, reason) = clamp_long_exposure(20000.0, 14000, 42000, 85000, 85000, 7.0);
    assert_eq!(sel, 42000);
    assert!(reason.contains("minimum"), "{reason}");
    // short × 5 = 70000 caps before the adaptive/hardware 85000.
    let (sel, reason) = clamp_long_exposure(100000.0, 14000, 42000, 85000, 85000, 5.0);
    assert_eq!(sel, 70000);
    assert!(reason.contains("hardware"), "{reason}");
}

#[test]
fn select_dense_rises_toward_max() {
    // Very dense: p05=1000 → proposed ≈ 140000 → clamp to 85000.
    let d = select_long_exposure(rgb(1000).view(), 14000, &LongExposureParams::default());
    assert!(d.proposed > 85000);
    assert_eq!(d.selected, 85000);
    assert!(d.reason.starts_with("clamped"), "{}", d.reason);
}

#[test]
fn select_thin_stays_at_adaptive_min() {
    let d = select_long_exposure(rgb(12000).view(), 14000, &LongExposureParams::default());
    assert_eq!(d.selected, 42000);
    assert!(d.reason.contains("minimum"), "{}", d.reason);
}

#[test]
fn select_empty_fallback() {
    let d = select_long_exposure(Array3::zeros((0, 0, 3)).view(), 14000, &LongExposureParams::default());
    assert_eq!(d.selected, 42000);
    assert_eq!(d.reason, "fallback");
}

#[test]
fn model_adaptive_envelope_defaults() {
    let m = se();
    let g = m.gl128.as_ref().unwrap();
    assert_eq!(m.exposure_long, Some(42000));
    assert_eq!((g.me_adaptive_min_exposure, g.me_adaptive_max_exposure, g.me_hardware_max_exposure), (42000, 85000, 85000));
    assert_eq!((m.me_long_clamp_min, m.me_long_clamp_max), (Some(14000), Some(85000)));
    assert_eq!(m.me_max_exposure_ratio, Some(7.0));
    assert_eq!(g.me_target_dense_dn, 10000.0);
}

/// V2 uses the same 14000–85000 envelope, still cut to 64000 at 7200 dpi.
#[test]
fn v2_me_long_clamp_matches_se() {
    let v2 = model_by_name("plustek-opticfilm-8100-v2").unwrap();
    assert_eq!((v2.me_long_clamp_min, v2.me_long_clamp_max), (Some(14000), Some(85000)));
    assert_eq!(clamp_me_long(&v2, 1800, 85000), 85000);
    assert_eq!(clamp_me_long(&v2, 1800, 90000), 85000);
    assert_eq!(clamp_me_long(&v2, 7200, 85000), 64000);
}

// --- test_manual_exposure.py: validation ---

#[test]
fn validate_manual_exposure_range() {
    assert_eq!(validate_manual_exposure(None, "x").unwrap(), None);
    assert_eq!(validate_manual_exposure(Some(MAX_EXPOSURE_REGISTER), "x").unwrap(), Some(MAX_EXPOSURE_REGISTER));
    for v in [0, -1, -100000, MAX_EXPOSURE_REGISTER + 1] {
        assert!(matches!(validate_manual_exposure(Some(v), "x"), Err(Error::Value(_))), "{v}");
    }
    assert!(validate_n_passes(0).is_err());
    assert_eq!(validate_n_passes(3).unwrap(), 3);
}

// --- single pass / ME short: what reaches REG_EXPOSURE ---

fn reg_exposure(a: &mut Gl128) -> i64 {
    let core = &find_transport::<MockScannerTransport>(a.protocol.transport()).unwrap().core;
    ((core.reg(0x7D) as i64) << 16) | ((core.reg(0x7E) as i64) << 8) | core.reg(0x7F) as i64
}

fn gl128(armed: bool) -> Gl128 {
    clock::use_virtual(1000.0);
    let Asic::Gl128(mut a) = Asic::create(Box::new(MockScannerTransport::new()), se()).unwrap() else { panic!() };
    a.motor_moves_enabled = armed;
    a.init(false).unwrap();
    a
}

/// `configure` with the given pass state (motor moves off) and the exposure it wrote.
fn configured(dpi: i64, exposure: i64, long: bool, manual: bool) -> i64 {
    let mut a = gl128(false);
    let mut s = Gl128ScanSession::new(se());
    s.set_pass(Some(exposure), long, manual);
    s.configure(&mut a, &compute_geometry(dpi, &se(), None).unwrap()).unwrap();
    reg_exposure(&mut a)
}

#[test]
fn single_pass_manual_reaches_register_unclamped() {
    assert_eq!(se().gl128.as_ref().unwrap().me_hardware_max_exposure, 85000);
    assert_eq!(configured(1200, 50000, false, true), 50000);
    assert_eq!(configured(1200, 100000, false, true), 100000);
}

/// No override: the existing hardware clamp still fires.
#[test]
fn single_pass_default_keeps_hardware_clamp() {
    assert_eq!(configured(1200, 100000, false, false), 85000);
}

#[test]
fn me_short_manual_reaches_register_unclamped() {
    assert_eq!(configured(1800, 30000, false, true), 30000);
    assert_eq!(configured(1800, 100000, false, true), 100000);
}

/// A manual pass must not leak into a later default-exposure scan.
#[test]
fn single_pass_manual_state_resets_between_scans() {
    clock::use_virtual(1000.0);
    let dir = common::tmp_path("manual-reset");
    let mut scanner = Scanner::open_fake(se(), None, Some(dir.join("c.json"))).unwrap();
    let req = ScanRequest { resolution: 150, area: Some(TINY), apply_calib: false, ..Default::default() };
    let exposure = |s: &mut Scanner| {
        let Asic::Gl128(a) = s.asic().unwrap() else { panic!() };
        reg_exposure(a)
    };
    scanner.scan(&ScanRequest { single_pass_exposure: Some(99999), ..req.clone() }, &mut ScanControl::default()).unwrap();
    assert_eq!(exposure(&mut scanner), 99999);
    scanner.scan(&req, &mut ScanControl::default()).unwrap();
    assert_eq!(exposure(&mut scanner), se().gl128.as_ref().unwrap().exposure_short);
}

// --- ME long: adaptive / DPI / hardware clamps bypassed end to end ---

fn me_run(dpi: i64, long: Option<i64>) -> (opticfilm::Result<opticfilm::ScanImage>, Gl128ScanSession, Gl128) {
    let mut a = gl128(true);
    let mut s = Gl128ScanSession::new(se());
    let req = ScanRequest { resolution: dpi, area: Some(TINY), apply_calib: false, multi_exposure: true, me_long_exposure: long, ..Default::default() };
    let r = s.run(&mut a, None, &req, &mut ScanControl::default());
    (r, s, a)
}

#[test]
fn me_short_default_uses_model_derived_value() {
    let (r, s, _) = me_run(1800, None);
    r.unwrap();
    assert_eq!(s.last_me_debug.unwrap().exposure_short, se().gl128.as_ref().unwrap().exposure_short);
}

#[test]
fn me_long_manual_below_normal_limits_not_raised_to_adaptive_floor() {
    let (r, s, _) = me_run(1800, Some(5000)); // below the 14000 floor / 42000 adaptive minimum
    r.unwrap();
    assert_eq!(s.last_me_debug.unwrap().exposure_long, 5000);
}

/// The 16-bit AHB exposure table's ceiling is reachable verbatim (131070 at 3600 dpi = word 65535).
#[test]
fn me_long_manual_skips_adaptive_clamp_and_reaches_register() {
    let (r, s, mut a) = me_run(3600, Some(131070));
    r.unwrap();
    let d = s.last_me_debug.unwrap();
    assert_eq!(d.exposure_long, 131070);
    assert_eq!(d.exposure_reason.as_deref(), Some("manual-override"));
    assert_eq!(d.exposure_proposed, None);
    // The long pass runs last, so its exposure is what the register still holds.
    assert_eq!(reg_exposure(&mut a), 131070);
}

/// A table word over 16 bits fails loudly instead of uploading a wrapped value.
#[test]
fn me_long_manual_above_16bit_table_raises() {
    let (r, _, _) = me_run(3600, Some(140000));
    let err = r.unwrap_err();
    assert!(matches!(err, Error::Value(_)) && err.to_string().contains("16-bit"), "{err}");
}

#[test]
fn me_long_manual_overrides_adaptive_selection() {
    let (r, s, _) = me_run(1800, Some(120000));
    r.unwrap();
    let d = s.last_me_debug.unwrap();
    assert_eq!(d.exposure_long, 120000);
    assert_eq!(d.exposure_reason.as_deref(), Some("manual-override"));
}

#[test]
fn me_long_default_uses_adaptive_selection() {
    let (r, s, _) = me_run(1800, None);
    r.unwrap();
    let d = s.last_me_debug.unwrap();
    assert_ne!(d.exposure_reason.as_deref(), Some("manual-override"));
    assert!(d.exposure_proposed.is_some());
    assert!((14000..=85000).contains(&d.exposure_long), "{}", d.exposure_long);
}

#[test]
fn scanner_rejects_invalid_manual_exposure_before_opening_asic() {
    clock::use_virtual(1000.0);
    let dir = common::tmp_path("manual-invalid");
    let mut scanner = Scanner::open_fake(se(), None, Some(dir.join("c.json"))).unwrap();
    let req = ScanRequest { resolution: 150, area: Some(TINY), apply_calib: false, me_long_exposure: Some(-5), ..Default::default() };
    assert!(matches!(scanner.scan(&req, &mut ScanControl::default()), Err(Error::Value(_))));
    assert!(!scanner.asic().unwrap().initialized());
    let core = &find_transport::<MockScannerTransport>(scanner.transport()).unwrap().core;
    assert!(core.transactions.is_empty(), "USB traffic before validation");
}

#[test]
fn single_pass_exposure_not_implemented_for_gl843() {
    clock::use_virtual(1000.0);
    let dir = common::tmp_path("manual-gl843");
    let mut scanner = Scanner::open_fake(model_by_name("plustek-opticfilm-7200i").unwrap(), None, Some(dir.join("c.json"))).unwrap();
    let req = ScanRequest { resolution: 900, area: Some(TINY), apply_calib: false, single_pass_exposure: Some(5000), ..Default::default() };
    assert!(matches!(scanner.scan(&req, &mut ScanControl::default()), Err(Error::NotImplemented(_))));
}
