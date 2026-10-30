//! Per-image persistence (`<image>.nlp.json` next to the scan — the
//! equivalent of the plugin's catalog metadata) and saved defaults.

use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::curve::Levels;
use super::pipeline::{Analysis, AutoWb, RollBalance};
use super::settings::{ExportPreset, ExportSettings, FilmMetadata, Settings};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RollData {
    pub id: String,
    /// The roll's levels at one exposure for the whole roll.
    pub levels: Levels,
    /// The roll's colour balance (film slopes and filtration).
    pub balance: RollBalance,
    pub base: [f32; 3],
    pub frames: usize,
    pub excluded: Vec<String>,
    /// Roll-wide auto WB (from the frames' averaged grey densities).
    #[serde(default)]
    pub auto_wb: Option<AutoWb>,
    /// The roll's common exposure value, when frames had EXIF.
    #[serde(default)]
    pub common_ev: Option<f64>,
}

/// Sidecar format version. Version 3 holds density analyses and roll
/// balances; analyses and rolls of older files are dropped when loading
/// (the frame needs a new Convert), their settings and metadata are kept.
pub const VERSION: u32 = 3;

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
    parse(&s)
}

/// Parse a sidecar, keeping whatever part of an older one still applies.
fn parse(s: &str) -> Option<Sidecar> {
    let v: serde_json::Value = serde_json::from_str(s).ok()?;
    let field = |k: &str| v.get(k).cloned().unwrap_or(serde_json::Value::Null);
    let version = v.get("version").and_then(|x| x.as_u64()).unwrap_or(0) as u32;
    let current = version >= VERSION;
    Some(Sidecar {
        version,
        settings: serde_json::from_value(field("settings")).unwrap_or_default(),
        analysis: current
            .then(|| serde_json::from_value(field("analysis")).ok())
            .flatten(),
        roll: current
            .then(|| serde_json::from_value(field("roll")).ok())
            .flatten(),
        metadata: serde_json::from_value(field("metadata")).unwrap_or_default(),
    })
}

pub fn save(image: &Path, sc: &Sidecar) -> Result<()> {
    let path = sidecar_path(image);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let mut sc = sc.clone();
    sc.version = VERSION;
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
    /// The user's own export presets (`ExportPreset::builtin` ship with the app).
    pub export_presets: Vec<ExportPreset>,
    pub last_folder: Option<PathBuf>,
    /// Scan folders opened before, most recent first (the Projects list).
    pub projects: Vec<PathBuf>,
    /// Convert frames only when asked, instead of as they load.
    pub manual_convert: bool,
}

/// Move `dir` to the front of the Projects list.
pub fn remember_project(d: &mut Defaults, dir: &Path) {
    d.projects.retain(|p| p != dir);
    d.projects.insert(0, dir.to_path_buf());
}

/// Details for a whole roll, kept in `<folder>/NLP/roll.json`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct RollInfo {
    pub name: String,
    /// Shoot date, free text.
    pub date: String,
    pub metadata: FilmMetadata,
    pub notes: String,
    /// Unix seconds when the roll was first set up.
    pub created: u64,
    /// Custom export folder for this roll.
    pub export_dir: Option<PathBuf>,
}

pub fn roll_info_path(dir: &Path) -> PathBuf {
    dir.join("NLP").join("roll.json")
}

pub fn load_roll_info(dir: &Path) -> Option<RollInfo> {
    let s = std::fs::read_to_string(roll_info_path(dir)).ok()?;
    serde_json::from_str(&s).ok()
}

pub fn save_roll_info(dir: &Path, info: &RollInfo) -> Result<()> {
    let path = roll_info_path(dir);
    if let Some(d) = path.parent() {
        std::fs::create_dir_all(d)?;
    }
    std::fs::write(&path, serde_json::to_string_pretty(info)?)?;
    Ok(())
}

/// Number of frames with a sidecar in a scan folder.
pub fn sidecar_count(dir: &Path) -> usize {
    std::fs::read_dir(dir.join("NLP"))
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .filter(|e| e.file_name().to_string_lossy().ends_with(".nlp.json"))
                .count()
        })
        .unwrap_or(0)
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

/// Disk cache for decoded analysis proxies (keyed by decoder version, path,
/// size and mtime).
pub fn proxy_cache_path(image: &Path) -> Option<PathBuf> {
    use std::hash::{Hash, Hasher};
    let md = std::fs::metadata(image).ok()?;
    let mut h = std::collections::hash_map::DefaultHasher::new();
    super::decode::DECODE_VERSION.hash(&mut h);
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
    fn roll_info_round_trip() {
        let tmp = std::env::temp_dir().join(format!("nlp_roll_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        assert!(load_roll_info(&tmp).is_none());
        let mut info = RollInfo {
            name: "Roll 7".into(),
            ..Default::default()
        };
        info.metadata.film_stock = "Portra 400".into();
        save_roll_info(&tmp, &info).unwrap();
        assert_eq!(load_roll_info(&tmp), Some(info));
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn old_sidecars_keep_settings_but_need_a_new_convert() {
        let old = r#"{"version": 1,
            "settings": {"contrast": 12, "tone_profile": "LogarithmicRich", "curve_points": "Auto"},
            "analysis": {"origins": []},
            "metadata": {"film_stock": "Portra 400"}}"#;
        let sc = parse(old).unwrap();
        assert_eq!(sc.settings.contrast, 12.0);
        assert_eq!(sc.metadata.film_stock, "Portra 400");
        assert!(sc.analysis.is_none() && sc.roll.is_none());
        let mut cur = sc.clone();
        cur.version = VERSION;
        let back = parse(&serde_json::to_string(&cur).unwrap()).unwrap();
        assert_eq!(back.settings, sc.settings);
    }

    #[test]
    fn defaults_without_projects_load() {
        let d: Defaults = serde_json::from_str(r#"{"last_folder":"/x"}"#).unwrap();
        assert!(d.projects.is_empty());
        let mut d = d;
        remember_project(&mut d, Path::new("/a"));
        remember_project(&mut d, Path::new("/b"));
        remember_project(&mut d, Path::new("/a"));
        assert_eq!(d.projects, vec![PathBuf::from("/a"), PathBuf::from("/b")]);
    }

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
        assert_eq!(loaded.version, VERSION);

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
        assert!(
            new_path.exists(),
            "Migrated sidecar must now exist in NLP/ folder"
        );
        assert!(
            !legacy_path.exists(),
            "Legacy sidecar in root should be removed"
        );

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
