// SPDX-License-Identifier: GPL-3.0-or-later
//! Port of `test_multi_model.py` and `test_gl128_firmware_variant.py`: model
//! selection from USB ids, ASIC routing, capability flags, and the per-firmware
//! GL128 register set (vendor `PID_xxxx` vs `PID_xxxx-20241113`).

mod common;

use opticfilm::asic::Asic;
use opticfilm::asic::gl845::Gl84xVariant;
use opticfilm::device::model::{GL128_FIRMWARE_CURRENT, GL128_REG36_LEGACY_FIRMWARE, MdKey, gl128_firmware_variant};
use opticfilm::device::select::model_for_device;
use opticfilm::device::{FilmModel, known_models, model_by_name};
use opticfilm::scan::calibrate::calib_fingerprint;
use opticfilm::scan::geometry::compute_geometry;
use opticfilm::usb::device::*;
use opticfilm::usb::fake::MockScannerTransport;
use opticfilm::{Scanner, clock};

fn m(name: &str) -> FilmModel {
    model_by_name(name).unwrap()
}

fn name_for(pid: u16, bcd: u16) -> &'static str {
    let n = model_for_device(pid, bcd).unwrap().name;
    known_models().iter().find(|k| k.name == n).unwrap().name
}

const GL128: [&str; 3] = ["plustek-opticfilm-8200i-se", "plustek-opticfilm-8100-v2", "plustek-opticfilm-8300i-se"];

// --- test_multi_model.py ---

#[test]
fn supported_ids_cover_complete_opticfilm() {
    for pid in [
        PID_OPTICFILM_8200I,
        PID_OPTICFILM_8200I_SE,
        PID_OPTICFILM_8300I_SE,
        PID_OPTICFILM_7200,
        PID_OPTICFILM_7200I,
        PID_OPTICFILM_7200_V2,
        PID_OPTICFILM_7300,
        PID_OPTICFILM_7400,
        PID_OPTICFILM_7500I,
        PID_OPTICFILM_7600I,
        PID_OPTICFILM_8100,
    ] {
        assert!(SUPPORTED_PIDS.contains(&pid), "{pid:#06x}");
    }
}

#[test]
fn bcd_disambiguation_7400_and_7600i() {
    assert_eq!(name_for(PID_OPTICFILM_7400, 0x0400), "plustek-opticfilm-7400-v1");
    assert_eq!(name_for(PID_OPTICFILM_7400, 0x0605), "plustek-opticfilm-7400-v2");
    assert_eq!(name_for(PID_OPTICFILM_7400, 0), "plustek-opticfilm-7400-v2");
    assert_eq!(name_for(PID_OPTICFILM_7600I, 0x0400), "plustek-opticfilm-7600i-v1");
    assert_eq!(name_for(PID_OPTICFILM_7600I, 0x0605), "plustek-opticfilm-7600i-v2");
}

#[test]
fn simple_pid_aliases() {
    for (pid, name) in [
        (PID_OPTICFILM_8200I, "plustek-opticfilm-8200i"),
        (PID_OPTICFILM_8200I_SE, "plustek-opticfilm-8200i-se"),
        (PID_OPTICFILM_8300I_SE, "plustek-opticfilm-8300i-se"),
        (PID_OPTICFILM_8100, "plustek-opticfilm-8100"),
        (PID_OPTICFILM_8100_V2, "plustek-opticfilm-8100-v2"),
        (PID_OPTICFILM_7200I, "plustek-opticfilm-7200i"),
        (PID_OPTICFILM_7200_V2, "plustek-opticfilm-7200-v2"),
        (PID_OPTICFILM_7300, "plustek-opticfilm-7300"),
        (PID_OPTICFILM_7500I, "plustek-opticfilm-7500i"),
        (PID_OPTICFILM_7200, "plustek-opticfilm-7200"),
    ] {
        assert_eq!(name_for(pid, 0), name, "{pid:#06x}");
    }
    assert!(model_for_device(0xFFFF, 0).is_err());
}

#[test]
fn create_asic_routing() {
    let variant = |name: &str| match Asic::create(Box::new(MockScannerTransport::new()), m(name)).unwrap() {
        Asic::Gl845(a) => Some(a.variant),
        _ => None,
    };
    assert_eq!(variant("plustek-opticfilm-8200i"), Some(Gl84xVariant::Gl845));
    assert_eq!(variant("plustek-opticfilm-7400-v2"), Some(Gl84xVariant::Gl845));
    assert_eq!(variant("plustek-opticfilm-7200i"), Some(Gl84xVariant::Gl843));
    assert_eq!(variant("plustek-opticfilm-7200"), Some(Gl84xVariant::Gl842));
}

#[test]
fn scan_ready_validate_set() {
    let ready = ["plustek-opticfilm-8200i-se", "plustek-opticfilm-8100-v2", "plustek-opticfilm-8300i-se", "plustek-opticfilm-7600i-v1", "plustek-opticfilm-7600i-v2"];
    for model in known_models() {
        assert_eq!(model.scan_ready, ready.contains(&model.name), "{} scan_ready", model.model);
    }
}

#[test]
fn geometry_for_each_canonical_model() {
    for name in ["plustek-opticfilm-8200i", "plustek-opticfilm-7400-v2", "plustek-opticfilm-7200i", "plustek-opticfilm-7300", "plustek-opticfilm-7500i", "plustek-opticfilm-7200"] {
        let model = m(name);
        let dpi = *model.resolutions_dpi.last().unwrap(); // typically lowest
        let g = compute_geometry(dpi, &model, None).unwrap();
        assert_eq!(g.resolution, dpi);
        assert!(g.pixels >= 16 && g.lines >= 1, "{name}");
        assert_eq!(Some(&g.register_dpiset), model.register_dpiset_by_dpi.get(&dpi), "{name}");
    }
}

#[test]
fn infrared_caps() {
    for (name, ir) in [
        ("plustek-opticfilm-8200i", true),
        ("plustek-opticfilm-7600i-v2", true),
        ("plustek-opticfilm-7400-v2", false),
        ("plustek-opticfilm-7200i", true),
        ("plustek-opticfilm-7200-v2", false),
        ("plustek-opticfilm-7300", false),
        ("plustek-opticfilm-7500i", true),
        ("plustek-opticfilm-7200", false),
    ] {
        assert_eq!(m(name).supports_infrared, ir, "{name}");
    }
}

#[test]
fn boot_maps_nonempty_for_complete_models() {
    for name in ["plustek-opticfilm-8200i", "plustek-opticfilm-7400-v2", "plustek-opticfilm-8100", "plustek-opticfilm-7200i", "plustek-opticfilm-7300", "plustek-opticfilm-7500i", "plustek-opticfilm-7200"] {
        let boot = m(name).boot_register_map();
        assert!(boot.get(&0x05).is_some(), "{name}");
    }
}

/// Capture-derived overrides in the 8100 V2 must match `04_color_7200.pcapng`.
#[test]
fn v2_8100_capture_derived_constants() {
    let (v2, se) = (m("plustek-opticfilm-8100-v2"), m("plustek-opticfilm-8200i-se"));
    let (g2, gse) = (v2.gl128.as_ref().unwrap(), se.gl128.as_ref().unwrap());
    // feed_to_scan_steps matches SE (both use the TA window top, see #67)
    assert_eq!(g2.feed_to_scan_steps, 13128);
    assert_eq!(gse.feed_to_scan_steps, 13128);
    // Full-frame lperiod at 7200 dpi is the capture value on every model.
    let lp = |m: &FilmModel, dpi| *m.lperiod_by_dpi.as_ref().unwrap().get(&MdKey(None, Some(dpi))).unwrap();
    assert_eq!(lp(&v2, 7200), 16035);
    for dpi in [600, 1200, 1800, 3600, 7200] {
        assert_eq!(lp(&v2, dpi), lp(&se, dpi), "{dpi}");
    }
    // White shading at 7200 dpi uses dummy=0x10 (SE would produce 0x17).
    assert_eq!(v2.shading_strip_clocks(7200, true, None), (0x10, 0x01, 0x01));
    // Dark shading at 7200 dpi uses dummy=0x17 on both V2 and SE.
    assert_eq!(v2.shading_strip_clocks(7200, false, None).0, 0x17);
    assert_eq!(se.shading_strip_clocks(7200, false, None).0, 0x17);
    // V2 full-frame LINCNT at feed2=13128: 29012; SE keeps its 1200 dpi preview value.
    assert_eq!(g2.max_image_lincnt_by_feed2.get(&13128), Some(&29012));
    assert_eq!(gse.max_image_lincnt_by_feed2.get(&13128), Some(&4836));
    // V2 ladder_feed2_steps matches feed_to_scan_steps (top of TA window).
    assert_eq!(g2.ladder_feed2_steps, 13128);
    assert_eq!(gse.ladder_feed2_steps, 13560);
    // Lower-DPI white shading delegates to SE (not the 0x10 override).
    assert_eq!(v2.shading_strip_clocks(1200, true, None).0, se.shading_strip_clocks(1200, true, None).0);
}

// --- test_gl128_firmware_variant.py ---

#[test]
fn init_regs_are_the_current_firmware_set() {
    for name in GL128 {
        assert_eq!(m(name).init_regs.reg(0x36), 0x1C, "{name}");
    }
}

#[test]
fn current_firmware_and_unreadable_serial_keep_model() {
    for name in GL128 {
        let model = m(name);
        for fw in [Some(GL128_FIRMWARE_CURRENT.to_string()), Some(format!(" {GL128_FIRMWARE_CURRENT} ")), None] {
            assert_eq!(gl128_firmware_variant(&model, fw.as_deref()).init_regs, model.init_regs, "{name} {fw:?}");
        }
    }
}

#[test]
fn other_firmware_uses_plain_register_set() {
    for name in GL128 {
        let model = m(name);
        for serial in ["20201119", "20110214", ""] {
            let variant = gl128_firmware_variant(&model, Some(serial));
            assert_eq!(variant.init_regs.reg(0x36), GL128_REG36_LEGACY_FIRMWARE);
            let diff: Vec<u16> = model.init_regs.iter().filter(|(a, v)| variant.init_regs.reg(**a) != **v).map(|(a, _)| *a).collect();
            assert_eq!(diff, [0x36], "{name} {serial:?}");
            assert_eq!(model.init_regs.reg(0x36), 0x1C);
            assert_eq!(variant.kind, model.kind);
        }
    }
}

#[test]
fn non_gl128_model_unchanged() {
    let model = m("plustek-opticfilm-7600i-v1");
    assert_eq!(gl128_firmware_variant(&model, Some("20201119")).init_regs, model.init_regs);
}

#[test]
fn fingerprint_separates_firmware_sets() {
    let model = m("plustek-opticfilm-8300i-se");
    assert_ne!(calib_fingerprint(&model), calib_fingerprint(&gl128_firmware_variant(&model, Some("20201119"))));
}

fn scanner_with_serial(serial: Option<&str>) -> Scanner {
    clock::use_virtual(1000.0);
    let model = m("plustek-opticfilm-8300i-se");
    let info = UsbDeviceInfo {
        vendor_id: VID_PLUSTEK,
        product_id: model.usb_product_id as u16,
        serial_number: serial.map(str::to_owned),
        ..Default::default()
    };
    let dir = common::tmp_path(&format!("firmware-{serial:?}"));
    Scanner::new(Box::new(MockScannerTransport::new()), info, Some(model), Some(dir.join("c.json"))).unwrap()
}

#[test]
fn scanner_applies_firmware_variant() {
    let mut legacy = scanner_with_serial(Some("20201119"));
    assert_eq!(legacy.model().init_regs.reg(0x36), GL128_REG36_LEGACY_FIRMWARE);
    assert_eq!(legacy.asic().unwrap().model().init_regs.reg(0x36), GL128_REG36_LEGACY_FIRMWARE);
    assert_eq!(scanner_with_serial(Some(GL128_FIRMWARE_CURRENT)).model().init_regs.reg(0x36), 0x1C);
    assert_eq!(scanner_with_serial(None).model().init_regs.reg(0x36), 0x1C);
}
