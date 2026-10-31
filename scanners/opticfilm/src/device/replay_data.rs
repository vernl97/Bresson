// SPDX-License-Identifier: GPL-3.0-or-later
//! OpticFilm 7600i v1 (GL843): SilverFast job sequences from USB captures.
//!
//! The profiles stay untyped JSON, as in Python, because the replay code
//! rewrites them op by op (see `scan::replay_gl843_v1`).

use std::sync::OnceLock;

use serde_json::Value;

use crate::error::{Error, Result};

static RAW: &[u8] = include_bytes!("data/opticfilm_7600i_v1.json.gz");

/// The bundled vendor data (decompressed and parsed once).
pub fn replay_data() -> &'static Value {
    static DATA: OnceLock<Value> = OnceLock::new();
    DATA.get_or_init(|| {
        let mut s = String::new();
        std::io::Read::read_to_string(&mut flate2::read::GzDecoder::new(RAW), &mut s).expect("7600i v1 data gzip");
        serde_json::from_str(&s).expect("7600i v1 data json")
    })
}

fn infrared_7200() -> &'static Value {
    static IR: OnceLock<Value> = OnceLock::new();
    IR.get_or_init(|| {
        let p = &replay_data()["profiles"];
        crate::scan::replay_gl843_v1::derive_infrared(&p["color_7200"], &p["color_3600"], &p["infrared_3600"])
            .expect("derive 7200 dpi infrared job")
    })
}

/// A copy of the vendor job for `mode` ("color" / "infrared") at `dpi`;
/// infrared at 7200 dpi is derived from the 7200 dpi colour job (not captured).
pub fn replay_profile(model: &str, mode: &str, dpi: i64) -> Result<Value> {
    let key = format!("{}_{dpi}", if mode == "infrared" { "infrared" } else { "color" });
    if key == "infrared_7200" {
        return Ok(infrared_7200().clone());
    }
    replay_data()["profiles"]
        .get(&key)
        .cloned()
        .ok_or_else(|| Error::Value(format!("{model} has no {mode} sequence at {dpi} dpi")))
}
