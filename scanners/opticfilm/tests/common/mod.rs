// SPDX-License-Identifier: GPL-3.0-or-later
//! Shared helpers for the parity tests against pyopticfilm fixtures.

#![allow(dead_code)]

use std::io::Read;
use std::path::{Path, PathBuf};

use base64::Engine;
use serde_json::Value;

pub fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

pub fn read_json(path: &Path) -> Value {
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let text = if path.extension().is_some_and(|e| e == "gz") {
        let mut s = String::new();
        flate2::read::GzDecoder::new(&bytes[..])
            .read_to_string(&mut s)
            .unwrap();
        s
    } else {
        String::from_utf8(bytes).unwrap()
    };
    serde_json::from_str(&text).unwrap()
}

pub fn unzlib_b64(s: &str) -> Vec<u8> {
    let z = base64::engine::general_purpose::STANDARD.decode(s).unwrap();
    let mut out = Vec::new();
    flate2::read::ZlibDecoder::new(&z[..])
        .read_to_end(&mut out)
        .unwrap();
    out
}

pub fn u16s(b: &[u8]) -> Vec<u16> {
    b.chunks_exact(2)
        .map(|p| u16::from_le_bytes([p[0], p[1]]))
        .collect()
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
            let mut x = i
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            x ^= x >> 33;
            x = x.wrapping_mul(0xFF51AFD7ED558CCD);
            x ^= x >> 33;
            (x >> 11) as f64 / (1u64 << 53) as f64 * 2.0 - 1.0
        })
        .collect()
}

/// Log records emitted on the current thread (the `caplog` fixture).
pub mod caplog {
    use std::cell::RefCell;
    use std::sync::Once;

    thread_local! {
        static RECORDS: RefCell<Vec<(log::Level, String)>> = const { RefCell::new(Vec::new()) };
    }

    struct Capture;

    impl log::Log for Capture {
        fn enabled(&self, _: &log::Metadata) -> bool {
            true
        }
        fn log(&self, record: &log::Record) {
            RECORDS.with(|r| {
                r.borrow_mut()
                    .push((record.level(), record.args().to_string()))
            });
        }
        fn flush(&self) {}
    }

    /// Start capturing on this thread, dropping anything captured before.
    pub fn start() {
        static INIT: Once = Once::new();
        INIT.call_once(|| {
            static CAPTURE: Capture = Capture;
            log::set_logger(&CAPTURE).unwrap();
            log::set_max_level(log::LevelFilter::Trace);
        });
        RECORDS.with(|r| r.borrow_mut().clear());
    }

    /// True when a record at `level` or more severe contains `needle`.
    pub fn contains(level: log::Level, needle: &str) -> bool {
        RECORDS.with(|r| {
            r.borrow()
                .iter()
                .any(|(l, m)| *l <= level && m.contains(needle))
        })
    }
}

/// A fresh, empty per-test directory (the `tmp_path` fixture).
pub fn tmp_path(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("opticfilm-test-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}
