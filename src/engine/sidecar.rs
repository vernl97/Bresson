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
    /// Roll-wide auto WB (from the frames' averaged zone colours).
    #[serde(default)]
    pub auto_wb: Option<AutoWb>,
    /// Roll colour ratios `[green, blue]` for Pro/Basic Lab processing;
    /// rolls from older versions lack them and render as Darkroom Paper.
    #[serde(default)]
    pub multipliers: Option<[f64; 2]>,
    /// The roll's common exposure value (`commonEV`), when frames had EXIF.
    #[serde(default)]
    pub common_ev: Option<f64>,
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

pub fn project_dir(image: &Path) -> PathBuf {
    let parent = image.parent().unwrap_or(Path::new(""));
    if parent.as_os_str().is_empty() {
        PathBuf::from("NLP")
    } else {
        parent.join("NLP")
    }
}

pub fn sidecar_path(image: &Path) -> PathBuf {
    let name = image
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("image");
    project_dir(image).join(format!("{name}.nlp.json"))
}

pub fn legacy_sidecar_path(image: &Path) -> PathBuf {
    let name = image
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("image");
    image.with_file_name(format!("{name}.nlp.json"))
}

pub fn load(image: &Path) -> Option<Sidecar> {
    let p = sidecar_path(image);
    let s = if let Ok(s) = std::fs::read_to_string(&p) {
        s
    } else {
        let legacy = legacy_sidecar_path(image);
        let s = std::fs::read_to_string(&legacy).ok()?;
        // Automatically move the legacy file to the NLP folder so the scan folder is clean
        if let Some(parent) = p.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if std::fs::rename(&legacy, &p).is_err() {
            let _ = std::fs::write(&p, &s);
            let _ = std::fs::remove_file(&legacy);
        }
        s
    };
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
    let path = sidecar_path(image);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let mut sc = sc.clone();
    sc.version = 1;
    std::fs::write(&path, serde_json::to_string_pretty(&sc)?)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sidecar_paths_use_nlp_subfolder() {
        let img = PathBuf::from("/photos/roll_1/frame_01.dng");
        assert_eq!(
            sidecar_path(&img),
            PathBuf::from("/photos/roll_1/NLP/frame_01.dng.nlp.json")
        );
        assert_eq!(
            legacy_sidecar_path(&img),
            PathBuf::from("/photos/roll_1/frame_01.dng.nlp.json")
        );

        let relative = PathBuf::from("frame_02.tif");
        assert_eq!(
            sidecar_path(&relative),
            PathBuf::from("NLP/frame_02.tif.nlp.json")
        );
    }

    #[test]
    fn save_and_load_in_nlp_folder() {
        let tmp = std::env::temp_dir().join(format!("nlp_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        let img = tmp.join("test.tif");
        std::fs::write(&img, b"fake_image_bytes").unwrap();

        let sc = Sidecar {
            version: 1,
            ..Default::default()
        };
        save(&img, &sc).expect("save sidecar");

        let expected_path = tmp.join("NLP").join("test.tif.nlp.json");
        assert!(expected_path.exists(), "Sidecar must be in NLP subfolder");

        let loaded = load(&img).expect("load sidecar");
        assert_eq!(loaded.version, 1);

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn legacy_sidecar_migrates_to_nlp_folder() {
        let tmp = std::env::temp_dir().join(format!("nlp_migrate_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        let img = tmp.join("test.tif");
        std::fs::write(&img, b"fake_image_bytes").unwrap();

        // Write legacy sidecar directly in the scan directory
        let legacy_path = tmp.join("test.tif.nlp.json");
        let sc = Sidecar {
            version: 1,
            ..Default::default()
        };
        std::fs::write(&legacy_path, serde_json::to_string(&sc).unwrap()).unwrap();
        assert!(legacy_path.exists());

        // Loading should find the legacy file and migrate it to NLP/
        let loaded = load(&img).expect("load migrated sidecar");
        assert_eq!(loaded.version, 1);

        let new_path = tmp.join("NLP").join("test.tif.nlp.json");
        assert!(new_path.exists(), "Migrated sidecar must now exist in NLP/ folder");
        assert!(!legacy_path.exists(), "Legacy sidecar in root should be removed");

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
