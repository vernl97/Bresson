// SPDX-License-Identifier: GPL-3.0-or-later
//! Dark / white / shading calibration and on-disk cache.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use base64::Engine;
use log::{info, warn};
use ndarray::{Array2, Axis};
use serde_json::{Value, json};
use sha1::{Digest, Sha1};

use super::calib_gl128::{AFE_GAIN_MAX, AfeFrontend, afe_timing_matches};
use super::geometry::{Area, MM_PER_INCH, ScanGeometry, compute_calib_geometry, compute_geometry};
use super::pipeline::ImagePipeline;
use super::request::{NoProgress, ScanMode};
use super::session::ScanSession;
use crate::asic::{Asic, Gl128, Gl845};
use crate::clock::sleep;
use crate::device::model::{FilmModel, ScanMethod};
use crate::device::sensor_lookup::{dummy_pixel_for, exposure_lperiod_for};
use crate::error::{Error, Result};
use crate::num::{U16Hist, py_round};

pub const CACHE_IDENT: &str = "pyopticfilm";
/// Same version as the Python cache so either driver can read the other's file.
pub const CACHE_VERSION: i64 = 22;
pub const DARK_SETTLE_S: f64 = 0.2;
pub const WHITE_SETTLE_S: f64 = 0.5;
/// Reject host calib when dark is this bright relative to white (lamp leak).
pub const DARK_WHITE_MAX_RATIO: f64 = 0.5;
pub const AFE_WHITE_GAIN_ROUNDS: usize = 4;
pub const AFE_WHITE_SATURATED: f64 = 64000.0;
pub const AFE_WHITE_TOLERANCE: f64 = 0.03;

/// AD9826 PGA gain for a 6-bit code (`6 / (6 - 5 * code / 63)`).
pub fn pga_gain(code: i64) -> f64 {
    6.0 / (6.0 - 5.0 * code as f64 / 63.0)
}

/// User-facing CalibrationError text mapped from ASIC reject reason.
pub fn colour_shading_failure_message(reason: &str) -> String {
    let text = if reason.trim().is_empty() {
        "ASIC shading validation failed".to_string()
    } else {
        reason.trim().to_string()
    };
    let lower = text.to_lowercase();
    if text.contains("white≈dark")
        || (lower.contains("white") && lower.contains("dark") && text.contains('≈'))
    {
        return format!(
            "Colour ASIC shading failed ({text}). Lamp-on white did not rise above dark — check illumination and retry."
        );
    }
    if lower.contains("dark mean") || lower.contains("lamp-off") {
        return format!(
            "Colour ASIC shading failed ({text}). AFE black level or illumination failed — retry; power-cycle if it persists."
        );
    }
    if lower.contains("film") {
        return format!(
            "Colour ASIC shading failed ({text}). Head is not on the clear home field — ensure the carriage can park at home, then retry."
        );
    }
    if lower.contains("home") || lower.contains("park") {
        return format!(
            "Colour ASIC shading failed ({text}). Ensure the carriage can park at home, then retry the scan."
        );
    }
    format!(
        "Colour ASIC shading failed ({text}). Retry the scan; power-cycle the scanner if it persists."
    )
}

/// `dirs::cache_dir()/opticfilm/calib_v2.json`.
pub fn default_cache_path() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("opticfilm")
        .join("calib_v2.json")
}

/// Python `json.dumps` of an int-or-None list.
fn py_list(v: &[Option<i64>]) -> String {
    let items: Vec<String> = v
        .iter()
        .map(|x| x.map_or("null".to_string(), |n| n.to_string()))
        .collect();
    format!("[{}]", items.join(", "))
}

fn py_pairs(mut v: Vec<(i64, i64)>) -> String {
    v.sort_unstable();
    let items: Vec<String> = v.iter().map(|(a, b)| format!("[{a}, {b}]")).collect();
    format!("[{}]", items.join(", "))
}

/// Hash of what a cached table depends on besides the scan window.
///
/// Byte-compatible with the Python `calib_fingerprint` (`json.dumps(sort_keys=True)`).
pub fn calib_fingerprint(model: &FilmModel) -> String {
    let g = model.gl128.as_deref();
    let pairs = |m: &crate::device::RegMap| {
        m.iter()
            .map(|(&a, &v)| (a as i64, v as i64))
            .collect::<Vec<_>>()
    };
    let text = format!(
        "{{\"clock\": {}, \"exposure\": {}, \"init\": {}, \"name\": {}, \"sensor\": {}}}",
        py_list(&[
            g.map(|g| g.asic_clock_boot_0x8c),
            g.map(|g| g.asic_clock_scan_0x8c)
        ]),
        py_list(&[Some(model.exposure_lperiod), g.map(|g| g.exposure_short)]),
        py_pairs(pairs(&model.init_regs)),
        serde_json::to_string(model.name).expect("name"),
        py_pairs(pairs(&model.sensor_custom_regs)),
    );
    let digest = Sha1::digest(text.as_bytes());
    digest
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>()[..16]
        .to_string()
}

fn now_s() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0.0, |d| d.as_secs_f64())
}

/// One shading result keyed like the SANE genesys cache.
#[derive(Debug, Clone, PartialEq)]
pub struct CalibEntry {
    pub method: ScanMethod,
    pub resolution: i64,
    pub startx: i64,
    pub pixels: i64,
    /// `(pixels, 3)` dark columns.
    pub dark: Array2<u16>,
    /// `(pixels, 3)` white columns.
    pub white: Array2<u16>,
    pub calibrated_at: f64,
    /// True when produced by GL128 ASIC shading (skip host stretch).
    pub asic_shading: bool,
    pub shading_blob: Option<Vec<u8>>,
    pub afe_offsets: Option<[i64; 3]>,
    pub afe_gains: Option<[i64; 3]>,
    pub fingerprint: String,
    /// Loaded from the file rather than measured this session.
    saved: bool,
}

impl CalibEntry {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        method: ScanMethod,
        resolution: i64,
        startx: i64,
        pixels: i64,
        dark: Array2<u16>,
        white: Array2<u16>,
    ) -> Self {
        Self {
            method,
            resolution,
            startx,
            pixels,
            dark,
            white,
            calibrated_at: now_s(),
            asic_shading: false,
            shading_blob: None,
            afe_offsets: None,
            afe_gains: None,
            fingerprint: String::new(),
            saved: false,
        }
    }

    pub fn has_asic_blob(&self) -> bool {
        self.asic_shading
            && self.shading_blob.as_ref().is_some_and(|b| !b.is_empty())
            && self.afe_offsets.is_some()
            && self.afe_gains.is_some()
    }

    pub fn matches(&self, method: ScanMethod, resolution: i64, startx: i64, pixels: i64) -> bool {
        self.method == method
            && self.resolution == resolution
            && self.startx == startx
            && self.pixels == pixels
    }

    pub fn to_json(&self) -> Value {
        let mut m = serde_json::Map::new();
        m.insert("method".into(), json!(self.method.as_str()));
        m.insert("resolution".into(), json!(self.resolution));
        m.insert("startx".into(), json!(self.startx));
        m.insert("pixels".into(), json!(self.pixels));
        m.insert("calibrated_at".into(), json!(self.calibrated_at));
        m.insert("asic_shading".into(), json!(self.asic_shading));
        m.insert("fingerprint".into(), json!(self.fingerprint));
        m.insert(
            "dark".into(),
            json!(self.dark.iter().copied().collect::<Vec<u16>>()),
        );
        m.insert(
            "white".into(),
            json!(self.white.iter().copied().collect::<Vec<u16>>()),
        );
        if let Some(b) = &self.shading_blob {
            m.insert(
                "shading_blob_b64".into(),
                json!(base64::engine::general_purpose::STANDARD.encode(b)),
            );
        }
        if let Some(o) = self.afe_offsets {
            m.insert("afe_offsets".into(), json!(o));
        }
        if let Some(g) = self.afe_gains {
            m.insert("afe_gains".into(), json!(g));
        }
        Value::Object(m)
    }

    pub fn from_json(d: &Value) -> Result<Self> {
        let bad = |what: &str| Error::Value(format!("calib entry: bad {what}"));
        let int = |k: &str| d.get(k).and_then(Value::as_i64).ok_or_else(|| bad(k));
        let pixels = int("pixels")?;
        let plane = |k: &str| -> Result<Array2<u16>> {
            let v: Vec<u16> = d
                .get(k)
                .and_then(Value::as_array)
                .ok_or_else(|| bad(k))?
                .iter()
                .map(|x| x.as_u64().map(|n| n as u16).ok_or_else(|| bad(k)))
                .collect::<Result<_>>()?;
            Array2::from_shape_vec((pixels as usize, 3), v).map_err(|_| bad(k))
        };
        let triple = |k: &str| -> Result<Option<[i64; 3]>> {
            match d.get(k) {
                None | Some(Value::Null) => Ok(None),
                Some(v) => {
                    let a = v.as_array().ok_or_else(|| bad(k))?;
                    if a.len() != 3 {
                        return Err(bad(k));
                    }
                    Ok(Some([0, 1, 2].map(|i| a[i].as_i64().unwrap_or(0))))
                }
            }
        };
        let blob = match d.get("shading_blob_b64").and_then(Value::as_str) {
            Some(s) if !s.is_empty() => Some(
                base64::engine::general_purpose::STANDARD
                    .decode(s)
                    .map_err(|_| bad("shading_blob_b64"))?,
            ),
            _ => None,
        };
        Ok(Self {
            method: ScanMethod::parse(
                d.get("method")
                    .and_then(Value::as_str)
                    .ok_or_else(|| bad("method"))?,
            )?,
            resolution: int("resolution")?,
            startx: int("startx")?,
            pixels,
            dark: plane("dark")?,
            white: plane("white")?,
            calibrated_at: d
                .get("calibrated_at")
                .and_then(Value::as_f64)
                .unwrap_or(0.0),
            asic_shading: d
                .get("asic_shading")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            shading_blob: blob,
            afe_offsets: triple("afe_offsets")?,
            afe_gains: triple("afe_gains")?,
            fingerprint: d
                .get("fingerprint")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            saved: false,
        })
    }
}

/// Versioned JSON cache of shading entries.
///
/// With a `fingerprint`, lookups only see entries stamped with it. Unless
/// `reuse_saved` is set, entries loaded from the file are kept for the file
/// but never returned by [`CalibCache::find`]: each session measures once per window.
pub struct CalibCache {
    pub path: PathBuf,
    pub fingerprint: Option<String>,
    pub reuse_saved: bool,
    pub entries: Vec<CalibEntry>,
}

impl CalibCache {
    pub fn new(path: Option<PathBuf>, fingerprint: Option<String>, reuse_saved: bool) -> Self {
        Self {
            path: path.unwrap_or_else(default_cache_path),
            fingerprint,
            reuse_saved,
            entries: Vec::new(),
        }
    }

    fn same_model(&self, e: &CalibEntry) -> bool {
        self.fingerprint
            .as_ref()
            .is_none_or(|f| &e.fingerprint == f)
    }

    fn visible(&self, e: &CalibEntry) -> bool {
        self.same_model(e) && (self.reuse_saved || !e.saved)
    }

    pub fn load(&mut self) -> bool {
        self.entries.clear();
        if !self.path.exists() {
            return false;
        }
        let raw: Value = match std::fs::read_to_string(&self.path)
            .map_err(|e| e.to_string())
            .and_then(|t| serde_json::from_str(&t).map_err(|e| e.to_string()))
        {
            Ok(v) => v,
            Err(e) => {
                warn!(
                    "Ignoring unreadable calib cache at {}: {e}",
                    self.path.display()
                );
                return false;
            }
        };
        if raw.get("ident").and_then(Value::as_str) != Some(CACHE_IDENT)
            || raw.get("version").and_then(Value::as_i64) != Some(CACHE_VERSION)
        {
            warn!(
                "Ignoring incompatible calib cache at {}",
                self.path.display()
            );
            return false;
        }
        let Some(entries) = raw.get("entries").and_then(Value::as_array) else {
            warn!(
                "Ignoring calib cache at {}: 'entries' is not a list",
                self.path.display()
            );
            return false;
        };
        for (index, d) in entries.iter().enumerate() {
            match CalibEntry::from_json(d) {
                Ok(mut e) => {
                    e.saved = true;
                    self.entries.push(e);
                }
                Err(e) => warn!(
                    "Skipping damaged calib cache entry {index} in {}: {e}",
                    self.path.display()
                ),
            }
        }
        info!(
            "Loaded {} calib entr(y/ies) from {}{}",
            self.entries.len(),
            self.path.display(),
            if self.reuse_saved {
                ""
            } else {
                " (earlier sessions: not reused)"
            }
        );
        true
    }

    pub fn save(&self) -> Result<()> {
        let io = |e: std::io::Error| {
            Error::Calibration(format!("calib cache {}: {e}", self.path.display()))
        };
        if let Some(dir) = self.path.parent() {
            std::fs::create_dir_all(dir).map_err(io)?;
        }
        let payload = json!({
            "ident": CACHE_IDENT,
            "version": CACHE_VERSION,
            "entries": self.entries.iter().map(CalibEntry::to_json).collect::<Vec<_>>(),
        });
        let tmp = self.path.with_extension("tmp");
        std::fs::write(&tmp, serde_json::to_string(&payload).expect("json")).map_err(io)?;
        std::fs::rename(&tmp, &self.path).map_err(io)?;
        info!(
            "Wrote calib cache {} ({} entries)",
            self.path.display(),
            self.entries.len()
        );
        Ok(())
    }

    pub fn find(
        &self,
        method: ScanMethod,
        resolution: i64,
        startx: i64,
        pixels: i64,
    ) -> Option<&CalibEntry> {
        self.entries
            .iter()
            .find(|e| self.visible(e) && e.matches(method, resolution, startx, pixels))
    }

    pub fn upsert(&mut self, mut entry: CalibEntry) {
        if let Some(f) = &self.fingerprint {
            entry.fingerprint = f.clone();
        }
        // Replace this model's entry for the window, saved or not.
        let fp = self.fingerprint.clone();
        self.entries.retain(|e| {
            let same_model = fp.as_ref().is_none_or(|f| &e.fingerprint == f);
            !same_model || !e.matches(entry.method, entry.resolution, entry.startx, entry.pixels)
        });
        entry.saved = false;
        self.entries.push(entry);
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        if self.path.exists() {
            let _ = std::fs::remove_file(&self.path);
            info!("Cleared calibration cache {}", self.path.display());
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// Acquire dark/white averages and manage the on-disk cache.
pub struct Calibrator {
    pub model: FilmModel,
    pub cache: CalibCache,
    pub pipeline: ImagePipeline,
    active: Option<CalibEntry>,
    /// Entry whose AHB blob is currently loaded in ASIC DVDSET RAM.
    loaded_asic_shading: Option<CalibEntry>,
    /// GL128: after ASIC shading upload, skip host dark/white stretch on scan.
    pub prefer_asic_shading: bool,
}

/// Guard: GL128 motor moves disarmed for stationary AFE/shading.
struct Stationary<'a> {
    asic: &'a mut Gl128,
    prev: bool,
}

impl<'a> Stationary<'a> {
    fn new(asic: &'a mut Gl128) -> Self {
        let prev = asic.motor_moves_enabled;
        asic.motor_moves_enabled = false;
        Self { asic, prev }
    }
}

impl Drop for Stationary<'_> {
    fn drop(&mut self) {
        self.asic.motor_moves_enabled = self.prev;
    }
}

impl Calibrator {
    pub fn new(model: FilmModel, cache_path: Option<PathBuf>, reuse_saved: bool) -> Self {
        let mut cache = CalibCache::new(cache_path, Some(calib_fingerprint(&model)), reuse_saved);
        cache.load();
        Self {
            pipeline: ImagePipeline::new(model.clone()),
            model,
            cache,
            active: None,
            loaded_asic_shading: None,
            prefer_asic_shading: false,
        }
    }

    pub fn cache_path(&self) -> &Path {
        &self.cache.path
    }

    pub fn active(&self) -> Option<&CalibEntry> {
        self.active.as_ref()
    }

    /// False when the active entry is ASIC-shading (or prefer flag + ready).
    pub fn should_apply_host_calib(&self, asic_shading_ready: bool) -> bool {
        if self.active.as_ref().is_some_and(|a| a.asic_shading) {
            return false;
        }
        if !self.prefer_asic_shading {
            return true;
        }
        !asic_shading_ready
    }

    pub fn load(&mut self) -> bool {
        self.cache.load()
    }

    pub fn clear(&mut self) {
        self.cache.clear();
        self.active = None;
        self.loaded_asic_shading = None;
    }

    pub fn find_for_scan(
        &mut self,
        method: ScanMethod,
        geometry: &ScanGeometry,
    ) -> Option<CalibEntry> {
        let mut entry = self
            .cache
            .find(
                method,
                geometry.resolution,
                geometry.startx,
                geometry.pixels,
            )
            .cloned();
        // Dummy v1 ASIC markers (no blob) are not usable — treat as miss.
        if entry
            .as_ref()
            .is_some_and(|e| e.asic_shading && !e.has_asic_blob())
        {
            info!(
                "Ignoring ASIC calib marker without shading blob dpi={}",
                geometry.resolution
            );
            entry = None;
        }
        self.active = entry.clone();
        entry
    }

    fn asic_dpi(&self, resolution: i64) -> i64 {
        self.model.asic_dpi_for(resolution)
    }

    fn ensure_at_home(asic: &mut Gl128) -> Result<()> {
        if asic.is_at_home()? {
            return Ok(());
        }
        info!("GL128 shading: carriage off home — parking before measure/apply");
        asic.home()
    }

    /// Re-upload a cached AHB blob + AFE (no strip re-acquire).
    pub fn apply_colour_asic_shading(
        &mut self,
        asic: &mut Gl128,
        entry: &CalibEntry,
    ) -> Result<()> {
        if !entry.has_asic_blob() {
            return Err(Error::Calibration(
                "Calib entry has no ASIC shading blob".into(),
            ));
        }
        let dpi = self.asic_dpi(entry.resolution);
        {
            let g = Stationary::new(asic);
            Self::ensure_at_home(g.asic)?;
            let fe = AfeFrontend {
                offsets: entry.afe_offsets.unwrap(),
                gains: entry.afe_gains.unwrap(),
            };
            g.asic.apply_frontend(fe)?;
            g.asic.last_afe = Some(fe);
            g.asic.last_afe_dpi = Some(dpi);
            g.asic
                .upload_shading_table(entry.shading_blob.as_deref().unwrap())?;
            g.asic.asic_shading_ready = true;
        }
        self.prefer_asic_shading = true;
        self.active = Some(entry.clone());
        self.loaded_asic_shading = Some(entry.clone());
        info!(
            "GL128 applied cached ASIC shading dpi={} blob={} bytes",
            entry.resolution,
            entry.shading_blob.as_ref().map_or(0, Vec::len)
        );
        Ok(())
    }

    /// AFE + dark/white at home; save blob. Retries once after re-home on film-like fail.
    pub fn measure_colour_asic_shading(
        &mut self,
        asic: &mut Gl128,
        geometry: &ScanGeometry,
        research_afe: bool,
    ) -> Result<CalibEntry> {
        let scan_dpi = self.asic_dpi(geometry.resolution);
        let mut last_reason = "unknown".to_string();
        for attempt in 0..2 {
            let blob = {
                let g = Stationary::new(asic);
                Self::ensure_at_home(g.asic)?;
                let reuse_afe = !(research_afe && attempt == 0)
                    && g.asic.last_afe.is_some()
                    && g.asic.scan_method == ScanMethod::Transparency
                    && afe_timing_matches(g.asic.last_afe_dpi, scan_dpi);
                if !reuse_afe {
                    info!(
                        "GL128 running stationary AFE search (attempt {})",
                        attempt + 1
                    );
                    g.asic
                        .search_afe(Some(ScanMethod::Transparency), Some(geometry.resolution))?;
                } else {
                    info!(
                        "GL128 reusing colour AFE codes for dpi={} shading remesure (attempt {})",
                        geometry.resolution,
                        attempt + 1
                    );
                }
                info!(
                    "GL128 measuring ASIC shading dpi={} window={}..{} dpiset={} (attempt {})",
                    geometry.resolution,
                    geometry.pixel_startx,
                    geometry.pixel_endx,
                    geometry.register_dpiset,
                    attempt + 1
                );
                g.asic.run_asic_shading(
                    geometry.resolution,
                    Some(ScanMethod::Transparency),
                    Some(geometry.pixel_startx),
                    Some(geometry.pixel_endx),
                    Some(geometry.register_dpiset),
                )?
            };
            let px = geometry.pixels as usize;
            if asic.asic_shading_ready {
                let afe = asic.last_afe.ok_or_else(|| {
                    Error::Calibration("ASIC shading ready but last_afe missing".into())
                })?;
                let mut entry = CalibEntry::new(
                    ScanMethod::Transparency,
                    geometry.resolution,
                    geometry.startx,
                    geometry.pixels,
                    Array2::zeros((px, 3)),
                    Array2::from_elem((px, 3), 65535),
                );
                entry.asic_shading = true;
                entry.shading_blob = Some(blob);
                entry.afe_offsets = Some(afe.offsets);
                entry.afe_gains = Some(afe.gains);
                self.cache.upsert(entry);
                self.cache.save()?;
                let entry = self.cache.entries.last().unwrap().clone();
                self.active = Some(entry.clone());
                self.loaded_asic_shading = Some(entry.clone());
                self.prefer_asic_shading = true;
                info!(
                    "GL128 ASIC shading measured dpi={} blob={} bytes",
                    geometry.resolution,
                    entry.shading_blob.as_ref().map_or(0, Vec::len)
                );
                return Ok(entry);
            }
            if asic.last_color_shading_host_ok {
                let afe = asic.last_afe.ok_or_else(|| {
                    Error::Calibration("Host shading fallback missing last_afe".into())
                })?;
                let dark_cols = asic.last_host_calib_dark.clone().unwrap_or_default();
                let white_cols = asic.last_host_calib_white.clone().unwrap_or_default();
                if dark_cols.len() < 8 || white_cols.len() < 8 {
                    return Err(Error::Calibration(
                        "Host shading fallback missing strip columns".into(),
                    ));
                }
                let to_arr = |cols: &[[i64; 3]]| {
                    Array2::from_shape_fn((px, 3), |(x, c)| cols[x.min(cols.len() - 1)][c] as u16)
                };
                let mut entry = CalibEntry::new(
                    ScanMethod::Transparency,
                    geometry.resolution,
                    geometry.startx,
                    geometry.pixels,
                    to_arr(&dark_cols),
                    to_arr(&white_cols),
                );
                entry.afe_offsets = Some(afe.offsets);
                entry.afe_gains = Some(afe.gains);
                self.cache.upsert(entry);
                self.cache.save()?;
                let entry = self.cache.entries.last().unwrap().clone();
                self.active = Some(entry.clone());
                self.loaded_asic_shading = None;
                self.prefer_asic_shading = false;
                info!(
                    "GL128 host dark/white calib dpi={} pixels={px} (ASIC DVDSET off)",
                    geometry.resolution
                );
                return Ok(entry);
            }
            last_reason = asic
                .last_color_shading_reject_reason
                .clone()
                .unwrap_or_else(|| "ASIC shading validation failed".into());
            if attempt == 0 {
                warn!(
                    "GL128 colour shading rejected ({last_reason}) — re-homing and retrying once"
                );
                let prev = asic.motor_moves_enabled;
                asic.motor_moves_enabled = true;
                let r = asic.home();
                asic.motor_moves_enabled = prev;
                r?;
            }
        }
        Err(Error::Calibration(colour_shading_failure_message(
            &last_reason,
        )))
    }

    /// True when DVDSET RAM already holds this entry's table (same window).
    fn asic_shading_matches(&self, asic: &Gl128, entry: &CalibEntry) -> bool {
        if !asic.asic_shading_ready {
            return false;
        }
        let Some(loaded) = self
            .loaded_asic_shading
            .as_ref()
            .filter(|l| l.has_asic_blob())
        else {
            return false;
        };
        loaded.resolution == entry.resolution
            && loaded.startx == entry.startx
            && loaded.pixels == entry.pixels
            && loaded.shading_blob == entry.shading_blob
    }

    /// Apply cached blob if present; otherwise measure at home (SF order).
    pub fn ensure_colour_asic_shading(
        &mut self,
        asic: &mut Gl128,
        geometry: &ScanGeometry,
    ) -> Result<CalibEntry> {
        let hit = self.find_for_scan(ScanMethod::Transparency, geometry);
        if let Some(h) = hit.as_ref().filter(|h| h.has_asic_blob()) {
            if self.asic_shading_matches(asic, h) {
                self.prefer_asic_shading = true;
                self.active = Some(h.clone());
            } else {
                self.apply_colour_asic_shading(asic, h)?;
            }
            return Ok(h.clone());
        }
        if let Some(h) =
            hit.filter(|h| !h.asic_shading && h.afe_offsets.is_some() && h.afe_gains.is_some())
        {
            let fe = AfeFrontend {
                offsets: h.afe_offsets.unwrap(),
                gains: h.afe_gains.unwrap(),
            };
            asic.apply_frontend(fe)?;
            asic.last_afe = Some(fe);
            asic.last_afe_dpi = Some(self.asic_dpi(h.resolution));
            asic.asic_shading_ready = false;
            self.prefer_asic_shading = false;
            self.active = Some(h.clone());
            self.loaded_asic_shading = None;
            info!(
                "GL128 using cached host dark/white calib dpi={}",
                geometry.resolution
            );
            return Ok(h);
        }
        self.measure_colour_asic_shading(asic, geometry, false)
    }

    /// GL128 IR: AFE search + IR white strip at home, before the IR pass.
    /// Returns `ir_host_flatten_ready`.
    pub fn measure_ir_host_flatten(
        &mut self,
        asic: &mut Gl128,
        geometry: &ScanGeometry,
    ) -> Result<bool> {
        let colour_afe = asic.last_afe;
        let colour_afe_dpi = asic.last_afe_dpi;
        let r = (|| -> Result<()> {
            let g = Stationary::new(asic);
            Self::ensure_at_home(g.asic)?;
            info!(
                "GL128 running stationary IR AFE search dpi={}",
                geometry.resolution
            );
            g.asic
                .search_afe(Some(ScanMethod::Infrared), Some(geometry.resolution))?;
            g.asic.run_asic_shading(
                geometry.resolution,
                Some(ScanMethod::Infrared),
                Some(geometry.pixel_startx),
                Some(geometry.pixel_endx),
                Some(geometry.register_dpiset),
            )?;
            Ok(())
        })();
        let reason = match r {
            Ok(()) => None,
            // Not an IR-calibration problem: the scanner needs a power-cycle.
            Err(e @ Error::ScannerStuck(_)) => return Err(e),
            Err(e @ (Error::Asic(_) | Error::Calibration(_) | Error::Scan(_))) => {
                Some(e.to_string())
            }
            Err(e) => return Err(e),
        };
        let ready = reason.is_none() && asic.ir_host_flatten_ready;
        if !ready {
            asic.ir_host_flatten_ready = false;
            asic.last_ir_host_white = None;
            if let Some(fe) = colour_afe {
                asic.apply_frontend(fe)?;
            }
            asic.last_afe = colour_afe;
            asic.last_afe_dpi = colour_afe_dpi;
            warn!(
                "GL128 IR calibration unavailable ({}); IR pass uses colour AFE codes and the per-image flatten",
                reason.as_deref().unwrap_or("IR white profile rejected")
            );
        }
        Ok(ready)
    }

    /// Public GL128 IR calibration as a [`CalibEntry`] (never cached).
    fn gl128_infrared_entry(
        &mut self,
        asic: &mut Gl128,
        geometry: &ScanGeometry,
    ) -> Result<CalibEntry> {
        if !self.measure_ir_host_flatten(asic, geometry)? {
            return Err(Error::Calibration(
                "GL128 infrared calibration failed (IR AFE search or IR white strip rejected); see the log. IR scans fall back to the per-image flatten.".into(),
            ));
        }
        let px = geometry.pixels as usize;
        let mut white = Array2::<u16>::zeros((px, 3));
        let profile: Vec<i64> = asic
            .last_ir_host_white
            .clone()
            .unwrap_or_default()
            .into_iter()
            .take(px)
            .collect();
        for (x, &v) in profile.iter().enumerate() {
            for c in 0..3 {
                white[[x, c]] = v as u16;
            }
        }
        let mut e = CalibEntry::new(
            ScanMethod::Infrared,
            geometry.resolution,
            geometry.startx,
            geometry.pixels,
            Array2::zeros((px, 3)),
            white,
        );
        e.afe_offsets = asic.last_afe.map(|a| a.offsets);
        e.afe_gains = asic.last_afe.map(|a| a.gains);
        Ok(e)
    }

    /// Re-apply cached FE codes so image configure matches the calib strips.
    fn apply_host_afe(asic: &mut Gl845, entry: &CalibEntry) -> Result<()> {
        if let (Some(o), Some(g)) = (entry.afe_offsets, entry.afe_gains) {
            asic.apply_afe_frontend(
                AfeFrontend {
                    offsets: o,
                    gains: g,
                },
                Some(entry.method),
                Some(entry.resolution),
            )?;
        }
        Ok(())
    }

    /// Genesys path: cached host dark/white, or measure via [`Calibrator::run`].
    pub fn ensure_host_calib(
        &mut self,
        asic: &mut Gl845,
        geometry: &ScanGeometry,
        method: ScanMethod,
        mode: ScanMode,
    ) -> Result<CalibEntry> {
        if mode == ScanMode::Gray {
            return Err(Error::Value(format!(
                "Unsupported mode {:?}",
                mode.as_str()
            )));
        }
        if let Some(hit) = self
            .find_for_scan(method, geometry)
            .filter(|h| !h.asic_shading)
        {
            self.prefer_asic_shading = false;
            self.active = Some(hit.clone());
            Self::apply_host_afe(asic, &hit)?;
            info!(
                "Using cached host calib method={} dpi={} pixels={}",
                method.as_str(),
                geometry.resolution,
                geometry.pixels
            );
            return Ok(hit);
        }
        self.run_genesys(asic, geometry.resolution, mode, false, None, Some(geometry))
    }

    /// Run dark (if color) + white shading for `resolution` / `mode`.
    pub fn run(
        &mut self,
        asic: &mut Asic,
        resolution: i64,
        mode: ScanMode,
        force: bool,
        area: Option<Area>,
        geometry: Option<&ScanGeometry>,
    ) -> Result<CalibEntry> {
        match asic {
            Asic::Gl845(a) => self.run_genesys(a, resolution, mode, force, area, geometry),
            Asic::Gl128(a) => self.run_gl128(a, resolution, mode, force, area, geometry),
            Asic::Gl843V1(_) => Err(Error::NotImplemented(
                "OpticFilm 7600i (v1) calibration runs inside the vendor scan job".into(),
            )),
        }
    }

    fn check_mode(mode: ScanMode) -> Result<ScanMethod> {
        match mode {
            ScanMode::Gray => Err(Error::Value(
                "Grayscale calibration is not implemented".into(),
            )),
            m => Ok(m.method()),
        }
    }

    fn run_gl128(
        &mut self,
        asic: &mut Gl128,
        resolution: i64,
        mode: ScanMode,
        force: bool,
        area: Option<Area>,
        geometry: Option<&ScanGeometry>,
    ) -> Result<CalibEntry> {
        let method = Self::check_mode(mode)?;
        let scan_geo = match geometry {
            Some(g) => g.clone(),
            None => compute_geometry(resolution, &self.model, area)?,
        };
        if method == ScanMethod::Infrared {
            // Every IR scan re-measures this before its IR pass; never cached.
            return self.gl128_infrared_entry(asic, &scan_geo);
        }
        if let Some(existing) = self
            .cache
            .find(
                method,
                scan_geo.resolution,
                scan_geo.startx,
                scan_geo.pixels,
            )
            .cloned()
            && !force
            && (!existing.asic_shading || existing.has_asic_blob())
        {
            info!(
                "Using cached calib method={} dpi={} pixels={}",
                method.as_str(),
                scan_geo.resolution,
                scan_geo.pixels
            );
            self.active = Some(existing.clone());
            if existing.has_asic_blob() && !asic.asic_shading_ready {
                self.apply_colour_asic_shading(asic, &existing)?;
            }
            return Ok(existing);
        }
        if !asic.initialized {
            asic.init(false)?;
        }
        asic.set_scan_method(method);
        if force {
            return self.measure_colour_asic_shading(asic, &scan_geo, true);
        }
        self.ensure_colour_asic_shading(asic, &scan_geo)
    }

    fn run_genesys(
        &mut self,
        asic: &mut Gl845,
        resolution: i64,
        mode: ScanMode,
        force: bool,
        area: Option<Area>,
        geometry: Option<&ScanGeometry>,
    ) -> Result<CalibEntry> {
        let method = Self::check_mode(mode)?;
        let scan_geo = match geometry {
            Some(g) => g.clone(),
            None => compute_geometry(resolution, &self.model, area)?,
        };
        let resolution = scan_geo.resolution;
        if let Some(existing) = self
            .cache
            .find(method, resolution, scan_geo.startx, scan_geo.pixels)
            .cloned()
            && !force
            && (!existing.asic_shading || existing.has_asic_blob())
        {
            info!(
                "Using cached calib method={} dpi={resolution} pixels={}",
                method.as_str(),
                scan_geo.pixels
            );
            self.active = Some(existing.clone());
            return Ok(existing);
        }
        if !asic.initialized {
            asic.init(false)?;
        }
        let mut session = ScanSession::new(self.model.clone(), asic.variant);
        let calib_geo = self.calib_geometry_for(&scan_geo, method, None)?;
        asic.set_scan_method(method);

        // AFE search before dark/white strips (ADI FESET: table defaults).
        if asic.afe_dichotomy_applicable()? {
            let pipeline = &mut self.pipeline;
            let mut measure = |a: &mut Gl845, _fe: AfeFrontend| -> Result<[f64; 3]> {
                let avg = capture_average(
                    pipeline,
                    &mut session,
                    a,
                    &calib_geo,
                    method,
                    true,
                    false,
                    0.05,
                    "afe",
                    Reduce::Median,
                )?;
                Ok([0, 1, 2].map(|c| crate::num::mean(avg.column(c).iter().map(|&v| v as f64))))
            };
            asic.search_afe(method, Some(resolution), Some(&mut measure), None, false)?;
        } else {
            asic.search_afe(method, Some(resolution), None, None, false)?;
        }

        if let Some(target) = self.model.afe_white_target {
            self.search_white_gains(&mut session, asic, &calib_geo, method, target)?;
        }
        let ir_dark = self.model.asic == "GL845";
        let px = scan_geo.pixels as usize;
        let dark = if mode == ScanMode::Infrared && !ir_dark {
            info!("IR calib: skipping dark (genesys behaviour)");
            Array2::zeros((px, 3))
        } else {
            let dg = self.dark_geometry_for(&scan_geo, &calib_geo, method)?;
            let d = capture_average(
                &mut self.pipeline,
                &mut session,
                asic,
                &dg,
                method,
                false,
                false,
                DARK_SETTLE_S,
                "dark",
                Reduce::Mean,
            )?;
            asic.home(crate::asic::gl845::DEFAULT_HOME_TIMEOUT_S, true)?;
            d
        };
        // GL845 white strip may use a short feed.
        let white = capture_average(
            &mut self.pipeline,
            &mut session,
            asic,
            &calib_geo,
            method,
            true,
            true,
            WHITE_SETTLE_S,
            "white",
            Reduce::Median,
        )?;
        asic.home(crate::asic::gl845::DEFAULT_HOME_TIMEOUT_S, true)?;
        let mean_u16 = |a: &Array2<u16>| crate::num::mean(a.iter().map(|&v| v as f64));
        if mode != ScanMode::Infrared || ir_dark {
            let (dm, wm) = (mean_u16(&dark), mean_u16(&white));
            if dm > DARK_WHITE_MAX_RATIO * wm {
                return Err(Error::Calibration(format!(
                    "Dark capture is not dark (dark mean {dm:.0} vs white {wm:.0}); the lamp stayed on during the dark capture"
                )));
            }
        }
        let (dmean, wmean) = (mean_u16(&dark), mean_u16(&white));
        let mut entry = CalibEntry::new(
            method,
            resolution,
            scan_geo.startx,
            scan_geo.pixels,
            dark,
            white,
        );
        entry.afe_offsets = asic.last_afe_offsets;
        entry.afe_gains = asic.last_afe_gains;
        self.cache.upsert(entry);
        self.cache.save()?;
        let entry = self.cache.entries.last().unwrap().clone();
        self.active = Some(entry.clone());
        self.prefer_asic_shading = false;
        info!(
            "Calib done method={} dpi={resolution} dark_mean={dmean:.0} white_mean={wmean:.0} afe_gains={:?}",
            method.as_str(),
            entry.afe_gains
        );
        Ok(entry)
    }

    /// Taller strip for the dark reference when the model asks for more lines.
    fn dark_geometry_for(
        &self,
        scan_geo: &ScanGeometry,
        calib_geo: &ScanGeometry,
        method: ScanMethod,
    ) -> Result<ScanGeometry> {
        let lines = self.model.calib_dark_lines.unwrap_or(0);
        if lines <= calib_geo.lines {
            return Ok(calib_geo.clone());
        }
        let asic_dpi = self.model.asic_dpi_for(scan_geo.resolution);
        self.calib_geometry_for(
            scan_geo,
            method,
            Some(lines as f64 * MM_PER_INCH / asic_dpi as f64),
        )
    }

    /// Calibration strip geometry whose columns match `scan_geo` 1:1.
    pub fn calib_geometry_for(
        &self,
        scan_geo: &ScanGeometry,
        method: ScanMethod,
        height_mm: Option<f64>,
    ) -> Result<ScanGeometry> {
        let res = scan_geo.resolution;
        let cg = compute_calib_geometry(res, &self.model, height_mm)?;
        let depth = self
            .model
            .gl128
            .as_ref()
            .map_or(cg.depth, |g| g.usb_calib_depth);
        Ok(ScanGeometry {
            pixels: scan_geo.pixels,
            startx: scan_geo.startx,
            pixel_startx: scan_geo.pixel_startx,
            pixel_endx: scan_geo.pixel_endx,
            optical_pixels: scan_geo.optical_pixels,
            output_pixel_offset: scan_geo.output_pixel_offset,
            depth,
            line_bytes: scan_geo.pixels * cg.channels * (depth / 8),
            disable_buffer_full_move: true,
            exposure_lperiod: exposure_lperiod_for(&self.model, res, method),
            dummy_pixel: dummy_pixel_for(&self.model, res),
            ..cg
        })
    }

    /// Set AD9826 PGA codes so the white strip's bright columns land at `target`.
    fn search_white_gains(
        &mut self,
        session: &mut ScanSession,
        asic: &mut Gl845,
        geometry: &ScanGeometry,
        method: ScanMethod,
        target: f64,
    ) -> Result<()> {
        let mut gains = asic.last_afe_gains.unwrap_or([0; 3]);
        let offsets = asic.last_afe_offsets.unwrap_or([0; 3]);
        for round_idx in 0..AFE_WHITE_GAIN_ROUNDS {
            asic.apply_afe_frontend(
                AfeFrontend { offsets, gains },
                Some(method),
                Some(geometry.resolution),
            )?;
            let dark: [f64; 3] = if method == ScanMethod::Infrared && self.model.asic != "GL845" {
                [0.0; 3]
            } else {
                let d = capture_average(
                    &mut self.pipeline,
                    session,
                    asic,
                    geometry,
                    method,
                    false,
                    false,
                    DARK_SETTLE_S,
                    "afe dark",
                    Reduce::Median,
                )?;
                // The white feed must begin at home or it lands on the holder edge.
                asic.home(crate::asic::gl845::DEFAULT_HOME_TIMEOUT_S, true)?;
                [0, 1, 2].map(|c| crate::num::mean(d.column(c).iter().map(|&v| v as f64)))
            };
            let white = capture_average(
                &mut self.pipeline,
                session,
                asic,
                geometry,
                method,
                true,
                true,
                WHITE_SETTLE_S,
                "afe white",
                Reduce::Median,
            )?;
            asic.home(crate::asic::gl845::DEFAULT_HOME_TIMEOUT_S, true)?;
            let peak =
                [0, 1, 2].map(|c| U16Hist::new(white.column(c).iter().copied()).percentile(99.0));
            let mut new_gains = [0i64; 3];
            for c in 0..3 {
                let current = pga_gain(gains[c]);
                let wanted = if peak[c] >= AFE_WHITE_SATURATED {
                    current * 0.75
                } else {
                    current * (target - dark[c]) / (peak[c] - dark[c]).max(1.0)
                };
                let wanted = wanted.clamp(1.0, 6.0);
                new_gains[c] = py_round(75.6 * (1.0 - 1.0 / wanted)).clamp(0, AFE_GAIN_MAX);
            }
            info!(
                "AFE white search {} round {round_idx}: gains={gains:?} p99={:?} dark={:?} → {new_gains:?}",
                method.as_str(),
                peak.map(|v| v as i64),
                dark.map(|v| v as i64)
            );
            let settled = (0..3).all(|c| (peak[c] - target).abs() <= AFE_WHITE_TOLERANCE * target);
            if new_gains == gains || settled {
                break;
            }
            gains = new_gains;
        }
        asic.apply_afe_frontend(
            AfeFrontend { offsets, gains },
            Some(method),
            Some(geometry.resolution),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reduce {
    Mean,
    Median,
}

/// One stationary (or short-feed) strip, averaged per column.
#[allow(clippy::too_many_arguments)]
pub fn capture_average(
    pipeline: &mut ImagePipeline,
    session: &mut ScanSession,
    asic: &mut Gl845,
    geometry: &ScanGeometry,
    method: ScanMethod,
    lamp_on: bool,
    start_motor: bool,
    settle_s: f64,
    label: &str,
    reduce: Reduce,
) -> Result<Array2<u16>> {
    asic.set_scan_method(method);
    if lamp_on {
        asic.lamp_on()?;
    } else {
        asic.lamp_off()?;
    }
    sleep(settle_s);
    let raw = session.acquire_raw(
        asic,
        geometry,
        method,
        lamp_on,
        start_motor,
        &mut NoProgress,
    )?;
    // Raw linear columns: the film-base stretch would lift dark to white's peak.
    let rgb = pipeline.assemble(&raw, geometry, None, None, None, false, None)?;
    let (h, w, _) = rgb.dim();
    let out = Array2::from_shape_fn((w, 3), |(x, c)| {
        let col = rgb.slice(ndarray::s![.., x, c]);
        let v = match reduce {
            // np.mean of float32 along axis 0 (pairwise, float32).
            Reduce::Mean => (col.iter().map(|&v| v as f32).sum::<f32>() / h as f32) as f64,
            Reduce::Median => {
                crate::num::median_f32(col.iter().map(|&v| v as f32).collect()) as f64
            }
        };
        crate::num::to_u16(v)
    });
    info!(
        "{label} calib average shape=({w}, 3) mean={:.1}",
        crate::num::mean(out.iter().map(|&v| v as f64))
    );
    let _ = Axis(0);
    Ok(out)
}
