//! Contact sheet: rendered frames tiled onto one sRGB JPEG.

use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use anyhow::{Context, Result};
use image::ImageEncoder;

/// A rendered frame (sRGB, 0..1).
pub struct Tile {
    pub name: String,
    pub w: usize,
    pub h: usize,
    pub px: Vec<[f32; 3]>,
}

const CELL_W: usize = 360;
const CELL_H: usize = 240;
const LABEL_H: usize = 34;
const GAP: usize = 16;
const MARGIN: usize = 32;
const FONT: &[u8] = include_bytes!("../../assets/fonts/Inter-Regular.ttf");
const FONT_PX: f32 = 15.0;

/// Rotate a portrait frame a quarter turn so every tile is landscape.
fn landscape(t: &Tile) -> (usize, usize, Vec<[f32; 3]>) {
    if t.h <= t.w {
        return (t.w, t.h, t.px.clone());
    }
    let (nw, nh) = (t.h, t.w);
    let mut out = vec![[0.0; 3]; nw * nh];
    for y in 0..t.h {
        for x in 0..t.w {
            out[(t.w - 1 - x) * nw + y] = t.px[y * t.w + x];
        }
    }
    (nw, nh, out)
}

/// Draw `text` centred on `cx`, with its top at `y`; long names are cut with "…".
fn label(buf: &mut [u8], w: usize, cx: usize, y: usize, text: &str) {
    use ab_glyph::{Font, FontRef, ScaleFont};
    let Ok(font) = FontRef::try_from_slice(FONT) else { return };
    let f = font.as_scaled(FONT_PX);
    let adv = |s: &str| s.chars().map(|c| f.h_advance(f.glyph_id(c))).sum::<f32>();
    let mut text = text.to_owned();
    while adv(&text) > CELL_W as f32 && text.chars().count() > 1 {
        text.pop();
        text.pop();
        text.push('…');
    }
    let mut x = cx as f32 - adv(&text) / 2.0;
    let base = y as f32 + f.ascent();
    for c in text.chars() {
        let id = f.glyph_id(c);
        let g = id.with_scale_and_position(FONT_PX, ab_glyph::point(x, base));
        if let Some(o) = font.outline_glyph(g) {
            let b = o.px_bounds();
            o.draw(|gx, gy, cov| {
                let (px, py) = (b.min.x as i32 + gx as i32, b.min.y as i32 + gy as i32);
                if px >= 0 && py >= 0 && (px as usize) < w {
                    let i = (py as usize * w + px as usize) * 3;
                    if i + 2 < buf.len() {
                        let v = (200.0 * cov) as u8;
                        for k in 0..3 {
                            buf[i + k] = buf[i + k].max(v);
                        }
                    }
                }
            });
        }
        x += f.h_advance(id);
    }
}

/// Tile the frames in order, each fitted into a square cell on a dark ground.
pub fn write_sheet(path: &Path, tiles: &[Tile]) -> Result<()> {
    anyhow::ensure!(!tiles.is_empty(), "no frames to place on the sheet");
    let n = tiles.len();
    // About a 3:2 sheet.
    let cols = (((n as f32) * 1.5).sqrt().ceil() as usize).clamp(1, n);
    let rows = n.div_ceil(cols);
    let row_h = CELL_H + LABEL_H;
    let w = MARGIN * 2 + cols * CELL_W + (cols - 1) * GAP;
    let h = MARGIN * 2 + rows * row_h + (rows - 1) * GAP;
    let mut buf = vec![0u8; w * h * 3];
    for (k, t) in tiles.iter().enumerate() {
        if t.w == 0 || t.h == 0 || t.px.len() < t.w * t.h {
            continue;
        }
        let (sw, sh, spx) = landscape(t);
        let t = &Tile { name: String::new(), w: sw, h: sh, px: spx };
        let scale = (CELL_W as f32 / t.w as f32).min(CELL_H as f32 / t.h as f32);
        let (tw, th) = (
            ((t.w as f32 * scale).round() as usize).clamp(1, CELL_W),
            ((t.h as f32 * scale).round() as usize).clamp(1, CELL_H),
        );
        let cx0 = MARGIN + (k % cols) * (CELL_W + GAP);
        let cy0 = MARGIN + (k / cols) * (row_h + GAP);
        let x0 = cx0 + (CELL_W - tw) / 2;
        let y0 = cy0 + (CELL_H - th) / 2;
        label(&mut buf, w, cx0 + CELL_W / 2, cy0 + CELL_H + 8, &tiles[k].name);
        for y in 0..th {
            // Box-average the source area behind each output pixel.
            let sy0 = y * t.h / th;
            let sy1 = ((y + 1) * t.h / th).clamp(sy0 + 1, t.h);
            for x in 0..tw {
                let sx0 = x * t.w / tw;
                let sx1 = ((x + 1) * t.w / tw).clamp(sx0 + 1, t.w);
                let mut acc = [0.0f32; 3];
                for sy in sy0..sy1 {
                    for sx in sx0..sx1 {
                        let p = t.px[sy * t.w + sx];
                        for c in 0..3 {
                            acc[c] += p[c];
                        }
                    }
                }
                let cnt = ((sy1 - sy0) * (sx1 - sx0)) as f32;
                let o = ((y0 + y) * w + x0 + x) * 3;
                for c in 0..3 {
                    buf[o + c] = ((acc[c] / cnt).clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
                }
            }
        }
    }
    let f = BufWriter::new(File::create(path).with_context(|| format!("create {}", path.display()))?);
    image::codecs::jpeg::JpegEncoder::new_with_quality(f, 92).write_image(
        &buf,
        w as u32,
        h as u32,
        image::ExtendedColorType::Rgb8,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_a_sheet() {
        let tile = |w, h| Tile { name: "a.tif".into(), w, h, px: vec![[0.5; 3]; w * h] };
        let p = std::env::temp_dir().join("nlp_contact_test.jpg");
        write_sheet(&p, &[tile(30, 20), tile(20, 30), tile(10, 10)]).unwrap();
        assert!(std::fs::metadata(&p).unwrap().len() > 0);
        let _ = std::fs::remove_file(p);
        assert!(write_sheet(&p_none(), &[]).is_err());
    }

    fn p_none() -> std::path::PathBuf {
        std::env::temp_dir().join("nlp_contact_none.jpg")
    }
}
