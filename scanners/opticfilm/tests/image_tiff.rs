// SPDX-License-Identifier: GPL-3.0-or-later
//! Port of `test_image.py` and `test_swap_16bit.py`: 16-bit TIFF export/import
//! and the GL843 byte-swapped decode. The Python wrong-shape test has no
//! counterpart: `ScanImage::rgb` is typed HxWx3.

mod common;

use std::fs::File;
use std::io::BufWriter;

use ndarray::{Array2, Array3};
use opticfilm::device::model_by_name;
use opticfilm::image::{ScanImage, load_rgb16_tiff, save_gray16_tiff, save_rgb16_tiff};
use opticfilm::scan::geometry::{ScanGeometry, compute_geometry};
use opticfilm::scan::pipeline::ImagePipeline;
use tiff::encoder::{Rational, TiffEncoder, colortype};
use tiff::tags::ResolutionUnit;

#[test]
fn save_tiff_writes_a_tif() {
    let dir = common::tmp_path("tiff-save");
    let image = ScanImage::new(Array3::zeros((4, 4, 3)), 3600, "x", None);
    let path = image.save_tiff(dir.join("frame.png")).unwrap();
    assert!(path.exists());
    assert_eq!(path.extension().unwrap(), "tif");
    let path = image.save_tiff(dir.join("frame.TIFF")).unwrap();
    assert_eq!(path.file_name().unwrap(), "frame.TIFF");
}

#[test]
fn save_rgb16_tiff_roundtrip() {
    let dir = common::tmp_path("tiff-roundtrip");
    let rgb = Array3::from_shape_vec((2, 2, 3), (0..12u16).map(|v| v * 5000).collect()).unwrap();
    let path = save_rgb16_tiff(&rgb, dir.join("out.tif"), 1800).unwrap();
    let (back, dpi) = load_rgb16_tiff(&path, 1).unwrap();
    assert_eq!(back, rgb);
    assert_eq!(dpi, 1800);
}

#[test]
fn gray16_tiff_loads_as_rgb() {
    let dir = common::tmp_path("tiff-gray");
    let gray = Array2::from_shape_fn((3, 5), |(y, x)| (y * 1000 + x) as u16);
    let path = save_gray16_tiff(&gray, dir.join("ir.tif"), 900).unwrap();
    let (back, dpi) = load_rgb16_tiff(&path, 1).unwrap();
    assert_eq!(dpi, 900);
    for c in 0..3 {
        assert_eq!(back.index_axis(ndarray::Axis(2), c), gray);
    }
}

/// 8-bit scans are expanded to 16 bits (`v << 8 | v`), and a centimetre
/// resolution is read as dpi.
#[test]
fn rgb8_tiff_in_centimetres_loads_as_rgb16() {
    let dir = common::tmp_path("tiff-rgb8");
    let path = dir.join("scan8.tif");
    let data: Vec<u8> = (0..2 * 3 * 3).map(|v| (v * 13) as u8).collect();
    {
        let mut enc = TiffEncoder::new(BufWriter::new(File::create(&path).unwrap())).unwrap();
        let mut img = enc.new_image::<colortype::RGB8>(3, 2).unwrap();
        img.resolution(ResolutionUnit::Centimeter, Rational { n: 70866, d: 100 }); // 1800 dpi
        img.write_data(&data).unwrap();
    }
    let (back, dpi) = load_rgb16_tiff(&path, 1).unwrap();
    assert_eq!(dpi, 1800);
    let want: Vec<u16> = data.iter().map(|&b| ((b as u16) << 8) | b as u16).collect();
    assert_eq!(back.iter().copied().collect::<Vec<_>>(), want);
}

fn one_line(name: &str) -> (ImagePipeline, ScanGeometry, Vec<u8>) {
    let model = model_by_name(name).unwrap();
    let g = compute_geometry(900, &model, Some([0.0, 0.0, 0.1, 0.05])).unwrap();
    let g = ScanGeometry { lines: 1, optical_line_count: 1, ..g };
    // One pixel RGB LE words 0x1234 / 0x5678 / 0x9ABC, repeated.
    let pixel: Vec<u8> = [0x1234u16, 0x5678, 0x9ABC].iter().flat_map(|v| v.to_le_bytes()).collect();
    let mut raw = pixel.repeat(g.pixels as usize);
    raw.resize(g.total_bytes() as usize, 0);
    (ImagePipeline::new(model), g, raw)
}

#[test]
fn gl843_7200i_decodes_with_16bit_byteswap() {
    assert_eq!(model_by_name("plustek-opticfilm-7200i").unwrap().swap_16bit_data, Some(true));
    let (pipe, g, raw) = one_line("plustek-opticfilm-7200i");
    let out = pipe.decode_rgb(&raw, &g, Some(false)).unwrap();
    assert_eq!([out[[0, 0, 0]], out[[0, 0, 1]], out[[0, 0, 2]]], [0x3412, 0x7856, 0xBC9A]);
}

#[test]
fn gl845_8200i_does_not_byteswap() {
    assert_ne!(model_by_name("plustek-opticfilm-8200i").unwrap().swap_16bit_data, Some(true));
    let (pipe, g, raw) = one_line("plustek-opticfilm-8200i");
    let out = pipe.decode_rgb(&raw, &g, Some(false)).unwrap();
    assert_eq!(out[[0, 0, 0]], 0x1234);
}
