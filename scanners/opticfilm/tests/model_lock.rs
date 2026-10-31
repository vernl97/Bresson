// SPDX-License-Identifier: GPL-3.0-or-later
//! Port of `tests/model_lock/`: frozen per-model oracles for the hardware-tested
//! GL128 models (geometry windows, bring-up gates, ME configure, slope order).

mod common;

use ndarray::Array3;
use opticfilm::asic::{Asic, Gl128};
use opticfilm::device::generated::{tables_8200i_se, tables_8300i_se};
use opticfilm::device::{FilmModel, model_by_name};
use opticfilm::error::Error;
use opticfilm::scan::bringup::{
    GeometryProfile, PRESCAN_DPI, bringup_scan_geometry, clamp_area, crop_scan_geometry, default_frame_crop_norm, effective_scan_area,
    image_crop_to_scan_area, preview_safe_scan_area, scan_area_to_image_crop,
};
use opticfilm::scan::geometry::{ScanGeometry, compute_calib_geometry, compute_geometry};
use opticfilm::scan::pipeline::{count_usb_end_dummy_columns, trim_to_optical_span};
use opticfilm::scan::request::{ScanControl, ScanRequest};
use opticfilm::scan::session_gl128::{Gl128ScanSession, clamp_me_long, image_feed2_steps};
use opticfilm::usb::fake::MockScannerTransport;
use opticfilm::usb::trace::UsbOp;
use opticfilm::usb::transport::find_transport;

const SE: &str = "plustek-opticfilm-8200i-se";
const V2: &str = "plustek-opticfilm-8100-v2";
const S83: &str = "plustek-opticfilm-8300i-se";
const GL128: [&str; 3] = [SE, V2, S83];

fn m(name: &str) -> FilmModel {
    model_by_name(name).unwrap()
}

fn gl128(model: &FilmModel, armed: bool) -> Gl128 {
    let Asic::Gl128(mut a) = Asic::create(Box::new(MockScannerTransport::new()), model.clone()).unwrap() else { panic!() };
    a.motor_moves_enabled = armed;
    a
}

fn regs(a: &mut Gl128) -> std::collections::HashMap<u16, u8> {
    find_transport::<MockScannerTransport>(a.protocol.transport()).unwrap().core.registers.clone()
}

fn u24(r: &std::collections::HashMap<u16, u8>, a: u16) -> i64 {
    let g = |k| *r.get(&k).unwrap_or(&0) as i64;
    (g(a) << 16) | (g(a + 1) << 8) | g(a + 2)
}

#[test]
fn identity_and_scan_flags() {
    for (name, pid, ir) in [(SE, 0x1825, true), (V2, 0x1824, false), (S83, 0x181F, true)] {
        let model = m(name);
        assert_eq!(model.asic, "GL128");
        assert_eq!(model.usb_vendor_id, 0x07B3);
        assert_eq!(model.usb_product_id, pid);
        assert!(model.scan_ready);
        assert_eq!(model.supports_infrared, ir, "{name}");
        assert!(model.mirror_x());
        assert!(model.g().strpixel_native_units);
        assert_eq!(model.g().optical_end_inactive_native, 0);
        assert_eq!(model.x_offset_ta_mm, 0.43);
        assert_eq!(model.x_size_ta_mm, 36.58);
        assert_eq!(model.g().feed_to_scan_top_steps, 13128);
    }
    let v2 = m(V2);
    assert_eq!(v2.g().feed_to_scan_steps, 13128);
    assert_eq!(v2.g().ladder_feed2_steps, 13128);
    assert_eq!(v2.lperiod_by_dpi.as_ref().unwrap().get(&opticfilm::device::MdKey(None, Some(7200))), Some(&16035));
}

#[test]
fn full_window_str_end_and_preview_safe_gate() {
    for name in GL128 {
        let model = m(name);
        let (area, meta) = preview_safe_scan_area(&model, 1200, 0.0);
        assert_eq!((area[0], area[2]), (0.0, 1.0));
        let want = if name == S83 { 1209 } else { 4836 };
        assert_eq!(meta.feed2, 13128, "{name}");
        assert_eq!((meta.target_lincnt, meta.max_lincnt), (want, want), "{name}");
        let (g, meta) = bringup_scan_geometry(&model, 1200, GeometryProfile::PreviewSafe).unwrap();
        assert_eq!(g.lincnt_register(), want);
        assert!(g.lincnt_register() <= model.max_lincnt_for(meta.feed2, 1200));
        let full = compute_geometry(1200, &model, None).unwrap();
        assert_eq!(model.feed_to_scan_steps_for_area(None), 13128);
        assert!(full.lincnt_register() <= model.max_lincnt_for(13128, 1200));
        let (g, _) = bringup_scan_geometry(&model, 1800, GeometryProfile::PreviewSafe).unwrap();
        assert_eq!((g.pixels, g.pixel_startx, g.pixel_endx, g.usb_end_drop), (2592, 242, 10610, 0), "{name}");
        let mut starts = Vec::new();
        for dpi in [1200, 1800, 3600, 7200] {
            let (g, _) = bringup_scan_geometry(&model, dpi, GeometryProfile::PreviewSafe).unwrap();
            starts.push(g.pixel_startx);
            assert_eq!(g.pixels % 2, 0);
            assert_eq!(g.pixel_endx - g.pixel_startx, g.optical_pixels);
            assert_eq!(g.line_bytes, g.pixels * 3 * (g.depth / 8));
            assert_eq!(g.usb_end_drop, 0);
        }
        assert!(starts.iter().all(|&s| s == 242), "{name}: {starts:?}");
    }
    assert_eq!(m(S83).feed_to_scan_steps_for_dpi(1200, false), 13112);
}

#[test]
fn session_windows_match_silverfast() {
    let model = m(SE);
    let g = compute_geometry(1200, &model, Some([0.0, 0.0, 1.0, 0.5])).unwrap();
    assert_eq!((g.pixel_startx, g.pixel_endx, g.pixels, g.usb_end_drop), (242, 10610, 1728, 0));
    assert_eq!(g.line_bytes, g.pixels * 6);
    // Session 04: STR 578 / 2478 px crop.
    let (str_, end, origin) = (578.0, 10490.0, 120.0);
    let tl_mm = (str_ - origin) * 25.4 / 7200.0;
    let x1 = (tl_mm - model.x_offset_ta_mm) / model.x_size_ta_mm;
    let x2 = x1 + (end - str_) * 25.4 / 7200.0 / model.x_size_ta_mm;
    let g = compute_geometry(1800, &model, Some([x1, 0.1, x2, 0.5])).unwrap();
    assert_eq!((g.pixel_startx, g.pixel_endx, g.pixels), (578, 10490, 2478));
    assert_eq!(g.optical_pixels % 4, 0);
    for name in GL128 {
        let model = m(name);
        let (a, _) = crop_scan_geometry(&model, 1800, [0.15, 0.1, 0.72, 0.8]).unwrap();
        let (b, _) = crop_scan_geometry(&model, 3600, [0.15, 0.1, 0.72, 0.8]).unwrap();
        assert_eq!(a.pixel_startx, b.pixel_startx, "{name}");
        assert!((a.pixel_endx - b.pixel_endx).abs() < 8);
        let c = compute_calib_geometry(1800, &model, None).unwrap();
        assert!(c.disable_buffer_full_move);
        assert_eq!((c.usb_end_drop, c.pixel_startx), (0, 120));
    }
}

#[test]
fn crop_widths_are_even_and_snap_to_output_pixels() {
    for name in GL128 {
        let model = m(name);
        for i in 0..80 {
            let x1 = (i % 40) as f64 / 100.0;
            let x2 = (x1 + 0.25 + (i % 20) as f64 / 100.0).min(1.0);
            let g = compute_geometry(1800, &model, Some([x1, 0.1, x2, 0.75])).unwrap();
            assert_eq!(g.pixels % 2, 0);
            assert_eq!(g.pixel_endx - g.pixel_startx, g.optical_pixels);
            assert_eq!(g.optical_pixels % 4, 0);
        }
        for dpi in [2400, 3600] {
            for i in 0..40 {
                let x1 = (i % 20) as f64 / 100.0;
                let x2 = (x1 + 0.25 + (i % 10) as f64 / 100.0).min(1.0);
                let g = compute_geometry(dpi, &model, Some([x1, 0.1, x2, 0.75])).unwrap();
                assert_eq!(g.pixels % 2, 0);
                assert_eq!(g.pixel_endx - g.pixel_startx, g.optical_pixels);
                if dpi == 2400 && name == SE {
                    assert_eq!((g.pixel_startx - 242) % 3, 0);
                }
            }
        }
        let (g, meta) = crop_scan_geometry(&model, 1800, [0.1, 0.0, 0.9, 1.0]).unwrap();
        assert_eq!(meta.profile, "crop");
        assert_eq!(g.pixels % 2, 0);
        assert!(g.lincnt_register() <= meta.max_lincnt);
        assert_eq!(g.lincnt_register(), meta.target_lincnt);
        assert!(meta.effective_area.is_some() && meta.requested_area.is_some());
        let requested = [0.1, 0.2, 0.9, 0.8];
        let g = compute_geometry(2400, &model, Some(requested)).unwrap();
        let e = effective_scan_area(&model, &g, requested);
        assert!((e[0] - 0.1).abs() < 1e-9 && (e[1] - 0.2).abs() < 1e-9);
        assert!(e[2] <= requested[2] + 1e-9);
        assert!((e[3] - requested[3]).abs() <= 1e-3, "{name}: {e:?}");
    }
}

#[test]
fn lperiod_and_shading_clocks_at_7200() {
    assert_eq!(m(V2).line_period_for(7200, None), 16035);
    assert_eq!(m(V2).shading_strip_clocks(7200, true, None), (0x10, 0x01, 0x01));
    assert_eq!(m(S83).line_period_for(7200, None), 16035);
    assert_eq!(m(S83).shading_strip_clocks(7200, true, None), (0x26, 0x02, 0x02));
}

#[test]
fn clamp_me_long_bounds() {
    let se = m(SE);
    for (dpi, v, want) in [
        (1800, 85000, 85000),
        (1800, 90000, 85000),
        (1800, 64000, 64000),
        (1800, 42000, 42000),
        (1800, 14000, 14000),
        (1800, 10000, 14000),
        (3600, 85000, 85000),
        (7200, 85000, 64000),
        (7200, 64000, 64000),
        (7200, 42000, 42000),
    ] {
        assert_eq!(clamp_me_long(&se, dpi, v), want, "{dpi} {v}");
    }
}

#[test]
fn image_feed2_uses_area_y1_when_area_missing() {
    for name in [SE, V2] {
        let model = m(name);
        let (geo, _) = crop_scan_geometry(&model, 1800, [0.1, 0.2, 0.9, 0.8]).unwrap();
        assert_eq!(image_feed2_steps(&model, &geo, false), model.feed_to_scan_steps_for_area(geo.area));
        let lost = ScanGeometry { area: None, ..geo.clone() };
        let from_y1 = model.feed_to_scan_steps_for_area(Some([0.0, lost.area_y1, 1.0, 1.0]));
        assert_eq!(image_feed2_steps(&model, &lost, false), from_y1);
        assert_ne!(from_y1, model.feed_to_scan_steps_for_area(None));
    }
}

fn configure_exposure(name: &str, exposure: Option<i64>, long: bool, dpi: i64) -> std::collections::HashMap<u16, u8> {
    let model = m(name);
    let mut a = gl128(&model, false);
    a.init(false).unwrap();
    let geo = compute_geometry(dpi, &model, if dpi == 7200 { Some([0.0, 0.0, 1.0, 0.12]) } else { None }).unwrap();
    let mut s = Gl128ScanSession::new(model);
    s.set_pass(exposure, long, false);
    s.configure(&mut a, &geo).unwrap();
    regs(&mut a)
}

#[test]
fn gl128_configure_exposure_registers() {
    for name in [SE, V2] {
        let r = configure_exposure(name, Some(42000), true, 1200);
        assert_eq!(u24(&r, 0x7D), 42000);
        assert_eq!((r[&0xA5], r[&0xAB]), (0x01, 0x01));
    }
    let r = configure_exposure(SE, Some(14000), false, 1200);
    assert_eq!(u24(&r, 0x7D), 14000);
    assert_eq!(r[&0xA5], 0x02);
    let r = configure_exposure(SE, Some(50000), true, 1200);
    assert_eq!((u24(&r, 0x7D), r[&0xA5], r[&0xAB]), (50000, 0x01, 0x01));
    // Driver-derived exposure above the hardware max is clamped.
    let r = configure_exposure(SE, Some(100000), true, 1200);
    assert_eq!(u24(&r, 0x7D), m(SE).g().me_hardware_max_exposure);
    let r = configure_exposure(V2, None, false, 7200);
    assert_eq!(u24(&r, 0x28), 16035);
}

#[test]
fn infrared_combined_scan_is_refused_on_the_8100_v2() {
    let model = m(V2);
    let mut a = gl128(&model, true);
    let req = ScanRequest { resolution: 150, area: Some([0.0, 0.0, 0.2, 0.1]), infrared: true, apply_calib: false, ..Default::default() };
    let e = Gl128ScanSession::new(model).run(&mut a, None, &req, &mut ScanControl::default()).unwrap_err();
    assert!(matches!(&e, Error::Scan(m) if m.contains("no infrared")), "{e:?}");
}

/// `(ahb address, payload)` per `write_ahb` in the transaction log.
fn ahb_writes(a: &mut Gl128) -> Vec<(u32, Vec<u8>)> {
    let t = &find_transport::<MockScannerTransport>(a.protocol.transport()).unwrap().core.transactions;
    let mut out: Vec<(u32, Vec<u8>)> = Vec::new();
    for x in t {
        match x.operation {
            UsbOp::ControlWrite if x.value == Some(0x82) && x.index == Some(0x01) => {
                let d = x.data.as_ref().unwrap();
                out.push((u32::from_le_bytes([d[0], d[1], d[2], d[3]]), Vec::new()));
            }
            UsbOp::BulkWrite => out.last_mut().unwrap().1.extend_from_slice(x.data.as_ref().unwrap()),
            _ => {}
        }
    }
    out
}

fn pack(w: &[i64]) -> Vec<u8> {
    w.iter().flat_map(|&v| [(v & 0xFF) as u8, (v >> 8) as u8]).collect()
}

#[test]
fn positioning_uploads_slow_then_fast_ramps() {
    for (name, steps, fast, slow) in [
        (SE, 13704, tables_8200i_se::SLOPE_TABLE_FAST, tables_8200i_se::SLOPE_TABLE_SLOW),
        (V2, 13128, tables_8200i_se::SLOPE_TABLE_FAST, tables_8200i_se::SLOPE_TABLE_SLOW),
        (S83, 13128, tables_8300i_se::SLOPE_TABLE_FAST, tables_8300i_se::SLOPE_TABLE_SLOW),
    ] {
        let model = m(name);
        let mut a = gl128(&model, true);
        a.position_for_full_frame_scan(Some(steps), 60.0, None).unwrap();
        let slope: Vec<Vec<u8>> = ahb_writes(&mut a).into_iter().filter(|(addr, _)| *addr == 0x1000_C000 || *addr == 0x1001_0000).map(|(_, d)| d).collect();
        assert_eq!(slope, vec![pack(slow), pack(slow), pack(fast), pack(fast)], "{name}");
    }
}

#[test]
fn prescan_crop_mapping() {
    let model = m(SE);
    assert_eq!(PRESCAN_DPI, 1200);
    let ta = image_crop_to_scan_area(&model, [0.1, 0.2, 0.4, 0.8]);
    let want = clamp_area([0.6, 0.2, 0.9, 0.8]);
    assert!(ta.iter().zip(want).all(|(a, b)| (a - b).abs() < 1e-12));
    let back = scan_area_to_image_crop(&model, ta);
    assert!(back.iter().zip([0.1, 0.2, 0.4, 0.8]).all(|(a, b)| (a - b).abs() < 1e-12));
    let [x1, y1, x2, y2] = default_frame_crop_norm(&model);
    assert!(0.0 <= x1 && x1 < x2 && x2 <= 1.0 && 0.0 <= y1 && y1 < y2 && y2 <= 1.0);
    assert_eq!(default_frame_crop_norm(&m("plustek-opticfilm-7600i-v1")), [0.0, 0.0, 1.0, 1.0]);
}

#[test]
fn usb_end_dummy_handling() {
    let model = m(SE);
    let geo = compute_geometry(1800, &model, Some([0.2, 0.2, 0.6, 0.5])).unwrap();
    let wide = Array3::<u16>::zeros((8, geo.pixels as usize + 5, 3));
    assert_eq!(trim_to_optical_span(wide, &geo).dim().1, geo.pixels as usize);
    for dpi in [1200, 1800, 3600, 7200] {
        assert_eq!(compute_geometry(dpi, &model, Some([0.15, 0.2, 0.7, 0.6])).unwrap().usb_end_drop, 0);
    }
    let mut rgb = Array3::<u16>::from_elem((8, 80, 3), 18_000);
    assert_eq!(count_usb_end_dummy_columns(&rgb, 24, 2048.0), 0);
    rgb.slice_mut(ndarray::s![.., 72.., ..]).fill(200);
    assert_eq!(count_usb_end_dummy_columns(&rgb, 24, 2048.0), 8);
}
