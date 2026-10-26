//! Loader for the plugin's Enhanced Profile looks
//! (`LR Files/CameraRaw/Settings/NLP Enhanced Settings/*.xmp`).
//!
//! Each look XMP embeds an Adobe "big table" in `crs:Table_<digest>`:
//! Adobe base85 → [u32 uncompressed length][zlib stream] → little-endian
//! `u32 type(1), u32 version, u32 dims(3), u32 divisions`, then
//! `divisions³ × 3` u16 samples stored as offsets from the identity grid
//! (r outermost, b innermost), followed by `u32 primaries, u32 gamma,
//! u32 gamut, f64 min_amount, f64 max_amount`.

use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock, RwLock};

use anyhow::{Context, Result, anyhow, bail};

use super::settings::Lut;

const B85: &[u8; 85] =
    b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?`'|()[]{}@%$#";

fn base85_decode(s: &str) -> Vec<u8> {
    let mut map = [255u8; 256];
    for (i, &c) in B85.iter().enumerate() {
        map[c as usize] = i as u8;
    }
    let mut out = Vec::with_capacity(s.len() * 4 / 5 + 4);
    let (mut x, mut n, mut mul) = (0u64, 0, 1u64);
    for &c in s.as_bytes() {
        let d = map[c as usize];
        if d == 255 {
            continue;
        }
        x += d as u64 * mul;
        mul *= 85;
        n += 1;
        if n == 5 {
            out.extend_from_slice(&(x as u32).to_le_bytes());
            (x, n, mul) = (0, 0, 1);
        }
    }
    if n > 1 {
        out.extend_from_slice(&(x as u32).to_le_bytes()[..n - 1]);
    }
    out
}

fn xml_unescape(s: &str) -> String {
    s.replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
}

/// Colour encoding of the table input/output (DNG SDK enums).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TablePrimaries {
    Srgb,
    AdobeRgb,
    ProPhoto,
    P3,
    Rec2020,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TableGamma {
    Linear,
    Srgb,
    G18,
    G22,
}

pub struct RgbTable {
    pub div: usize,
    /// Samples normalised to 0..1, index ((r*div)+g)*div+b.
    pub data: Vec<[f32; 3]>,
    pub primaries: TablePrimaries,
    pub gamma: TableGamma,
    pub max_amount: f32,
}

impl RgbTable {
    pub fn parse_xmp(path: &Path) -> Result<Self> {
        let text =
            std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
        let start = text
            .find("crs:Table_")
            .ok_or_else(|| anyhow!("no RGB table in {}", path.display()))?;
        let rest = &text[start..];
        let q0 = rest.find('"').ok_or_else(|| anyhow!("malformed table"))? + 1;
        let q1 = rest[q0..]
            .find('"')
            .ok_or_else(|| anyhow!("malformed table"))?
            + q0;
        let bin = base85_decode(&xml_unescape(&rest[q0..q1]));
        if bin.len() < 8 {
            bail!("table too short");
        }
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(&bin[4..])
            .read_to_end(&mut raw)
            .context("inflate table")?;
        let u32_at = |o: usize| -> Result<u32> {
            raw.get(o..o + 4)
                .map(|b| u32::from_le_bytes(b.try_into().unwrap()))
                .ok_or_else(|| anyhow!("truncated table"))
        };
        let (kind, dims, div) = (u32_at(0)?, u32_at(8)?, u32_at(12)? as usize);
        if kind != 1 || dims != 3 || !(2..=64).contains(&div) {
            bail!("unsupported table (type {kind}, dims {dims}, div {div})");
        }
        let n = div * div * div;
        let body = 16;
        if raw.len() < body + n * 6 {
            bail!("truncated table body");
        }
        let nop = |i: usize| ((i * 0xFFFF + (div - 1) / 2) / (div - 1)) as u32;
        let mut data = Vec::with_capacity(n);
        let mut o = body;
        for r in 0..div {
            for g in 0..div {
                for b in 0..div {
                    let mut px = [0.0f32; 3];
                    for (k, idx) in [r, g, b].into_iter().enumerate() {
                        let d = u16::from_le_bytes([raw[o], raw[o + 1]]) as u32;
                        o += 2;
                        px[k] = (d.wrapping_add(nop(idx)) & 0xFFFF) as f32 / 65535.0;
                    }
                    data.push(px);
                }
            }
        }
        let tail = |i: usize| u32_at(o + i * 4).unwrap_or(0);
        let primaries = match tail(0) {
            0 => TablePrimaries::Srgb,
            1 => TablePrimaries::AdobeRgb,
            3 => TablePrimaries::P3,
            4 => TablePrimaries::Rec2020,
            _ => TablePrimaries::ProPhoto,
        };
        let gamma = match tail(1) {
            0 => TableGamma::Linear,
            1 => TableGamma::Srgb,
            3 => TableGamma::G22,
            _ => TableGamma::G18,
        };
        let max_amount = raw
            .get(o + 20..o + 28)
            .map(|b| f64::from_le_bytes(b.try_into().unwrap()) as f32)
            .unwrap_or(2.0);
        Ok(Self {
            div,
            data,
            primaries,
            gamma,
            max_amount,
        })
    }

    /// Tetrahedral interpolation; input/output in the table's encoding (0..1).
    #[inline]
    pub fn lookup(&self, c: [f32; 3]) -> [f32; 3] {
        let m = (self.div - 1) as f32;
        let f = [
            c[0].clamp(0.0, 1.0) * m,
            c[1].clamp(0.0, 1.0) * m,
            c[2].clamp(0.0, 1.0) * m,
        ];
        let i = f.map(|v| (v as usize).min(self.div - 2));
        let t = [f[0] - i[0] as f32, f[1] - i[1] as f32, f[2] - i[2] as f32];
        let d = self.div;
        let at = |dr: usize, dg: usize, db: usize| {
            self.data[((i[0] + dr) * d + i[1] + dg) * d + i[2] + db]
        };
        let c000 = at(0, 0, 0);
        let c111 = at(1, 1, 1);
        let (a, b, ca, cb, wa, wb, wc);
        let (tr, tg, tb) = (t[0], t[1], t[2]);
        if tr >= tg {
            if tg >= tb {
                (a, b) = (at(1, 0, 0), at(1, 1, 0));
                (wa, wb, wc) = (tr - tg, tg - tb, tb);
            } else if tr >= tb {
                (a, b) = (at(1, 0, 0), at(1, 0, 1));
                (wa, wb, wc) = (tr - tb, tb - tg, tg);
            } else {
                (a, b) = (at(0, 0, 1), at(1, 0, 1));
                (wa, wb, wc) = (tb - tr, tr - tg, tg);
            }
        } else if tb > tg {
            (a, b) = (at(0, 0, 1), at(0, 1, 1));
            (wa, wb, wc) = (tb - tg, tg - tr, tr);
        } else if tb > tr {
            (a, b) = (at(0, 1, 0), at(0, 1, 1));
            (wa, wb, wc) = (tg - tb, tb - tr, tr);
        } else {
            (a, b) = (at(0, 1, 0), at(1, 1, 0));
            (wa, wb, wc) = (tg - tr, tr - tb, tb);
        }
        (ca, cb) = (a, b);
        let w0 = 1.0 - wa - wb - wc;
        [0, 1, 2].map(|k| w0 * c000[k] + wa * ca[k] + wb * cb[k] + wc * c111[k])
    }
}

/// File stem of the look for a LUT + post-saturation level (1..=10, 5 = base),
/// matching `enhanced_profile_tables` in NegativePresets.lua.
pub fn look_file_stem(lut: Lut, sat: u8) -> Option<String> {
    let base = match lut {
        Lut::None => return None,
        Lut::Natural => "Natural",
        Lut::Frontier => "Frontier",
        Lut::Crystal => "Crystal",
        Lut::Pakon => "Pakon",
    };
    let sat = sat.clamp(1, 10);
    Some(if sat == 5 {
        format!("Negative Lab v2.3 - {base}")
    } else {
        format!("Negative Lab v2.3 - {base} - Sat{sat}")
    })
}

/// Directory holding the looks, discovered once and overridable from the UI.
pub struct LutLibrary {
    dir: Option<PathBuf>,
    cache: HashMap<String, Option<Arc<RgbTable>>>,
}

const SUBDIR: &str = "NLP Enhanced Settings";

fn candidate_dirs() -> Vec<PathBuf> {
    let mut v = Vec::new();
    if let Ok(d) = std::env::var("NLP_LUT_DIR") {
        v.push(PathBuf::from(d));
    }
    if let Some(cfg) = dirs::config_dir() {
        if let Ok(s) = std::fs::read_to_string(cfg.join("nlp-studio").join("lut_dir.txt")) {
            v.push(PathBuf::from(s.trim()));
        }
    }
    let plugin_rel = Path::new("NLP Lightroom Plugin")
        .join("LR Files")
        .join("CameraRaw")
        .join("Settings")
        .join(SUBDIR);
    if let Ok(exe) = std::env::current_exe() {
        for anc in exe.ancestors().skip(1).take(6) {
            v.push(anc.join("luts"));
            v.push(anc.join(&plugin_rel));
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        for anc in cwd.ancestors().take(4) {
            v.push(anc.join(&plugin_rel));
        }
    }
    if let Some(home) = dirs::home_dir() {
        v.push(
            home.join("Library/Application Support/Adobe/CameraRaw/Settings")
                .join(SUBDIR),
        );
    }
    if let Some(data) = dirs::data_dir() {
        v.push(data.join("Adobe/CameraRaw/Settings").join(SUBDIR));
    }
    v
}

fn dir_has_looks(d: &Path) -> bool {
    d.join("Negative Lab v2.3 - Frontier.xmp").is_file()
}

impl LutLibrary {
    fn discover() -> Self {
        let dir = candidate_dirs().into_iter().find(|d| dir_has_looks(d));
        Self {
            dir,
            cache: HashMap::new(),
        }
    }

    pub fn global() -> &'static RwLock<LutLibrary> {
        static L: OnceLock<RwLock<LutLibrary>> = OnceLock::new();
        L.get_or_init(|| RwLock::new(LutLibrary::discover()))
    }

    pub fn dir(&self) -> Option<&Path> {
        self.dir.as_deref()
    }

    /// Point the library at a folder (persisted for future runs).
    pub fn set_dir(&mut self, d: PathBuf) -> Result<()> {
        let d = if dir_has_looks(&d) {
            d
        } else if dir_has_looks(&d.join(SUBDIR)) {
            d.join(SUBDIR)
        } else {
            bail!(
                "No 'Negative Lab v2.3 - *.xmp' looks found in {}",
                d.display()
            )
        };
        if let Some(cfg) = dirs::config_dir() {
            let p = cfg.join("nlp-studio");
            let _ = std::fs::create_dir_all(&p);
            let _ = std::fs::write(p.join("lut_dir.txt"), d.to_string_lossy().as_bytes());
        }
        self.dir = Some(d);
        self.cache.clear();
        Ok(())
    }

    pub fn get(&mut self, lut: Lut, sat: u8) -> Option<Arc<RgbTable>> {
        let stem = look_file_stem(lut, sat)?;
        let dir = self.dir.clone()?;
        self.cache
            .entry(stem.clone())
            .or_insert_with(|| {
                let p = dir.join(format!("{stem}.xmp"));
                match RgbTable::parse_xmp(&p) {
                    Ok(t) => Some(Arc::new(t)),
                    Err(e) => {
                        eprintln!("LUT load failed: {e:#}");
                        None
                    }
                }
            })
            .clone()
    }
}

/// Convenience: fetch a table from the global library.
pub fn table_for(lut: Lut, sat: u8) -> Option<Arc<RgbTable>> {
    LutLibrary::global().write().ok()?.get(lut, sat)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base85_roundtrip_small() {
        // encode 4 bytes manually: little-endian u32 written as 5 base85 digits, LSD first
        let x: u32 = 0x1234_5678;
        let mut s = String::new();
        let mut v = x as u64;
        for _ in 0..5 {
            s.push(B85[(v % 85) as usize] as char);
            v /= 85;
        }
        assert_eq!(base85_decode(&s), x.to_le_bytes().to_vec());
    }

    #[test]
    fn plugin_tables_load_if_present() {
        let mut lib = LutLibrary::discover();
        if lib.dir().is_none() {
            return;
        }
        let t = lib.get(Lut::Frontier, 5).expect("frontier table");
        assert_eq!(t.div, 32);
        let g = t.lookup([0.5, 0.5, 0.5]);
        assert!(
            (g[0] - g[1]).abs() < 0.01 && (g[1] - g[2]).abs() < 0.01,
            "gray stays gray: {g:?}"
        );
        for s in 1..=10 {
            assert!(lib.get(Lut::Pakon, s).is_some(), "pakon sat{s}");
        }
    }
}
