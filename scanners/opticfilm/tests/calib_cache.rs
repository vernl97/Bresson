// SPDX-License-Identifier: GPL-3.0-or-later
//! Port of `test_calib_cache_corruption.py`, `test_calib_cache_load.py` and
//! `test_calib_fingerprint.py`: a damaged cache never stops the scanner from
//! opening, saves are atomic, and entries are only reused on the model and
//! timing they were measured with, within one session.
//!
//! `test_calib_cache_isolation.py` checks a pytest fixture; no Rust test opens
//! the default cache path (every one passes its own file).

mod common;

use std::path::Path;

use common::{caplog, tmp_path};
use ndarray::Array2;
use opticfilm::device::{FilmModel, ScanMethod, model_by_name};
use opticfilm::scan::calibrate::{
    CACHE_IDENT, CACHE_VERSION, CalibCache, CalibEntry, Calibrator, calib_fingerprint,
};
use opticfilm::{Scanner, clock};
use serde_json::{Value, json};

fn m(name: &str) -> FilmModel {
    model_by_name(name).unwrap()
}

const SE: &str = "plustek-opticfilm-8200i-se";
const V2: &str = "plustek-opticfilm-8100-v2";
const S83: &str = "plustek-opticfilm-8300i-se";

fn write(path: &Path, payload: &Value) {
    std::fs::write(path, serde_json::to_string(payload).unwrap()).unwrap();
}

fn file(entries: Vec<Value>) -> Value {
    json!({"ident": CACHE_IDENT, "version": CACHE_VERSION, "entries": entries})
}

fn plain(resolution: i64, pixels: usize) -> CalibEntry {
    CalibEntry::new(
        ScanMethod::Transparency,
        resolution,
        0,
        pixels as i64,
        Array2::zeros((pixels, 3)),
        Array2::from_elem((pixels, 3), 65535),
    )
}

fn good(resolution: i64) -> Value {
    plain(resolution, 4).to_json()
}

fn with(mut v: Value, key: &str, val: Value) -> Value {
    v[key] = val;
    v
}

// --- test_calib_cache_corruption.py ---

#[test]
fn load_with_corrupt_json_treated_as_missing() {
    let path = tmp_path("cache-corrupt").join("calib.json");
    std::fs::write(&path, "{not valid json").unwrap();
    let mut cache = CalibCache::new(Some(path), None, false);
    assert!(!cache.load());
    assert!(cache.entries.is_empty());
}

#[test]
fn calibrator_construction_survives_corrupt_cache() {
    let path = tmp_path("cache-corrupt-cal").join("calib.json");
    std::fs::write(&path, "{not valid json").unwrap();
    let cal = Calibrator::new(m("plustek-opticfilm-8200i"), Some(path), false);
    assert!(cal.cache.entries.is_empty());
}

#[test]
fn save_is_atomic_no_leftover_tmp_file() {
    let path = tmp_path("cache-atomic").join("calib.json");
    CalibCache::new(Some(path.clone()), None, false)
        .save()
        .unwrap();
    assert!(path.exists());
    assert!(!path.with_extension("tmp").exists());
}

/// A leftover partial tmp file from a crash mid-write is not mistaken for the cache.
#[test]
fn save_survives_crash_mid_write_leaving_old_file_intact() {
    let path = tmp_path("cache-crash").join("calib.json");
    CalibCache::new(Some(path.clone()), None, false)
        .save()
        .unwrap();
    let good_contents = std::fs::read_to_string(&path).unwrap();
    std::fs::write(path.with_extension("tmp"), "{truncat").unwrap();
    assert_eq!(std::fs::read_to_string(&path).unwrap(), good_contents);
    assert!(CalibCache::new(Some(path), None, false).load());
}

// --- test_calib_cache_load.py ---

fn damaged() -> Vec<Value> {
    let mut missing = good(1800);
    missing.as_object_mut().unwrap().remove("pixels");
    vec![
        with(good(1800), "dark", json!(vec![0; 11])), // wrong length: reshape fails
        missing,                                      // missing key
        with(good(1800), "shading_blob_b64", json!("not base64!")), // bad base64
        with(good(1800), "pixels", json!("four")),    // wrong type
        json!(["not", "an", "entry"]),
        Value::Null,
    ]
}

#[test]
fn damaged_entries_are_skipped_and_good_ones_kept() {
    let path = tmp_path("cache-damaged").join("calib.json");
    let bad = damaged();
    let n_bad = bad.len();
    write(
        &path,
        &file([vec![good(1800)], bad, vec![good(3600)]].concat()),
    );
    let mut cache = CalibCache::new(Some(path.clone()), None, false);
    caplog::start();
    assert!(cache.load());
    let mut res: Vec<i64> = cache.entries.iter().map(|e| e.resolution).collect();
    res.sort();
    assert_eq!(res, [1800, 3600]);
    for i in 1..=n_bad {
        assert!(
            caplog::contains(
                log::Level::Warn,
                &format!("Skipping damaged calib cache entry {i} ")
            ),
            "entry {i}"
        );
    }

    cache.save().unwrap(); // the damaged entries are not written back
    let saved: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(saved["entries"].as_array().unwrap().len(), 2);
}

#[test]
fn malformed_file_is_ignored() {
    let payloads = [
        json!(["a", "list"]),
        json!(42),
        json!({"ident": CACHE_IDENT, "version": "not a number", "entries": []}),
        json!({"ident": CACHE_IDENT, "version": CACHE_VERSION, "entries": {"not": "a list"}}),
    ];
    for (i, payload) in payloads.iter().enumerate() {
        let path = tmp_path(&format!("cache-malformed-{i}")).join("calib.json");
        write(&path, payload);
        let mut cache = CalibCache::new(Some(path), None, false);
        assert!(!cache.load(), "{payload}");
        assert!(cache.entries.is_empty());
    }
}

/// The reported repro: one entry whose dark length does not match pixels.
#[test]
fn scanner_opens_with_a_damaged_cache() {
    clock::use_virtual(1000.0);
    let path = tmp_path("cache-scanner").join("calib.json");
    write(
        &path,
        &file(vec![with(
            with(good(1800), "dark", json!(vec![0; 29])),
            "pixels",
            json!(10),
        )]),
    );
    let mut scanner = Scanner::open_fake(m(SE), None, Some(path)).unwrap();
    assert!(scanner.calibrator().unwrap().cache.entries.is_empty());
    scanner.close();
}

// --- test_calib_fingerprint.py ---

fn entry(blob: &[u8]) -> CalibEntry {
    let mut e = plain(1800, 8);
    e.asic_shading = true;
    e.shading_blob = Some(blob.to_vec());
    e.afe_gains = Some([38, 30, 36]);
    e.afe_offsets = Some([19, 31, 23]);
    e
}

fn find(cache: &CalibCache) -> Option<&[u8]> {
    cache
        .find(ScanMethod::Transparency, 1800, 0, 8)
        .map(|e| e.shading_blob.as_deref().unwrap())
}

/// The 8300i SE with the 8200i SE's sensor timing.
fn old_timing() -> FilmModel {
    let mut model = m(S83);
    model.sensor_custom_regs = m(SE).sensor_custom_regs.clone();
    model
}

#[test]
fn fingerprint_tracks_model_exposure_and_sensor_regs() {
    let prints: std::collections::HashSet<String> = [SE, V2, S83]
        .iter()
        .map(|n| calib_fingerprint(&m(n)))
        .collect();
    assert_eq!(prints.len(), 3);
    assert_eq!(
        calib_fingerprint(&m(S83)),
        calib_fingerprint(&m(S83).clone())
    );
    let mut exposure = m(S83);
    exposure.gl128.as_mut().unwrap().exposure_short = 14000;
    assert_ne!(calib_fingerprint(&m(S83)), calib_fingerprint(&exposure));
    assert_ne!(calib_fingerprint(&m(S83)), calib_fingerprint(&old_timing()));
}

#[test]
fn entries_from_other_timing_are_ignored_but_kept() {
    let path = tmp_path("fp-timing").join("calib.json");
    let mut stale = Calibrator::new(old_timing(), Some(path.clone()), false);
    stale.cache.upsert(entry(b"OLD"));
    stale.cache.save().unwrap();

    let mut current = Calibrator::new(m(S83), Some(path.clone()), true);
    assert!(find(&current.cache).is_none());

    current.cache.upsert(entry(b"NEW"));
    current.cache.save().unwrap();
    let reloaded = Calibrator::new(m(S83), Some(path.clone()), true);
    assert_eq!(find(&reloaded.cache), Some(&b"NEW"[..]));
    let mut blobs: Vec<&[u8]> = reloaded
        .cache
        .entries
        .iter()
        .map(|e| e.shading_blob.as_deref().unwrap())
        .collect();
    blobs.sort();
    assert_eq!(blobs, [&b"NEW"[..], &b"OLD"[..]]);
    assert_eq!(
        find(&Calibrator::new(old_timing(), Some(path), true).cache),
        Some(&b"OLD"[..])
    );
}

#[test]
fn pre_fingerprint_cache_file_is_ignored() {
    let path = tmp_path("fp-legacy").join("calib.json");
    let mut legacy = entry(b"LEGACY").to_json();
    legacy.as_object_mut().unwrap().remove("fingerprint");
    write(
        &path,
        &json!({"ident": CACHE_IDENT, "version": 10, "entries": [legacy]}),
    );
    let cal = Calibrator::new(m(SE), Some(path), false);
    assert!(cal.cache.entries.is_empty());
}

/// GL128 units share one USB serial and drift between sessions: measure once per session.
#[test]
fn saved_entries_are_not_reused_by_a_new_session() {
    let path = tmp_path("fp-session").join("calib.json");
    let mut first = Calibrator::new(m(SE), Some(path.clone()), false);
    first.cache.upsert(entry(b"SESSION1"));
    first.cache.save().unwrap();
    assert_eq!(find(&first.cache), Some(&b"SESSION1"[..])); // same session: reused

    let mut second = Calibrator::new(m(SE), Some(path.clone()), false);
    assert!(find(&second.cache).is_none());
    second.cache.upsert(entry(b"SESSION2"));
    second.cache.save().unwrap();
    assert_eq!(find(&second.cache), Some(&b"SESSION2"[..]));
    // Replaced, not appended: the file keeps one entry per window.
    let reloaded = Calibrator::new(m(SE), Some(path), true);
    let blobs: Vec<&[u8]> = reloaded
        .cache
        .entries
        .iter()
        .map(|e| e.shading_blob.as_deref().unwrap())
        .collect();
    assert_eq!(blobs, [&b"SESSION2"[..]]);
}
