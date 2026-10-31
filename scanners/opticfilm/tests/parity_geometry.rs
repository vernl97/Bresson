// SPDX-License-Identifier: GPL-3.0-or-later
//! Scan geometry and calibration fingerprints match pyopticfilm for every model.

mod common;

use opticfilm::device::model_by_name;
use opticfilm::scan::calibrate::calib_fingerprint;
use opticfilm::scan::geometry::{ScanGeometry, compute_calib_geometry, compute_geometry};
use serde_json::{Value, json};

fn as_json(g: &ScanGeometry) -> Value {
    json!({
        "resolution": g.resolution, "pixels": g.pixels, "lines": g.lines, "startx": g.startx, "starty": g.starty,
        "pixel_startx": g.pixel_startx, "pixel_endx": g.pixel_endx, "optical_pixels": g.optical_pixels,
        "register_dpiset": g.register_dpiset, "output_pixel_offset": g.output_pixel_offset,
        "shift_r": g.shift_r, "shift_g": g.shift_g, "shift_b": g.shift_b,
        "optical_line_count": g.optical_line_count, "line_bytes": g.line_bytes, "stagger_y": g.stagger_y,
        "num_staggered_lines": g.num_staggered_lines, "channels": g.channels, "depth": g.depth,
        "exposure_lperiod": g.exposure_lperiod, "dummy_pixel": g.dummy_pixel,
        "disable_buffer_full_move": g.disable_buffer_full_move, "register_lincnt": g.register_lincnt,
        "lincnt_per_line": g.lincnt_per_line, "y_oversample": g.y_oversample, "area_y1": g.area_y1,
        "is_default_full_frame": g.is_default_full_frame, "area": g.area, "host_downsample": g.host_downsample,
        "usb_end_drop": g.usb_end_drop, "frame_lines": g.frame_lines,
    })
}

fn num_eq(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => {
            (x.as_f64().unwrap() - y.as_f64().unwrap()).abs() < 1e-9
        }
        (Value::Array(x), Value::Array(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(p, q)| num_eq(p, q))
        }
        _ => a == b,
    }
}

#[test]
fn geometry_matches_python_for_every_model_and_dpi() {
    let rows = common::read_json(&common::fixtures().join("geometry.json"));
    let mut checked = 0;
    for row in rows.as_array().unwrap() {
        let model = model_by_name(row["model"].as_str().unwrap()).unwrap();
        let dpi = row["dpi"].as_i64().unwrap();
        let got = if row.get("calib").is_some() {
            compute_calib_geometry(dpi, &model, None)
        } else {
            let area = row["area"]
                .as_array()
                .map(|a| [0, 1, 2, 3].map(|i| a[i].as_f64().unwrap()));
            compute_geometry(dpi, &model, area)
        };
        let ctx = format!("{} {dpi} {:?}", model.name, row.get("area"));
        match (got, row.get("error")) {
            (Ok(_), Some(e)) => panic!("{ctx}: python raised {e}, rust did not"),
            (Err(e), None) => panic!("{ctx}: rust error {e}"),
            (Err(_), Some(_)) => {}
            (Ok(g), None) => {
                let want = &row["geometry"];
                let got = as_json(&g);
                for (k, v) in want.as_object().unwrap() {
                    assert!(
                        num_eq(&got[k], v),
                        "{ctx}: field {k}: rust {} python {v}",
                        got[k]
                    );
                }
            }
        }
        checked += 1;
    }
    assert!(checked > 400);
}

#[test]
fn calibration_fingerprints_match_python() {
    let fps = common::read_json(&common::fixtures().join("fingerprints.json"));
    for (name, fp) in fps.as_object().unwrap() {
        let model = model_by_name(name).unwrap();
        assert_eq!(calib_fingerprint(&model), fp.as_str().unwrap(), "{name}");
    }
}
