//! Post-inversion "look" stage: the plugin's enhanced-profile LUTs
//! (Natural / Frontier / Crystal / Pakon) with amount and saturation
//! variants, plus sharpening.
//!
//! The real 3D tables are read from the plugin's `NLP Enhanced Settings`
//! XMPs (see [`super::lut`]). When they cannot be found, a parametric
//! approximation of each look's documented character is used instead.

use std::sync::Arc;

use super::color::{self, hsv_to_rgb, luma, rgb_to_hsv};
use super::lut::{self, RgbTable, TableGamma, TablePrimaries};
use super::presets::SharpenDef;
use super::settings::{Lut, Settings};

/// Hue anchors (degrees) used by the hue-dependent tables.
const ANCHORS: [f32; 8] = [0.0, 30.0, 60.0, 120.0, 180.0, 210.0, 240.0, 300.0];

#[derive(Clone, Copy)]
struct LookDef {
    /// Hue shift in degrees at each anchor.
    hue: [f32; 8],
    /// Saturation multiplier at each anchor.
    sat: [f32; 8],
    /// Global saturation multiplier.
    global_sat: f32,
    /// S-curve amount on value (0..1).
    contrast: f32,
    /// Exponent on value (< 1 brightens).
    gamma: f32,
    /// Highlight desaturation amount (0..1) above `hl_start`.
    hl_desat: f32,
    hl_start: f32,
    /// Extra saturation in the shadows.
    shadow_sat: f32,
    /// RGB tint added to the deep shadows.
    shadow_tint: [f32; 3],
}

const IDENT: LookDef = LookDef {
    hue: [0.0; 8],
    sat: [1.0; 8],
    global_sat: 1.0,
    contrast: 0.0,
    gamma: 1.0,
    hl_desat: 0.0,
    hl_start: 0.9,
    shadow_sat: 0.0,
    shadow_tint: [0.0; 3],
};

fn def(l: Lut) -> LookDef {
    match l {
        Lut::None => IDENT,
        // Colorimetric: true blues, natural greens, neutral skin.
        Lut::Natural => LookDef {
            sat: [1.0, 0.98, 1.02, 1.04, 1.0, 1.04, 1.03, 1.0],
            global_sat: 1.02,
            ..IDENT
        },
        // Peachy skin, golden yellows, teal-blue skies, suppressed cherry reds,
        // purer neutral highlights.
        Lut::Frontier => LookDef {
            hue: [5.0, 4.0, -4.0, 10.0, 12.0, 2.0, -4.0, -2.0],
            sat: [0.86, 1.06, 1.10, 1.08, 0.88, 1.06, 1.0, 0.94],
            global_sat: 1.03,
            contrast: 0.08,
            gamma: 1.0,
            hl_desat: 0.35,
            hl_start: 0.82,
            shadow_sat: 0.0,
            shadow_tint: [0.0; 3],
        },
        // Fuji Crystal Archive paper: subtractive, deeper contrast, green-ish shadows.
        Lut::Crystal => LookDef {
            hue: [-2.0, 0.0, 0.0, 4.0, 2.0, 3.0, 3.0, 0.0],
            sat: [1.04, 1.0, 1.02, 1.06, 1.02, 1.04, 1.06, 1.0],
            global_sat: 1.04,
            contrast: 0.18,
            gamma: 1.04,
            hl_desat: 0.1,
            hl_start: 0.9,
            shadow_sat: 0.12,
            shadow_tint: [-0.006, 0.012, -0.002],
        },
        // Pakon F135: luminous, clean separation, bright whites.
        Lut::Pakon => LookDef {
            hue: [2.0, 1.0, 2.0, -3.0, 0.0, 0.0, 2.0, 0.0],
            sat: [1.04, 1.02, 1.04, 1.02, 1.04, 1.06, 1.04, 1.02],
            global_sat: 1.05,
            contrast: 0.1,
            gamma: 0.93,
            hl_desat: 0.45,
            hl_start: 0.8,
            shadow_sat: 0.0,
            shadow_tint: [0.0; 3],
        },
    }
}

fn interp(table: &[f32; 8], h: f32) -> f32 {
    let h = h.rem_euclid(360.0);
    for i in 0..8 {
        let a = ANCHORS[i];
        let b = if i == 7 { 360.0 } else { ANCHORS[i + 1] };
        if h >= a && h < b {
            let t = (h - a) / (b - a);
            let vb = table[(i + 1) % 8];
            return table[i] + (vb - table[i]) * t;
        }
    }
    table[0]
}

/// A real Adobe RGB table plus the transforms into and out of its encoding.
#[derive(Clone)]
struct TableLook {
    table: Arc<RgbTable>,
    to_table: [[f32; 3]; 3],
    from_table: [[f32; 3]; 3],
}

impl TableLook {
    fn new(table: Arc<RgbTable>) -> Self {
        let p = match table.primaries {
            TablePrimaries::Srgb => color::SRGB,
            TablePrimaries::AdobeRgb => color::ADOBE_RGB,
            TablePrimaries::ProPhoto => color::PROPHOTO,
            TablePrimaries::P3 => color::Primaries {
                r: [0.68, 0.32],
                g: [0.265, 0.69],
                b: [0.15, 0.06],
                white: color::D65,
            },
            TablePrimaries::Rec2020 => color::Primaries {
                r: [0.708, 0.292],
                g: [0.170, 0.797],
                b: [0.131, 0.046],
                white: color::D65,
            },
        };
        let to_table = color::to_f32(&color::rgb_to_rgb(&color::MELISSA, &p));
        let from_table = color::to_f32(&color::rgb_to_rgb(&p, &color::MELISSA));
        Self {
            table,
            to_table,
            from_table,
        }
    }

    fn encode(&self, v: f32) -> f32 {
        let v = v.max(0.0);
        match self.table.gamma {
            TableGamma::Linear => v,
            TableGamma::Srgb => color::srgb_encode(v),
            TableGamma::G18 => v.powf(1.0 / 1.8),
            TableGamma::G22 => v.powf(1.0 / 2.2),
        }
    }

    fn decode(&self, v: f32) -> f32 {
        let v = v.max(0.0);
        match self.table.gamma {
            TableGamma::Linear => v,
            TableGamma::Srgb => color::srgb_decode(v),
            TableGamma::G18 => v.powf(1.8),
            TableGamma::G22 => v.powf(2.2),
        }
    }

    /// `c` is encoded MelissaRGB; `k` the amount (0..=max_amount).
    #[inline]
    fn apply(&self, c: [f32; 3], k: f32) -> [f32; 3] {
        let dec = color::srgb_decode_lut();
        let lin = [dec.get(c[0]), dec.get(c[1]), dec.get(c[2])];
        let t = color::mul3(&self.to_table, lin);
        let e = [
            self.encode(t[0].min(1.0)),
            self.encode(t[1].min(1.0)),
            self.encode(t[2].min(1.0)),
        ];
        let l = self.table.lookup(e);
        let k = k.min(self.table.max_amount);
        let o = [0, 1, 2].map(|i| self.decode(e[i] + (l[i] - e[i]) * k));
        let m = color::mul3(&self.from_table, o);
        let enc = color::srgb_encode_lut();
        [enc.get(m[0]), enc.get(m[1]), enc.get(m[2])]
    }
}

/// A prepared look (strength folded in).
#[derive(Clone)]
pub struct Look {
    d: LookDef,
    table: Option<TableLook>,
    k: f32,
    post_sat: f32,
    identity: bool,
}

impl Look {
    pub fn new(s: &Settings) -> Self {
        let mono = s.is_monochrome();
        let lut = if mono { Lut::None } else { s.lut };
        let k = (s.lut_strength / 100.0).clamp(0.0, 2.0);
        // Saturation 1..=10 selects the table's "Sat" variant; only the
        // parametric fallback needs an explicit saturation factor.
        let table = lut::table_for(lut, s.post_saturation).map(TableLook::new);
        // Saturation only exists as LUT variants: no LUT, no effect.
        let post_sat = if mono || table.is_some() || lut == Lut::None {
            1.0
        } else {
            1.0 + (s.post_saturation as f32 - 5.0) * 0.06
        };
        let identity = (lut == Lut::None || k == 0.0) && (post_sat - 1.0).abs() < 1e-6;
        Self {
            d: def(lut),
            table,
            k,
            post_sat,
            identity,
        }
    }

    /// Whether the plugin's real LUT data is in use.
    pub fn uses_table(&self) -> bool {
        self.table.is_some()
    }

    /// Apply to an encoded positive MelissaRGB pixel.
    #[inline]
    pub fn apply(&self, c: [f32; 3]) -> [f32; 3] {
        if self.identity {
            return c;
        }
        if let Some(t) = &self.table {
            return t.apply(c, self.k);
        }
        let d = &self.d;
        let k = self.k;
        let (h, s, v) = rgb_to_hsv([
            c[0].clamp(0.0, 1.0),
            c[1].clamp(0.0, 1.0),
            c[2].clamp(0.0, 1.0),
        ]);
        let h2 = h + interp(&d.hue, h) * k * s.min(1.0).sqrt();
        let mut s2 = s * (1.0 + (interp(&d.sat, h) * d.global_sat - 1.0) * k);
        let mut v2 = v;
        if d.contrast != 0.0 {
            let sc = v * v * (3.0 - 2.0 * v);
            v2 += (sc - v) * d.contrast * k;
        }
        if d.gamma != 1.0 {
            v2 = v2.max(0.0).powf(1.0 + (d.gamma - 1.0) * k);
        }
        if d.hl_desat > 0.0 && v2 > d.hl_start {
            let t = ((v2 - d.hl_start) / (1.0 - d.hl_start)).min(1.0);
            s2 *= 1.0 - d.hl_desat * k * t * t;
        }
        if d.shadow_sat > 0.0 {
            s2 *= 1.0 + d.shadow_sat * k * (1.0 - v2).powi(2);
        }
        s2 = (s2 * self.post_sat).clamp(0.0, 1.0);
        let mut o = hsv_to_rgb(h2, s2, v2.clamp(0.0, 1.0));
        if d.shadow_tint != [0.0; 3] {
            let w = (1.0 - v2).powi(3) * k;
            for i in 0..3 {
                o[i] += d.shadow_tint[i] * w;
            }
        }
        o
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
