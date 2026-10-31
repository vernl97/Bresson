// SPDX-License-Identifier: GPL-3.0-or-later
//! Shared helpers for the parity tests against pyopticfilm fixtures.

#![allow(dead_code)]

use std::io::Read;
use std::path::{Path, PathBuf};

use base64::Engine;
use serde_json::Value;

pub fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("fixtures")
}

pub fn read_json(path: &Path) -> Value {
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let text = if path.extension().is_some_and(|e| e == "gz") {
        let mut s = String::new();
        flate2::read::GzDecoder::new(&bytes[..]).read_to_string(&mut s).unwrap();
        s
    } else {
        String::from_utf8(bytes).unwrap()
    };
    serde_json::from_str(&text).unwrap()
}

pub fn unzlib_b64(s: &str) -> Vec<u8> {
    let z = base64::engine::general_purpose::STANDARD.decode(s).unwrap();
    let mut out = Vec::new();
    flate2::read::ZlibDecoder::new(&z[..]).read_to_end(&mut out).unwrap();
    out
}

pub fn u16s(b: &[u8]) -> Vec<u16> {
    b.chunks_exact(2).map(|p| u16::from_le_bytes([p[0], p[1]])).collect()
}

pub fn sha256_u16(v: impl IntoIterator<Item = u16>) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    for x in v {
        h.update(x.to_le_bytes());
    }
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

pub fn sha1_hex(b: &[u8]) -> String {
    use sha1::{Digest, Sha1};
    Sha1::digest(b).iter().map(|x| format!("{x:02x}")).collect()
}

/// The fixture generator's deterministic integer-hash noise in [-1, 1).
pub fn noise(n: usize, seed: u64) -> Vec<f64> {
    (0..n as u64)
        .map(|i| {
            let i = i.wrapping_add(seed.wrapping_mul(0x9E3779B97F4A7C15));
            let mut x = i.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            x ^= x >> 33;
            x = x.wrapping_mul(0xFF51AFD7ED558CCD);
            x ^= x >> 33;
            (x >> 11) as f64 / (1u64 << 53) as f64 * 2.0 - 1.0
        })
        .collect()
}
