//! Film scanner output as a LinearRaw DNG.
//!
//! The scanner's 16-bit linear RGB goes into one three-sample LinearRaw image
//! (lossless JPEG), the form VueScan and SilverFast DNGs take, so it opens
//! here as a VueScan / SilverFast DNG source and in other raw software. The
//! scanner's RGB is tagged as sRGB primaries (`ColorMatrix1`, D65), the same
//! reading an untagged scan gets.

use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use anyhow::{Context, Result};
use rawler::dng::writer::DngWriter;
use rawler::dng::{DNG_VERSION_V1_4, DngCompression};
use rawler::formats::tiff::{Rational, SRational};
use rawler::imgop::xyz::Illuminant;
use rawler::tags::{DngTag, TiffCommonTag};

use super::color;

/// What the DNG says about where it came from.
pub struct ScanDngInfo<'a> {
    pub make: &'a str,
    pub model: &'a str,
    pub software: &'a str,
    pub dpi: u32,
}

/// Write `rgb` (row-major RGB triples, `width` × `height`) as a LinearRaw DNG.
pub fn write(
    path: &Path,
    rgb: &[u16],
    width: usize,
    height: usize,
    info: &ScanDngInfo,
) -> Result<()> {
    anyhow::ensure!(
        rgb.len() == width * height * 3,
        "scan is {} samples, expected {width}×{height}×3",
        rgb.len()
    );
    let file = File::create(path).with_context(|| format!("create {}", path.display()))?;
    let mut dng = DngWriter::new(BufWriter::new(file), DNG_VERSION_V1_4)
        .map_err(|e| anyhow::anyhow!("DNG: {e:?}"))?;
    let mut raw = dng.subframe_on_root(0);
    raw.rgb_image_u16(rgb, width, height, DngCompression::Lossless, 1)
        .map_err(|e| anyhow::anyhow!("DNG image: {e:?}"))?;
    raw.finalize()
        .map_err(|e| anyhow::anyhow!("DNG image: {e:?}"))?;

    let xyz2cam = color::mat_inv(&color::rgb_to_xyz(&color::SRGB));
    let m: Vec<SRational> = xyz2cam
        .iter()
        .flatten()
        .map(|v| SRational::new((v * 10_000.0).round() as i32, 10_000))
        .collect();
    dng.color_matrix(1, Illuminant::D65, m);
    let root = dng.root_ifd_mut();
    root.add_tag(TiffCommonTag::Make, info.make);
    root.add_tag(TiffCommonTag::Model, info.model);
    root.add_tag(
        DngTag::UniqueCameraModel,
        format!("{} {}", info.make, info.model).as_str(),
    );
    root.add_tag(TiffCommonTag::Software, info.software);
    root.add_tag(TiffCommonTag::XResolution, Rational::new(info.dpi, 1));
    root.add_tag(TiffCommonTag::YResolution, Rational::new(info.dpi, 1));
    root.add_tag(TiffCommonTag::ResolutionUnit, 2u16);
    dng.close().map_err(|e| anyhow::anyhow!("DNG: {e:?}"))?;
    Ok(())
}

/// Write `rgb` as a 16-bit RGB TIFF (deflate) for scans with a tone curve
/// already applied; without the `opticfilm` Software tag it opens as a TIF Scan.
pub fn write_tiff16(
    path: &Path,
    rgb: &[u16],
    width: usize,
    height: usize,
    dpi: u32,
    software: &str,
) -> Result<()> {
    use tiff::encoder::{Compression, DeflateLevel, Rational as TRational, TiffEncoder, colortype};
    use tiff::tags::{ResolutionUnit, Tag};
    anyhow::ensure!(
        rgb.len() == width * height * 3,
        "scan is {} samples, expected {width}×{height}×3",
        rgb.len()
    );
    let file =
        BufWriter::new(File::create(path).with_context(|| format!("create {}", path.display()))?);
    let mut enc =
        TiffEncoder::new(file)?.with_compression(Compression::Deflate(DeflateLevel::Balanced));
    let mut img = enc.new_image::<colortype::RGB16>(width as u32, height as u32)?;
    img.resolution(ResolutionUnit::Inch, TRational { n: dpi, d: 1 });
    img.encoder().write_tag(Tag::Software, software)?;
    img.write_data(rgb)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::decode;
    use crate::engine::pipeline::detect_source;
    use crate::engine::settings::Source;

    #[test]
    fn scan_dng_opens_as_a_linear_scanner_dng() {
        let (w, h) = (37, 23);
        let rgb: Vec<u16> = (0..w * h * 3)
            .map(|i| ((i * 977) % 60000 + 1000) as u16)
            .collect();
        let dir = std::env::temp_dir().join(format!("nlp-scan-dng-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("scan_001.dng");
        let info = ScanDngInfo {
            make: "Plustek",
            model: "OpticFilm 8200i SE",
            software: "opticfilm 0.1.0",
            dpi: 3600,
        };
        write(&path, &rgb, w, h, &info).unwrap();

        let src = decode::load(&path, None).unwrap();
        assert_eq!((src.width, src.height), (w, h));
        assert!(src.is_raw && src.info.linear_raw);
        assert_eq!(src.info.software, "opticfilm 0.1.0");
        assert_eq!(
            (src.info.make.as_str(), src.info.model.as_str()),
            ("Plustek", "OpticFilm 8200i SE")
        );
        assert_eq!(detect_source(&src), Source::VueScanRaw);
        // Lossless: the linear values come back (as fractions of the 16-bit white level).
        for (i, px) in src.data.iter().enumerate().take(200) {
            for c in 0..3 {
                let want = rgb[i * 3 + c] as f32 / 65535.0;
                assert!(
                    (px[c] - want).abs() < 1e-4,
                    "pixel {i} ch {c}: {} vs {want}",
                    px[c]
                );
            }
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}
