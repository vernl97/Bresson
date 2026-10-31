// SPDX-License-Identifier: GPL-3.0-or-later
//! Scan buffer helpers and TIFF export.

use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use ndarray::{Array2, Array3, Axis};
use tiff::decoder::{Decoder, DecodingResult};
use tiff::encoder::{Compression, DeflateLevel, Rational, TiffEncoder, colortype};
use tiff::tags::{Predictor, ResolutionUnit, Tag};

use crate::error::{Error, Result};
use crate::num::{median, percentile};

/// Host-side scan result.
///
/// `rgb` is HxWx3 uint16 linear data (colour or infrared-illuminated CCD
/// frame). With infrared, `ir` holds an HxW uint16 plane for dust removal.
/// All arrays are **linear film-negative measurements** (bright sensor
/// values correspond to dense emulsion).
#[derive(Debug, Clone)]
pub struct ScanImage {
    pub rgb: Array3<u16>,
    pub dpi: i64,
    pub device_model: String,
    pub ir: Option<Array2<u16>>,
    ir_tracks: OnceLock<Option<bool>>,
}

impl PartialEq for ScanImage {
    fn eq(&self, o: &Self) -> bool {
        self.rgb == o.rgb && self.dpi == o.dpi && self.device_model == o.device_model && self.ir == o.ir
    }
}

impl ScanImage {
    pub fn new(rgb: Array3<u16>, dpi: i64, device_model: impl Into<String>, ir: Option<Array2<u16>>) -> Self {
        Self { rgb, dpi, device_model: device_model.into(), ir, ir_tracks: OnceLock::new() }
    }

    /// True when `ir` follows the picture (silver film), `None` without IR.
    pub fn ir_tracks_image(&self) -> Option<bool> {
        *self.ir_tracks.get_or_init(|| self.ir.as_ref().map(|ir| infrared_tracks_image(&self.rgb, ir, 0.06, 8).unwrap_or(false)))
    }

    /// Write a 16-bit RGB TIFF (the scanned frame only; no IR sidecar).
    pub fn save_tiff(&self, path: impl AsRef<Path>) -> Result<PathBuf> {
        save_rgb16_tiff(&self.rgb, path, self.dpi)
    }
}

/// IR p95-p5 spread over its median above which IR is not just a dust field.
pub const IR_IMAGE_SPREAD: f64 = 0.2;
/// Block-averaged IR vs visible correlation above which IR follows the picture.
pub const IR_IMAGE_CORRELATION: f64 = 0.8;
/// IR spread relative to the visible spread for low-contrast frames.
pub const IR_IMAGE_SPREAD_RATIO: f64 = 0.5;
/// Visible spread below which the ratio test is skipped.
pub const VISIBLE_MIN_SPREAD: f64 = 0.1;

/// True when the IR plane records the picture, so it is unusable as a dust map.
pub fn infrared_tracks_image(rgb: &Array3<u16>, ir: &Array2<u16>, inset: f64, block: usize) -> Result<bool> {
    let vis: Array2<f64> = rgb.mapv(|v| v as f64).mean_axis(Axis(2)).expect("channels");
    if vis.dim() != ir.dim() {
        return Err(Error::Value(format!("rgb {:?} and ir {:?} differ in size", vis.dim(), ir.dim())));
    }
    let (h, w) = ir.dim();
    let (y0, y1) = ((h as f64 * inset) as usize, (h as f64 * (1.0 - inset)) as usize);
    let (x0, x1) = ((w as f64 * inset) as usize, (w as f64 * (1.0 - inset)) as usize);
    let hh = ((y1 - y0) / block) * block;
    let ww = ((x1 - x0) / block) * block;
    if hh < 2 * block || ww < 2 * block {
        return Ok(false);
    }
    let pool = |get: &dyn Fn(usize, usize) -> f64| -> Vec<f64> {
        let mut out = Vec::with_capacity((hh / block) * (ww / block));
        for by in 0..hh / block {
            for bx in 0..ww / block {
                let mut s = 0.0;
                for dy in 0..block {
                    for dx in 0..block {
                        s += get(y0 + by * block + dy, x0 + bx * block + dx);
                    }
                }
                out.push(s / (block * block) as f64);
            }
        }
        out
    };
    let v = pool(&|y, x| vis[[y, x]]);
    let i = pool(&|y, x| ir[[y, x]] as f64);
    let std = |a: &[f64]| {
        let m = a.iter().sum::<f64>() / a.len() as f64;
        (a.iter().map(|x| (x - m).powi(2)).sum::<f64>() / a.len() as f64).sqrt()
    };
    let (mi, mv) = (median(i.iter().copied()), median(v.iter().copied()));
    if mi <= 0.0 || mv <= 0.0 || std(&v) == 0.0 || std(&i) == 0.0 {
        return Ok(false);
    }
    let spread = |a: &[f64], med: f64| (percentile(a.iter().copied(), 95.0) - percentile(a.iter().copied(), 5.0)) / med;
    let (ir_spread, vis_spread) = (spread(&i, mi), spread(&v, mv));
    let varies = ir_spread >= IR_IMAGE_SPREAD || (vis_spread >= VISIBLE_MIN_SPREAD && ir_spread >= IR_IMAGE_SPREAD_RATIO * vis_spread);
    Ok(varies && corrcoef(&v, &i) > IR_IMAGE_CORRELATION)
}

fn corrcoef(a: &[f64], b: &[f64]) -> f64 {
    let n = a.len() as f64;
    let (ma, mb) = (a.iter().sum::<f64>() / n, b.iter().sum::<f64>() / n);
    let (mut sab, mut saa, mut sbb) = (0.0, 0.0, 0.0);
    for (x, y) in a.iter().zip(b) {
        sab += (x - ma) * (y - mb);
        saa += (x - ma).powi(2);
        sbb += (y - mb).powi(2);
    }
    sab / (saa * sbb).sqrt()
}

/// TIFF `Software` tag of every file written here. NLP Studio reads it to
/// open these scans as linear (gamma 1.0).
pub const SOFTWARE: &str = concat!("opticfilm ", env!("CARGO_PKG_VERSION"));

fn tiff_path(path: &Path) -> PathBuf {
    match path.extension().and_then(|e| e.to_str()).map(str::to_lowercase).as_deref() {
        Some("tif") | Some("tiff") => path.to_path_buf(),
        _ => path.with_extension("tif"),
    }
}

fn terr(e: impl std::fmt::Display) -> Error {
    Error::Scan(format!("TIFF: {e}"))
}

/// Write HxWx3 uint16 linear RGB to a deflate-compressed TIFF with resolution tags.
pub fn save_rgb16_tiff(rgb: &Array3<u16>, path: impl AsRef<Path>, dpi: i64) -> Result<PathBuf> {
    let (h, w, c) = rgb.dim();
    if c != 3 {
        return Err(Error::Scan(format!("rgb must be HxWx3, got shape ({h}, {w}, {c})")));
    }
    let out = tiff_path(path.as_ref());
    let f = BufWriter::new(File::create(&out).map_err(terr)?);
    let mut enc = TiffEncoder::new(f)
        .map_err(terr)?
        .with_compression(Compression::Deflate(DeflateLevel::Balanced))
        .with_predictor(Predictor::Horizontal);
    let mut img = enc.new_image::<colortype::RGB16>(w as u32, h as u32).map_err(terr)?;
    img.resolution(ResolutionUnit::Inch, Rational { n: dpi as u32, d: 1 });
    img.encoder().write_tag(Tag::Software, SOFTWARE).map_err(terr)?;
    let data: Vec<u16> = rgb.as_standard_layout().iter().copied().collect();
    img.write_data(&data).map_err(terr)?;
    Ok(out)
}

/// Write HxW uint16 grayscale to a deflate-compressed TIFF with resolution tags.
pub fn save_gray16_tiff(gray: &Array2<u16>, path: impl AsRef<Path>, dpi: i64) -> Result<PathBuf> {
    let (h, w) = gray.dim();
    let out = tiff_path(path.as_ref());
    let f = BufWriter::new(File::create(&out).map_err(terr)?);
    let mut enc = TiffEncoder::new(f)
        .map_err(terr)?
        .with_compression(Compression::Deflate(DeflateLevel::Balanced))
        .with_predictor(Predictor::Horizontal);
    let mut img = enc.new_image::<colortype::Gray16>(w as u32, h as u32).map_err(terr)?;
    img.resolution(ResolutionUnit::Inch, Rational { n: dpi as u32, d: 1 });
    img.encoder().write_tag(Tag::Software, SOFTWARE).map_err(terr)?;
    let data: Vec<u16> = gray.as_standard_layout().iter().copied().collect();
    img.write_data(&data).map_err(terr)?;
    Ok(out)
}

/// Read a 16-bit RGB TIFF written by [`save_rgb16_tiff`] (gray is broadcast to RGB).
pub fn load_rgb16_tiff(path: impl AsRef<Path>, default_dpi: i64) -> Result<(Array3<u16>, i64)> {
    let f = BufReader::new(File::open(path.as_ref()).map_err(terr)?);
    let mut dec = Decoder::new(f).map_err(terr)?;
    let (w, h) = dec.dimensions().map_err(terr)?;
    let (w, h) = (w as usize, h as usize);
    let dpi = tiff_dpi(&mut dec);
    let samples: Vec<u16> = match dec.read_image().map_err(terr)? {
        DecodingResult::U16(v) => v,
        DecodingResult::U8(v) => v.iter().map(|&b| ((b as u16) << 8) | b as u16).collect(),
        DecodingResult::F32(v) => v.iter().map(|&x| x.clamp(0.0, 65535.0) as u16).collect(),
        other => return Err(Error::Scan(format!("unsupported TIFF sample type {:?}", std::mem::discriminant(&other)))),
    };
    let c = samples.len() / (w * h).max(1);
    let arr = match c {
        1 => Array3::from_shape_fn((h, w, 3), |(y, x, _)| samples[y * w + x]),
        3 => Array3::from_shape_vec((h, w, 3), samples).map_err(terr)?,
        _ => return Err(Error::Scan(format!("expected HxWx3 RGB TIFF, got {c} channels"))),
    };
    Ok((arr, dpi.unwrap_or(default_dpi)))
}

fn tiff_dpi<R: std::io::Read + std::io::Seek>(dec: &mut Decoder<R>) -> Option<i64> {
    let v = dec.get_tag(Tag::XResolution).ok()?;
    let x = match v {
        tiff::decoder::ifd::Value::Rational(n, d) if d != 0 => n as f64 / d as f64,
        other => other.into_f64().ok()?,
    };
    let unit = dec.get_tag_u32(Tag::ResolutionUnit).ok();
    let x = if unit == Some(3) { x * 2.54 } else { x };
    (x > 0.0).then(|| crate::num::py_round(x))
}
