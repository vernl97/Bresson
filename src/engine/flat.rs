//! Flat-field correction: divides out uneven illumination and lens falloff,
//! measured from a frame of the bare light source (or of clear film leader)
//! shot with the same camera, lens and light as the negatives.
//!
//! Uneven light is a multiplicative, spatially varying factor on the scan.
//! In density it becomes an offset that changes across the frame, which the
//! per-channel levels cannot absorb: corners go denser and change colour.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

use anyhow::Result;

use super::decode::{self, SourceImage};
use super::look;

/// Long edge of the stored gain map; illumination varies slowly.
const MAP_SIZE: usize = 64;

/// Per-channel gain map, 1 in the centre of the frame.
pub struct FlatField {
    w: usize,
    h: usize,
    gain: Vec<[f32; 3]>,
}

impl FlatField {
    /// Build from a flat frame: smoothed, then each channel normalised to
    /// the mean of the central tenth of the frame.
    pub fn from_source(src: &SourceImage) -> Self {
        let (d, w, h) = decode::downscale(&src.data, src.width, src.height, 4 * MAP_SIZE);
        let d = look::gaussian_blur_3(&d, w, h, 6.0);
        let (d, w, h) = decode::downscale(&d, w, h, MAP_SIZE);
        let (cx, cy) = (w / 2, h / 2);
        let (rx, ry) = ((w / 10).max(1), (h / 10).max(1));
        let mut centre = [0.0f64; 3];
        let mut n = 0.0;
        for y in cy.saturating_sub(ry)..(cy + ry).min(h) {
            for x in cx.saturating_sub(rx)..(cx + rx).min(w) {
                for c in 0..3 {
                    centre[c] += d[y * w + x][c] as f64;
                }
                n += 1.0;
            }
        }
        let centre = centre.map(|v| (v / n) as f32);
        let gain = d
            .iter()
            .map(|p| std::array::from_fn(|c| centre[c] / p[c].max(centre[c] * 0.05)))
            .collect();
        Self { w, h, gain }
    }

    pub fn load(path: &Path) -> Result<Self> {
        Ok(Self::from_source(&decode::load(path, Some(4 * MAP_SIZE))?))
    }

    /// Gain at normalised coordinates (0..1), bilinear.
    pub fn at(&self, u: f32, v: f32) -> [f32; 3] {
        let x = (u * self.w as f32 - 0.5).clamp(0.0, (self.w - 1) as f32);
        let y = (v * self.h as f32 - 0.5).clamp(0.0, (self.h - 1) as f32);
        let (x0, y0) = (x as usize, y as usize);
        let (x1, y1) = ((x0 + 1).min(self.w - 1), (y0 + 1).min(self.h - 1));
        let (tx, ty) = (x - x0 as f32, y - y0 as f32);
        let g = |x: usize, y: usize| self.gain[y * self.w + x];
        let (a, b, c, d) = (g(x0, y0), g(x1, y0), g(x0, y1), g(x1, y1));
        std::array::from_fn(|k| {
            let top = a[k] + (b[k] - a[k]) * tx;
            let bottom = c[k] + (d[k] - c[k]) * tx;
            top + (bottom - top) * ty
        })
    }

    /// Apply to a source-sized buffer.
    pub fn apply(&self, data: &[[f32; 3]], w: usize, h: usize) -> Vec<[f32; 3]> {
        use rayon::prelude::*;
        data.par_iter()
            .enumerate()
            .map(|(i, p)| {
                let (x, y) = (i % w, i / w);
                let g = self.at((x as f32 + 0.5) / w as f32, (y as f32 + 0.5) / h as f32);
                [p[0] * g[0], p[1] * g[1], p[2] * g[2]]
            })
            .collect()
    }
}

/// The flat field at `path`, loaded once.
pub fn get(path: &Path) -> Option<Arc<FlatField>> {
    static CACHE: OnceLock<Mutex<HashMap<PathBuf, Option<Arc<FlatField>>>>> = OnceLock::new();
    let cache = CACHE.get_or_init(Default::default);
    let mut c = cache.lock().ok()?;
    c.entry(path.to_path_buf())
        .or_insert_with(|| match FlatField::load(path) {
            Ok(f) => Some(Arc::new(f)),
            Err(e) => {
                eprintln!("flat field {}: {e:#}", path.display());
                None
            }
        })
        .clone()
}
