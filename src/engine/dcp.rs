//! DNG Camera Profiles (`.dcp`) from the plugin's
//! `LR Files/CameraRaw/CameraProfiles` folder ("Negative Lab v2.3" profiles),
//! matched to a raw file by camera make/model (or the DNG
//! `UniqueCameraModel`). They identify scanner DNGs; the conversion itself
//! does not use them.

use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

use anyhow::{Result, bail};

use super::decode::SourceInfo;

// ---- minimal TIFF / DCP IFD reader ----------------------------------------------

/// Read the ASCII tags `only` of IFD0 of a TIFF-structured file (DCP magic
/// 0x4352 or TIFF 42).
fn read_ifd0(path: &Path, only: &[u16]) -> Result<HashMap<u16, String>> {
    let mut f = File::open(path)?;
    let mut hdr = [0u8; 8];
    f.read_exact(&mut hdr)?;
    let le = match &hdr[..2] {
        b"II" => true,
        b"MM" => false,
        _ => bail!("not a TIFF/DCP file"),
    };
    let u16_ = |b: &[u8]| {
        if le {
            u16::from_le_bytes([b[0], b[1]])
        } else {
            u16::from_be_bytes([b[0], b[1]])
        }
    };
    let u32_ = |b: &[u8]| {
        if le {
            u32::from_le_bytes([b[0], b[1], b[2], b[3]])
        } else {
            u32::from_be_bytes([b[0], b[1], b[2], b[3]])
        }
    };
    let magic = u16_(&hdr[2..4]);
    if magic != 0x4352 && magic != 42 && magic != 0x4f52 && magic != 0x5352 {
        bail!("unexpected TIFF magic {magic:#x}");
    }
    let off = u32_(&hdr[4..8]) as u64;
    f.seek(SeekFrom::Start(off))?;
    let mut nb = [0u8; 2];
    f.read_exact(&mut nb)?;
    let n = u16_(&nb) as usize;
    let mut entries = vec![0u8; n * 12];
    f.read_exact(&mut entries)?;
    let mut out = HashMap::new();
    for e in entries.chunks_exact(12) {
        let tag = u16_(&e[0..2]);
        if !only.is_empty() && !only.contains(&tag) {
            continue;
        }
        if u16_(&e[2..4]) != 2 {
            continue;
        }
        let total = u32_(&e[4..8]) as usize;
        if total > 1 << 16 {
            continue;
        }
        let data = if total <= 4 {
            e[8..8 + total].to_vec()
        } else {
            let mut d = vec![0u8; total];
            f.seek(SeekFrom::Start(u32_(&e[8..12]) as u64))?;
            f.read_exact(&mut d)?;
            d
        };
        let val = String::from_utf8_lossy(&data)
            .trim_end_matches('\0')
            .trim()
            .to_string();
        out.insert(tag, val);
    }
    Ok(out)
}

mod tag {
    pub const MAKE: u16 = 271;
    pub const MODEL: u16 = 272;
    pub const SOFTWARE: u16 = 305;
    pub const UNIQUE_CAMERA_MODEL: u16 = 50708;
    pub const PROFILE_NAME: u16 = 50936;
}

/// Camera identification strings from a TIFF-based raw (DNG, NEF, CR2, ARW …).
pub fn tiff_camera_ids(path: &Path) -> Vec<String> {
    let Ok(t) = read_ifd0(
        path,
        &[
            tag::MAKE,
            tag::MODEL,
            tag::UNIQUE_CAMERA_MODEL,
            tag::SOFTWARE,
        ],
    ) else {
        return vec![];
    };
    let s = |k| t.get(&k).cloned().unwrap_or_default();
    let (make, model, ucm, sw) = (
        s(tag::MAKE),
        s(tag::MODEL),
        s(tag::UNIQUE_CAMERA_MODEL),
        s(tag::SOFTWARE),
    );
    // The plugin's "Update VueScan/SilverFast DNGs" utility rewrites these
    // DNGs' UniqueCameraModel so Lightroom picks the generic scanner profile.
    let scanner = if sw.contains("VueScan") {
        "Vuescan RAW"
    } else if sw.contains("SilverFast") {
        "Silverfast RAW"
    } else {
        ""
    };
    let mut v = vec![scanner.to_string(), ucm, format!("{make} {model}"), model];
    v.retain(|x| !x.trim().is_empty());
    v
}

// ---- profile ----------------------------------------------------------------------

/// A camera profile matched to a raw file. Only its identity is used: the
/// conversion works on the camera matrix and density, so the profile's
/// tone curve and look tables (designed for scenes, not negatives) are not.
pub struct DcpProfile {
    pub camera: String,
    pub path: PathBuf,
}

impl DcpProfile {
    pub fn load(path: &Path) -> Result<Self> {
        let t = read_ifd0(path, &[tag::UNIQUE_CAMERA_MODEL])?;
        let Some(camera) = t.get(&tag::UNIQUE_CAMERA_MODEL) else {
            bail!("not a camera profile");
        };
        Ok(Self {
            camera: camera.clone(),
            path: path.to_path_buf(),
        })
    }
}

// ---- library / matching -------------------------------------------------------------

fn normalize(s: &str) -> String {
    let lower = s.to_lowercase();
    let mut words: Vec<&str> = lower
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .collect();
    words.retain(|w| {
        !matches!(
            *w,
            "corporation"
                | "corp"
                | "company"
                | "co"
                | "ltd"
                | "inc"
                | "imaging"
                | "camera"
                | "digital"
                | "optical"
        )
    });
    // "NIKON NIKON D750" → "nikon d750"
    words.dedup();
    words.concat()
}

/// Ranking of profile versions: the plugin uses "Negative Lab v2.3".
fn version_rank(name: &str, file: &str) -> u8 {
    let f = file.to_lowercase();
    if name.contains("v2.3") || f.contains("2.3") {
        4
    } else if f.contains("v2.1") || f.contains(" 2.1") {
        3
    } else if f.contains(" v2") {
        2
    } else {
        1
    }
}

struct Library {
    /// normalised camera name → (rank, path)
    index: HashMap<String, (u8, PathBuf)>,
    cache: Mutex<HashMap<PathBuf, Option<Arc<DcpProfile>>>>,
}

fn profile_dirs() -> Vec<PathBuf> {
    let mut dirs_found = Vec::new();
    let rel = Path::new("NLP Lightroom Plugin")
        .join("LR Files")
        .join("CameraRaw")
        .join("CameraProfiles");
    let mut cands: Vec<PathBuf> = Vec::new();
    if let Ok(d) = std::env::var("NLP_DCP_DIR") {
        cands.push(PathBuf::from(d));
    }
    if let Some(l) = super::lut::LutLibrary::global()
        .read()
        .ok()
        .and_then(|l| l.dir().map(|d| d.to_path_buf()))
    {
        // …/CameraRaw/Settings/NLP Enhanced Settings → …/CameraRaw/CameraProfiles
        if let Some(cr) = l.parent().and_then(|p| p.parent()) {
            cands.push(cr.join("CameraProfiles"));
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        for a in exe.ancestors().skip(1).take(6) {
            cands.push(a.join(&rel));
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        for a in cwd.ancestors().take(4) {
            cands.push(a.join(&rel));
        }
    }
    if let Some(h) = dirs::home_dir() {
        cands.push(h.join("Library/Application Support/Adobe/CameraRaw/CameraProfiles"));
    }
    if let Some(d) = dirs::data_dir() {
        cands.push(d.join("Adobe/CameraRaw/CameraProfiles"));
    }
    for c in cands {
        if c.is_dir() {
            let c = c.canonicalize().unwrap_or(c);
            if !dirs_found.contains(&c) {
                dirs_found.push(c);
            }
        }
    }
    dirs_found
}

fn collect_dcps(dir: &Path, out: &mut Vec<PathBuf>, depth: usize) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() && depth < 4 {
            collect_dcps(&p, out, depth + 1);
        } else if p
            .extension()
            .and_then(|x| x.to_str())
            .map(|x| x.eq_ignore_ascii_case("dcp"))
            .unwrap_or(false)
        {
            out.push(p);
        }
    }
}

fn library() -> &'static Library {
    static L: OnceLock<Library> = OnceLock::new();
    L.get_or_init(|| {
        use rayon::prelude::*;
        let mut files = Vec::new();
        for d in profile_dirs() {
            collect_dcps(&d, &mut files, 0);
        }
        let entries: Vec<(String, u8, PathBuf)> = files
            .par_iter()
            .filter_map(|p| {
                let t = read_ifd0(p, &[tag::UNIQUE_CAMERA_MODEL, tag::PROFILE_NAME]).ok()?;
                let name = t.get(&tag::PROFILE_NAME).cloned().unwrap_or_default();
                if !name.starts_with("Negative Lab") {
                    return None;
                }
                let cam = t.get(&tag::UNIQUE_CAMERA_MODEL)?.clone();
                let file = p.file_name()?.to_string_lossy().to_string();
                Some((normalize(&cam), version_rank(&name, &file), p.clone()))
            })
            .collect();
        let mut index: HashMap<String, (u8, PathBuf)> = HashMap::new();
        for (k, r, p) in entries {
            match index.get(&k) {
                Some((r0, _)) if *r0 >= r => {}
                _ => {
                    index.insert(k, (r, p));
                }
            }
        }
        Library {
            index,
            cache: Mutex::new(HashMap::new()),
        }
    })
}

/// Candidate names for matching, most specific first.
fn candidates(info: &SourceInfo) -> Vec<String> {
    let mut v: Vec<String> = info.camera_ids.clone();
    v.push(format!("{} {}", info.make, info.model));
    v.push(info.model.clone());
    v.into_iter()
        .map(|s| normalize(&s))
        .filter(|s| !s.is_empty())
        .collect()
}

/// Find and load the Negative Lab profile for a camera, if one exists.
pub fn profile_for(info: &SourceInfo) -> Option<Arc<DcpProfile>> {
    let lib = library();
    let path = candidates(info)
        .iter()
        .find_map(|k| lib.index.get(k).map(|(_, p)| p.clone()))?;
    let mut cache = lib.cache.lock().ok()?;
    cache
        .entry(path.clone())
        .or_insert_with(|| match DcpProfile::load(&path) {
            Ok(p) => Some(Arc::new(p)),
            Err(e) => {
                eprintln!("DCP load failed for {}: {e:#}", path.display());
                None
            }
        })
        .clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalization_matches_common_names() {
        assert_eq!(
            normalize("NIKON CORPORATION NIKON D750"),
            normalize("Nikon D750")
        );
        assert_eq!(
            normalize("Canon Canon EOS 5D Mark III"),
            normalize("Canon EOS 5D Mark III")
        );
        assert_eq!(normalize("SONY ILCE-7M3"), normalize("Sony ILCE-7M3"));
        assert_eq!(
            normalize("OLYMPUS IMAGING CORP. E-M1"),
            normalize("Olympus E-M1")
        );
    }

    #[test]
    fn plugin_profiles_match_cameras() {
        if library().index.is_empty() {
            return;
        }
        let info = |make: &str, model: &str| SourceInfo {
            make: make.into(),
            model: model.into(),
            ..Default::default()
        };
        let p = profile_for(&info("Nikon", "D750")).expect("Nikon D750 profile");
        assert!(
            p.path.to_string_lossy().contains("2.3"),
            "{}",
            p.path.display()
        );
        let p = profile_for(&SourceInfo {
            camera_ids: vec!["Vuescan RAW".into(), "Plustek OpticFilm 7600i".into()],
            ..Default::default()
        })
        .expect("scanner profile");
        assert_eq!(p.camera, "Vuescan RAW");
        let p = profile_for(&SourceInfo {
            camera_ids: vec!["Plustek OpticFilm 7600i".into()],
            ..Default::default()
        })
        .expect("scanner model profile");
        assert!(p.camera.contains("7600i"));
        assert!(profile_for(&info("Nonexistent", "Camera 9000")).is_none());
    }
}
