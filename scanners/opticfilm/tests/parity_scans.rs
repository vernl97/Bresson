// SPDX-License-Identifier: GPL-3.0-or-later
//! Every mock scan recorded from pyopticfilm replays with byte-identical USB
//! traffic and identical images (see `tools/gen_fixtures.py`).

mod common;

use std::sync::{Arc, Mutex};

use ndarray::{Array2, Array3};
use opticfilm::device::model_by_name;
use opticfilm::error::Error;
use opticfilm::scan::me_exposure::MeLongMode;
use opticfilm::scan::request::{ScanControl, ScanMode, ScanRequest};
use opticfilm::usb::fake::MockScannerTransport;
use opticfilm::usb::fake_gl843_v1::SimulatedGl843V1Transport;
use opticfilm::usb::trace::{UsbOp, UsbTransaction};
use opticfilm::usb::transport::find_transport;
use opticfilm::{Scanner, clock};
use serde_json::{Value, json};

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

fn row(t: &UsbTransaction) -> Value {
    match t.operation {
        UsbOp::ControlRead | UsbOp::ControlWrite => json!([
            t.operation.as_str(),
            t.request_type,
            t.request,
            t.value,
            t.index,
            t.data.as_deref().map(hex),
            t.length
        ]),
        UsbOp::BulkWrite => {
            let d = t.data.as_deref().unwrap_or(&[]);
            json!(["bulk_write", d.len(), common::sha1_hex(d)])
        }
        UsbOp::BulkRead => json!(["bulk_read", t.length]),
    }
}

fn error_name(e: &Error) -> &'static str {
    match e {
        Error::DeviceNotFound(_) => "DeviceNotFoundError",
        Error::UnsupportedDevice(_) => "UnsupportedDeviceError",
        Error::DriverBinding(_) => "DriverBindingError",
        Error::Usb(_) => "UsbError",
        Error::Scan(_) => "ScanError",
        Error::ScannerStuck(_) => "ScannerStuckError",
        Error::ScanCancelled(_) => "ScanCancelled",
        Error::Calibration(_) => "CalibrationError",
        Error::MotorTimeout(_) => "MotorTimeoutError",
        Error::Asic(_) => "AsicError",
        Error::Value(_) => "ValueError",
        Error::NotImplemented(_) => "NotImplementedError",
    }
}

fn sha(a: impl Iterator<Item = u16>) -> String {
    common::sha256_u16(a)
}

/// Max abs difference vs the stored array, when the fixture kept the data.
fn diff_stats(want: &Value, got: &[u16]) -> String {
    let Some(d) = want.get("data").and_then(Value::as_str) else { return "no data kept".into() };
    let w = common::u16s(&common::unzlib_b64(d));
    if w.len() != got.len() {
        return format!("len {} vs {}", got.len(), w.len());
    }
    let mut n = 0;
    let mut maxd = 0i32;
    for (a, b) in got.iter().zip(&w) {
        let dd = (*a as i32 - *b as i32).abs();
        if dd > 0 {
            n += 1;
            maxd = maxd.max(dd);
        }
    }
    format!("{n} of {} samples differ, max |d| = {maxd}", w.len())
}

fn check_array(name: &str, want: &Value, shape: &[usize], data: Vec<u16>, fails: &mut Vec<String>) {
    let ws: Vec<usize> = want["shape"].as_array().unwrap().iter().map(|v| v.as_u64().unwrap() as usize).collect();
    if ws != shape {
        fails.push(format!("{name}: shape {shape:?} vs python {ws:?}"));
        return;
    }
    if sha(data.iter().copied()) != want["sha256"].as_str().unwrap() {
        fails.push(format!("{name}: pixels differ ({})", diff_stats(want, &data)));
    }
}

fn request(step: &Value) -> ScanRequest {
    let b = |k: &str, d: bool| step.get(k).and_then(Value::as_bool).unwrap_or(d);
    let i = |k: &str| step.get(k).and_then(Value::as_i64);
    ScanRequest {
        resolution: i("resolution").unwrap_or(1800),
        mode: step.get("mode").and_then(Value::as_str).map_or(ScanMode::Color, |m| ScanMode::parse(m).unwrap()),
        area: step.get("area").and_then(Value::as_array).map(|a| [0, 1, 2, 3].map(|k| a[k].as_f64().unwrap())),
        apply_calib: b("apply_calib", true),
        multi_exposure: b("multi_exposure", false),
        infrared: b("infrared", false),
        align_passes: b("align_passes", true),
        single_pass_exposure: i("single_pass_exposure"),
        me_short_exposure: i("me_short_exposure"),
        me_long_exposure: i("me_long_exposure"),
        me_long_mode: step.get("me_long_mode").and_then(Value::as_str).map_or(MeLongMode::Adaptive, |m| MeLongMode::parse(m).unwrap()),
        n_passes: i("n_passes").unwrap_or(1),
        ..Default::default()
    }
}

fn run(path: &std::path::Path) -> Vec<String> {
    let fx = common::read_json(path);
    let spec = &fx["spec"];
    let model = model_by_name(spec["model"].as_str().unwrap()).unwrap();
    clock::use_virtual(1000.0);
    let dir = std::env::temp_dir().join(format!("opticfilm-parity-{}-{}", std::process::id(), spec["name"].as_str().unwrap()));
    let _ = std::fs::remove_dir_all(&dir);
    let mut scanner = Scanner::open_fake(model, None, Some(dir.join("calib.json"))).unwrap();
    let mut fails = Vec::new();
    for (k, (step, want)) in spec["steps"].as_array().unwrap().iter().zip(fx["results"].as_array().unwrap()).enumerate() {
        let op = step["op"].as_str().unwrap();
        let tag = format!("step {k} {op}");
        let outcome: Result<(), Error> = match op {
            "warmup" => scanner.warmup(true, true),
            "home" => scanner.home(30.0),
            "calibrate" => {
                let mode = step.get("mode").and_then(Value::as_str).map_or(ScanMode::Color, |m| ScanMode::parse(m).unwrap());
                scanner.calibrate(step["resolution"].as_i64().unwrap(), mode, step.get("force").and_then(Value::as_bool).unwrap_or(false), None, None).map(|e| {
                    let we = &want["entry"];
                    if e.asic_shading != we["asic_shading"].as_bool().unwrap() || e.pixels != we["pixels"].as_i64().unwrap() {
                        fails.push(format!("{tag}: entry differs"));
                    }
                    check_array(&format!("{tag} dark"), &we["dark"], e.dark.shape(), e.dark.iter().copied().collect(), &mut fails);
                    check_array(&format!("{tag} white"), &we["white"], e.white.shape(), e.white.iter().copied().collect(), &mut fails);
                    let blob = e.shading_blob.as_deref().map(common::sha1_hex);
                    if blob.as_deref() != we["blob_sha1"].as_str() {
                        fails.push(format!("{tag}: shading blob differs"));
                    }
                })
            }
            "scan" => {
                let seen: Arc<Mutex<Vec<f64>>> = Arc::default();
                let s2 = seen.clone();
                let mut ctl = ScanControl { progress: Some(Box::new(move |p| s2.lock().unwrap().push(p))), ..Default::default() };
                scanner.scan(&request(step), &mut ctl).map(|img| {
                    let rgb: &Array3<u16> = &img.rgb;
                    check_array(&format!("{tag} rgb"), &want["rgb"], rgb.shape(), rgb.iter().copied().collect(), &mut fails);
                    match (&img.ir, want.get("ir").filter(|v| !v.is_null())) {
                        (Some(ir), Some(w)) => {
                            let ir: &Array2<u16> = ir;
                            check_array(&format!("{tag} ir"), w, ir.shape(), ir.iter().copied().collect(), &mut fails);
                        }
                        (None, None) => {}
                        (a, b) => fails.push(format!("{tag}: ir presence rust={} python={}", a.is_some(), b.is_some())),
                    }
                    if img.dpi != want["dpi"].as_i64().unwrap() || img.device_model != want["device_model"].as_str().unwrap() {
                        fails.push(format!("{tag}: dpi/device_model differ"));
                    }
                    let p = seen.lock().unwrap();
                    if p.len() as i64 != want["progress_n"].as_i64().unwrap() || p.last().copied() != want["progress_last"].as_f64() {
                        fails.push(format!("{tag}: progress {} calls last {:?} vs python {} {:?}", p.len(), p.last(), want["progress_n"], want["progress_last"]));
                    }
                    if let Some(me) = want.get("me") {
                        match scanner.last_me_debug() {
                            None => fails.push(format!("{tag}: no ME debug")),
                            Some(d) => {
                                if d.exposure_long != me["exposure_long"].as_i64().unwrap() || d.exposure_reason.as_deref() != me["exposure_reason"].as_str() {
                                    fails.push(format!("{tag}: ME long {} {:?} vs python {} {}", d.exposure_long, d.exposure_reason, me["exposure_long"], me["exposure_reason"]));
                                }
                            }
                        }
                    }
                })
            }
            other => panic!("unknown op {other}"),
        };
        match (outcome, want.get("error").and_then(Value::as_str)) {
            (Ok(()), None) => {}
            (Ok(()), Some(e)) => fails.push(format!("{tag}: python raised {e}: {}", want["message"])),
            (Err(e), None) => fails.push(format!("{tag}: rust error {e:?}")),
            (Err(e), Some(name)) => {
                if error_name(&e) != name {
                    fails.push(format!("{tag}: rust {e:?} vs python {name}: {}", want["message"]));
                }
            }
        }
    }
    let t = scanner.transport();
    let txns: Vec<Value> = if let Some(m) = find_transport::<MockScannerTransport>(t) {
        m.core.transactions.iter().map(row).collect()
    } else if let Some(s) = find_transport::<SimulatedGl843V1Transport>(t) {
        s.transactions.iter().map(row).collect()
    } else {
        Vec::new()
    };
    let want: &Vec<Value> = fx["transactions"].as_array().unwrap();
    if let Some(i) = (0..txns.len().min(want.len())).find(|&i| txns[i] != want[i]) {
        fails.push(format!("transaction {i} of {}/{} differs: rust {} python {}", txns.len(), want.len(), txns[i], want[i]));
    } else if txns.len() != want.len() {
        fails.push(format!("transaction count rust {} python {}", txns.len(), want.len()));
    }
    drop(scanner);
    let _ = std::fs::remove_dir_all(&dir);
    clock::use_real();
    fails
}

#[test]
fn mock_scans_match_python() {
    let mut paths: Vec<_> = std::fs::read_dir(common::fixtures().join("scans")).unwrap().map(|e| e.unwrap().path()).collect();
    paths.sort();
    let only = std::env::var("PARITY_ONLY").ok();
    let mut all = Vec::new();
    for p in &paths {
        let name = p.file_name().unwrap().to_string_lossy().replace(".json.gz", "");
        if only.as_ref().is_some_and(|o| !o.split(',').any(|x| x == name)) {
            continue;
        }
        let fails = run(p);
        if fails.is_empty() {
            eprintln!("ok   {name}");
        } else {
            eprintln!("FAIL {name}");
            for f in &fails {
                eprintln!("     {f}");
            }
            all.push(name);
        }
    }
    assert!(all.is_empty(), "{} scenario(s) differ from pyopticfilm: {all:?}", all.len());
}
