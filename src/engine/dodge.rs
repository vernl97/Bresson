//! Automatic dodging and burning, after the Frontier's Hyper Tone (Fuji,
//! US 6,807,316): the print's large-area tones are compressed towards the
//! frame's key while the detail on top of them is kept, the way a printer
//! holds back a dark foreground and burns in a bright sky by hand.
//!
//! The large-area tone is a low-pass of the green density, made from two
//! blurs: a wide one for smooth areas and a narrow one near strong edges,
//! blended by how much they disagree, so the correction does not spill
//! across an edge as a halo. It depends only on the negative, so it is made
//! once per negative (the GPU preview carries it in the density texture's
//! fourth channel). Its departure from the key becomes a density offset
//! added to every channel before the print curves: proportional far from
//! the key, nothing near it, so midtone contrast stays as it was.

use super::curve::Levels;
use super::decode;
use super::look;
use super::pipeline::Negative;
use super::settings::{ImageType, Settings};

/// Long edge of the low-pass map; large-area tone varies slowly.
const MAP_SIZE: usize = 256;
/// Blur radii as a share of the map's long edge: smooth areas, edges.
const WIDE: f32 = 0.12;
const NARROW: f32 = 0.05;
/// Disagreement of the two blurs (D) at which the narrow one takes over.
const EDGE: f32 = 0.1;
/// Departure from the key (D) below which tones are left alone.
pub const KNEE: f32 = 0.3;
/// Largest compression (share of the departure taken away) at 100.
const MAX_COMPRESSION: f32 = 0.4;

/// Low-pass green density of `neg`, one value per pixel.
pub fn low_pass(neg: &Negative) -> Vec<f32> {
    let (w, h) = (neg.width, neg.height);
    if w == 0 || h == 0 {
        return Vec::new();
    }
    let (map, mw, mh) = decode::downscale(&neg.data, w, h, MAP_SIZE);
    let long = mw.max(mh) as f32;
    let wide = look::gaussian_blur_3(&map, mw, mh, WIDE * long);
    let narrow = look::gaussian_blur_3(&map, mw, mh, NARROW * long);
    let low: Vec<f32> = wide
        .iter()
        .zip(&narrow)
        .map(|(a, b)| {
            let (l, s) = (a[1], b[1]);
            let k = ((l - s) / EDGE).powi(2);
            s + (l - s) * (-k).exp()
        })
        .collect();
    // Bilinear back to the negative's size (pixel centres aligned).
    let (sx, sy) = (mw as f32 / w as f32, mh as f32 / h as f32);
    let mut out = vec![0.0; w * h];
    for y in 0..h {
        let fy = ((y as f32 + 0.5) * sy - 0.5).clamp(0.0, (mh - 1) as f32);
        let (y0, ty) = (fy as usize, fy.fract());
        let y1 = (y0 + 1).min(mh - 1);
        for x in 0..w {
            let fx = ((x as f32 + 0.5) * sx - 0.5).clamp(0.0, (mw - 1) as f32);
            let (x0, tx) = (fx as usize, fx.fract());
            let x1 = (x0 + 1).min(mw - 1);
            let top = low[y0 * mw + x0] + (low[y0 * mw + x1] - low[y0 * mw + x0]) * tx;
            let bottom = low[y1 * mw + x0] + (low[y1 * mw + x1] - low[y1 * mw + x0]) * tx;
            out[y * w + x] = top + (bottom - top) * ty;
        }
    }
    out
}

/// The dodge a renderer applies.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Dodge {
    /// Green density of the frame's key tone.
    pub key: f32,
    /// Compression on the dense (print-light) and thin (print-dark) side.
    pub light: f32,
    pub dark: f32,
    /// Density per unit of green density in each channel (the channels'
    /// ranges against green's), so the offset leaves greys grey.
    pub gain: [f32; 3],
}

impl Dodge {
    /// `None` when the settings ask for none or the frame is not a negative.
    pub fn new(s: &Settings, l: &Levels) -> Option<Self> {
        let k = |v: f32| v.clamp(0.0, 100.0) / 100.0 * MAX_COMPRESSION;
        let (light, dark) = (k(s.hyper_tone_highlights), k(s.hyper_tone_shadows));
        if s.image_type != ImageType::Negative || (light == 0.0 && dark == 0.0) {
            return None;
        }
        let span_g = (l[1].hi - l[1].lo).max(1e-3);
        Some(Self {
            key: l[1].mid as f32,
            light,
            dark,
            gain: std::array::from_fn(|c| ((l[c].hi - l[c].lo) / span_g) as f32),
        })
    }

    /// Green density offset for a low-pass density `low`.
    #[inline]
    pub fn offset(&self, low: f32) -> f32 {
        let u = low - self.key;
        let g = if u > 0.0 { self.light } else { self.dark };
        -g * u * u * u / (u * u + KNEE * KNEE)
    }
}
