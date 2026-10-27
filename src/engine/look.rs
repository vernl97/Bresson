//! Post-inversion "look" stage (Natural / Frontier / Crystal / Pakon) with
//! amount and saturation, plus sharpening.
//!
//! The looks are native data (`assets/looks.json`, built into the binary),
//! measured from the Negative Lab Pro v2.3 enhanced looks the plugin
//! applied: per channel a grey curve, and a hue / saturation / value map
//! like a DNG profile's look table (hue shift, saturation scale and value
//! shift at 24 hues × 5 saturations × 8 values). On colours sampled across
//! the gamut, including hues left out of the fit, they reproduce the
//! plugin's looks to a mean of 2–4.5 and a 95th percentile of 6–14 (RGB
//! distance out of 255), where leaving the look out is 15–26 / 47–70 off.

use std::collections::HashMap;
use std::sync::OnceLock;

use serde::Deserialize;

use super::color::{hsv_to_rgb, luma, rgb_to_hsv};
use super::presets::SharpenDef;
use super::settings::{Lut, Settings};

const HUES: usize = 24;
const SATS: usize = 5;
const VALUES: usize = 8;

#[derive(Deserialize)]
struct LookData {
    /// Largest amount (strength) the look supports.
    max_amount: f32,
    /// Encoded output of each channel for a grey of value 0, 0.1 … 1.
    grey: [[f32; 11]; 3],
    /// `[hue][saturation][value]` → [hue shift (degrees), saturation
    /// scale, value shift].
    grid: Vec<Vec<Vec<[f32; 3]>>>,
    /// Chroma scale of saturation levels 1..=10 (level 5 = 1).
    saturation: [f32; 10],
}

#[derive(Deserialize)]
struct LooksFile {
    looks: HashMap<String, LookData>,
}

fn looks() -> &'static HashMap<String, LookData> {
    static L: OnceLock<HashMap<String, LookData>> = OnceLock::new();
    L.get_or_init(|| {
        let f: LooksFile = serde_json::from_str(include_str!("../../assets/looks.json"))
            .expect("assets/looks.json");
        for (name, l) in &f.looks {
            assert!(
                l.grid.len() == HUES
                    && l.grid
                        .iter()
                        .all(|h| h.len() == SATS && h.iter().all(|s| s.len() == VALUES)),
                "look {name}: grid must be {HUES}x{SATS}x{VALUES}"
            );
        }
        f.looks
    })
}

fn data(l: Lut) -> Option<&'static LookData> {
    let name = match l {
        Lut::None => return None,
        Lut::Natural => "Natural",
        Lut::Frontier => "Frontier",
        Lut::Crystal => "Crystal",
        Lut::Pakon => "Pakon",
    };
    looks().get(name)
}

/// Linear interpolation weights of `x` (clamped to 0..1) between `n` evenly
/// spaced nodes.
fn node_weights(x: f32, n: usize) -> [(usize, f32); 2] {
    let p = x.clamp(0.0, 1.0) * (n - 1) as f32;
    let i = (p as usize).min(n - 2);
    let t = p - i as f32;
    [(i, 1.0 - t), (i + 1, t)]
}

fn grey_curve(k: &[f32; 11], v: f32) -> f32 {
    let [(i, a), (j, b)] = node_weights(v, 11);
    k[i] * a + k[j] * b
}

/// A look as the GPU preview needs it: the hue / saturation / value map
/// flattened as `[hue][saturation][value]` (xyz = hue shift, saturation
/// scale, value shift) and the grey curves.
pub struct LookTables {
    pub grid: Vec<[f32; 4]>,
    pub grey: [[f32; 11]; 3],
}

/// Map size the GPU preview indexes with: hues, saturations, values.
pub const LOOK_GRID: [usize; 3] = [HUES, SATS, VALUES];

/// A prepared look (strength and saturation folded in).
#[derive(Clone, Copy)]
pub struct Look {
    data: Option<&'static LookData>,
    k: f32,
    saturation: f32,
    identity: bool,
}

impl Look {
    pub fn new(s: &Settings) -> Self {
        let mono = s.is_monochrome();
        let data = if mono { None } else { data(s.lut) };
        let k = data
            .map(|d| (s.lut_strength / 100.0).clamp(0.0, d.max_amount))
            .unwrap_or(0.0);
        // Without a look, saturation follows the looks' average levels.
        let level = s.post_saturation.clamp(1, 10) as usize - 1;
        let saturation = if mono {
            1.0
        } else {
            match data {
                Some(d) => d.saturation[level],
                None => {
                    let all = looks();
                    all.values().map(|d| d.saturation[level]).sum::<f32>() / all.len() as f32
                }
            }
        };
        let identity = k == 0.0 && (saturation - 1.0).abs() < 1e-6;
        Self {
            data,
            k,
            saturation,
            identity,
        }
    }

    /// Strength, saturation and whether the look changes anything.
    pub fn params(&self) -> (f32, f32, bool) {
        (self.k, self.saturation, self.identity)
    }

    /// The look's tables, when one is selected.
    pub fn tables(&self) -> Option<LookTables> {
        let d = self.data?;
        Some(LookTables {
            grid: d
                .grid
                .iter()
                .flatten()
                .flatten()
                .map(|g| [g[0], g[1], g[2], 0.0])
                .collect(),
            grey: d.grey,
        })
    }

    /// Apply to an encoded positive MelissaRGB pixel.
    #[inline]
    pub fn apply(&self, c: [f32; 3]) -> [f32; 3] {
        if self.identity {
            return c;
        }
        let (h, s, v) = rgb_to_hsv([
            c[0].clamp(0.0, 1.0),
            c[1].clamp(0.0, 1.0),
            c[2].clamp(0.0, 1.0),
        ]);
        let Some(d) = self.data.filter(|_| self.k > 0.0) else {
            return hsv_to_rgb(h, (s * self.saturation).min(1.0), v);
        };
        let k = self.k;
        // Hue / saturation / value map.
        let hp = h.rem_euclid(360.0) / 360.0 * HUES as f32;
        let hi = (hp as usize) % HUES;
        let ht = hp - hp.floor();
        let (mut dh, mut ds, mut dv) = (0.0f32, 0.0f32, 0.0f32);
        for (hn, hw) in [(hi, 1.0 - ht), ((hi + 1) % HUES, ht)] {
            for (sn, sw) in node_weights(s, SATS) {
                for (vn, vw) in node_weights(v, VALUES) {
                    let w = hw * sw * vw;
                    let g = d.grid[hn][sn][vn];
                    dh += w * g[0];
                    ds += w * g[1];
                    dv += w * g[2];
                }
            }
        }
        // Grey curves: their mean sets the tone, their spread the tint.
        let grey: [f32; 3] = std::array::from_fn(|c| grey_curve(&d.grey[c], v));
        let gv = (grey[0] + grey[1] + grey[2]) / 3.0;
        let v2 = v + (gv + dv - v) * k;
        let s2 = (s * (1.0 + (ds - 1.0) * k) * self.saturation).clamp(0.0, 1.0);
        let mut o = hsv_to_rgb(h + dh * k, s2, v2.clamp(0.0, 1.0));
        for c in 0..3 {
            o[c] += (grey[c] - gv) * (1.0 - s) * k;
        }
        o
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn look(lut: Lut, sat: u8, strength: f32) -> Look {
        Look::new(&Settings {
            lut,
            post_saturation: sat,
            lut_strength: strength,
            ..Settings::default()
        })
    }

    #[test]
    fn every_look_loads() {
        for l in [Lut::Natural, Lut::Frontier, Lut::Crystal, Lut::Pakon] {
            assert!(data(l).is_some(), "{l:?}");
        }
    }

    #[test]
    fn none_at_default_saturation_is_identity() {
        let c = [0.3, 0.5, 0.7];
        assert_eq!(look(Lut::None, 5, 100.0).apply(c), c);
        assert_eq!(look(Lut::Frontier, 5, 0.0).apply(c), c);
    }

    #[test]
    fn greys_stay_nearly_grey() {
        for l in [Lut::Natural, Lut::Frontier, Lut::Pakon] {
            for v in [0.1f32, 0.4, 0.8] {
                let o = look(l, 5, 100.0).apply([v; 3]);
                let spread = o.iter().fold(f32::MIN, |a, &b| a.max(b))
                    - o.iter().fold(f32::MAX, |a, &b| a.min(b));
                assert!(spread < 0.01, "{l:?} {v}: {o:?}");
            }
        }
    }

    #[test]
    fn saturation_levels_rise_monotonically() {
        let c = [0.7, 0.45, 0.3];
        let chroma = |o: [f32; 3]| {
            o.iter().fold(f32::MIN, |a, &b| a.max(b)) - o.iter().fold(f32::MAX, |a, &b| a.min(b))
        };
        for l in [Lut::None, Lut::Frontier, Lut::Pakon] {
            let v: Vec<f32> = (1..=10)
                .map(|s| chroma(look(l, s, 100.0).apply(c)))
                .collect();
            assert!(v.windows(2).all(|w| w[1] > w[0]), "{l:?}: {v:?}");
        }
    }

    #[test]
    fn strength_scales_the_change() {
        let c = [0.62, 0.48, 0.35];
        let d = |a: [f32; 3]| (0..3).map(|i| (a[i] - c[i]).abs()).sum::<f32>();
        let half = d(look(Lut::Frontier, 5, 50.0).apply(c));
        let full = d(look(Lut::Frontier, 5, 100.0).apply(c));
        assert!(half > 0.0 && half < full, "{half} {full}");
    }
}

/// Separable Gaussian blur of a single channel.
pub fn gaussian_blur_1(data: &[f32], w: usize, h: usize, sigma: f32) -> Vec<f32> {
    use rayon::prelude::*;
    if sigma <= 0.05 || w == 0 || h == 0 {
        return data.to_vec();
    }
    let r = (sigma * 3.0).ceil() as isize;
    let kernel: Vec<f32> = (-r..=r)
        .map(|i| (-(i * i) as f32 / (2.0 * sigma * sigma)).exp())
        .collect();
    let ks: f32 = kernel.iter().sum();
    let kernel: Vec<f32> = kernel.iter().map(|v| v / ks).collect();
    let mut tmp = vec![0.0f32; w * h];
    tmp.par_chunks_mut(w).enumerate().for_each(|(y, row)| {
        for x in 0..w {
            let mut acc = 0.0;
            for (i, k) in kernel.iter().enumerate() {
                let xx = (x as isize + i as isize - r).clamp(0, w as isize - 1) as usize;
                acc += data[y * w + xx] * k;
            }
            row[x] = acc;
        }
    });
    let mut out = vec![0.0f32; w * h];
    out.par_chunks_mut(w).enumerate().for_each(|(y, row)| {
        for x in 0..w {
            let mut acc = 0.0;
            for (i, k) in kernel.iter().enumerate() {
                let yy = (y as isize + i as isize - r).clamp(0, h as isize - 1) as usize;
                acc += tmp[yy * w + x] * k;
            }
            row[x] = acc;
        }
    });
    out
}

/// Separable Gaussian blur of RGB data.
pub fn gaussian_blur_3(data: &[[f32; 3]], w: usize, h: usize, sigma: f32) -> Vec<[f32; 3]> {
    let mut out = vec![[0.0f32; 3]; data.len()];
    for c in 0..3 {
        let ch: Vec<f32> = data.iter().map(|p| p[c]).collect();
        let b = gaussian_blur_1(&ch, w, h, sigma);
        for (o, v) in out.iter_mut().zip(b) {
            o[c] = v;
        }
    }
    out
}

/// Luminance unsharp mask on encoded output pixels. `scale` < 1 shrinks the
/// radius for downscaled previews.
pub fn sharpen(data: &mut [[f32; 3]], w: usize, h: usize, def: SharpenDef, scale: f32) {
    let radius = def.radius * scale;
    if radius < 0.35 {
        return;
    }
    let l: Vec<f32> = data.iter().map(|p| luma(*p)).collect();
    let b = gaussian_blur_1(&l, w, h, radius);
    let th = def.threshold;
    use rayon::prelude::*;
    data.par_iter_mut()
        .zip(l.par_iter().zip(b.par_iter()))
        .for_each(|(p, (l, b))| {
            let mut d = l - b;
            if th > 0.0 && d.abs() < th {
                d *= d.abs() / th;
            }
            let a = def.amount * d;
            for v in p.iter_mut() {
                *v = (*v + a).clamp(0.0, 1.0);
            }
        });
}
