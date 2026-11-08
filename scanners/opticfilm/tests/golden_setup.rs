// SPDX-License-Identifier: GPL-3.0-or-later
//! Port of `tests/scanners/protocol/test_gl128_setup.py` and
//! `test_8200i_setup.py`: init + configure register programs over the fake
//! USB transports match the committed pyopticfilm goldens.

mod common;

use std::collections::HashMap;

use opticfilm::asic::{Asic, Gl845};
use opticfilm::device::model_by_name;
use opticfilm::device::sensor_lookup::maxwd_register_value;
use opticfilm::scan::geometry::{ScanGeometry, compute_geometry};
use opticfilm::scan::session::ScanSession;
use opticfilm::scan::session_gl128::Gl128ScanSession;
use opticfilm::usb::GenesysUsbProtocol;
use opticfilm::usb::fake::{FakeUsbTransport, MockScannerTransport, NoDevice};
use opticfilm::usb::trace::{UsbOp, UsbTransaction};
use opticfilm::usb::transport::find_transport;
use serde_json::Value;

const GL128_OPTICAL_REGISTER_KEYS: [u16; 22] = [
    0x01, 0x02, 0x2B, 0x2C, 0x2D, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2A, 0x82, 0x83, 0x84, 0x85, 0x86,
    0x87, 0x7D, 0x7E, 0x7F, 0xA5, 0xAB,
];
const GL128_SENSOR_REGISTER_KEYS: [u16; 15] = [
    0x06, 0x0B, 0x1D, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57, 0x5A, 0x5B, 0x70, 0x71, 0x72, 0x73,
];
const OPTICAL_REGISTER_KEYS: [u16; 22] = [
    0x01, 0x02, 0x03, 0x04, 0x05, 0x2C, 0x2D, 0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38,
    0x39, 0x25, 0x26, 0x27, 0x3D, 0x3E,
];

const CASES: [(&str, &str, i64); 9] = [
    ("8200i_se", "plustek-opticfilm-8200i-se", 1200),
    ("8200i_se", "plustek-opticfilm-8200i-se", 1800),
    ("8200i_se", "plustek-opticfilm-8200i-se", 7200),
    ("8100_v2", "plustek-opticfilm-8100-v2", 1200),
    ("8100_v2", "plustek-opticfilm-8100-v2", 1800),
    ("8100_v2", "plustek-opticfilm-8100-v2", 7200),
    ("8300i_se", "plustek-opticfilm-8300i-se", 1200),
    ("8300i_se", "plustek-opticfilm-8300i-se", 1800),
    ("8300i_se", "plustek-opticfilm-8300i-se", 7200),
];

fn load_registers(path: &str) -> (HashMap<u16, u8>, Vec<UsbTransaction>) {
    let raw = common::read_json(&common::fixtures().join("traces").join(path));
    let regs = raw["registers"]
        .as_object()
        .unwrap()
        .iter()
        .map(|(k, v)| {
            (
                u16::from_str_radix(k.trim_start_matches("0x"), 16).unwrap(),
                u8::from_str_radix(v.as_str().unwrap(), 16).unwrap(),
            )
        })
        .collect();
    let txns = raw
        .get("transactions")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .map(|t| UsbTransaction::from_json(t).unwrap())
                .collect()
        })
        .unwrap_or_default();
    (regs, txns)
}

/// ASIC `init` + GL128 configure (motors gated; no bulk image).
fn run_gl128_setup(
    model_name: &str,
    dpi: i64,
) -> (HashMap<u16, u8>, ScanGeometry, opticfilm::FilmModel) {
    let model = model_by_name(model_name).unwrap();
    let Asic::Gl128(mut asic) =
        Asic::create(Box::new(MockScannerTransport::new()), model.clone()).unwrap()
    else {
        panic!()
    };
    asic.motor_moves_enabled = false;
    asic.init(false).unwrap();
    let geo = compute_geometry(dpi, &model, Some([0.0, 0.0, 1.0, 0.12])).unwrap();
    Gl128ScanSession::new(model.clone())
        .configure(&mut asic, &geo)
        .unwrap();
    let usb = find_transport::<MockScannerTransport>(asic.protocol.transport()).unwrap();
    (usb.core.registers.clone(), geo, model)
}

fn u24(r: &HashMap<u16, u8>, a: u16) -> i64 {
    let g = |k| *r.get(&k).unwrap_or(&0) as i64;
    (g(a) << 16) | (g(a + 1) << 8) | g(a + 2)
}

fn u16r(r: &HashMap<u16, u8>, a: u16) -> i64 {
    ((r[&a] as i64) << 8) | r[&(a + 1)] as i64
}

fn compare_registers(
    expected: &HashMap<u16, u8>,
    actual: &HashMap<u16, u8>,
    keys: &[u16],
) -> Option<String> {
    let mut diffs = Vec::new();
    for a in keys {
        match (expected.get(a), actual.get(a)) {
            (Some(e), Some(v)) if e != v => diffs.push(format!(
                "register 0x{a:02x}: expected 0x{e:02x} actual 0x{v:02x}"
            )),
            (Some(e), None) => diffs.push(format!(
                "register 0x{a:02x}: expected 0x{e:02x}, missing in actual"
            )),
            (None, Some(v)) => diffs.push(format!("register 0x{a:02x}: unexpected 0x{v:02x}")),
            _ => {}
        }
    }
    (!diffs.is_empty()).then(|| format!("Register program differs\n{}", diffs.join("\n")))
}

#[test]
fn gl128_setup_optical_registers_match_geometry() {
    for (_, name, dpi) in CASES {
        let (r, geo, model) = run_gl128_setup(name, dpi);
        assert_eq!(u24(&r, 0x25), geo.lincnt_register(), "{name} {dpi}");
        assert_eq!(
            u24(&r, 0x28),
            model.line_period_for(dpi, None),
            "{name} {dpi}"
        );
        assert_eq!(u24(&r, 0x82), geo.pixel_startx, "{name} {dpi}");
        assert_eq!(u24(&r, 0x85), geo.pixel_endx, "{name} {dpi}");
        assert_eq!(u16r(&r, 0x2C), geo.register_dpiset, "{name} {dpi}");
    }
}

#[test]
fn gl128_setup_matches_golden_registers() {
    let keys: Vec<u16> = GL128_OPTICAL_REGISTER_KEYS
        .iter()
        .chain(&GL128_SENSOR_REGISTER_KEYS)
        .copied()
        .collect();
    for (folder, name, dpi) in CASES {
        let (golden, _) = load_registers(&format!("{folder}/{dpi}_rgb16_setup.json"));
        let (written, _, _) = run_gl128_setup(name, dpi);
        assert_eq!(
            compare_registers(&golden, &written, &keys),
            None,
            "{name} {dpi}"
        );
    }
}

/// Final register map from GL845 SET_REGISTER control writes.
fn extract_register_writes(txns: &[UsbTransaction]) -> HashMap<u16, u8> {
    let mut regs = HashMap::new();
    for t in txns
        .iter()
        .filter(|t| t.operation == UsbOp::ControlWrite && t.request == Some(0x04))
    {
        let value = t.value.unwrap_or(0);
        if value & 0xFF != 0x83 {
            continue;
        }
        let high = if value & 0x100 != 0 { 0x100 } else { 0 };
        for p in t.data.as_deref().unwrap_or(&[]).chunks_exact(2) {
            regs.insert(high | p[0] as u16, p[1]);
        }
    }
    regs
}

const STATUS_READ_INDEX: u16 = 0x22 + (0x41 << 8);

/// Keep the first of each consecutive GL845 status (0x41) read run.
fn collapse_status_polls(txns: &[UsbTransaction]) -> Vec<&UsbTransaction> {
    let mut out = Vec::new();
    let mut in_run = false;
    for t in txns {
        let poll = t.operation == UsbOp::ControlRead
            && t.request_type == Some(0xC0)
            && t.request == Some(0x04)
            && t.value.unwrap_or(0) & 0xFF == 0x8E
            && t.index == Some(STATUS_READ_INDEX);
        if poll && in_run {
            continue;
        }
        in_run = poll;
        out.push(t);
    }
    out
}

#[allow(clippy::type_complexity)] // the key mirrors the Python tuple
fn key(
    t: &UsbTransaction,
) -> (
    UsbOp,
    Option<u8>,
    Option<u8>,
    Option<u16>,
    Option<u16>,
    Option<u8>,
    Option<Vec<u8>>,
    Option<usize>,
) {
    (
        t.operation,
        t.request_type,
        t.request,
        t.value,
        t.index,
        t.endpoint,
        t.data.clone(),
        t.length,
    )
}

fn run_8200i_setup() -> (Vec<UsbTransaction>, ScanGeometry) {
    let model = model_by_name("plustek-opticfilm-8200i").unwrap();
    let mut asic = Gl845::new(
        GenesysUsbProtocol::new(Box::new(FakeUsbTransport::<NoDevice>::new())),
        model.clone(),
        opticfilm::asic::Gl84xVariant::Gl845,
    );
    asic.init(false).unwrap();
    let geo = compute_geometry(1800, &model, None).unwrap();
    ScanSession::new(model, asic.variant)
        .configure(&mut asic, &geo, false)
        .unwrap();
    let usb = find_transport::<FakeUsbTransport>(asic.protocol.transport()).unwrap();
    (usb.core.transactions.clone(), geo)
}

#[test]
fn gl845_8200i_setup_optical_registers_match_geometry() {
    let (txns, geo) = run_8200i_setup();
    let r = extract_register_writes(&txns);
    let model = model_by_name("plustek-opticfilm-8200i").unwrap();
    assert_eq!(u16r(&r, 0x2C), geo.register_dpiset);
    assert_eq!(u16r(&r, 0x30), geo.pixel_startx);
    assert_eq!(u16r(&r, 0x32), geo.pixel_endx);
    assert_eq!(u16r(&r, 0x38), geo.exposure_lperiod);
    assert_eq!(r[&0x34] as i64, geo.dummy_pixel);
    assert_eq!(
        u24(&r, 0x35),
        maxwd_register_value(&model, geo.line_bytes, geo.channels)
    );
    assert_eq!(u24(&r, 0x25), geo.lincnt_register());
    assert!(r[&0x01] & 0x02 != 0);
    assert!(r[&0x01] & 0x20 == 0);
    assert!(r[&0x01] & 0x01 == 0);
}

#[test]
fn gl845_8200i_setup_matches_golden_usb_trace() {
    let (golden_regs, golden) = load_registers("8200i/1800_rgb16_setup.json");
    let (txns, _) = run_8200i_setup();
    let (l, r) = (collapse_status_polls(&golden), collapse_status_polls(&txns));
    for (i, (a, b)) in l.iter().zip(&r).enumerate() {
        assert_eq!(key(a), key(b), "transaction {i} differs");
    }
    assert_eq!(l.len(), r.len(), "trace length");
    let written = extract_register_writes(&txns);
    assert_eq!(
        compare_registers(&golden_regs, &written, &OPTICAL_REGISTER_KEYS),
        None
    );
}

#[test]
fn fake_transport_records_control_and_bulk() {
    let (txns, geo) = run_8200i_setup();
    assert!(txns.iter().any(|t| t.operation == UsbOp::ControlWrite));
    assert!(txns.iter().any(|t| t.operation == UsbOp::BulkWrite));
    assert_eq!(geo.resolution, 1800);
}
