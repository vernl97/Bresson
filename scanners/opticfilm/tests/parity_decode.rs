// SPDX-License-Identifier: GPL-3.0-or-later
//! Trace JSON, USB log lines and decoded events must match pyopticfilm
//! transaction for transaction (`decode.json.gz`, `tools/gen_fixtures.py`).
//! Python tests these only through Scan Lab; the fixture records a short
//! GL128 and GL845 mock scan.

mod common;

use opticfilm::device::model_by_name;
use opticfilm::scan::request::{ScanControl, ScanRequest};
use opticfilm::usb::decode::{decode_transaction, format_decoded_line};
use opticfilm::usb::fake::MockScannerTransport;
use opticfilm::usb::trace::{RecordingTransport, UsbTransaction, format_usb_log_line};
use opticfilm::usb::transport::find_transport;
use opticfilm::{Scanner, clock};
use serde_json::Value;

fn record(name: &str, dpi: i64) -> Vec<UsbTransaction> {
    clock::use_virtual(1000.0);
    let rec = RecordingTransport::new(Box::new(MockScannerTransport::new()));
    let dir = common::tmp_path(&format!("decode-{name}"));
    let mut scanner = Scanner::open_fake(
        model_by_name(name).unwrap(),
        Some(Box::new(rec)),
        Some(dir.join("c.json")),
    )
    .unwrap();
    let req = ScanRequest {
        resolution: dpi,
        area: Some([0.0, 0.0, 0.08, 0.08]),
        apply_calib: false,
        ..Default::default()
    };
    scanner.scan(&req, &mut ScanControl::default()).unwrap();
    scanner.close();
    std::mem::take(
        &mut find_transport::<RecordingTransport>(scanner.transport())
            .unwrap()
            .transactions,
    )
}

/// The recorder stamps transfers with the wall clock in both ports; drop the stamps.
fn untimed(v: &Value) -> Value {
    let mut v = v.clone();
    if let Some(o) = v.as_object_mut() {
        for k in ["t0", "t1", "latency_ms", "raw_t0", "raw_t1"] {
            o.remove(k);
        }
    }
    v
}

#[test]
fn trace_lines_and_decoded_events_match_python() {
    let fx = common::read_json(&common::fixtures().join("decode.json.gz"));
    for (name, dpi) in [
        ("plustek-opticfilm-8200i-se", 150),
        ("plustek-opticfilm-8200i", 900),
    ] {
        let rows = fx[name].as_array().unwrap();
        let txns = record(name, dpi);
        assert_eq!(txns.len(), rows.len(), "{name}: transaction count");
        for (i, (t, row)) in txns.iter().zip(rows).enumerate() {
            assert_eq!(
                untimed(&t.to_json()),
                untimed(&row["txn"]),
                "{name} #{i}: trace JSON"
            );
            let back = UsbTransaction::from_json(&row["txn"]).unwrap();
            // Like Python, from_json leaves out the recording timestamps.
            assert_eq!(
                back.to_json(),
                untimed(&row["txn"]),
                "{name} #{i}: JSON round trip"
            );
            assert_eq!(
                format_usb_log_line(t, 24),
                row["line"].as_str().unwrap(),
                "{name} #{i}: log line"
            );
            let ev = decode_transaction(t);
            assert_eq!(
                untimed(&ev.as_ref().map(|e| e.to_json()).unwrap_or(Value::Null)),
                untimed(&row["event"]),
                "{name} #{i}: decoded event"
            );
            assert_eq!(
                ev.as_ref().map(format_decoded_line),
                row["decoded"].as_str().map(str::to_owned),
                "{name} #{i}: decoded line"
            );
        }
    }
}
