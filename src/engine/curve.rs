//! Port of `CurveEngineV3.lua` — the parametric tone-curve inversion engine.
//!
//! The engine works on per-channel "node arrays": `N` evenly spaced values in
//! `[0, 1]` that represent *positive* brightness. Each tonal / colour layer maps
//! node values to new node values. Finally the nodes are reversed (inverted)
//! and paired with x positions spread between each channel's white and black
//! origin (in 8-bit MelissaRGB units of the negative), producing the three
//! `ToneCurvePV2012{Red,Green,Blue}` curves Lightroom would receive.

use serde::{Deserialize, Serialize};

use super::presets::{TEMP_MULTIPLIER, tone_profile, tone_profile_v3};
use super::settings::{
    ClipMethod, CorrectionType, CurvePoints, ImageType, LayerOrder, Settings, ToningMethod,
    WbMethod, WbTonality,
};

/// Analysis result for one channel (`whitePointOrigin`, `blackPointOrigin`, `meanPoint`).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ChannelOrigin {
    pub white: f64,
    pub black: f64,
    pub mean: f64,
}

pub type Origins = [ChannelOrigin; 3];

type Nodes = [Vec<f64>; 3];

const L8: f64 = 0.1; // L8_1
const L9: f64 = 0.8; // L9_1

fn is_positive(s: &Settings) -> bool {
    matches!(s.image_type, ImageType::Positive | ImageType::EditOnly)
}

fn clamp01(v: f64) -> f64 {
    v.clamp(0.0, 1.0)
}

fn map_nodes(n: &Nodes, f: impl Fn(f64, usize, usize) -> f64) -> Nodes {
    let mut out: Nodes = Default::default();
    for c in 0..3 {
        out[c] = n[c].iter().enumerate().map(|(k, &v)| f(v, c, k)).collect();
    }
    out
}

// ---- sigmoid helpers (L13_1, L14_1, L15_1) ---------------------------------------

fn t(a: f64, b: f64, x: f64) -> f64 {
    (0.5 * a * (x - b)).tanh()
}

/// Normalised tanh sigmoid.
pub fn sigmoid(a: f64, b: f64, x: f64, s: f64) -> f64 {
    let s = if s == 0.0 { 1.0 } else { s };
    let den = t(a, b, 1.0) - t(a, b, 0.0);
    if den.abs() < 1e-12 {
        return x;
    }
    (t(a, b, x / s) - t(a, b, 0.0)) / den * s
}

/// Inverse of [`sigmoid`] using `atanh`.
pub fn inv_sigmoid(a: f64, b: f64, x: f64, s: f64) -> f64 {
    let s = if s == 0.0 { 1.0 } else { s };
    let t0 = t(a, b, 0.0);
    let t1 = t(a, b, 1.0);
    let u = ((t1 - t0) * (x / s) + t0).clamp(-0.99999, 0.99999);
    (b + 2.0 / a * u.atanh()) * s
}

/// AutoTone gamma (L16_1): centre the mean of the three channel means at 0.5.
pub fn auto_tone_gamma(o: &Origins, s: &Settings) -> f64 {
    let mean = (o[0].mean + o[1].mean + o[2].mean) / 3.0;
    if !(mean > 0.0 && mean < 1.0) {
        return 1.0;
    }
    let logb = 0.5f64.ln() / mean.ln(); // logb(0.5, mean)
    let g = if is_positive(s) { logb } else { 1.0 / logb };
    g.clamp(0.8, 1.1)
}

/// Pre-computed engine values (L10_1).
struct EngineValues {
    cyan: f64,
    tint: f64,
    temp: f64,
    brightness: f64,
    exposure: f64,
    contrast: f64,
    highlights: f64,
    shadows: f64,
    blacks: f64,
    whites: f64,
    shadow_range: f64,
}

fn engine_values(o: &Origins, s: &Settings) -> EngineValues {
    let p = tone_profile(s.tone_profile);
    let auto = if p.auto_tone {
        auto_tone_gamma(o, s)
    } else {
        1.0
    };
    let f = |v: f32| v as f64;
    EngineValues {
        cyan: 1.0 - f(s.mid_cyan) * 0.01,
        tint: 1.0 - f(s.mid_tint) * 0.01,
        temp: 1.0 - f(s.mid_temp) * 0.01,
        brightness: (1.0 / (1.0 + f(s.brightness) * 0.02).max(0.02))
            * (1.0 / p.default_gamma)
            * (1.0 / auto),
        exposure: 1.0 / (1.0 + f(s.exposure) * 0.02).max(0.02),
        contrast: f(s.contrast) + p.default_contrast,
        highlights: f(s.highlights) + p.default_highlights + f(s.glow) / 1.5,
        shadows: f(s.shadows) + p.default_shadows - f(s.fade) / 1.5,
        blacks: f(s.blacks) + p.default_blacks + f(s.fade),
        whites: f(s.whites) + p.default_whites - f(s.glow),
        shadow_range: f(s.shadow_range),
    }
}

/// Number of curve points (L11_1).
fn node_count(min_width: f64, s: &Settings) -> usize {
    match s.curve_points {
        CurvePoints::Auto => {
            if is_positive(s) {
                7
            } else if min_width <= 30.0 {
                3
            } else if min_width <= 70.0 {
                5
            } else if min_width < 128.0 {
                7
            } else {
                9
            }
        }
        CurvePoints::Smooth => 4,
        CurvePoints::Precise => 14,
        CurvePoints::Manual => s.curve_resolution.clamp(2, 14) as usize,
    }
}

/// `makeNodeArray` (L17_1): `n` evenly spaced values 0..=1 per channel.
fn make_nodes(n: usize) -> Nodes {
    let n = n.max(2);
    let v: Vec<f64> = (0..n).map(|i| i as f64 / (n - 1) as f64).collect();
    [v.clone(), v.clone(), v]
}

// ---- clipping ---------------------------------------------------------------------

/// Extra clipping from Basic Lab roll processing (`rollWhiteClip`,
/// `rollBlackClip` in NLPRolls), in 8-bit origin units; zero otherwise.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct RollClip {
    pub white: f64,
    pub black: f64,
}

#[derive(Debug, Clone, Copy)]
struct Clip {
    start: [f64; 3],
    width: [f64; 3],
    start_over: [f64; 3],
    end_over: [f64; 3],
}

/// L31_1 (with L30_1): move white/black origins by the clip amounts, keeping
/// green and blue proportional to red and compensating for the WB offsets.
fn adjust_origins(o: &Origins, s: &Settings, a: f64, b: f64) -> ([f64; 3], [f64; 3]) {
    let nz = |v: f64| {
        if v.abs() < 1e-6 {
            1e-6f64.copysign(if v == 0.0 { 1.0 } else { v })
        } else {
            v
        }
    };
    let r_origin = nz(o[0].black - o[0].white);
    let g_origin = nz(o[1].black - o[1].white);
    let b_origin = nz(o[2].black - o[2].white);
    let after_white = o[0].black - (o[0].white + a);
    let after_black = (o[0].black - b) - o[0].white;
    let (wb_temp, wb_tint) = (s.wb_temp as f64, s.wb_tint as f64);
    let temp_a = a * (wb_temp / b_origin);
    let temp_b = b * (wb_temp / (b_origin / 2.0));
    let tint_a = a * (wb_tint / g_origin);
    let tint_b = b * (wb_tint / (g_origin / 2.0));
    let rw = after_white / r_origin;
    let rb = after_black / r_origin;
    let start = [
        o[0].white + a,
        o[1].black - rw * g_origin - tint_a,
        o[2].black - rw * b_origin - temp_a,
    ];
    let end = [
        o[0].black - b,
        o[1].white + rb * g_origin - tint_b,
        o[2].white + rb * b_origin - temp_b,
    ];
    (start, end)
}

/// Colour-protected clipping (L32_1).
fn color_protected_clip(o: &Origins, s: &Settings, rc: RollClip) -> Clip {
    let p = tone_profile_v3(s.tone_profile);
    let (wc, bc) = (s.white_clip as f64, s.black_clip as f64);
    let (a, b) = if is_positive(s) {
        (
            bc + p.default_soft_low + rc.black,
            wc + p.default_soft_high + rc.white,
        )
    } else {
        (
            wc + p.default_soft_high + rc.white,
            bc + p.default_soft_low + rc.black,
        )
    };
    let (mut start, mut end) = adjust_origins(o, s, a, b);
    let mut start_over = [0.0; 3];
    let mut end_over = [0.0; 3];
    for c in 0..3 {
        if start[c] < 0.0 {
            let slope = 255.0 / (start[c] - end[c]);
            let y = slope * (0.0 - end[c]);
            start_over[c] = 255.0 - y;
            start[c] = 0.0;
        }
    }
    for c in 0..3 {
        if end[c] > 255.0 {
            let slope = 255.0 / (start[c] - end[c]);
            end_over[c] = slope * (255.0 - end[c]);
            end[c] = 255.0;
        }
    }
    let width = [end[0] - start[0], end[1] - start[1], end[2] - start[2]];
    Clip {
        start,
        width,
        start_over,
        end_over,
    }
}

/// Linear clipping (L33_1).
fn linear_clip(o: &Origins, s: &Settings, rc: RollClip) -> Clip {
    let p = tone_profile(s.tone_profile);
    let (wc, bc) = (s.white_clip as f64, s.black_clip as f64);
    let mut start = [0.0; 3];
    let mut end = [0.0; 3];
    for c in 0..3 {
        if is_positive(s) {
            start[c] = o[c].white + bc + p.default_soft_low;
            end[c] = o[c].black - wc - p.default_soft_high;
        } else {
            start[c] = o[c].white + wc + p.default_soft_high;
            end[c] = o[c].black - bc - p.default_soft_low;
        }
    }
    // With roll clip offsets, L33_1 recomputes start/end from the roll
    // amounts alone (via L31_1), dropping WhiteClip/BlackClip and the soft defaults.
    if rc.white != 0.0 || rc.black != 0.0 {
        (start, end) = adjust_origins(o, s, rc.white, rc.black);
    }
    let mut start_over = [0.0; 3];
    let mut end_over = [0.0; 3];
    for c in 0..3 {
        if start[c] < 0.0 {
            start_over[c] = -start[c];
            start[c] = 0.0;
        }
        if end[c] > 255.0 {
            end_over[c] = end[c] - 255.0;
            end[c] = 255.0;
        }
    }
    let width = [end[0] - start[0], end[1] - start[1], end[2] - start[2]];
    Clip {
        start,
        width,
        start_over,
        end_over,
    }
}

/// Clipping as used by `updateToneCurve` (curve offsets and node count): the
/// clip method only, regardless of monochrome.
fn clip_for_curve(o: &Origins, s: &Settings, rc: RollClip) -> Clip {
    if s.clip_method == ClipMethod::ColorProtected {
        color_protected_clip(o, s, rc)
    } else {
        linear_clip(o, s, rc)
    }
}

/// Clipping as used by `getRGBNodesForXAxis`: colour-protected only for colour.
fn clip(o: &Origins, s: &Settings, rc: RollClip) -> Clip {
    if s.clip_method == ClipMethod::ColorProtected && !s.is_monochrome() {
        color_protected_clip(o, s, rc)
    } else {
        linear_clip(o, s, rc)
    }
}

/// `getRGBNodesForXAxis` (L34_1).
fn x_nodes(cl: &Clip, s: &Settings, nodes: &Nodes) -> Nodes {
    const SOFT: f64 = 6.0;
    let (first_soft, last_soft) = if is_positive(s) {
        (s.soft_shadows, s.soft_highlights)
    } else {
        (s.soft_highlights, s.soft_shadows)
    };
    let mut x = map_nodes(nodes, |v, c, _| cl.start[c] + v * cl.width[c]);
    let n = x[0].len();
    if first_soft && (0..3).all(|c| x[c][0] > SOFT) {
        for c in 0..3 {
            x[c][0] -= SOFT;
        }
    }
    if last_soft && (0..3).all(|c| x[c][n - 1] < 255.0 - SOFT) {
        for c in 0..3 {
            x[c][n - 1] += SOFT;
        }
    }
    x
}

// ---- layers -------------------------------------------------------------------------

/// `colorLinearLayer` — the WB Temp / Tint balance.
fn color_linear_layer(n: &Nodes, s: &Settings) -> Nodes {
    let k = 255.0 / TEMP_MULTIPLIER;
    let l2 = -(s.wb_tint as f64) / k;
    let l3 = -(s.wb_temp as f64) / k;
    let l7 = s.wb_tint as f64 * 0.01;
    let l8 = s.wb_temp as f64 * 0.01;
    let (offs, factors): ([f64; 3], [f64; 3]) = match s.wb_tonality {
        WbTonality::NeutralDensity => (
            [
                -l3 / 2.0 + -l2 / 2.0,
                -l3 / 2.0 + l2 / 2.0,
                l3 / 2.0 + -l2 / 2.0,
            ],
            [
                (1.0 / (1.0 - l8 * 0.5)) * (1.0 / (1.0 - l7 * 0.5)),
                (1.0 / (1.0 - l8 * 0.5)) * (1.0 - l7 * 0.5),
                (1.0 - l8 * 0.5) * (1.0 / (1.0 - l7 * 0.5)),
            ],
        ),
        WbTonality::SubtractDensity => (
            [-l3 + -l2, -l3, -l2],
            [
                (1.0 / (1.0 - l8)) * (1.0 / (1.0 - l7)),
                1.0 / (1.0 - l8),
                1.0 / (1.0 - l7),
            ],
        ),
        WbTonality::AddDensity => ([0.0, l2, l3], [1.0, 1.0 - l7, 1.0 - l8]),
    };
    let hi = 1.0 - L9;
    map_nodes(n, |x, c, _| {
        let off = offs[c];
        let fac = factors[c].max(1e-6);
        match s.wb_method {
            WbMethod::LinearFixed => {
                if x > 0.0 && x < 1.0 {
                    if x > L9 {
                        clamp01(off + (hi * x - off * x + off * L9) / hi)
                    } else if x < L8 {
                        clamp01(x + off * x / L8)
                    } else {
                        clamp01(x + off)
                    }
                } else {
                    clamp01(x)
                }
            }
            WbMethod::LinearDynamic => clamp01(x + off),
            WbMethod::ShadowWeighted => clamp01(x.max(0.0).powf(1.0 / fac)),
            WbMethod::HighlightWeighted => clamp01(1.0 - (1.0 - x).max(0.0).powf(fac)),
            WbMethod::MidtoneWeighted => {
                clamp01((x.max(0.0).powf(1.0 / fac) + (1.0 - (1.0 - x).max(0.0).powf(fac))) / 2.0)
            }
        }
    })
}

/// `colorGammaLayer` — the Mids R/C, G/M, B/Y balance.
fn color_gamma_layer(n: &Nodes, f: [f64; 3]) -> Nodes {
    const L3: f64 = 0.2;
    map_nodes(n, |x, c, _| {
        let shift = (1.0 - f[c]) / 4.0;
        let v = x - shift;
        if x > 0.0 && x < 1.0 {
            if v >= 1.0 {
                1.0
            } else if x > 1.0 - L3 {
                x - shift * ((1.0 - x) / L3)
            } else if v <= 0.0 {
                0.0
            } else {
                v
            }
        } else {
            x
        }
    })
}

fn exposure_layer(n: &Nodes, e: f64) -> Nodes {
    map_nodes(n, |x, _, _| {
        if e < 1.0 {
            1.0 - (1.0 - x).max(0.0).powf(1.0 / e)
        } else if e > 1.0 {
            x * (1.0 - (1.0 - (2.0 - e)) * 0.4)
        } else {
            x
        }
    })
}

fn gamma_layer(n: &Nodes, g: f64) -> Nodes {
    map_nodes(n, |x, _, _| x.max(0.0).powf(g))
}

fn contrast_layer(n: &Nodes, c: f64) -> Nodes {
    if c >= 1.0 {
        let a = 1.0 + c * 0.2;
        map_nodes(n, |x, _, _| sigmoid(a, 0.5, x, 1.0))
    } else if c <= -1.0 {
        let a = 1.0 + c.abs() * 0.2 * 0.5;
        map_nodes(n, |x, _, _| inv_sigmoid(a, 0.5, x, 1.0))
    } else {
        n.clone()
    }
}

/// Lights (L20_1).
fn highlights_layer(n: &Nodes, h: f64) -> Nodes {
    let alpha = 0.5 + h.abs() * 0.1;
    if h >= 1.0 {
        map_nodes(n, |x, _, _| {
            if x <= 0.1 {
                x
            } else {
                1.0 - sigmoid(alpha, 0.75, 1.0 - x, 0.9)
            }
        })
    } else if h <= -1.0 {
        map_nodes(n, |x, _, _| {
            if x <= 0.1 {
                x
            } else {
                1.0 - inv_sigmoid(alpha, 0.75, 1.0 - x, 0.9)
            }
        })
    } else {
        n.clone()
    }
}

/// Darks (L21_1).
fn shadows_layer(n: &Nodes, s: f64) -> Nodes {
    let alpha = 0.5 + s.abs() * 0.1;
    if s > 0.0 {
        map_nodes(n, |x, _, _| {
            if x >= 0.9 {
                x
            } else {
                inv_sigmoid(alpha, 0.75, x, 0.9)
            }
        })
    } else if s < 0.0 {
        map_nodes(n, |x, _, _| {
            if x >= 0.9 {
                x
            } else {
                sigmoid(alpha, 0.75, x, 0.9)
            }
        })
    } else {
        n.clone()
    }
}

/// Blacks (L22_1): exponential fog lift / sigmoid crush.
fn blacks_layer(n: &Nodes, ev: &EngineValues) -> Nodes {
    let k = (1.0 - ev.shadow_range) * 0.33 + 5.0;
    let b = ev.blacks;
    if b >= 1.0 {
        let d = b / 255.0;
        map_nodes(
            n,
            |x, _, _| if x >= 0.9 { x } else { d * (-x * k).exp() + x },
        )
    } else if b <= -1.0 {
        let alpha = 0.5 + b.abs() * 0.1;
        map_nodes(n, |x, _, _| {
            if x > 0.5 {
                x
            } else {
                sigmoid(alpha, 0.75, x, 0.5)
            }
        })
    } else {
        n.clone()
    }
}

/// Whites (L28_1): exponential shoulder roll-off / sigmoid expansion.
fn whites_layer(n: &Nodes, s: &Settings, ev: &EngineValues) -> Nodes {
    let k = (1.0 - s.high_range as f64) * 0.33 + 5.0;
    let mut w = ev.whites;
    if (s.highlights as f64) < -5.0 {
        w += s.highlights as f64 / 5.0;
    }
    if w <= -1.0 {
        let d = -w / 255.0;
        map_nodes(n, |x, _, _| {
            if x < 0.1 {
                x
            } else {
                1.0 - (d * (-(1.0 - x) * k).exp() + (1.0 - x))
            }
        })
    } else if w >= 1.0 {
        let alpha = 0.5 + w.abs() * 0.1;
        map_nodes(n, |x, _, _| {
            if x < 0.5 {
                x
            } else {
                1.0 - sigmoid(alpha, 0.75, 1.0 - x, 0.5)
            }
        })
    } else {
        n.clone()
    }
}

/// L29_1: rescale so clipped origins keep their colour ratios.
fn clip_offset_layer(n: &Nodes, a: [f64; 3], b: [f64; 3]) -> Nodes {
    map_nodes(n, |x, c, _| {
        if a[c] == 0.0 && b[c] == 0.0 {
            x
        } else {
            let hi = (255.0 - a[c]) / 255.0;
            let lo = b[c] / 255.0;
            x * (hi - lo) + lo
        }
    })
}

/// L19_1: add a layer's delta (computed on `base`) onto `n`.
fn add_with_base(n: &Nodes, layer: &Nodes, base: &Nodes) -> Nodes {
    map_nodes(n, |x, c, k| clamp01(x + layer[c][k] - base[c][k]))
}

/// L24_1: shift a triple so its minimum is zero (or max zero).
fn shift_nonneg(a: f64, b: f64, c: f64) -> [f64; 3] {
    let m = a.min(b).min(c);
    if m < 0.0 {
        [a - m, b - m, c - m]
    } else {
        [a, b, c]
    }
}

/// L23_1 linear map.
fn lerp_map(v: f64, i0: f64, i1: f64, o0: f64, o1: f64) -> f64 {
    (v - i0) / (i1 - i0) * (o1 - o0) + o0
}

/// L25_1 vintage toning curve.
fn vintage(x: f64, a: f64, b: f64) -> f64 {
    let g = (5.0 * a - 2.0 * b + 1.8).clamp(1.0, 2.5);
    let xc = x.max(0.0);
    if b >= xc && b > 0.0 {
        a + (xc / b).powf(g) * (b - a)
    } else {
        x
    }
}

/// Highlight colour toning (L27_1).
fn highlight_toning(n: &Nodes, s: &Settings) -> Nodes {
    let range = s.high_range as f64;
    let l6 = 0.9 - (10.0 - range) * 0.0444;
    let colors = [
        -(s.high_cyan as f64),
        -(s.high_tint as f64),
        -(s.high_temp as f64),
    ];
    let vint = shift_nonneg(s.high_cyan as f64, s.high_tint as f64, s.high_temp as f64);
    map_nodes(n, |x, c, _| {
        if s.toning_method == ToningMethod::Vintage {
            if vint[c] == 0.0 {
                return x;
            }
            let v = vint[c] / 150.0;
            let top = (lerp_map(range, 1.0, 10.0, 0.15, 0.6) + v).min(0.99);
            return 1.0 - vintage(1.0 - x, v, top);
        }
        let w = colors[c];
        let alpha = 0.75 + w.abs() * (1.0 + (10.0 - range) / 18.0) * 0.125;
        if w == 0.0 || x <= 1.0 - l6 {
            x
        } else if w > 0.0 {
            1.0 - sigmoid(alpha, 0.75, 1.0 - x, l6)
        } else {
            1.0 - inv_sigmoid(alpha, 0.75, 1.0 - x, l6)
        }
    })
}

/// Shadow colour toning (L26_1).
fn shadow_toning(n: &Nodes, s: &Settings) -> Nodes {
    let range = s.shadow_range as f64;
    let l6 = 0.9 - (10.0 - range) * 0.0444;
    let colors = [
        -(s.shadow_cyan as f64),
        -(s.shadow_tint as f64),
        -(s.shadow_temp as f64),
    ];
    let vint = shift_nonneg(colors[0], colors[1], colors[2]);
    map_nodes(n, |x, c, _| {
        if s.toning_method == ToningMethod::Vintage {
            if vint[c] == 0.0 {
                return x;
            }
            let v = vint[c] / 150.0;
            let top = (lerp_map(range, 1.0, 10.0, 0.15, 0.6) + v).min(0.99);
            return vintage(x, v, top);
        }
        let w = colors[c];
        let alpha = 0.75 + w.abs() * (1.0 + (10.0 - range) / 18.0) * 0.125;
        if w == 0.0 || x >= l6 {
            x
        } else if w > 0.0 {
            inv_sigmoid(alpha, 0.75, x, l6)
        } else {
            sigmoid(alpha, 0.75, x, l6)
        }
    })
}

/// `invert` (L30_1): reverse the node order of every channel.
fn invert(n: &Nodes) -> Nodes {
    let mut o = n.clone();
    for c in o.iter_mut() {
        c.reverse();
    }
    o
}

// ---- public API ---------------------------------------------------------------------

/// Resulting per-channel curves.
#[derive(Debug, Clone, PartialEq)]
pub enum ToneCurves {
    /// Classic single-pass `ToneCurvePV2012*` point curves: x in 0..255 (negative),
    /// y in 0..1 (positive).
    Single { x: Nodes, y: Nodes },
    /// v3.1 multi-pass: a linear inversion from `start` (→ white) to `end`
    /// (→ black) followed by a full-frame mask curve with evenly spaced nodes.
    Multi {
        start: [f64; 3],
        end: [f64; 3],
        y: Nodes,
    },
}

/// `updateToneCurve`: build the three channel curves for a frame.
#[cfg(test)]
pub fn update_tone_curve(o: &Origins, s: &Settings) -> ToneCurves {
    update_tone_curve_roll(o, s, RollClip::default())
}

/// `updateToneCurve` with the roll clip offsets (`rollWhiteClip`,
/// `rollBlackClip`) that Basic Lab roll processing passes in.
pub fn update_tone_curve_roll(o: &Origins, s: &Settings, rc: RollClip) -> ToneCurves {
    let ev = engine_values(o, s);
    let cl = clip_for_curve(o, s, rc);
    let n = node_count(cl.width[0].min(cl.width[1]).min(cl.width[2]), s);
    let mut nodes = make_nodes(n);
    let xs = x_nodes(&clip(o, s, rc), s, &nodes);
    let mids = [ev.cyan, ev.tint, ev.temp];
    let has_wb = s.wb_tint != 0.0 || s.wb_temp != 0.0;

    if s.layer_order == LayerOrder::ColorFirst {
        if has_wb {
            nodes = color_linear_layer(&nodes, s);
        }
        nodes = color_gamma_layer(&nodes, mids);
    }
    nodes = exposure_layer(&nodes, ev.exposure);
    nodes = gamma_layer(&nodes, ev.brightness);
    nodes = contrast_layer(&nodes, ev.contrast);
    nodes = highlights_layer(&nodes, ev.highlights);
    nodes = shadows_layer(&nodes, ev.shadows);
    nodes = blacks_layer(&nodes, &ev);
    nodes = whites_layer(&nodes, s, &ev);
    let (a, b) = if is_positive(s) {
        (cl.end_over, cl.start_over)
    } else {
        (cl.start_over, cl.end_over)
    };
    nodes = clip_offset_layer(&nodes, a, b);

    if s.layer_order == LayerOrder::TonesFirst {
        let base = make_nodes(n);
        if has_wb {
            let col = color_linear_layer(&base, s);
            nodes = add_with_base(&nodes, &col, &base);
        }
        let cg = color_gamma_layer(&base, mids);
        nodes = add_with_base(&nodes, &cg, &base);
    }

    if s.high_cyan != 0.0 || s.high_tint != 0.0 || s.high_temp != 0.0 {
        nodes = highlight_toning(&nodes, s);
    }
    if s.shadow_cyan != 0.0 || s.shadow_tint != 0.0 || s.shadow_temp != 0.0 {
        nodes = shadow_toning(&nodes, s);
    }

    if is_positive(s) {
        return ToneCurves::Single { x: xs, y: nodes };
    }
    if s.correction_type == CorrectionType::MultiPass {
        let last = xs[0].len() - 1;
        return ToneCurves::Multi {
            start: [xs[0][0], xs[1][0], xs[2][0]],
            end: [xs[0][last], xs[1][last], xs[2][last]],
            y: nodes,
        };
    }
    ToneCurves::Single {
        x: xs,
        y: invert(&nodes),
    }
}

/// `revertColorLinearLayer` (first L20_1): undo the linear-fixed WB offsets.
fn revert_color_linear_layer(n: &Nodes, offs: [f64; 3]) -> Nodes {
    let hi = 1.0 - L9;
    map_nodes(n, |x, c, _| {
        let off = offs[c];
        let v = x - off;
        if v > L9 {
            (hi * x - hi * L9 - hi * off) / (hi - off) + L9
        } else if v < L8 {
            x * L8 / (L8 + off)
        } else {
            v
        }
    })
}

/// The plugin's colour picker (`NLPViewController` L44_1): take the picked
/// positive colour, undo contrast, brightness/auto-tone gamma and the current
/// WB offsets, then linear-balance it. Returns (temp, tint).
pub fn eyedropper_wb(
    color: [f64; 3],
    o: &Origins,
    s: &Settings,
) -> Result<(f32, f32), &'static str> {
    let p = tone_profile(s.tone_profile);
    let nodes: Nodes = [vec![color[0]], vec![color[1]], vec![color[2]]];
    let nodes = contrast_layer(&nodes, -(s.contrast as f64) - p.default_contrast);
    let auto = if p.auto_tone {
        auto_tone_gamma(o, s)
    } else {
        1.0
    };
    let g = (1.0 + s.brightness as f64 * 0.02) * p.default_gamma * auto;
    let nodes = gamma_layer(&nodes, g);
    let offs = [
        0.0,
        -(s.wb_tint as f64) / 255.0 * TEMP_MULTIPLIER,
        -(s.wb_temp as f64) / 255.0 * TEMP_MULTIPLIER,
    ];
    let nodes = revert_color_linear_layer(&nodes, offs);
    let c = [nodes[0][0], nodes[1][0], nodes[2][0]];
    if c.iter().any(|v| *v > 0.95) {
        return Err(
            "The area you've selected is too bright. Please try something closer to neutral grey.",
        );
    }
    if c.iter().any(|v| *v < 0.05) {
        return Err(
            "The area you've selected is too dark. Please try something closer to neutral grey.",
        );
    }
    let v2 = 255.0 * (c[0] - c[1]);
    let v3 = 255.0 * (c[0] - c[2]);
    Ok((
        (-v3 / TEMP_MULTIPLIER) as f32,
        (-v2 / TEMP_MULTIPLIER) as f32,
    ))
}

// ---- spline + LUT -------------------------------------------------------------------

// ---- darkroom print exposure ----------------------------------------------------

/// Photographic paper model: density vs. log10 exposure is a logistic
/// (S-shaped) characteristic curve (grade-2-like: ~1.1 log units from paper
/// white to maximum black). The image's black and white map to the paper's
/// maximum black `IMG_DMAX` and paper white (density 0), which lie just
/// inside the curve's asymptotes `PAPER_DMIN` / `PAPER_DMAX`.
const IMG_DMAX: f64 = 2.1;
const PAPER_DMIN: f64 = -0.05;
const PAPER_DMAX: f64 = 2.15;
const PAPER_K: f64 = 5.35;

/// Re-print a positive value (encoded, 0..1) with the print time changed by
/// `ev` stops: + EV = less light on the paper = lighter print. The value is
/// turned into print density, traced back through the paper curve to its log
/// exposure, shifted by `ev · log10(2)` and printed again.
pub fn print_exposure(y: f32, ev: f32) -> f32 {
    let r_min = 10f64.powf(-IMG_DMAX);
    let lin = super::color::srgb_decode(y.clamp(0.0, 1.0)) as f64;
    let d = -(r_min + (1.0 - r_min) * lin).log10();
    let range = PAPER_DMAX - PAPER_DMIN;
    // Inverse logistic → relative log exposure.
    let log_e = -(range / (d - PAPER_DMIN) - 1.0).ln() / PAPER_K;
    let log_e2 = log_e - ev as f64 * std::f64::consts::LOG10_2;
    let d2 = PAPER_DMIN + range / (1.0 + (-PAPER_K * log_e2).exp());
    let lin2 = ((10f64.powf(-d2) - r_min) / (1.0 - r_min)).clamp(0.0, 1.0);
    super::color::srgb_encode(lin2 as f32)
}

/// Natural cubic spline through (x, y) as Lightroom/Photoshop use for point curves.
pub struct Spline {
    x: Vec<f64>,
    y: Vec<f64>,
    m: Vec<f64>,
}

impl Spline {
    pub fn new(xs: &[f64], ys: &[f64]) -> Self {
        // Force strictly increasing x.
        let mut x: Vec<f64> = Vec::with_capacity(xs.len());
        let mut y: Vec<f64> = Vec::with_capacity(ys.len());
        for (i, (&xi, &yi)) in xs.iter().zip(ys).enumerate() {
            let xi = if i > 0 && xi <= x[i - 1] {
                x[i - 1] + 1e-3
            } else {
                xi
            };
            x.push(xi);
            y.push(yi);
        }
        let n = x.len();
        let mut m = vec![0.0; n];
        if n > 2 {
            // Solve tridiagonal system for second derivatives.
            let mut a = vec![0.0; n];
            let mut b = vec![0.0; n];
            let mut c = vec![0.0; n];
            let mut d = vec![0.0; n];
            b[0] = 1.0;
            b[n - 1] = 1.0;
            for i in 1..n - 1 {
                let h0 = x[i] - x[i - 1];
                let h1 = x[i + 1] - x[i];
                a[i] = h0;
                b[i] = 2.0 * (h0 + h1);
                c[i] = h1;
                d[i] = 6.0 * ((y[i + 1] - y[i]) / h1 - (y[i] - y[i - 1]) / h0);
            }
            for i in 1..n {
                let w = a[i] / b[i - 1];
                b[i] -= w * c[i - 1];
                d[i] -= w * d[i - 1];
            }
            m[n - 1] = d[n - 1] / b[n - 1];
            for i in (0..n - 1).rev() {
                m[i] = (d[i] - c[i] * m[i + 1]) / b[i];
            }
        }
        Self { x, y, m }
    }

    pub fn eval(&self, v: f64) -> f64 {
        let n = self.x.len();
        if v <= self.x[0] {
            return self.y[0];
        }
        if v >= self.x[n - 1] {
            return self.y[n - 1];
        }
        let i = match self.x.binary_search_by(|p| p.partial_cmp(&v).unwrap()) {
            Ok(i) => return self.y[i],
            Err(i) => i - 1,
        };
        let h = self.x[i + 1] - self.x[i];
        let t = (v - self.x[i]) / h;
        let a = 1.0 - t;
        a * self.y[i]
            + t * self.y[i + 1]
            + ((a * a * a - a) * self.m[i] + (t * t * t - t) * self.m[i + 1]) * h * h / 6.0
    }
}

pub const LUT_SIZE: usize = 4096;

/// Per-channel lookup table: encoded negative value (0..1) → encoded positive (0..1).
#[derive(Clone)]
pub struct CurveLut {
    pub lut: [Vec<f32>; 3],
}

impl CurveLut {
    pub fn from_curves(c: &ToneCurves) -> Self {
        let mut lut: [Vec<f32>; 3] = Default::default();
        for ch in 0..3 {
            lut[ch] = match c {
                ToneCurves::Single { x, y } => {
                    let sp = Spline::new(&x[ch], &y[ch]);
                    (0..LUT_SIZE)
                        .map(|i| {
                            sp.eval(i as f64 / (LUT_SIZE - 1) as f64 * 255.0)
                                .clamp(0.0, 1.0) as f32
                        })
                        .collect()
                }
                ToneCurves::Multi { start, end, y } => {
                    let n = y[ch].len();
                    let xs: Vec<f64> = (0..n).map(|i| i as f64 / (n - 1) as f64).collect();
                    let sp = Spline::new(&xs, &y[ch]);
                    let (s0, e0) = (start[ch], end[ch]);
                    let w = if (e0 - s0).abs() < 1e-6 {
                        1e-6
                    } else {
                        e0 - s0
                    };
                    (0..LUT_SIZE)
                        .map(|i| {
                            let x = i as f64 / (LUT_SIZE - 1) as f64 * 255.0;
                            let p = (1.0 - (x - s0) / w).clamp(0.0, 1.0);
                            sp.eval(p).clamp(0.0, 1.0) as f32
                        })
                        .collect()
                }
            };
        }
        Self { lut }
    }

    /// Apply a darkroom print-exposure change (stops) on top of the curves.
    pub fn with_print_exposure(mut self, ev: f32) -> Self {
        if ev.abs() > 1e-4 {
            for ch in self.lut.iter_mut() {
                for v in ch.iter_mut() {
                    *v = print_exposure(*v, ev);
                }
            }
        }
        self
    }

    #[inline]
    pub fn apply(&self, c: usize, v: f32) -> f32 {
        let x = v.clamp(0.0, 1.0) * (LUT_SIZE - 1) as f32;
        let i = x as usize;
        let l = &self.lut[c];
        if i >= LUT_SIZE - 1 {
            return l[LUT_SIZE - 1];
        }
        let t = x - i as f32;
        l[i] + (l[i + 1] - l[i]) * t
    }

    /// Curves sampled for display (x: 0..1 negative input, y per channel).
    pub fn sample(&self, n: usize) -> Vec<[f32; 4]> {
        (0..n)
            .map(|i| {
                let x = i as f32 / (n - 1) as f32;
                [x, self.apply(0, x), self.apply(1, x), self.apply(2, x)]
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn origins() -> Origins {
        [
            ChannelOrigin {
                white: 40.0,
                black: 200.0,
                mean: 0.45,
            },
            ChannelOrigin {
                white: 30.0,
                black: 180.0,
                mean: 0.45,
            },
            ChannelOrigin {
                white: 20.0,
                black: 150.0,
                mean: 0.45,
            },
        ]
    }

    #[test]
    fn sigmoid_is_normalised() {
        assert!((sigmoid(3.0, 0.5, 0.0, 1.0)).abs() < 1e-9);
        assert!((sigmoid(3.0, 0.5, 1.0, 1.0) - 1.0).abs() < 1e-9);
        let y = sigmoid(3.0, 0.5, 0.3, 1.0);
        assert!((inv_sigmoid(3.0, 0.5, y, 1.0) - 0.3).abs() < 1e-6);
    }

    #[test]
    fn negative_curve_inverts() {
        let s = Settings::default();
        for corr in [CorrectionType::SinglePass, CorrectionType::MultiPass] {
            let mut s = s.clone();
            s.correction_type = corr;
            let lut = CurveLut::from_curves(&update_tone_curve(&origins(), &s));
            for c in 0..3 {
                let lo = lut.apply(c, 0.0);
                let hi = lut.apply(c, 1.0);
                assert!(lo > 0.9, "white end {lo}");
                assert!(hi < 0.1, "black end {hi}");
                // monotonic decreasing (roughly)
                assert!(lut.apply(c, 0.3) > lut.apply(c, 0.5));
            }
        }
    }

    #[test]
    fn print_exposure_behaves_like_paper() {
        for y in [0.0f32, 0.1, 0.5, 0.9, 1.0] {
            assert!(
                (print_exposure(y, 0.0) - y).abs() < 2e-3,
                "identity at 0 EV for {y}"
            );
        }
        // One stop up lightens, one down darkens, monotonic in EV.
        let m = 0.45;
        assert!(print_exposure(m, 1.0) > m && print_exposure(m, -1.0) < m);
        assert!(print_exposure(m, 2.0) > print_exposure(m, 1.0));
        // Extremes saturate towards paper white / max black, but never flip.
        assert!(print_exposure(m, 5.0) > 0.95);
        assert!(print_exposure(m, -5.0) < 0.05);
        assert!(print_exposure(0.2, -3.0) <= print_exposure(0.3, -3.0));
    }

    #[test]
    fn warm_temp_reduces_blue() {
        let o = origins();
        let s0 = Settings::default();
        let mut s1 = s0.clone();
        s1.wb_temp = 20.0;
        let l0 = CurveLut::from_curves(&update_tone_curve(&o, &s0));
        let l1 = CurveLut::from_curves(&update_tone_curve(&o, &s1));
        let mid = 95.0 / 255.0;
        assert!(l1.apply(2, mid) < l0.apply(2, mid));
    }
}
