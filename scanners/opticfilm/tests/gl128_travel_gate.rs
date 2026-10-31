// SPDX-License-Identifier: GPL-3.0-or-later
//! Port of `test_gl128_travel_gate.py`: default full frames pass the GL128
//! image-pass motor gate, overruns do not.
//!
//! SilverFast's own full-frame LINCNTs run up to 12 steps past
//! `scan_window_end_steps`; the gate allows that and nothing more.

mod common;

use opticfilm::asic::{Asic, Gl128};
use opticfilm::clock;
use opticfilm::device::{FilmModel, model_by_name};
use opticfilm::scan::bringup::apply_target_lincnt;
use opticfilm::scan::geometry::compute_geometry;
use opticfilm::scan::session_gl128::{Gl128ScanSession, image_feed2_steps};
use opticfilm::usb::fake::MockScannerTransport;

const SE: &str = "plustek-opticfilm-8200i-se";
const V2: &str = "plustek-opticfilm-8100-v2";
const S83: &str = "plustek-opticfilm-8300i-se";
const LADDER_DPI: [i64; 9] = [600, 720, 900, 1200, 1440, 1800, 2400, 3600, 7200];

/// SilverFast full-frame image LINCNT at feed2 13128 (8100 V2 captures).
const SF_LINCNT_SE_V2: [(i64, i64); 9] =
    [(600, 2420), (720, 2904), (900, 3628), (1200, 4836), (1440, 5804), (1800, 7252), (2400, 9668), (3600, 14500), (7200, 29012)];
/// SilverFast full-frame image LINCNT on the 8300i SE (PPI ladder).
const SF_LINCNT_8300I: [(i64, i64); 7] = [(600, 605), (900, 907), (1200, 1209), (1800, 1813), (2400, 2417), (3600, 3625), (7200, 7253)];

fn m(name: &str) -> FilmModel {
    model_by_name(name).unwrap()
}

fn armed(model: &FilmModel) -> (Gl128, Gl128ScanSession) {
    clock::use_virtual(1000.0);
    let Asic::Gl128(mut a) = Asic::create(Box::new(MockScannerTransport::new()), model.clone()).unwrap() else { panic!() };
    a.motor_moves_enabled = true;
    a.init(false).unwrap();
    (a, Gl128ScanSession::new(model.clone()))
}

#[test]
fn default_full_frame_fits_the_motor_gate() {
    for name in [SE, V2, S83] {
        let model = m(name);
        for dpi in LADDER_DPI {
            let g = compute_geometry(dpi, &model, None).unwrap();
            let feed2 = image_feed2_steps(&model, &g, false);
            assert!(g.lincnt_register() <= model.max_safe_lincnt_for(feed2, dpi), "{name} {dpi}");
        }
    }
}

#[test]
fn silverfast_full_frame_lincnt_fits_se_v2() {
    for name in [SE, V2] {
        for (dpi, lincnt) in SF_LINCNT_SE_V2 {
            assert!(lincnt <= m(name).max_safe_lincnt_for(13128, dpi), "{name} {dpi}");
        }
    }
}

#[test]
fn silverfast_full_frame_lincnt_fits_8300i() {
    let model = m(S83);
    for (dpi, lincnt) in SF_LINCNT_8300I {
        let feed2 = model.feed_to_scan_steps_for_dpi(dpi, false);
        assert!(lincnt <= model.max_safe_lincnt_for(feed2, dpi), "{dpi}");
    }
}

/// 12 steps past the window end; crop sizing keeps the window end.
#[test]
fn safe_limit_is_only_the_silverfast_overrun() {
    let model = m(V2);
    let g = model.gl128.as_ref().unwrap();
    assert_eq!(g.max_travel_end_steps - g.scan_window_end_steps, 12);
    assert_eq!(model.max_lincnt_for(13128, 1440), 5800);
    assert_eq!(model.max_safe_lincnt_for(13128, 1440), 5808);
}

/// These raised a scan error before the gate used SilverFast's travel.
#[test]
fn configure_accepts_default_full_frame() {
    for (name, dpi) in [(SE, 1440), (V2, 1440), (V2, 7200)] {
        let model = m(name);
        let (mut a, mut session) = armed(&model);
        session.configure(&mut a, &compute_geometry(dpi, &model, None).unwrap()).unwrap_or_else(|e| panic!("{name} {dpi}: {e}"));
    }
}

#[test]
fn configure_still_refuses_an_overrun() {
    let model = m(V2);
    let g = compute_geometry(1440, &model, None).unwrap();
    let feed2 = image_feed2_steps(&model, &g, false);
    let too_long = model.max_safe_lincnt_for(feed2, 1440) + model.gl128.as_ref().unwrap().image_lincnt_per_line;
    let (mut a, mut session) = armed(&model);
    let err = session.configure(&mut a, &apply_target_lincnt(&g, too_long)).unwrap_err();
    assert!(err.is_scan_error() && err.to_string().contains("travel limit"), "{err}");
}
