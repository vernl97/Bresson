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
        let sat = skin_kept(h, s, self.saturation);
        let Some(d) = self.data.filter(|_| self.k > 0.0) else {
            return hsv_to_rgb(h, (s * sat).min(1.0), v);
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
        let s2 = (s * (1.0 + (ds - 1.0) * k) * sat).clamp(0.0, 1.0);
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

    /// More saturation reaches a blue but not a face.
    #[test]
    fn saturation_keeps_skin() {
        let l = look(Lut::None, 9, 100.0);
        let (_, s0, _) = rgb_to_hsv([0.85, 0.65, 0.55]);
        let (_, s1, _) = rgb_to_hsv(l.apply([0.85, 0.65, 0.55]));
        let (_, b0, _) = rgb_to_hsv([0.3, 0.4, 0.7]);
        let (_, b1, _) = rgb_to_hsv(l.apply([0.3, 0.4, 0.7]));
        assert!(b1 > b0 * 1.1, "blue {b0} -> {b1}");
        assert!((s1 - s0).abs() < 0.01, "skin {s0} -> {s1}");
    }

    /// An edge under film grain (each channel its own): the edge is
    /// sharpened about as much as an unsharp mask does it, the grain is
    /// not sharpened at all.
    #[test]
    fn sharpening_spares_the_grain() {
        let (w, h) = (256usize, 128usize);
        let mut seed = 12345u32;
        let mut rnd = || {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            (seed >> 8) as f32 / (1u32 << 24) as f32 - 0.5
        };
        let noise: Vec<[f32; 3]> = (0..w * h).map(|_| [rnd(), rnd(), rnd()]).collect();
        let grain = gaussian_blur_3(&noise, w, h, 0.8);
        let img: Vec<[f32; 3]> = (0..w * h)
            .map(|i| {
                let v = if i % w < w / 2 { 0.3 } else { 0.7 };
                grain[i].map(|g| v + g * 0.12)
            })
            .collect();
        let def =
            crate::engine::presets::sharpening(crate::engine::settings::Sharpening::Lab).unwrap();
        let usm = |data: &mut [[f32; 3]]| {
            let l: Vec<f32> = data.iter().map(|p| luma(*p)).collect();
            let b = gaussian_blur_1(&l, w, h, def.radius);
            for (p, (l, b)) in data.iter_mut().zip(l.iter().zip(&b)) {
                let mut d = l - b;
                if d.abs() < def.threshold {
                    d *= d.abs() / def.threshold;
                }
                for v in p.iter_mut() {
                    *v = (*v + def.amount * d).clamp(0.0, 1.0);
                }
            }
        };
        let flat_sd = |d: &[[f32; 3]]| {
            let v: Vec<f32> = (0..w * h)
                .filter(|i| (20..100).contains(&(i % w)))
                .map(|i| luma(d[i]))
                .collect();
            let m = v.iter().sum::<f32>() / v.len() as f32;
            (v.iter().map(|x| (x - m) * (x - m)).sum::<f32>() / v.len() as f32).sqrt()
        };
        // Edge contrast: mean luminance two pixels either side of the edge.
        let edge = |d: &[[f32; 3]]| {
            let col = |x: usize| (0..h).map(|y| luma(d[y * w + x])).sum::<f32>() / h as f32;
            col(w / 2 + 1) - col(w / 2 - 2)
        };
        let mut a = img.clone();
        usm(&mut a);
        let mut b = img.clone();
        sharpen(&mut b, w, h, def, 1.0);
        eprintln!(
            "grain sd {:.4}: usm {:.4}, grain-aware {:.4}; edge {:.3}: usm {:.3}, grain-aware {:.3}",
            flat_sd(&img),
            flat_sd(&a),
            flat_sd(&b),
            edge(&img),
            edge(&a),
            edge(&b)
        );
        assert!(flat_sd(&b) <= flat_sd(&img), "grain is not sharpened");
        assert!(
            edge(&b) - edge(&img) > 0.7 * (edge(&a) - edge(&img)),
            "the edge is sharpened"
        );
    }

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

/// The saturation scale `sat` for a colour of hue `h` (degrees) and
/// saturation `s`, eased back to 1 on skin tones, like the Frontier's
/// Bright Mode: primaries get more (or less) colour, faces keep theirs.
/// Skin: orange hues (about 5–45°) of low to middle saturation.
#[inline]
pub fn skin_kept(h: f32, s: f32, sat: f32) -> f32 {
    let dh = ((h - 25.0 + 180.0).rem_euclid(360.0) - 180.0).abs();
    let hue = (1.0 - dh / 20.0).clamp(0.0, 1.0);
    let chroma = ((0.75 - s) / 0.25).clamp(0.0, 1.0);
    let skin = hue * hue * (3.0 - 2.0 * hue) * chroma;
    sat + (1.0 - sat) * skin
}

/// Grain-aware sharpening on encoded output pixels, after the Frontier's
/// Hyper Sharpness (Fuji, US 6,445,831 / US 5,739,922). Detail is split
/// into a fine band (edges, at the radius) and a middle band (two to three
/// times coarser, where film grain sits). An edge shows in all three
/// channels at once; grain does not, because each dye layer has its own
/// grain. So the local red-green-blue correlation of the bands says how
/// much of a pixel's detail is edge: edges are sharpened (luminance, like
/// an unsharp mask), grain-only areas get their middle band softened a
/// little instead. `scale` < 1 shrinks the radius for downscaled previews.
pub fn sharpen(data: &mut [[f32; 3]], w: usize, h: usize, def: SharpenDef, scale: f32) {
    use rayon::prelude::*;
    let radius = def.radius * scale;
    if radius < 0.35 {
        return;
    }
    let blur = |c: usize, sigma: f32| {
        let ch: Vec<f32> = data.iter().map(|p| p[c]).collect();
        gaussian_blur_1(&ch, w, h, sigma)
    };
    // Per channel: fine band (detail) and fine + middle band (for the
    // correlation, which needs the grain's scale too).
    let mut fine = Vec::with_capacity(3);
    let mut band = Vec::with_capacity(3);
    for c in 0..3 {
        let (b1, b2) = (blur(c, radius), blur(c, radius * GRAIN_SCALE));
        fine.push(
            data.iter()
                .zip(&b1)
                .map(|(p, b)| p[c] - b)
                .collect::<Vec<f32>>(),
        );
        band.push(
            data.iter()
                .zip(&b2)
                .map(|(p, b)| p[c] - b)
                .collect::<Vec<f32>>(),
        );
    }
    // Local (co)variances of the bands.
    let window = radius * GRAIN_SCALE;
    let local = |a: usize, b: usize| {
        let prod: Vec<f32> = band[a].iter().zip(&band[b]).map(|(x, y)| x * y).collect();
        gaussian_blur_1(&prod, w, h, window)
    };
    let (rr, gg, bb) = (local(0, 0), local(1, 1), local(2, 2));
    let (rg, gb, rb) = (local(0, 1), local(1, 2), local(0, 2));
    let th = def.threshold;
    data.par_iter_mut().enumerate().for_each(|(i, p)| {
        let corr = |xy: f32, xx: f32, yy: f32| xy / (xx * yy).sqrt().max(1e-12);
        let r = (corr(rg[i], rr[i], gg[i]) + corr(gb[i], gg[i], bb[i]) + corr(rb[i], rr[i], bb[i]))
            / 3.0;
        // 0 for grain (uncorrelated) … 1 for an edge.
        let t = ((r - 0.3) / 0.5).clamp(0.0, 1.0);
        let edge = t * t * (3.0 - 2.0 * t);
        let f = [0, 1, 2].map(|c| fine[c][i]);
        let m = [0, 1, 2].map(|c| band[c][i] - fine[c][i]);
        let mut d = luma(f);
        if th > 0.0 && d.abs() < th {
            d *= d.abs() / th;
        }
        let a = def.amount * edge * d;
        for c in 0..3 {
            p[c] = (p[c] + a - GRAIN_DAMP * (1.0 - edge) * m[c]).clamp(0.0, 1.0);
        }
    });
}

/// The grain band's blur, in multiples of the sharpening radius.
const GRAIN_SCALE: f32 = 2.5;
/// Share of the grain band taken out where there is no edge.
const GRAIN_DAMP: f32 = 0.25;
