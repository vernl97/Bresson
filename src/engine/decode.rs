//! Image ingestion: camera RAW (via `rawler`) and TIFF / JPEG / PNG scans.
//!
//! RAW files are kept in linear *camera* space together with a matrix to
//! linear MelissaRGB, so that the film-base white balance can be applied in
//! camera space exactly like Lightroom's WB eyedropper on the film border.

use std::path::Path;

use anyhow::{Context, Result, anyhow};
use rawler::Orientation;
use rawler::imgop::develop::{Intermediate, ProcessingStep, RawDevelop};
use rawler::rawimage::RawPhotometricInterpretation;
use serde::{Deserialize, Serialize};

use super::color::{self, Mat3};

pub const RAW_EXTENSIONS: &[&str] = &[
    "dng", "cr2", "cr3", "crw", "nef", "nrw", "arw", "srf", "sr2", "raf", "orf", "rw2", "pef",
    "srw", "3fr", "fff", "iiq", "erf", "kdc", "dcr", "mos", "mef", "mrw", "x3f", "rwl", "raw",
];
pub const BITMAP_EXTENSIONS: &[&str] = &["tif", "tiff", "jpg", "jpeg", "png"];

pub fn is_supported(path: &Path) -> bool {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    RAW_EXTENSIONS.contains(&ext.as_str()) || BITMAP_EXTENSIONS.contains(&ext.as_str())
}

/// RAW or TIFF: the file types listed when a whole folder is opened.
/// JPEG/PNG are still accepted when opened individually.
pub fn is_folder_scan(path: &Path) -> bool {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    RAW_EXTENSIONS.contains(&ext.as_str()) || matches!(ext.as_str(), "tif" | "tiff")
}

pub fn is_raw(path: &Path) -> bool {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    RAW_EXTENSIONS.contains(&ext.as_str())
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SourceInfo {
    pub make: String,
    pub model: String,
    pub full_width: usize,
    pub full_height: usize,
    /// EXIF exposure of the capture: shutter (s), f-number, ISO. Used to
    /// match exposure across a roll (NLPRolls `L19_1`).
    #[serde(default)]
    pub exposure: Option<[f64; 3]>,
}

/// Linear source pixels (camera RGB for RAW, linear RGB for bitmaps).
#[derive(Clone, Serialize, Deserialize)]
pub struct SourceImage {
    pub width: usize,
    pub height: usize,
    pub data: Vec<[f32; 3]>,
    /// Source space → linear MelissaRGB.
    pub to_melissa: [[f32; 3]; 3],
    /// Camera multipliers recorded by the camera (normalised to green = 1).
    pub as_shot_wb: [f32; 3],
    pub is_raw: bool,
    pub info: SourceInfo,
}

impl SourceImage {
    pub fn long_edge(&self) -> usize {
        self.width.max(self.height)
    }
}

/// Load an image. With `max_long_edge`, RAW files use a fast binning path and
/// the result is downscaled so its long edge is at most that size.
pub fn load(path: &Path, max_long_edge: Option<usize>) -> Result<SourceImage> {
    let mut img = if is_raw(path) {
        load_raw(path, max_long_edge)?
    } else {
        load_bitmap(path)?
    };
    if let Some(m) = max_long_edge {
        if img.long_edge() > m {
            let (d, w, h) = downscale(&img.data, img.width, img.height, m);
            img.data = d;
            img.width = w;
            img.height = h;
        }
    }
    Ok(img)
}

fn load_bitmap(path: &Path) -> Result<SourceImage> {
    let dynimg = image::ImageReader::open(path)
        .with_context(|| format!("open {}", path.display()))?
        .with_guessed_format()?
        .decode()
        .with_context(|| format!("decode {}", path.display()))?;
    let (w, h) = (dynimg.width() as usize, dynimg.height() as usize);
    let rgb = dynimg.into_rgb32f();
    let dec = color::srgb_decode_lut();
    let data: Vec<[f32; 3]> = rgb
        .pixels()
        .map(|p| [dec.get(p.0[0]), dec.get(p.0[1]), dec.get(p.0[2])])
        .collect();
    // Scans are assumed to be sRGB-primaried and gamma encoded.
    let to_mel = color::to_f32(&color::rgb_to_rgb(&color::SRGB, &color::MELISSA));
    Ok(SourceImage {
        width: w,
        height: h,
        data,
        to_melissa: to_mel,
        as_shot_wb: [1.0; 3],
        is_raw: false,
        info: SourceInfo {
            make: String::new(),
            model: String::new(),
            full_width: w,
            full_height: h,
            exposure: None,
        },
    })
}

/// Shutter, aperture and ISO from the raw file's EXIF, when all three exist.
fn raw_exposure(path: &Path) -> Option<[f64; 3]> {
    let src = rawler::rawsource::RawSource::new(path).ok()?;
    let dec = rawler::get_decoder(&src).ok()?;
    let exif = dec
        .raw_metadata(&src, &rawler::decoders::RawDecodeParams::default())
        .ok()?
        .exif;
    let ratio = |r: rawler::formats::tiff::Rational| (r.d != 0).then(|| r.n as f64 / r.d as f64);
    let t = exif.exposure_time.and_then(ratio)?;
    let n = exif.fnumber.and_then(ratio)?;
    let iso = exif
        .iso_speed_ratings
        .map(f64::from)
        .or(exif.iso_speed.map(f64::from))?;
    (t > 0.0 && n > 0.0 && iso > 0.0).then_some([t, n, iso])
}

fn camera_matrix(raw: &rawler::RawImage) -> Mat3 {
    use rawler::imgop::xyz::Illuminant;
    let found = raw.color_matrix_find_first([
        Illuminant::D65,
        Illuminant::D55,
        Illuminant::Daylight,
        Illuminant::D50,
        Illuminant::D75,
        Illuminant::Flash,
        Illuminant::A,
    ]);
    let xyz2cam: Mat3 = match found {
        Some((_, m)) if m.len() >= 9 => [
            [m[0] as f64, m[1] as f64, m[2] as f64],
            [m[3] as f64, m[4] as f64, m[5] as f64],
            [m[6] as f64, m[7] as f64, m[8] as f64],
        ],
        _ => color::mat_inv(&color::rgb_to_xyz(&color::SRGB)),
    };
    // rgb (Melissa) -> cam, rows normalised so that RGB white maps to camera white.
    let mut rgb2cam = color::mat_mul(&xyz2cam, &color::rgb_to_xyz(&color::MELISSA));
    for row in rgb2cam.iter_mut() {
        let s: f64 = row.iter().sum();
        if s.abs() > 1e-9 {
            for v in row.iter_mut() {
                *v /= s;
            }
        }
    }
    color::mat_inv(&rgb2cam)
}

fn load_raw(path: &Path, max_long_edge: Option<usize>) -> Result<SourceImage> {
    let raw = rawler::decode_file(path).map_err(|e| anyhow!("{}: {e}", path.display()))?;
    let cam2mel = camera_matrix(&raw);
    let wb = if raw.wb_coeffs[0].is_nan() || raw.wb_coeffs[1] <= 0.0 {
        [1.0, 1.0, 1.0]
    } else {
        let g = raw.wb_coeffs[1];
        [raw.wb_coeffs[0] / g, 1.0, raw.wb_coeffs[2] / g]
    };
    let info = SourceInfo {
        make: raw.clean_make.clone(),
        model: raw.clean_model.clone(),
        full_width: raw.width,
        full_height: raw.height,
        exposure: raw_exposure(path),
    };

    let fast = matches!(&raw.photometric, RawPhotometricInterpretation::Cfa(c) if c.cfa.is_rgb())
        && raw.cpp == 1;
    let (mut data, mut w, mut h) = match max_long_edge {
        Some(m) if fast => bin_cfa(&raw, m)?,
        _ => develop_full(&raw)?,
    };
    let orient = raw.orientation;
    orient_in_place(&mut data, &mut w, &mut h, orient);
    let mut info = info;
    if matches!(
        orient,
        Orientation::Rotate90
            | Orientation::Rotate270
            | Orientation::Transpose
            | Orientation::Transverse
    ) {
        std::mem::swap(&mut info.full_width, &mut info.full_height);
    }
    Ok(SourceImage {
        width: w,
        height: h,
        data,
        to_melissa: color::to_f32(&cam2mel),
        as_shot_wb: wb,
        is_raw: true,
        info,
    })
}

/// Full-resolution develop: our own demosaic for Bayer (RCD-style, see
/// `demosaic.rs`) and rawler's Markesteijn for X-Trans, where rawler's
/// develop would use PPG and bilinear; anything else goes through rawler.
fn develop_full(raw: &rawler::RawImage) -> Result<(Vec<[f32; 3]>, usize, usize)> {
    match develop_cfa(raw)? {
        Some(r) => Ok(r),
        None => develop_rawler(raw),
    }
}

fn develop_cfa(raw: &rawler::RawImage) -> Result<Option<(Vec<[f32; 3]>, usize, usize)>> {
    use rawler::imgop::sensor::{Demosaic, SensorType};
    use rawler::imgop::{Dim2, Point, Rect};
    let RawPhotometricInterpretation::Cfa(config) = &raw.photometric else {
        return Ok(None);
    };
    if raw.cpp != 1 || !config.cfa.is_rgb() || raw.fuji_rotation_width.is_some() {
        return Ok(None);
    }
    let mut raw = raw.clone();
    raw.apply_scaling().map_err(|e| anyhow!("scaling: {e}"))?;
    let (w, h) = (raw.width, raw.height);
    let data = raw.data.as_f32();
    let roi = raw
        .active_area
        .unwrap_or(Rect::new(Point::new(0, 0), Dim2::new(w, h)));
    let (rx, ry, rw, rh) = (roi.p.x, roi.p.y, roi.d.w.min(w - roi.p.x), roi.d.h.min(h - roi.p.y));
    let mut out = match config.sensor {
        SensorType::Bayer => {
            let m: Vec<f32> = (0..rh)
                .flat_map(|y| data[(ry + y) * w + rx..][..rw].iter().copied())
                .collect();
            let cfa = &config.cfa;
            super::demosaic::rcd(&m, rw, rh, |r, c| cfa.color_at(ry + r, rx + c).min(2))
        }
        SensorType::Xtrans => {
            use rawler::imgop::sensor::xtrans::markesteijn::XTransMarkesteijnDemosaic;
            let pixels = rawler::pixarray::PixF32::new_with(data.into_owned(), w, h);
            XTransMarkesteijnDemosaic::new_pass_1()
                .demosaic(&pixels, &config.cfa, &config.colors, Rect::new(Point::new(rx, ry), Dim2::new(rw, rh)))
                .data
        }
    };
    // Default crop, relative to the active area (as rawler's develop does).
    let (mut ow, mut oh) = (rw, rh);
    if let Some(mut crop) = raw.crop_area.or(raw.active_area) {
        if let Some(a) = &raw.active_area {
            crop = crop.adapt(a);
        }
        let (cx, cy) = (crop.p.x.min(ow), crop.p.y.min(oh));
        let (cw, ch) = (crop.d.w.min(ow - cx), crop.d.h.min(oh - cy));
        if (cw, ch) != (ow, oh) && cw > 0 && ch > 0 {
            out = (0..ch)
                .flat_map(|y| out[(cy + y) * ow + cx..][..cw].iter().copied())
                .collect();
            (ow, oh) = (cw, ch);
        }
    }
    Ok(Some((out, ow, oh)))
}

fn develop_rawler(raw: &rawler::RawImage) -> Result<(Vec<[f32; 3]>, usize, usize)> {
    let dev = RawDevelop::new_with(&[
        ProcessingStep::Rescale,
        ProcessingStep::Demosaic,
        ProcessingStep::FujiRotate,
        ProcessingStep::CropActiveArea,
        ProcessingStep::CropDefault,
    ]);
    let inter = dev
        .develop_intermediate(raw)
        .map_err(|e| anyhow!("develop: {e}"))?;
    Ok(match inter {
        Intermediate::ThreeColor(px) => (px.data, px.width, px.height),
        Intermediate::FourColor(px) => (
            px.data
                .iter()
                .map(|p| [p[0], (p[1] + p[3]) * 0.5, p[2]])
                .collect(),
            px.width,
            px.height,
        ),
        Intermediate::Monochrome(px) => (
            px.data.iter().map(|&v| [v, v, v]).collect(),
            px.width,
            px.height,
        ),
    })
}

/// Fast proxy path: black/white scaling then per-colour block averaging of the
/// CFA mosaic (no demosaic).
fn bin_cfa(raw: &rawler::RawImage, max_long_edge: usize) -> Result<(Vec<[f32; 3]>, usize, usize)> {
    let mut raw = raw.clone();
    raw.apply_scaling().map_err(|e| anyhow!("scaling: {e}"))?;
    let cfa = match &raw.photometric {
        RawPhotometricInterpretation::Cfa(c) => c.cfa.clone(),
        _ => unreachable!(),
    };
    let src = raw.data.as_f32();
    let rect = raw.crop_area.or(raw.active_area);
    let (x0, y0, cw, ch) = match rect {
        Some(r) => (
            r.p.x,
            r.p.y,
            r.d.w.min(raw.width - r.p.x),
            r.d.h.min(raw.height - r.p.y),
        ),
        None => (0, 0, raw.width, raw.height),
    };
    let long = cw.max(ch);
    // Block size: even (Bayer) and a multiple of 6 for X-Trans friendliness when large.
    let mut block = (long / max_long_edge).max(2);
    if block % 2 == 1 {
        block += 1;
    }
    let ow = cw / block;
    let oh = ch / block;
    let rw = raw.width;
    use rayon::prelude::*;
    let data: Vec<[f32; 3]> = (0..oh)
        .into_par_iter()
        .flat_map_iter(|by| {
            let src = &src;
            let cfa = &cfa;
            (0..ow).map(move |bx| {
                let mut sum = [0.0f32; 3];
                let mut cnt = [0u32; 3];
                for yy in 0..block {
                    let row = y0 + by * block + yy;
                    let base = row * rw;
                    for xx in 0..block {
                        let col = x0 + bx * block + xx;
                        let c = cfa.color_at(row, col).min(3);
                        let c = if c == 3 { 1 } else { c };
                        sum[c] += src[base + col];
                        cnt[c] += 1;
                    }
                }
                [0, 1, 2].map(|c| {
                    if cnt[c] > 0 {
                        sum[c] / cnt[c] as f32
                    } else {
                        0.0
                    }
                })
            })
        })
        .collect();
    Ok((data, ow, oh))
}

/// Apply an EXIF orientation.
pub fn orient_in_place(data: &mut Vec<[f32; 3]>, w: &mut usize, h: &mut usize, o: Orientation) {
    let (sw, sh) = (*w, *h);
    let map: Option<(usize, usize, Box<dyn Fn(usize, usize) -> (usize, usize)>)> = match o {
        Orientation::Normal | Orientation::Unknown => None,
        Orientation::HorizontalFlip => Some((sw, sh, Box::new(move |x, y| (sw - 1 - x, y)))),
        Orientation::Rotate180 => Some((sw, sh, Box::new(move |x, y| (sw - 1 - x, sh - 1 - y)))),
        Orientation::VerticalFlip => Some((sw, sh, Box::new(move |x, y| (x, sh - 1 - y)))),
        Orientation::Transpose => Some((sh, sw, Box::new(move |x, y| (y, x)))),
        Orientation::Rotate90 => Some((sh, sw, Box::new(move |x, y| (y, sh - 1 - x)))),
        Orientation::Transverse => Some((sh, sw, Box::new(move |x, y| (sw - 1 - y, sh - 1 - x)))),
        Orientation::Rotate270 => Some((sh, sw, Box::new(move |x, y| (sw - 1 - y, x)))),
    };
    if let Some((nw, nh, f)) = map {
        let mut out = vec![[0.0f32; 3]; nw * nh];
        for y in 0..nh {
            for x in 0..nw {
                let (sx, sy) = f(x, y);
                out[y * nw + x] = data[sy * sw + sx];
            }
        }
        *data = out;
        *w = nw;
        *h = nh;
    }
}

/// Rotate by quarter turns clockwise and optionally flip horizontally.
pub fn rotate_flip(
    data: &[[f32; 3]],
    w: usize,
    h: usize,
    quarter_turns: u8,
    flip_h: bool,
) -> (Vec<[f32; 3]>, usize, usize) {
    let mut d = data.to_vec();
    let (mut nw, mut nh) = (w, h);
    let o = match quarter_turns % 4 {
        1 => Orientation::Rotate90,
        2 => Orientation::Rotate180,
        3 => Orientation::Rotate270,
        _ => Orientation::Normal,
    };
    orient_in_place(&mut d, &mut nw, &mut nh, o);
    if flip_h {
        orient_in_place(&mut d, &mut nw, &mut nh, Orientation::HorizontalFlip);
    }
    (d, nw, nh)
}

/// Area-averaging downscale so the long edge is at most `max`.
pub fn downscale(
    data: &[[f32; 3]],
    w: usize,
    h: usize,
    max: usize,
) -> (Vec<[f32; 3]>, usize, usize) {
    let long = w.max(h);
    if long <= max || w == 0 || h == 0 {
        return (data.to_vec(), w, h);
    }
    let scale = max as f64 / long as f64;
    let nw = ((w as f64 * scale).round() as usize).max(1);
    let nh = ((h as f64 * scale).round() as usize).max(1);
    use rayon::prelude::*;
    let out: Vec<[f32; 3]> = (0..nh)
        .into_par_iter()
        .flat_map_iter(|y| {
            let y0 = y * h / nh;
            let y1 = ((y + 1) * h / nh).max(y0 + 1).min(h);
            (0..nw).map(move |x| {
                let x0 = x * w / nw;
                let x1 = ((x + 1) * w / nw).max(x0 + 1).min(w);
                let mut s = [0.0f32; 3];
                for yy in y0..y1 {
                    for p in &data[yy * w + x0..yy * w + x1] {
                        s[0] += p[0];
                        s[1] += p[1];
                        s[2] += p[2];
                    }
                }
                let n = ((y1 - y0) * (x1 - x0)) as f32;
                [s[0] / n, s[1] / n, s[2] / n]
            })
        })
        .collect();
    (out, nw, nh)
}

/// Crop by insets in percent [left, top, right, bottom].
pub fn crop(
    data: &[[f32; 3]],
    w: usize,
    h: usize,
    insets: [f32; 4],
) -> (Vec<[f32; 3]>, usize, usize) {
    if insets.iter().all(|v| *v <= 0.0) {
        return (data.to_vec(), w, h);
    }
    let l = ((insets[0].clamp(0.0, 45.0) / 100.0) * w as f32) as usize;
    let t = ((insets[1].clamp(0.0, 45.0) / 100.0) * h as f32) as usize;
    let r = ((insets[2].clamp(0.0, 45.0) / 100.0) * w as f32) as usize;
    let b = ((insets[3].clamp(0.0, 45.0) / 100.0) * h as f32) as usize;
    let nw = w.saturating_sub(l + r).max(1);
    let nh = h.saturating_sub(t + b).max(1);
    let mut out = Vec::with_capacity(nw * nh);
    for y in t..t + nh {
        out.extend_from_slice(&data[y * w + l..y * w + l + nw]);
    }
    (out, nw, nh)
}

/// Fine rotation (degrees, clockwise) with bilinear sampling, cropped to the
/// largest rectangle of the original aspect ratio that contains no border.
pub fn straighten(
    data: &[[f32; 3]],
    w: usize,
    h: usize,
    deg: f32,
) -> (Vec<[f32; 3]>, usize, usize) {
    if deg.abs() < 0.01 || w < 2 || h < 2 {
        return (data.to_vec(), w, h);
    }
    let th = deg.to_radians();
    let (c, s) = (th.cos(), th.sin());
    let (wf, hf) = (w as f32, h as f32);
    let sc = (hf / (hf * c.abs() + wf * s.abs())).min(wf / (wf * c.abs() + hf * s.abs()));
    let nw = ((wf * sc) as usize).max(1);
    let nh = ((hf * sc) as usize).max(1);
    let (cx, cy) = ((wf - 1.0) / 2.0, (hf - 1.0) / 2.0);
    let (ox, oy) = ((nw as f32 - 1.0) / 2.0, (nh as f32 - 1.0) / 2.0);
    use rayon::prelude::*;
    let out = (0..nh)
        .into_par_iter()
        .flat_map_iter(|y| {
            (0..nw).map(move |x| {
                let (dx, dy) = (x as f32 - ox, y as f32 - oy);
                let sx = (cx + dx * c + dy * s).clamp(0.0, wf - 1.001);
                let sy = (cy - dx * s + dy * c).clamp(0.0, hf - 1.001);
                let (x0, y0) = (sx as usize, sy as usize);
                let (fx, fy) = (sx - x0 as f32, sy - y0 as f32);
                let p = |xx: usize, yy: usize| data[yy * w + xx];
                let (a, b, cc, d) = (p(x0, y0), p(x0 + 1, y0), p(x0, y0 + 1), p(x0 + 1, y0 + 1));
                [0, 1, 2].map(|k| {
                    (a[k] * (1.0 - fx) + b[k] * fx) * (1.0 - fy)
                        + (cc[k] * (1.0 - fx) + d[k] * fx) * fy
                })
            })
        })
        .collect();
    (out, nw, nh)
}

/// Full geometry chain: quarter turns, flip, straighten, crop.
pub fn geometry(
    data: &[[f32; 3]],
    w: usize,
    h: usize,
    rotation: u8,
    flip_h: bool,
    straighten_deg: f32,
    insets: [f32; 4],
) -> (Vec<[f32; 3]>, usize, usize) {
    let (d, w, h) = rotate_flip(data, w, h, rotation, flip_h);
    let (d, w, h) = straighten(&d, w, h, straighten_deg);
    crop(&d, w, h, insets)
}
