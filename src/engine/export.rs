//! Positive copies (NLPExportServiceProviderV3.lua / ExifV3.lua) and the
//! TIF Scan Prep utility (GammaUtilityV3.lua).

use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use image::ImageEncoder;
use tiff::encoder::{Compression, DeflateLevel, TiffEncoder, colortype};
use tiff::tags::Tag;

use super::color;
use super::decode;
use super::pipeline::{self, Analysis, Renderer};
use super::settings::{ExportFormat, ExportSettings, FilmMetadata, Placement, Settings};
use super::sidecar::RollData;

pub const SOFTWARE: &str = concat!("NLP Studio ", env!("CARGO_PKG_VERSION"));

/// One frame of an export batch, for the name template.
pub struct NameCtx<'a> {
    /// 1-based position in the batch.
    pub n: usize,
    pub meta: &'a FilmMetadata,
}

/// Output file name from `ex.name_template`. Tokens: `{name}` (source file
/// stem), `{suffix}`, `{n}` or `{n:03}` (position in the batch), `{stock}`,
/// `{iso}` and `{date}` (the scan's file date, YYYY-MM-DD). Unknown tokens
/// stay as typed.
pub fn file_name(src: &Path, ex: &ExportSettings, ctx: &NameCtx) -> String {
    let stem = src.file_stem().and_then(|s| s.to_str()).unwrap_or("image");
    let mut out = String::new();
    let mut rest = ex.name_template.as_str();
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let Some(len) = rest[open..].find('}') else {
            break;
        };
        let token = &rest[open + 1..open + len];
        match token {
            "name" => out.push_str(stem),
            "suffix" => out.push_str(&ex.suffix),
            "stock" => out.push_str(ctx.meta.film_stock.trim()),
            "iso" => out.push_str(ctx.meta.film_iso.trim()),
            "date" => out.push_str(&file_date(src).unwrap_or_default()),
            "n" => out.push_str(&ctx.n.to_string()),
            t => match t.strip_prefix("n:").and_then(|w| w.parse::<usize>().ok()) {
                Some(w) => out.push_str(&format!("{:0w$}", ctx.n)),
                None => out.push_str(&rest[open..=open + len]),
            },
        }
        rest = &rest[open + len + 1..];
    }
    out.push_str(rest);
    let out: String = out
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '-',
            c if c.is_control() => '-',
            c => c,
        })
        .collect();
    let out = out.trim_matches(|c: char| c.is_whitespace() || c == '_' || c == '-' || c == '.');
    let name = if out.is_empty() { stem } else { out };
    let ext = match ex.format {
        ExportFormat::Tiff => "tif",
        ExportFormat::Jpeg => "jpg",
    };
    format!("{name}.{ext}")
}

/// The source file's modification date (the scan date), YYYY-MM-DD.
fn file_date(src: &Path) -> Option<String> {
    let t = std::fs::metadata(src).ok()?.modified().ok()?;
    let days = t.duration_since(std::time::UNIX_EPOCH).ok()?.as_secs() as i64 / 86_400;
    // Civil date from days since 1970-01-01 (Howard Hinnant's algorithm).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    Some(format!("{y:04}-{m:02}-{d:02}"))
}

/// Output paths for an export batch. When the template gives two frames the
/// same name, the later ones get `-2`, `-3`, … so none overwrites another.
pub fn batch_paths(items: &[(&Path, &FilmMetadata)], ex: &ExportSettings) -> Vec<PathBuf> {
    let mut seen = std::collections::HashSet::new();
    items
        .iter()
        .enumerate()
        .map(|(i, (src, meta))| {
            let p = output_path(src, ex, &NameCtx { n: i + 1, meta });
            let (stem, ext) = (
                p.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("image")
                    .to_string(),
                p.extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string(),
            );
            let mut q = p.clone();
            let mut k = 2;
            while !seen.insert(q.clone()) {
                q = p.with_file_name(format!("{stem}-{k}.{ext}"));
                k += 1;
            }
            q
        })
        .collect()
}

pub fn output_path(src: &Path, ex: &ExportSettings, ctx: &NameCtx) -> PathBuf {
    let dir = if let Some(ref out_dir) = ex.output_dir {
        if !out_dir.as_os_str().is_empty() {
            out_dir.clone()
        } else {
            default_dir(src, ex)
        }
    } else {
        default_dir(src, ex)
    };
    dir.join(file_name(src, ex, ctx))
}

fn default_dir(src: &Path, ex: &ExportSettings) -> PathBuf {
    let dir = src.parent().unwrap_or(Path::new("."));
    match ex.placement {
        Placement::Subfolder => dir.join(if ex.subfolder.trim().is_empty() {
            "Positives"
        } else {
            ex.subfolder.trim()
        }),
        Placement::SameFolder => dir.to_path_buf(),
    }
}

/// Render a frame at full resolution and write the positive copy to `out`
/// (`output_path` / `batch_paths`).
pub fn export_frame(
    src_path: &Path,
    s: &Settings,
    analysis: Option<&Analysis>,
    roll: Option<&RollData>,
    meta: &FilmMetadata,
    ex: &ExportSettings,
    out: &Path,
) -> Result<PathBuf> {
    let src = decode::load(src_path, None)?;
    let base = analysis
        .map(|a| a.base)
        .unwrap_or_else(|| pipeline::resolve_base(&src, s, None));
    let mut neg = pipeline::prepare_negative(&src, s, base);
    let full_long = neg.width.max(neg.height);
    if ex.long_edge > 0 && (ex.long_edge as usize) < full_long {
        let (d, w, h) = decode::downscale(&neg.data, neg.width, neg.height, ex.long_edge as usize);
        neg = pipeline::Negative {
            width: w,
            height: h,
            data: d,
        };
    }
    let scale = neg.width.max(neg.height) as f32 / full_long as f32;
    let origins = pipeline::origins_for(s, analysis, roll);
    let mp = (src.info.full_width * src.info.full_height) as f32 / 1e6;
    let r = Renderer::new(s, origins.as_ref(), ex.space).with_megapixels(mp);
    let px = r.render(&neg, scale);
    let out = out.to_path_buf();
    if let Some(p) = out.parent() {
        std::fs::create_dir_all(p).with_context(|| format!("create {}", p.display()))?;
    }
    let icc = color::icc_for(ex.space);
    match ex.format {
        ExportFormat::Tiff => write_tiff(
            &out,
            neg.width,
            neg.height,
            &px,
            ex.sixteen_bit,
            ex.zip_compression,
            &icc,
            meta,
            &src.info,
        )?,
        ExportFormat::Jpeg => write_jpeg(
            &out,
            neg.width,
            neg.height,
            &px,
            ex.jpeg_quality,
            &icc,
            meta,
        )?,
    }
    Ok(out)
}

fn description(meta: &FilmMetadata) -> String {
    let mut parts = Vec::new();
    if !meta.film_stock.is_empty() {
        parts.push(format!("Film: {}", meta.film_stock));
    }
    if !meta.shot_at_iso.is_empty() {
        parts.push(format!("Shot at ISO {}", meta.shot_at_iso));
    }
    if !meta.developer.is_empty() {
        parts.push(format!("Developer: {}", meta.developer));
    }
    if !meta.development_notes.is_empty() {
        parts.push(meta.development_notes.clone());
    }
    parts.join("; ")
}

#[allow(clippy::too_many_arguments)]
pub fn write_tiff(
    path: &Path,
    w: usize,
    h: usize,
    px: &[[f32; 3]],
    sixteen: bool,
    zip: bool,
    icc: &[u8],
    meta: &FilmMetadata,
    info: &decode::SourceInfo,
) -> Result<()> {
    let f =
        BufWriter::new(File::create(path).with_context(|| format!("create {}", path.display()))?);
    let mut enc = TiffEncoder::new(f)?;
    if zip {
        enc = enc.with_compression(Compression::Deflate(DeflateLevel::Balanced));
    }
    let make = if meta.camera_make.is_empty() {
        info.make.clone()
    } else {
        meta.camera_make.clone()
    };
    let model = if meta.camera_model.is_empty() {
        info.model.clone()
    } else {
        meta.camera_model.clone()
    };
    let desc = description(meta);
    macro_rules! write_img {
        ($ct:ty, $data:expr) => {{
            let mut img = enc.new_image::<$ct>(w as u32, h as u32)?;
            let d = img.encoder();
            d.write_tag(Tag::IccProfile, icc)?;
            d.write_tag(Tag::Software, SOFTWARE)?;
            if !make.is_empty() {
                d.write_tag(Tag::Make, make.as_str())?;
            }
            if !model.is_empty() {
                d.write_tag(Tag::Model, model.as_str())?;
            }
            if !desc.is_empty() {
                d.write_tag(Tag::ImageDescription, desc.as_str())?;
            }
            img.write_data(&$data)?;
        }};
    }
    if sixteen {
        let data: Vec<u16> = px
            .iter()
            .flat_map(|p| p.map(|v| (v.clamp(0.0, 1.0) * 65535.0).round() as u16))
            .collect();
        write_img!(colortype::RGB16, data);
    } else {
        let data: Vec<u8> = px
            .iter()
            .flat_map(|p| p.map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8))
            .collect();
        write_img!(colortype::RGB8, data);
    }
    Ok(())
}

pub fn write_jpeg(
    path: &Path,
    w: usize,
    h: usize,
    px: &[[f32; 3]],
    quality: u8,
    icc: &[u8],
    meta: &FilmMetadata,
) -> Result<()> {
    let data: Vec<u8> = px
        .iter()
        .flat_map(|p| p.map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8))
        .collect();
    let f =
        BufWriter::new(File::create(path).with_context(|| format!("create {}", path.display()))?);
    let mut enc = image::codecs::jpeg::JpegEncoder::new_with_quality(f, quality.clamp(1, 100));
    let _ = enc.set_icc_profile(icc.to_vec());
    let _ = enc.set_exif_metadata(build_exif(meta));
    enc.write_image(&data, w as u32, h as u32, image::ExtendedColorType::Rgb8)?;
    Ok(())
}

// ---- EXIF ------------------------------------------------------------------------

enum Val {
    Ascii(String),
    Short(u16),
    Long(u32),
    Rational(u32, u32),
    Undefined(Vec<u8>),
}

impl Val {
    fn kind_count_bytes(&self) -> (u16, u32, Vec<u8>) {
        match self {
            Val::Ascii(s) => {
                let mut b = s.as_bytes().to_vec();
                b.push(0);
                (2, b.len() as u32, b)
            }
            Val::Short(v) => (3, 1, v.to_le_bytes().to_vec()),
            Val::Long(v) => (4, 1, v.to_le_bytes().to_vec()),
            Val::Rational(n, d) => (5, 1, [n.to_le_bytes(), d.to_le_bytes()].concat()),
            Val::Undefined(b) => (7, b.len() as u32, b.clone()),
        }
    }
}

/// Serialise an IFD at `offset`; returns bytes (IFD + its out-of-line data).
fn ifd(entries: &[(u16, Val)], offset: u32) -> Vec<u8> {
    let n = entries.len();
    let header_len = 2 + n * 12 + 4;
    let mut out = (n as u16).to_le_bytes().to_vec();
    let mut extra = Vec::new();
    for (tag, v) in entries {
        let (kind, count, bytes) = v.kind_count_bytes();
        out.extend_from_slice(&tag.to_le_bytes());
        out.extend_from_slice(&kind.to_le_bytes());
        out.extend_from_slice(&count.to_le_bytes());
        if bytes.len() <= 4 {
            let mut b = bytes;
            b.resize(4, 0);
            out.extend_from_slice(&b);
        } else {
            let off = offset + header_len as u32 + extra.len() as u32;
            out.extend_from_slice(&off.to_le_bytes());
            extra.extend_from_slice(&bytes);
            if extra.len() % 2 == 1 {
                extra.push(0);
            }
        }
    }
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&extra);
    out
}

fn parse_rational(s: &str) -> Option<(u32, u32)> {
    let s = s
        .trim()
        .trim_start_matches(['f', 'F', '/'])
        .trim_end_matches(['s', '"'])
        .trim();
    if s.is_empty() {
        return None;
    }
    if let Some((a, b)) = s.split_once('/') {
        let (a, b) = (a.trim().parse::<f64>().ok()?, b.trim().parse::<f64>().ok()?);
        if b <= 0.0 {
            return None;
        }
        return Some(((a * 1000.0).round() as u32, (b * 1000.0).round() as u32));
    }
    let v: f64 = s.parse().ok()?;
    Some(((v * 1000.0).round() as u32, 1000))
}

/// TIFF-structured EXIF block (without the "Exif\0\0" prefix).
pub fn build_exif(meta: &FilmMetadata) -> Vec<u8> {
    let mut ifd0: Vec<(u16, Val)> = Vec::new();
    let desc = description(meta);
    if !desc.is_empty() {
        ifd0.push((0x010E, Val::Ascii(desc)));
    }
    if !meta.camera_make.is_empty() {
        ifd0.push((0x010F, Val::Ascii(meta.camera_make.clone())));
    }
    if !meta.camera_model.is_empty() {
        ifd0.push((0x0110, Val::Ascii(meta.camera_model.clone())));
    }
    ifd0.push((0x0131, Val::Ascii(SOFTWARE.into())));
    let mut exif: Vec<(u16, Val)> = Vec::new();
    if let Some((n, d)) = parse_rational(&meta.shutter_speed) {
        exif.push((0x829A, Val::Rational(n, d)));
    }
    if let Some((n, d)) = parse_rational(&meta.aperture) {
        exif.push((0x829D, Val::Rational(n, d)));
    }
    if let Ok(iso) = meta.film_iso.trim().parse::<u16>() {
        exif.push((0x8827, Val::Short(iso)));
    }
    if !meta.film_stock.is_empty() {
        let mut b = b"ASCII\0\0\0".to_vec();
        b.extend_from_slice(meta.film_stock.as_bytes());
        exif.push((0x9286, Val::Undefined(b)));
    }
    if !meta.lens.is_empty() {
        exif.push((0xA434, Val::Ascii(meta.lens.clone())));
    }
    if !exif.is_empty() {
        ifd0.push((0x8769, Val::Long(0))); // patched below
    }
    // Layout: header(8) + IFD0 + Exif IFD.
    let mut first = ifd(&ifd0, 8);
    if !exif.is_empty() {
        let exif_off = 8 + first.len() as u32;
        ifd0.last_mut().unwrap().1 = Val::Long(exif_off);
        first = ifd(&ifd0, 8);
        let mut out = b"II*\0".to_vec();
        out.extend_from_slice(&8u32.to_le_bytes());
        out.extend_from_slice(&first);
        out.extend_from_slice(&ifd(&exif, exif_off));
        return out;
    }
    let mut out = b"II*\0".to_vec();
    out.extend_from_slice(&8u32.to_le_bytes());
    out.extend_from_slice(&first);
    out
}

// ---- TIF Scan Prep -------------------------------------------------------------------

/// Normalise a linear (gamma 1.0) flatbed scan: level each channel to its
/// maximum and re-encode with gamma 2.2. Writes `<stem>-prepped.tif` and returns it.
pub fn tif_scan_prep(path: &Path) -> Result<PathBuf> {
    let img = image::ImageReader::open(path)?
        .with_guessed_format()?
        .decode()
        .with_context(|| format!("decode {}", path.display()))?;
    let (w, h) = (img.width() as usize, img.height() as usize);
    let rgb = img.into_rgb32f();
    let mut max = [1e-6f32; 3];
    for p in rgb.pixels() {
        for c in 0..3 {
            max[c] = max[c].max(p.0[c]);
        }
    }
    let px: Vec<[f32; 3]> = rgb
        .pixels()
        .map(|p| [0, 1, 2].map(|c| (p.0[c] / max[c]).clamp(0.0, 1.0).powf(1.0 / 2.2)))
        .collect();
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("scan");
    let out = path.with_file_name(format!("{stem}-prepped.tif"));
    let icc = color::make_icc(
        "Gamma 2.2 sRGB primaries (NLP Studio)",
        &color::SRGB,
        color::Trc::Gamma(2.2),
    );
    write_tiff(
        &out,
        w,
        h,
        &px,
        true,
        true,
        &icc,
        &FilmMetadata::default(),
        &decode::SourceInfo::default(),
    )?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exif_is_well_formed() {
        let m = FilmMetadata {
            camera_make: "Leica".into(),
            camera_model: "M6".into(),
            film_stock: "Portra 400".into(),
            film_iso: "400".into(),
            shutter_speed: "1/125".into(),
            aperture: "f/2.8".into(),
            ..Default::default()
        };
        let e = build_exif(&m);
        assert_eq!(&e[..4], b"II*\0");
        assert!(e.windows(5).any(|w| w == b"Leica"));
        assert_eq!(parse_rational("1/125"), Some((1000, 125000)));
        assert_eq!(parse_rational("f/2.8"), Some((2800, 1000)));
    }

    #[test]
    fn name_template_tokens() {
        let src = Path::new("/scans/roll7/raw0070.dng");
        let meta = FilmMetadata {
            film_stock: "Portra 400".into(),
            film_iso: "400".into(),
            ..Default::default()
        };
        let ctx = NameCtx { n: 7, meta: &meta };
        let name = |t: &str| {
            let ex = ExportSettings {
                name_template: t.into(),
                ..Default::default()
            };
            file_name(src, &ex, &ctx)
        };
        assert_eq!(name("{name}{suffix}"), "raw0070-positive.jpg");
        assert_eq!(name("{stock}_{n:03}"), "Portra 400_007.jpg");
        assert_eq!(name("roll {n} iso{iso}"), "roll 7 iso400.jpg");
        assert_eq!(name("{name}/{what}"), "raw0070-{what}.jpg");
        assert_eq!(
            name("{name"),
            "{name.jpg",
            "unterminated tokens stay as typed"
        );
        // Empty tokens fall back to the source name.
        let none = FilmMetadata::default();
        let ex = ExportSettings {
            name_template: "{stock}_".into(),
            ..Default::default()
        };
        assert_eq!(
            file_name(src, &ex, &NameCtx { n: 1, meta: &none }),
            "raw0070.jpg"
        );
    }

    #[test]
    fn batch_names_never_collide() {
        let meta = FilmMetadata::default();
        let ex = ExportSettings {
            name_template: "roll".into(),
            output_dir: Some(PathBuf::from("/out")),
            ..Default::default()
        };
        let (a, b, c) = (
            Path::new("/s/a.dng"),
            Path::new("/s/b.dng"),
            Path::new("/s/c.dng"),
        );
        let p = batch_paths(&[(a, &meta), (b, &meta), (c, &meta)], &ex);
        let names: Vec<_> = p
            .iter()
            .map(|p| p.file_name().unwrap().to_str().unwrap())
            .collect();
        assert_eq!(names, ["roll.jpg", "roll-2.jpg", "roll-3.jpg"]);
    }

    #[test]
    fn file_date_is_a_calendar_date() {
        let d = file_date(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("Cargo.toml")
                .as_path(),
        )
        .unwrap();
        assert_eq!(d.len(), 10);
        assert!(d.starts_with("20"), "{d}");
    }

    #[test]
    fn output_path_uses_custom_output_dir() {
        let src = Path::new("/path/to/scans/test-01.nef");
        let mut ex = ExportSettings::default();
        ex.format = ExportFormat::Jpeg;
        ex.suffix = "-positive".into();
        ex.output_dir = Some(PathBuf::from("/Users/abe/Pictures/ExportDir"));

        let meta = FilmMetadata::default();
        let p = output_path(src, &ex, &NameCtx { n: 1, meta: &meta });
        assert_eq!(
            p,
            PathBuf::from("/Users/abe/Pictures/ExportDir/test-01-positive.jpg")
        );
    }

    #[test]
    fn output_path_defaults_when_output_dir_none() {
        let src = Path::new("/path/to/scans/test-01.nef");
        let mut ex = ExportSettings::default();
        ex.format = ExportFormat::Tiff;
        ex.placement = Placement::Subfolder;
        ex.subfolder = "Positives".into();
        ex.suffix = "-positive".into();
        ex.output_dir = None;

        let meta = FilmMetadata::default();
        let p = output_path(src, &ex, &NameCtx { n: 1, meta: &meta });
        assert_eq!(
            p,
            PathBuf::from("/path/to/scans/Positives/test-01-positive.tif")
        );
    }
}
