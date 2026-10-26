//! Per-image persistence (`<image>.nlp.json` next to the scan — the
//! equivalent of the plugin's catalog metadata) and saved defaults.

use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::curve::Origins;
use super::pipeline::{Analysis, AutoWb};
use super::settings::{ExportSettings, FilmMetadata, Settings};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RollData {
    pub id: String,
    pub origins: Origins,
    pub base: [f32; 3],
    pub frames: usize,
    pub excluded: Vec<String>,
    /// Roll-wide auto WB (average of the frames used for the aggregate).
    #[serde(default)]
    pub auto_wb: Option<AutoWb>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Sidecar {
    pub version: u32,
    pub settings: Settings,
    pub analysis: Option<Analysis>,
    pub roll: Option<RollData>,
    pub metadata: FilmMetadata,
}

impl Sidecar {
    pub fn is_converted(&self) -> bool {
        self.analysis.is_some()
    }
}

pub fn sidecar_path(image: &Path) -> PathBuf {
    let name = image
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("image");
    image.with_file_name(format!("{name}.nlp.json"))
}

pub fn load(image: &Path) -> Option<Sidecar> {
    let s = std::fs::read_to_string(sidecar_path(image)).ok()?;
    let mut sc: Sidecar = serde_json::from_str(&s).ok()?;
    // Migrate the plugin-style Exposure to print exposure in stops.
    if sc.settings.exposure != 0.0 {
        sc.settings.print_exposure =
            (sc.settings.print_exposure + sc.settings.exposure / 10.0).clamp(-5.0, 5.0);
        sc.settings.exposure = 0.0;
    }
    Some(sc)
}

pub fn save(image: &Path, sc: &Sidecar) -> Result<()> {
    let mut sc = sc.clone();
    sc.version = 1;
    std::fs::write(sidecar_path(image), serde_json::to_string_pretty(&sc)?)?;
    Ok(())
}

fn config_dir() -> Option<PathBuf> {
    let d = dirs::config_dir()?.join("nlp-studio");
    std::fs::create_dir_all(&d).ok()?;
    Some(d)
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Defaults {
    pub settings: Settings,
    pub export: ExportSettings,
    pub last_folder: Option<PathBuf>,
}

pub fn load_defaults() -> Defaults {
    config_dir()
        .and_then(|d| std::fs::read_to_string(d.join("defaults.json")).ok())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_defaults(d: &Defaults) -> Result<()> {
    if let Some(dir) = config_dir() {
        std::fs::write(dir.join("defaults.json"), serde_json::to_string_pretty(d)?)?;
    }
    Ok(())
}

/// Disk cache for decoded analysis proxies (keyed by path, size and mtime).
pub fn proxy_cache_path(image: &Path) -> Option<PathBuf> {
    use std::hash::{Hash, Hasher};
    let md = std::fs::metadata(image).ok()?;
    let mut h = std::collections::hash_map::DefaultHasher::new();
    image.hash(&mut h);
    md.len().hash(&mut h);
    md.modified().ok()?.hash(&mut h);
    let dir = dirs::cache_dir()?.join("nlp-studio").join("proxies");
    std::fs::create_dir_all(&dir).ok()?;
    Some(dir.join(format!("{:016x}.json", h.finish())))
}
