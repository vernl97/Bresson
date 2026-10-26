//! DNG Camera Profiles (`.dcp`) from the plugin's
//! `LR Files/CameraRaw/CameraProfiles` folder ("Negative Lab v2.3" profiles).
//!
//! A profile is matched to a raw file by camera make/model (or the DNG
//! `UniqueCameraModel`) and applied following the DNG specification:
//! camera neutral → white point → interpolated colour matrix → XYZ D50 →
//! ProPhoto linear → hue/sat map → baseline exposure offset → look table →
//! profile tone curve (Adobe's hue-preserving RGB tone).

use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

use anyhow::{Result, anyhow, bail};

use super::color::{self, Mat3};
use super::curve::Spline;
use super::decode::SourceInfo;

// ---- minimal TIFF / DCP IFD reader ----------------------------------------------

#[derive(Debug, Clone)]
enum Val {
    Ascii(String),
    Ints(Vec<i64>),
    Floats(Vec<f64>),
}

impl Val {
    fn ascii(&self) -> Option<&str> {
        match self {
            Val::Ascii(s) => Some(s.as_str()),
            _ => None,
        }
    }
    fn floats(&self) -> Vec<f64> {
        match self {
            Val::Floats(v) => v.clone(),
            Val::Ints(v) => v.iter().map(|x| *x as f64).collect(),
            Val::Ascii(_) => vec![],
        }
    }
    fn int(&self) -> Option<i64> {
        match self {
            Val::Ints(v) => v.first().copied(),
            Val::Floats(v) => v.first().map(|x| *x as i64),
            _ => None,
        }
    }
}

/// Read IFD0 of a TIFF-structured file (DCP magic 0x4352 or TIFF 42).
/// `only` restricts which tags are loaded (empty = all).
fn read_ifd0(path: &Path, only: &[u16]) -> Result<HashMap<u16, Val>> {
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
        let typ = u16_(&e[2..4]);
        let cnt = u32_(&e[4..8]) as usize;
        let size = match typ {
            1 | 2 | 6 | 7 => 1,
            3 | 8 => 2,
            4 | 9 | 11 => 4,
            5 | 10 | 12 => 8,
            _ => continue,
        };
        let total = size * cnt;
        if total > 64 << 20 {
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
        let ch = |i: usize, w: usize| &data[i * w..i * w + w];
        let val = match typ {
            2 => Val::Ascii(
                String::from_utf8_lossy(&data)
                    .trim_end_matches('\0')
                    .trim()
                    .to_string(),
            ),
            1 | 7 => Val::Ints(data.iter().map(|b| *b as i64).collect()),
            6 => Val::Ints(data.iter().map(|b| *b as i8 as i64).collect()),
            3 => Val::Ints((0..cnt).map(|i| u16_(ch(i, 2)) as i64).collect()),
            8 => Val::Ints((0..cnt).map(|i| u16_(ch(i, 2)) as i16 as i64).collect()),
            4 => Val::Ints((0..cnt).map(|i| u32_(ch(i, 4)) as i64).collect()),
            9 => Val::Ints((0..cnt).map(|i| u32_(ch(i, 4)) as i32 as i64).collect()),
            5 => Val::Floats(
                (0..cnt)
                    .map(|i| {
                        let (a, b) = (u32_(&ch(i, 8)[..4]), u32_(&ch(i, 8)[4..]));
                        if b == 0 { 0.0 } else { a as f64 / b as f64 }
                    })
                    .collect(),
            ),
            10 => Val::Floats(
                (0..cnt)
                    .map(|i| {
                        let (a, b) = (u32_(&ch(i, 8)[..4]) as i32, u32_(&ch(i, 8)[4..]) as i32);
                        if b == 0 { 0.0 } else { a as f64 / b as f64 }
                    })
                    .collect(),
            ),
            11 => Val::Floats(
                (0..cnt)
                    .map(|i| f32::from_bits(u32_(ch(i, 4))) as f64)
                    .collect(),
            ),
            12 => Val::Floats(
                (0..cnt)
                    .map(|i| {
                        let b = ch(i, 8);
                        let bits = if le {
                            u64::from_le_bytes(b.try_into().unwrap())
                        } else {
                            u64::from_be_bytes(b.try_into().unwrap())
                        };
                        f64::from_bits(bits)
                    })
                    .collect(),
            ),
            _ => continue,
        };
        out.insert(tag, val);
    }
    Ok(out)
}

mod tag {
    pub const MAKE: u16 = 271;
    pub const MODEL: u16 = 272;
    pub const SOFTWARE: u16 = 305;
    pub const UNIQUE_CAMERA_MODEL: u16 = 50708;
    pub const COLOR_MATRIX_1: u16 = 50721;
    pub const COLOR_MATRIX_2: u16 = 50722;
    pub const CALIBRATION_ILLUMINANT_1: u16 = 50778;
    pub const CALIBRATION_ILLUMINANT_2: u16 = 50779;
    pub const PROFILE_NAME: u16 = 50936;
    pub const HUE_SAT_MAP_DIMS: u16 = 50937;
    pub const HUE_SAT_MAP_DATA_1: u16 = 50938;
    pub const HUE_SAT_MAP_DATA_2: u16 = 50939;
    pub const PROFILE_TONE_CURVE: u16 = 50940;
    pub const FORWARD_MATRIX_1: u16 = 50964;
    pub const FORWARD_MATRIX_2: u16 = 50965;
    pub const LOOK_TABLE_DIMS: u16 = 50981;
    pub const LOOK_TABLE_DATA: u16 = 50982;
    pub const HUE_SAT_MAP_ENCODING: u16 = 51107;
    pub const LOOK_TABLE_ENCODING: u16 = 51108;
    pub const BASELINE_EXPOSURE_OFFSET: u16 = 51109;
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
    let s = |k| t.get(&k).and_then(|v| v.ascii()).unwrap_or("").to_string();
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

/// A hue / saturation / value delta table (HueSatMap or LookTable).
pub struct HsvTable {
    hue_div: usize,
    sat_div: usize,
    val_div: usize,
    /// [hue shift (degrees), saturation scale, value scale], index
    /// ((v * hue_div) + h) * sat_div + s.
    data: Vec<[f32; 3]>,
    srgb_value: bool,
}

impl HsvTable {
    fn from(dims: &Val, data: &Val, srgb: bool) -> Option<Self> {
        let d = dims.floats();
        if d.len() < 3 {
            return None;
        }
        let (h, s, v) = (d[0] as usize, d[1] as usize, d[2] as usize);
        let f = data.floats();
        if h == 0 || s < 2 || v == 0 || f.len() != h * s * v * 3 {
            return None;
        }
        let data = f
            .chunks_exact(3)
            .map(|c| [c[0] as f32, c[1] as f32, c[2] as f32])
            .collect();
        Some(Self {
            hue_div: h,
            sat_div: s,
            val_div: v,
            data,
            srgb_value: srgb,
        })
    }

    /// `RefBaselineHueSatMap` from the DNG SDK, on linear ProPhoto RGB.
    fn apply(&self, rgb: [f32; 3]) -> [f32; 3] {
        let (mut h, mut s, v) = rgb_to_hsv_dng(rgb);
        let mut vv = if self.srgb_value {
            color::srgb_encode(v.min(1.0))
        } else {
            v
        };
        let h_scaled = h * self.hue_div as f32 / 6.0;
        let s_scaled = s * (self.sat_div - 1) as f32;
        let mut h0 = h_scaled as usize;
        let mut h1 = h0 + 1;
        if h0 >= self.hue_div - 1 {
            h0 = self.hue_div - 1;
            h1 = 0;
        }
        let hf = (h_scaled - h_scaled.floor()).clamp(0.0, 1.0);
        let s0 = (s_scaled as usize).min(self.sat_div - 2);
        let sf = (s_scaled - s0 as f32).clamp(0.0, 1.0);
        let at = |vi: usize, hi: usize, si: usize| {
            self.data[(vi * self.hue_div + hi) * self.sat_div + si]
        };
        let lerp3 = |a: [f32; 3], b: [f32; 3], t: f32| {
            [
                a[0] + (b[0] - a[0]) * t,
                a[1] + (b[1] - a[1]) * t,
                a[2] + (b[2] - a[2]) * t,
            ]
        };
        let plane = |vi: usize| {
            let a = lerp3(at(vi, h0, s0), at(vi, h0, s0 + 1), sf);
            let b = lerp3(at(vi, h1, s0), at(vi, h1, s0 + 1), sf);
            lerp3(a, b, hf)
        };
        let d = if self.val_div > 1 {
            let v_scaled = vv.min(1.0) * (self.val_div - 1) as f32;
            let v0 = (v_scaled as usize).min(self.val_div - 2);
            let vf = (v_scaled - v0 as f32).clamp(0.0, 1.0);
            lerp3(plane(v0), plane(v0 + 1), vf)
        } else {
            plane(0)
        };
        h += d[0] * 6.0 / 360.0;
        h = h.rem_euclid(6.0);
        s = (s * d[1]).clamp(0.0, 1.0);
        vv *= d[2];
        let v2 = if self.srgb_value {
            color::srgb_decode(vv)
        } else {
            vv
        };
        hsv_to_rgb_dng(h, s, v2)
    }
}

fn rgb_to_hsv_dng(c: [f32; 3]) -> (f32, f32, f32) {
    let (r, g, b) = (c[0], c[1], c[2]);
    let v = r.max(g).max(b);
    let gap = v - r.min(g).min(b);
    if v <= 0.0 || gap <= 0.0 {
        return (0.0, 0.0, v.max(0.0));
    }
    let mut h = if r == v {
        let h = (g - b) / gap;
        if h < 0.0 { h + 6.0 } else { h }
    } else if g == v {
        2.0 + (b - r) / gap
    } else {
        4.0 + (r - g) / gap
    };
    if h >= 6.0 {
        h -= 6.0;
    }
    (h, gap / v, v)
}

fn hsv_to_rgb_dng(h: f32, s: f32, v: f32) -> [f32; 3] {
    if s <= 0.0 {
        return [v, v, v];
    }
    let i = (h.floor() as i32).rem_euclid(6);
    let f = h - h.floor();
    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));
    match i {
        0 => [v, t, p],
        1 => [q, v, p],
        2 => [p, v, t],
        3 => [p, q, v],
        4 => [t, p, v],
        _ => [v, p, q],
    }
}

pub struct DcpProfile {
    pub name: String,
    pub camera: String,
    pub path: PathBuf,
    cm1: Mat3,
    cm2: Option<Mat3>,
    fm1: Option<Mat3>,
    fm2: Option<Mat3>,
    temp1: f64,
    temp2: f64,
    hue_sat: Option<HsvTable>,
    look: Option<HsvTable>,
    tone: Option<Vec<f32>>,
    exposure_offset: f64,
}

fn mat(v: Option<&Val>) -> Option<Mat3> {
    let f = v?.floats();
    (f.len() >= 9).then(|| [[f[0], f[1], f[2]], [f[3], f[4], f[5]], [f[6], f[7], f[8]]])
}

/// Correlated colour temperature (K) of an EXIF light source code.
fn illuminant_temp(code: i64) -> f64 {
    match code {
        1 | 4 | 9 => 5500.0,
        2 | 14 => 4150.0,
        3 | 17 => 2856.0,
        10 => 6500.0,
        11 => 7500.0,
        12 => 6430.0,
        13 => 5000.0,
        15 => 3450.0,
        18 => 4874.0,
        19 => 6774.0,
        20 => 5503.0,
        21 => 6504.0,
        22 => 7504.0,
        23 => 5003.0,
        24 => 3200.0,
        _ => 5000.0,
    }
}

/// McCamy's approximation of CCT from xy.
fn cct(xy: [f64; 2]) -> f64 {
    let n = (xy[0] - 0.3320) / (0.1858 - xy[1]);
    (449.0 * n.powi(3) + 3525.0 * n.powi(2) + 6823.3 * n + 5520.33).clamp(1500.0, 50000.0)
}

const TONE_LUT: usize = 4096;

impl DcpProfile {
    pub fn load(path: &Path) -> Result<Self> {
        let t = read_ifd0(path, &[])?;
        let cm1 = mat(t.get(&tag::COLOR_MATRIX_1))
            .ok_or_else(|| anyhow!("profile has no ColorMatrix1"))?;
        let srgb = |k| t.get(&k).and_then(|v| v.int()) == Some(1);
        let hue_sat = match (
            t.get(&tag::HUE_SAT_MAP_DIMS),
            t.get(&tag::HUE_SAT_MAP_DATA_2)
                .or(t.get(&tag::HUE_SAT_MAP_DATA_1)),
        ) {
            (Some(d), Some(v)) => HsvTable::from(d, v, srgb(tag::HUE_SAT_MAP_ENCODING)),
            _ => None,
        };
        let look = match (t.get(&tag::LOOK_TABLE_DIMS), t.get(&tag::LOOK_TABLE_DATA)) {
            (Some(d), Some(v)) => HsvTable::from(d, v, srgb(tag::LOOK_TABLE_ENCODING)),
            _ => None,
        };
        let tone = t
            .get(&tag::PROFILE_TONE_CURVE)
            .map(|v| v.floats())
            .filter(|f| f.len() >= 4)
            .map(|f| {
                let xs: Vec<f64> = f.iter().step_by(2).copied().collect();
                let ys: Vec<f64> = f.iter().skip(1).step_by(2).copied().collect();
                let sp = Spline::new(&xs, &ys);
                (0..TONE_LUT)
                    .map(|i| sp.eval(i as f64 / (TONE_LUT - 1) as f64).clamp(0.0, 1.0) as f32)
                    .collect()
            });
        let s = |k| t.get(&k).and_then(|v| v.ascii()).unwrap_or("").to_string();
        Ok(Self {
            name: s(tag::PROFILE_NAME),
            camera: s(tag::UNIQUE_CAMERA_MODEL),
            path: path.to_path_buf(),
            cm1,
            cm2: mat(t.get(&tag::COLOR_MATRIX_2)),
            fm1: mat(t.get(&tag::FORWARD_MATRIX_1)),
            fm2: mat(t.get(&tag::FORWARD_MATRIX_2)),
            temp1: illuminant_temp(
                t.get(&tag::CALIBRATION_ILLUMINANT_1)
                    .and_then(|v| v.int())
                    .unwrap_or(21),
            ),
            temp2: illuminant_temp(
                t.get(&tag::CALIBRATION_ILLUMINANT_2)
                    .and_then(|v| v.int())
                    .unwrap_or(21),
            ),
            hue_sat,
            look,
            tone,
            exposure_offset: t
                .get(&tag::BASELINE_EXPOSURE_OFFSET)
                .map(|v| v.floats())
                .and_then(|f| f.first().copied())
                .unwrap_or(0.0),
        })
    }

    /// Weight of matrix 1 for a white temperature (inverse-temperature interpolation).
    fn weight1(&self, temp: f64) -> f64 {
        if self.cm2.is_none() || (self.temp1 - self.temp2).abs() < 1.0 {
            return 1.0;
        }
        let (t1, t2) = (self.temp1, self.temp2);
        let g = (1.0 / temp - 1.0 / t2) / (1.0 / t1 - 1.0 / t2);
        g.clamp(0.0, 1.0)
    }

    fn blend(a: &Mat3, b: Option<&Mat3>, w: f64) -> Mat3 {
        match b {
            Some(b) => {
                std::array::from_fn(|i| std::array::from_fn(|j| a[i][j] * w + b[i][j] * (1.0 - w)))
            }
            None => *a,
        }
    }

    fn color_matrix(&self, temp: f64) -> Mat3 {
        Self::blend(&self.cm1, self.cm2.as_ref(), self.weight1(temp))
    }

    /// White point (xy) of a camera neutral, found iteratively (DNG spec).
    fn neutral_to_xy(&self, neutral: [f64; 3]) -> [f64; 2] {
        let mut xy = color::D50;
        for _ in 0..30 {
            let cm = self.color_matrix(cct(xy));
            let xyz = color::mat_vec(&color::mat_inv(&cm), neutral);
            let sum = xyz[0] + xyz[1] + xyz[2];
            if sum <= 0.0 {
                break;
            }
            let next = [xyz[0] / sum, xyz[1] / sum];
            if (next[0] - xy[0]).abs() + (next[1] - xy[1]).abs() < 1e-7 {
                xy = next;
                break;
            }
            xy = next;
        }
        xy
    }

    /// Camera neutral of a white point (used when no film base is applied).
    pub fn xy_to_neutral(&self, xy: [f64; 2]) -> [f64; 3] {
        let cm = self.color_matrix(cct(xy));
        let n = color::mat_vec(&cm, color::xy_to_xyz(xy));
        let m = n[0].max(n[1]).max(n[2]).max(1e-9);
        n.map(|v| v / m)
    }

    /// Camera → linear ProPhoto (D50) matrix for a camera neutral; the
    /// neutral maps to equal RGB with green = 1.
    fn camera_to_prophoto(&self, neutral: [f64; 3]) -> Mat3 {
        let xy = self.neutral_to_xy(neutral);
        let temp = cct(xy);
        let xyz_d50 = if let Some(fm1) = &self.fm1 {
            let fm = Self::blend(fm1, self.fm2.as_ref(), self.weight1(temp));
            let d = [
                [1.0 / neutral[0].max(1e-9), 0.0, 0.0],
                [0.0, 1.0 / neutral[1].max(1e-9), 0.0],
                [0.0, 0.0, 1.0 / neutral[2].max(1e-9)],
            ];
            color::mat_mul(&fm, &d)
        } else {
            let cam_to_xyz = color::mat_inv(&self.color_matrix(temp));
            color::mat_mul(&color::bradford(xy, color::D50), &cam_to_xyz)
        };
        let mut m = color::mat_mul(
            &color::mat_inv(&color::rgb_to_xyz(&color::PROPHOTO)),
            &xyz_d50,
        );
        let w = color::mat_vec(&m, neutral);
        let k = 1.0 / w[1].max(1e-9);
        for row in m.iter_mut() {
            for v in row.iter_mut() {
                *v *= k;
            }
        }
        m
    }
}

/// A profile prepared for one image: camera → linear MelissaRGB.
pub struct DcpTransform {
    profile: Arc<DcpProfile>,
    cam_to_pp: [[f32; 3]; 3],
    pp_to_mel: [[f32; 3]; 3],
    exposure: f32,
}

impl DcpTransform {
    /// `neutral`: camera values of something that should be neutral (the film
    /// base). `level_ref`: camera colour that should land at `level` (the
    /// brightest channel), to keep the film base just below clipping.
    pub fn new(
        profile: Arc<DcpProfile>,
        neutral: [f64; 3],
        level_ref: Option<[f32; 3]>,
        level: f32,
    ) -> Self {
        let mut m = profile.camera_to_prophoto(neutral);
        if let Some(r) = level_ref {
            let v = color::mat_vec(&m, r.map(|x| x as f64));
            let k = level as f64 / v[0].max(v[1]).max(v[2]).max(1e-9);
            for row in m.iter_mut() {
                for x in row.iter_mut() {
                    *x *= k;
                }
            }
        }
        Self {
            exposure: 2f32.powf(profile.exposure_offset as f32),
            cam_to_pp: color::to_f32(&m),
            pp_to_mel: color::to_f32(&color::rgb_to_rgb(&color::PROPHOTO, &color::MELISSA)),
            profile,
        }
    }

    /// Camera RGB → linear MelissaRGB.
    #[inline]
    pub fn apply(&self, cam: [f32; 3]) -> [f32; 3] {
        let p = &self.profile;
        let mut rgb = color::mul3(&self.cam_to_pp, cam).map(|v| v.max(0.0));
        if let Some(t) = &p.hue_sat {
            rgb = t.apply(rgb);
        }
        rgb = rgb.map(|v| v * self.exposure);
        if let Some(t) = &p.look {
            rgb = t.apply(rgb);
        }
        if let Some(lut) = &p.tone {
            rgb = rgb_tone(rgb, lut);
        }
        color::mul3(&self.pp_to_mel, rgb)
    }
}

#[inline]
fn tone_eval(lut: &[f32], v: f32) -> f32 {
    if v >= 1.0 {
        return lut[TONE_LUT - 1] + (v - 1.0);
    }
    let x = v.max(0.0) * (TONE_LUT - 1) as f32;
    let i = (x as usize).min(TONE_LUT - 2);
    let t = x - i as f32;
    lut[i] + (lut[i + 1] - lut[i]) * t
}

/// Adobe's hue-preserving RGB tone curve (`RefBaselineRGBTone`): the curve is
/// applied to the largest and smallest channel, the middle one is placed
/// proportionally between them.
fn rgb_tone(c: [f32; 3], lut: &[f32]) -> [f32; 3] {
    let mut idx = [0usize, 1, 2];
    idx.sort_by(|a, b| {
        c[*b]
            .partial_cmp(&c[*a])
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let (lg, md, sm) = (c[idx[0]], c[idx[1]], c[idx[2]]);
    let lg2 = tone_eval(lut, lg);
    let sm2 = tone_eval(lut, sm);
    let md2 = if lg > sm {
        sm2 + (lg2 - sm2) * (md - sm) / (lg - sm)
    } else {
        sm2
    };
    let mut o = [0.0f32; 3];
    o[idx[0]] = lg2;
    o[idx[1]] = md2;
    o[idx[2]] = sm2;
    o
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
                let name = t
                    .get(&tag::PROFILE_NAME)
                    .and_then(|v| v.ascii())
                    .unwrap_or("")
                    .to_string();
                if !name.starts_with("Negative Lab") {
                    return None;
                }
                let cam = t.get(&tag::UNIQUE_CAMERA_MODEL)?.ascii()?.to_string();
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
    fn hsv_roundtrip_dng() {
        let c = [0.2, 0.6, 0.4];
        let (h, s, v) = rgb_to_hsv_dng(c);
        let r = hsv_to_rgb_dng(h, s, v);
        for i in 0..3 {
            assert!((r[i] - c[i]).abs() < 1e-5);
        }
    }

    #[test]
    fn rgb_tone_preserves_order_and_identity() {
        let lut: Vec<f32> = (0..TONE_LUT)
            .map(|i| i as f32 / (TONE_LUT - 1) as f32)
            .collect();
        let c = [0.7, 0.2, 0.4];
        let o = rgb_tone(c, &lut);
        for i in 0..3 {
            assert!((o[i] - c[i]).abs() < 1e-3, "{o:?}");
        }
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
        assert_eq!(p.name, "Negative Lab v2.3");
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
        // Neutral stays neutral through the whole profile.
        let neutral = [0.8, 0.45, 0.3];
        let t = DcpTransform::new(p, neutral, Some([0.8, 0.45, 0.3]), 0.9);
        let o = t.apply([0.4, 0.225, 0.15]);
        assert!(
            (o[0] - o[1]).abs() < 0.02 && (o[1] - o[2]).abs() < 0.02,
            "{o:?}"
        );
        assert!(profile_for(&info("Nonexistent", "Camera 9000")).is_none());
    }
}
