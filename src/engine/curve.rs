//! The print engine: turns film density into a positive the way an enlarger
//! prints a negative onto paper.
//!
//! The input is per-channel density above the film base (`pipeline::Negative`).
//! Each channel's analysed density range ([`ChannelLevels`]) is normalised to
//! 0 (deepest shadow, printed black) … 1 (brightest highlight, printed white).
//! Normalising every channel to its own range matches the layers' different
//! H&D slopes, so neutrals stay neutral from shadows to highlights. Tone and
//! colour controls shape the normalised density. It then becomes a paper log
//! exposure (plus printing filtration and print time), and the tone profile
//! (the plugin's, measured) gives the printed value.
//!
//! Every channel is a pure function of its own density, so the whole engine
//! bakes into one density-indexed lookup table per channel ([`CurveLut`]).

use serde::{Deserialize, Serialize};

use super::color;
use super::settings::{ImageType, Settings, ToneProfile};

/// Analysed density range of one channel (density above the film base).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ChannelLevels {
    /// Shadow end: the thinnest image density, printed black.
    pub lo: f64,
    /// Highlight end: the densest image density, printed white.
    pub hi: f64,
    /// Median image density (sets the auto midtone placement).
    pub mid: f64,
    /// Bow of the channel's neutral curve against green (crossover): a grey
    /// at normalised green `x` has this channel's density at
    /// `lo + (hi − lo) · ((1 + bow) · x − bow · x²)`. 0 is a straight axis.
    #[serde(default)]
    pub bow: f64,
}

/// Largest bow: keeps the neutral curve monotone with slope ≥ ½ of the
/// straight line's at either end.
pub const MAX_BOW: f64 = 0.5;

impl ChannelLevels {
    /// Normalised density of a negative (0 = `lo` … 1 = `hi`) along the
    /// channel's neutral curve, so a grey has the same value in every
    /// channel. Straight beyond the ends, with the curve's end slopes.
    pub fn normalise(&self, d: f64) -> f64 {
        let y = (d - self.lo) / (self.hi - self.lo).max(1e-6);
        let q = self.bow.clamp(-MAX_BOW, MAX_BOW);
        if q.abs() < 1e-9 {
            y
        } else if y <= 0.0 {
            y / (1.0 + q)
        } else if y >= 1.0 {
            1.0 + (y - 1.0) / (1.0 - q)
        } else {
            // Root of q·x² − (1 + q)·x + y = 0 in 0..1.
            let b = 1.0 + q;
            2.0 * y / (b + (b * b - 4.0 * q * y).max(0.0).sqrt())
        }
    }

    /// Density at normalised `x` (the inverse of [`Self::normalise`]).
    pub fn density(&self, x: f64) -> f64 {
        let q = self.bow.clamp(-MAX_BOW, MAX_BOW);
        let y = if x <= 0.0 {
            x * (1.0 + q)
        } else if x >= 1.0 {
            1.0 + (x - 1.0) * (1.0 - q)
        } else {
            (1.0 + q) * x - q * x * x
        };
        self.lo + (self.hi - self.lo) * y
    }
}

pub type Levels = [ChannelLevels; 3];

/// Log exposure range of the paper between printed white and printed black
/// (ISO R ≈ 105, a grade-2 paper): the span print exposure and filtration
/// move along.
pub const PAPER_RANGE: f64 = 1.05;
/// One CC unit of printing filtration, in log exposure.
pub const CC: f64 = 0.01;

/// Density range the lookup tables cover.
pub const LUT_DMIN: f64 = -0.5;
pub const LUT_DMAX: f64 = 4.5;
pub const LUT_SIZE: usize = 8192;
/// Span of print tone (0 = black, 1 = white, before the tone profile)
/// the profile table covers; strong tone edits reach past 0..1.
pub const PROFILE_XMIN: f64 = -0.5;
pub const PROFILE_XMAX: f64 = 1.5;

/// Number of knots of a tone curve, evenly spaced over normalised density.
const KNOTS: usize = 21;

/// A tone profile: the encoded output at evenly spaced normalised densities
/// (0 = black end … 1 = white end), and whether the median density is
/// auto-placed on the midtone first.
///
/// The curves are the Lightroom plugin's tone profiles, which were tuned on
/// real rolls, measured on the modelled ColorChecker negative: each knot is
/// the plugin's printed grey (local regression over the chart's 18 neutral
/// patches at −1, 0 and +1 EV) at that normalised density of this engine.
fn profile(p: ToneProfile) -> (bool, [f64; KNOTS]) {
    use ToneProfile::*;
    match p {
        Logarithmic => (
            false,
            [
                0.0000, 0.0511, 0.1022, 0.1526, 0.2024, 0.2519, 0.3008, 0.3495, 0.3979, 0.4446,
                0.4901, 0.5349, 0.5789, 0.6227, 0.6667, 0.7124, 0.7619, 0.8133, 0.8656, 0.9185,
                0.9713,
            ],
        ),
        LogarithmicRich => (
            false,
            [
                0.0000, 0.0356, 0.0740, 0.1185, 0.1677, 0.2202, 0.2753, 0.3322, 0.3896, 0.4441,
                0.4962, 0.5464, 0.5937, 0.6394, 0.6850, 0.7313, 0.7796, 0.8285, 0.8775, 0.9259,
                0.9740,
            ],
        ),
        LogarithmicFlat => (
            false,
            [
                0.0187, 0.0669, 0.1153, 0.1627, 0.2092, 0.2553, 0.3007, 0.3459, 0.3905, 0.4336,
                0.4756, 0.5168, 0.5567, 0.5958, 0.6347, 0.6740, 0.7155, 0.7585, 0.8027, 0.8486,
                0.8949,
            ],
        ),
        Standard => (
            true,
            [
                0.0072, 0.0581, 0.1145, 0.1768, 0.2417, 0.3082, 0.3751, 0.4414, 0.5050, 0.5663,
                0.6252, 0.6789, 0.7269, 0.7725, 0.8155, 0.8519, 0.8846, 0.9153, 0.9427, 0.9674,
                0.9919,
            ],
        ),
        AllSoft => (
            true,
            [
                0.0389, 0.0966, 0.1569, 0.2186, 0.2795, 0.3389, 0.3966, 0.4517, 0.5040, 0.5544,
                0.6031, 0.6486, 0.6909, 0.7319, 0.7714, 0.8073, 0.8413, 0.8739, 0.9043, 0.9325,
                0.9604,
            ],
        ),
        AllHard => (
            true,
            [
                0.0069, 0.0338, 0.0703, 0.1206, 0.1815, 0.2531, 0.3323, 0.4200, 0.5068, 0.5903,
                0.6690, 0.7358, 0.7888, 0.8360, 0.8782, 0.9076, 0.9308, 0.9508, 0.9668, 0.9796,
                0.9921,
            ],
        ),
        HighlightHard => (
            true,
            [
                0.0071, 0.0582, 0.1160, 0.1814, 0.2504, 0.3216, 0.3936, 0.4645, 0.5316, 0.5956,
                0.6563, 0.7101, 0.7566, 0.8000, 0.8405, 0.8731, 0.9018, 0.9282, 0.9513, 0.9718,
                0.9919,
            ],
        ),
        HighlightSoft => (
            true,
            [
                0.0075, 0.0618, 0.1187, 0.1784, 0.2387, 0.2990, 0.3587, 0.4170, 0.4732, 0.5277,
                0.5806, 0.6302, 0.6763, 0.7209, 0.7638, 0.8020, 0.8377, 0.8718, 0.9030, 0.9319,
                0.9604,
            ],
        ),
        ShadowHard => (
            true,
            [
                0.0072, 0.0493, 0.0978, 0.1538, 0.2142, 0.2781, 0.3442, 0.4122, 0.4789, 0.5439,
                0.6068, 0.6646, 0.7162, 0.7652, 0.8113, 0.8494, 0.8834, 0.9149, 0.9427, 0.9674,
                0.9919,
            ],
        ),
        ShadowSoft => (
            true,
            [
                0.0348, 0.0948, 0.1583, 0.2245, 0.2906, 0.3553, 0.4185, 0.4786, 0.5351, 0.5892,
                0.6411, 0.6885, 0.7318, 0.7733, 0.8130, 0.8480, 0.8805, 0.9114, 0.9398, 0.9659,
                0.9919,
            ],
        ),
        Linear => (
            false,
            [
                0.0001, 0.0720, 0.1434, 0.2118, 0.2768, 0.3394, 0.3989, 0.4562, 0.5105, 0.5610,
                0.6093, 0.6560, 0.6999, 0.7405, 0.7798, 0.8174, 0.8523, 0.8858, 0.9177, 0.9475,
                0.9768,
            ],
        ),
        LinearGamma => (
            false,
            [
                0.0000, 0.0257, 0.0569, 0.0995, 0.1482, 0.2000, 0.2539, 0.3094, 0.3666, 0.4232,
                0.4792, 0.5349, 0.5899, 0.6429, 0.6949, 0.7458, 0.7950, 0.8429, 0.8893, 0.9333,
                0.9769,
            ],
        ),
        LinearDeep => (
            false,
            [
                0.0000, 0.0584, 0.1181, 0.1796, 0.2413, 0.3027, 0.3632, 0.4230, 0.4812, 0.5363,
                0.5893, 0.6408, 0.6895, 0.7344, 0.7777, 0.8187, 0.8560, 0.8912, 0.9243, 0.9546,
                0.9844,
            ],
        ),
        LinearFlat => (
            false,
            [
                0.0257, 0.0911, 0.1560, 0.2176, 0.2759, 0.3320, 0.3855, 0.4367, 0.4851, 0.5305,
                0.5740, 0.6158, 0.6548, 0.6912, 0.7267, 0.7602, 0.7914, 0.8215, 0.8500, 0.8766,
                0.9031,
            ],
        ),
    }
}

/// Monotone cubic (Fritsch–Carlson) through the knots at `x`, continuing
/// smoothly towards black below 0 and towards paper white above 1.
fn tone_curve(k: &[f64; KNOTS], x: f64) -> f64 {
    let h = 1.0 / (KNOTS - 1) as f64;
    let d: Vec<f64> = k.windows(2).map(|w| (w[1] - w[0]) / h).collect();
    let tangent = |i: usize| -> f64 {
        if i == 0 {
            d[0]
        } else if i == KNOTS - 1 {
            d[KNOTS - 2]
        } else if d[i - 1] * d[i] <= 0.0 {
            0.0
        } else {
            2.0 / (1.0 / d[i - 1] + 1.0 / d[i])
        }
    };
    if x <= 0.0 {
        let s = tangent(0).max(1e-6);
        return if k[0] > 1e-6 {
            k[0] * (s * x / k[0]).exp()
        } else {
            (s * x).max(0.0)
        };
    }
    if x >= 1.0 {
        let room = (1.0 - k[KNOTS - 1]).max(1e-6);
        let s = tangent(KNOTS - 1).max(1e-6);
        return 1.0 - room * (-s * (x - 1.0) / room).exp();
    }
    let i = ((x / h) as usize).min(KNOTS - 2);
    let t = (x - i as f64 * h) / h;
    let (m0, m1) = (tangent(i) * h, tangent(i + 1) * h);
    let (t2, t3) = (t * t, t * t * t);
    (2.0 * t3 - 3.0 * t2 + 1.0) * k[i]
        + (t3 - 2.0 * t2 + t) * m0
        + (-2.0 * t3 + 3.0 * t2) * k[i + 1]
        + (t3 - t2) * m1
}

/// Power that leaves 0 and 1 in place and continues straight outside them.
fn pow_ext(x: f64, g: f64) -> f64 {
    if x <= 0.0 {
        x
    } else if x >= 1.0 {
        1.0 + (x - 1.0) * g
    } else {
        x.powf(g)
    }
}

/// Smooth bump on 0..1, zero at both ends and 1 at its peak `a / (a + 2)`.
/// Flat at 1, so a highlight bump leaves the white end's slope alone instead
/// of folding the brightest tones together.
fn bump(x: f64, a: f64) -> f64 {
    if x <= 0.0 || x >= 1.0 {
        return 0.0;
    }
    let peak = a / (a + 2.0);
    let max = peak.powf(a) * (1.0 - peak).powi(2);
    x.powf(a) * (1.0 - x).powi(2) / max
}

/// Highlight bump; `range` 1..=10 widens it towards the midtones.
fn highlight_bump(x: f64, range: f64) -> f64 {
    bump(x, 2.0 + (11.0 - range.clamp(1.0, 10.0)) * 0.8)
}

fn shadow_bump(x: f64, range: f64) -> f64 {
    highlight_bump(1.0 - x, range)
}

/// Steepest slope of [`highlight_bump`] at `range`: an amount above its
/// reciprocal would fold the tone curve.
fn bump_steepness(range: f64) -> f64 {
    const N: usize = 256;
    (0..N)
        .map(|i| {
            let (a, b) = (i as f64 / N as f64, (i + 1) as f64 / N as f64);
            (highlight_bump(b, range) - highlight_bump(a, range)).abs() * N as f64
        })
        .fold(0.0, f64::max)
}

/// A bump amount limited so the curve keeps a tenth of its slope.
fn unfolded(amount: f64, range: f64) -> f64 {
    let max = 0.9 / bump_steepness(range);
    amount.clamp(-max, max)
}

/// Auto midtone gamma: moves the mean normalised median density towards
/// 0.5. Bright (high-key) frames may come down further than dark ones go
/// up, like a lab printer exposing for the frame's average density.
pub fn auto_gamma(l: &Levels) -> f64 {
    let m: f64 = (0..3).map(|c| l[c].normalise(l[c].mid)).sum::<f64>() / 3.0;
    if !(m > 0.0 && m < 1.0) {
        return 1.0;
    }
    (0.5f64.ln() / m.ln()).clamp(0.8, 1.6)
}

/// Everything one channel's curve depends on.
#[derive(Debug, Clone, Copy)]
struct ChannelCurve {
    kind: ImageType,
    /// The analysed density range and neutral curve.
    levels: ChannelLevels,
    /// Black and white clip, as fractions of the range.
    clip: (f64, f64),
    gamma: f64,
    /// Mid-tone colour gamma of this channel.
    mid_gamma: f64,
    whites: f64,
    blacks: f64,
    /// Lights / Darks bump amounts.
    highlights: f64,
    shadows: f64,
    /// Highlight / shadow colour toning amounts and ranges.
    tone_high: f64,
    tone_shadow: f64,
    high_range: f64,
    shadow_range: f64,
    contrast: f64,
    /// Paper log exposure added (filtration, white balance, print time).
    exposure: f64,
}

impl ChannelCurve {
    fn new(l: &Levels, s: &Settings, c: usize) -> Self {
        let (auto_tone, _) = profile(s.tone_profile);
        let kind = s.image_type;
        // Clip moves the black / white end into the range (− recovers).
        let clip = (s.black_clip as f64 * 0.006, s.white_clip as f64 * 0.006);
        let auto = if auto_tone && kind == ImageType::Negative {
            auto_gamma(l)
        } else {
            1.0
        };
        let brightness = 2f64.powf(-(s.brightness as f64) / 50.0 * 0.7);
        let f = |v: f32| v as f64;
        // Mids, highlight and shadow toning per channel: + cyan / magenta /
        // yellow takes away red / green / blue.
        let (mid, high, shadow) = match c {
            0 => (s.mid_cyan, s.high_cyan, s.shadow_cyan),
            1 => (s.mid_tint, s.high_tint, s.shadow_tint),
            _ => (s.mid_temp, s.high_temp, s.shadow_temp),
        };
        let wb = [0.0, f(s.wb_tint), f(s.wb_temp)][c];
        let filtration = f(s.filtration[c]) + wb;
        Self {
            kind,
            levels: l[c],
            clip,
            gamma: auto * brightness,
            mid_gamma: 2f64.powf(f(mid) / 75.0 * 0.5),
            whites: f(s.whites) - f(s.glow),
            blacks: f(s.blacks) + f(s.fade),
            // Lab Glow / Fade add to Lights / Darks, so keep both unfolded.
            highlights: unfolded(0.10 * (f(s.highlights) + f(s.glow) / 1.5) / 50.0, 5.0),
            shadows: unfolded(0.10 * (f(s.shadows) - f(s.fade) / 1.5) / 50.0, 5.0),
            tone_high: unfolded(-f(high) / 75.0 * 0.12, f(s.high_range)),
            tone_shadow: unfolded(-f(shadow) / 75.0 * 0.12, f(s.shadow_range)),
            high_range: f(s.high_range),
            shadow_range: f(s.shadow_range),
            // Reciprocal below 0, so −50 flattens as much as +50 steepens.
            contrast: if s.contrast >= 0.0 {
                1.0 + f(s.contrast) * 0.015
            } else {
                1.0 / (1.0 - f(s.contrast) * 0.015)
            },
            exposure: filtration * CC - f(s.print_exposure) * std::f64::consts::LOG10_2,
        }
    }

    /// Normalised density: 0 prints black, 1 prints white.
    fn normalised(&self, d: f64) -> f64 {
        let x = match self.kind {
            ImageType::Negative => self.levels.normalise(d),
            ImageType::Positive => 1.0 - self.levels.normalise(d),
            // Already positive: d is −log10 of the linear value.
            ImageType::EditOnly => return color::srgb_encode(10f64.powf(-d) as f32) as f64,
        };
        let (b, w) = self.clip;
        (x - b) / (1.0 - b - w).max(1e-3)
    }

    /// Shaped tone before the paper (1 = white).
    fn tone(&self, d: f64) -> f64 {
        let mut x = self.normalised(d);
        x = x * (1.0 + self.whites * 0.006) + self.blacks * 0.004 * (1.0 - x);
        x = pow_ext(x, self.gamma * self.mid_gamma);
        x += self.highlights * highlight_bump(x, 5.0);
        x += self.shadows * shadow_bump(x, 5.0);
        x += self.tone_high * highlight_bump(x, self.high_range);
        x += self.tone_shadow * shadow_bump(x, self.shadow_range);
        0.5 + (x - 0.5) * self.contrast
    }

    /// Paper log exposure (0 = white … `PAPER_RANGE` = black).
    fn exposure(&self, d: f64) -> f64 {
        (1.0 - self.tone(d)) * PAPER_RANGE + self.exposure
    }

    /// Print tone before the tone profile (1 = white).
    fn print_tone(&self, d: f64) -> f64 {
        1.0 - self.exposure(d) / PAPER_RANGE
    }
}

/// A tone curve `f` applied to a colour as Lightroom applies a master
/// curve (the DNG spec's RGBTone): to the largest and the smallest channel,
/// with the middle one kept at its place between them. A grey gets `f`
/// exactly; a colour keeps its hue where curving each channel on its own
/// would crush the weak channels in a toe or shoulder and pull it towards
/// a primary.
#[inline]
fn rgb_tone(x: [f32; 3], f: impl Fn(f32) -> f32) -> [f32; 3] {
    let (hi, lo) = if x[0] >= x[1] { (0, 1) } else { (1, 0) };
    let (hi, mid, lo) = if x[2] > x[hi] {
        (2, hi, lo)
    } else if x[2] < x[lo] {
        (hi, lo, 2)
    } else {
        (hi, 2, lo)
    };
    let mut o = [0.0f32; 3];
    o[hi] = f(x[hi]);
    o[lo] = f(x[lo]);
    let span = x[hi] - x[lo];
    o[mid] = if span > 1e-6 {
        o[lo] + (o[hi] - o[lo]) * (x[mid] - x[lo]) / span
    } else {
        o[hi]
    };
    o
}

/// Lookup tables of the print: per channel, density above base → print
/// tone before the tone profile; then the profile, shared by the channels
/// and applied hue-preservingly ([`CurveLut::apply_rgb`]).
#[derive(Clone)]
pub struct CurveLut {
    /// Per channel: density → print tone, by [`lut_density`].
    pub tone: [Vec<f32>; 3],
    /// Print tone → encoded positive, by [`profile_x`]. Identity for
    /// already positive scans and the unconverted view.
    pub profile: Vec<f32>,
    /// Each channel through the profile alone (0..1): the neutral curves.
    pub lut: [Vec<f32>; 3],
    /// Density span shown by [`CurveLut::sample`].
    view: (f64, f64),
}

/// Strong tone adjustments must never fold a table back.
fn monotonic(v: &mut [f32]) {
    if v[v.len() - 1] >= v[0] {
        for i in 1..v.len() {
            v[i] = v[i].max(v[i - 1]);
        }
    } else {
        for i in 1..v.len() {
            v[i] = v[i].min(v[i - 1]);
        }
    }
}

#[inline]
fn lerp_table(v: &[f32], x: f32) -> f32 {
    let x = x.clamp(0.0, (v.len() - 1) as f32);
    let i = (x as usize).min(v.len() - 2);
    let t = x - i as f32;
    v[i] + (v[i + 1] - v[i]) * t
}

impl CurveLut {
    pub fn new(l: &Levels, s: &Settings) -> Self {
        let tone = std::array::from_fn(|c| {
            let cc = ChannelCurve::new(l, s, c);
            let mut v: Vec<f32> = (0..LUT_SIZE)
                .map(|i| cc.print_tone(lut_density(i)) as f32)
                .collect();
            monotonic(&mut v);
            v
        });
        let profile = if s.image_type == ImageType::EditOnly {
            identity_profile()
        } else {
            let (_, knots) = profile(s.tone_profile);
            let mut v: Vec<f32> = (0..LUT_SIZE)
                .map(|i| tone_curve(&knots, profile_x(i)) as f32)
                .collect();
            monotonic(&mut v);
            v
        };
        let view = if s.image_type == ImageType::EditOnly {
            (0.0, 2.5)
        } else {
            let lo = l.iter().map(|c| c.lo).fold(f64::MAX, f64::min);
            let hi = l.iter().map(|c| c.hi).fold(f64::MIN, f64::max);
            let pad = (hi - lo) * 0.1;
            (lo - pad, hi + pad)
        };
        Self::with(tone, profile, view)
    }

    /// The unconverted negative as scanned: base at 0.97 linear.
    pub fn negative_view() -> Self {
        let tone = std::array::from_fn(|_| {
            (0..LUT_SIZE)
                .map(|i| color::srgb_encode(0.97 * 10f32.powf(-lut_density(i) as f32)))
                .collect()
        });
        Self::with(tone, identity_profile(), (0.0, 2.5))
    }

    fn with(tone: [Vec<f32>; 3], profile: Vec<f32>, view: (f64, f64)) -> Self {
        let lut = std::array::from_fn(|c| {
            tone[c]
                .iter()
                .map(|&x| lerp_table(&profile, profile_index(x)).clamp(0.0, 1.0))
                .collect()
        });
        Self {
            tone,
            profile,
            lut,
            view,
        }
    }

    #[inline]
    fn index(d: f32) -> f32 {
        (d - LUT_DMIN as f32) / (LUT_DMAX - LUT_DMIN) as f32 * (LUT_SIZE - 1) as f32
    }

    /// One channel through the profile alone: exact for neutrals.
    #[inline]
    pub fn apply(&self, c: usize, d: f32) -> f32 {
        lerp_table(&self.lut[c], Self::index(d))
    }

    /// The print of a density triplet: each channel's print tone, then the
    /// tone profile, hue-preservingly ([`rgb_tone`]).
    #[inline]
    pub fn apply_rgb(&self, d: [f32; 3]) -> [f32; 3] {
        let x: [f32; 3] = std::array::from_fn(|c| lerp_table(&self.tone[c], Self::index(d[c])));
        rgb_tone(x, |v| lerp_table(&self.profile, profile_index(v))).map(|v| v.clamp(0.0, 1.0))
    }

    /// Curves sampled for display: x 0..1 across the analysed density span
    /// (thin to dense), y per channel.
    pub fn sample(&self, n: usize) -> Vec<[f32; 4]> {
        (0..n)
            .map(|i| {
                let t = i as f64 / (n - 1) as f64;
                let d = (self.view.0 + t * (self.view.1 - self.view.0)) as f32;
                [
                    t as f32,
                    self.apply(0, d),
                    self.apply(1, d),
                    self.apply(2, d),
                ]
            })
            .collect()
    }
}

fn lut_density(i: usize) -> f64 {
    LUT_DMIN + i as f64 / (LUT_SIZE - 1) as f64 * (LUT_DMAX - LUT_DMIN)
}

/// Print tone at entry `i` of the profile table.
fn profile_x(i: usize) -> f64 {
    PROFILE_XMIN + i as f64 / (LUT_SIZE - 1) as f64 * (PROFILE_XMAX - PROFILE_XMIN)
}

#[inline]
fn profile_index(x: f32) -> f32 {
    (x - PROFILE_XMIN as f32) / (PROFILE_XMAX - PROFILE_XMIN) as f32 * (LUT_SIZE - 1) as f32
}

fn identity_profile() -> Vec<f32> {
    (0..LUT_SIZE).map(|i| profile_x(i) as f32).collect()
}

/// The colour picker: white balance (temp, tint) that prints the sampled
/// density `d` neutral. With equal paper exposure in all three channels
/// the print is grey, so temp and tint are the blue and green exposure
/// differences from red, in CC.
pub fn eyedropper_wb(d: [f64; 3], l: &Levels, s: &Settings) -> Result<(f32, f32), &'static str> {
    let s = Settings {
        wb_temp: 0.0,
        wb_tint: 0.0,
        ..s.clone()
    };
    let cc: [ChannelCurve; 3] = std::array::from_fn(|c| ChannelCurve::new(l, &s, c));
    let x: [f64; 3] = std::array::from_fn(|c| cc[c].tone(d[c]));
    if x.iter().any(|v| *v > 0.97) {
        return Err(
            "The area you've selected is too bright. Please try something closer to neutral grey.",
        );
    }
    if x.iter().any(|v| *v < 0.03) {
        return Err(
            "The area you've selected is too dark. Please try something closer to neutral grey.",
        );
    }
    let u: [f64; 3] = std::array::from_fn(|c| cc[c].exposure(d[c]));
    Ok((((u[0] - u[2]) / CC) as f32, ((u[0] - u[1]) / CC) as f32))
}

/// Temp / tint (CC) that bring normalised channel values `x` to neutral.
pub fn balance_wb(x: [f64; 3]) -> (f32, f32) {
    let u = x.map(|v| (1.0 - v) * PAPER_RANGE);
    (((u[0] - u[2]) / CC) as f32, ((u[0] - u[1]) / CC) as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn levels() -> Levels {
        [
            ChannelLevels {
                lo: 0.10,
                hi: 1.00,
                mid: 0.50,
                bow: 0.0,
            },
            ChannelLevels {
                lo: 0.12,
                hi: 1.10,
                mid: 0.58,
                bow: 0.0,
            },
            ChannelLevels {
                lo: 0.15,
                hi: 1.25,
                mid: 0.66,
                bow: 0.0,
            },
        ]
    }

    fn mid_grey(l: &Levels, c: usize) -> f32 {
        (l[c].lo + (l[c].hi - l[c].lo) * 0.5) as f32
    }

    /// A bowed channel still maps its ends to 0 and 1, stays monotone
    /// beyond them, and reads a grey on its curve as the green value.
    #[test]
    fn bowed_levels_follow_the_neutral_curve() {
        for q in [-0.4, 0.3] {
            let l = ChannelLevels {
                lo: 0.2,
                hi: 1.4,
                mid: 0.8,
                bow: q,
            };
            assert!(l.normalise(0.2).abs() < 1e-9 && (l.normalise(1.4) - 1.0).abs() < 1e-9);
            let mut prev = f64::MIN;
            for i in -20..=140 {
                let x = l.normalise(i as f64 * 0.01);
                assert!(x > prev, "bow {q}: not monotone at {i}");
                prev = x;
            }
            for x in [-0.2, 0.1, 0.5, 0.9, 1.3] {
                let d = l.density(x);
                assert!((l.normalise(d) - x).abs() < 1e-9, "bow {q} at {x}");
            }
            let d = l.lo + (l.hi - l.lo) * ((1.0 + q) * 0.3 - q * 0.09);
            assert!((l.normalise(d) - 0.3).abs() < 1e-9, "bow {q}");
        }
    }

    #[test]
    fn negative_prints_thin_dark_and_dense_light() {
        let l = levels();
        let lut = CurveLut::new(&l, &Settings::default());
        for c in 0..3 {
            assert!(lut.apply(c, l[c].lo as f32) < 0.1, "black end");
            assert!(lut.apply(c, l[c].hi as f32) > 0.9, "white end");
            assert!(lut.apply(c, 0.4) < lut.apply(c, 0.7));
        }
    }

    #[test]
    fn tone_profile_keeps_the_hue_of_saturated_shadows() {
        let l = levels();
        let lut = CurveLut::new(&l, &Settings::default());
        // A grey prints as each channel's own curve.
        let g = std::array::from_fn(|c| mid_grey(&l, c));
        let p = lut.apply_rgb(g);
        for c in 0..3 {
            assert!((p[c] - lut.apply(c, g[c])).abs() < 1e-3, "{p:?}");
        }
        // A yellow-green keeps where its red sits between blue and green;
        // its strongest and weakest channels print as on their own.
        let d = [0.5, 0.75, 0.4];
        let x: [f32; 3] = std::array::from_fn(|c| lerp_table(&lut.tone[c], CurveLut::index(d[c])));
        let place = |v: [f32; 3]| (v[0] - v[2]) / (v[1] - v[2]);
        let p = lut.apply_rgb(d);
        assert!((place(p) - place(x)).abs() < 1e-3, "{p:?} {x:?}");
        for c in [1, 2] {
            assert!((p[c] - lut.apply(c, d[c])).abs() < 1e-5, "{p:?}");
        }
    }

    #[test]
    fn normalised_mid_prints_mid_grey_in_every_channel() {
        let l = levels();
        let s = Settings {
            tone_profile: ToneProfile::AllSoft,
            ..Settings::default()
        };
        let lut = CurveLut::new(&l, &s);
        let y: Vec<f32> = (0..3).map(|c| lut.apply(c, mid_grey(&l, c))).collect();
        // Every channel lands on the same value: the slopes are matched.
        assert!(
            (y[0] - y[1]).abs() < 1e-3 && (y[1] - y[2]).abs() < 1e-3,
            "{y:?}"
        );
    }

    #[test]
    fn print_exposure_behaves_like_paper() {
        let l = levels();
        let at = |ev: f32| {
            let s = Settings {
                print_exposure: ev,
                ..Settings::default()
            };
            CurveLut::new(&l, &s).apply(1, mid_grey(&l, 1))
        };
        let m = at(0.0);
        assert!(at(1.0) > m && at(-1.0) < m);
        assert!(at(2.0) > at(1.0));
        // Extremes saturate towards paper white / max black, but never flip.
        assert!(at(5.0) > 0.95);
        assert!(at(-5.0) < 0.05);
    }

    #[test]
    fn warm_temp_reduces_blue() {
        let l = levels();
        let s0 = Settings::default();
        let s1 = Settings {
            wb_temp: 10.0,
            ..s0.clone()
        };
        let d = mid_grey(&l, 2);
        assert!(CurveLut::new(&l, &s1).apply(2, d) < CurveLut::new(&l, &s0).apply(2, d));
    }

    #[test]
    fn eyedropper_neutralises_the_sample() {
        let l = levels();
        let s = Settings::default();
        // A grey with a cast: green and blue a little dense.
        let d = [0.5, 0.65, 0.78];
        let (t, n) = eyedropper_wb(d, &l, &s).unwrap();
        let s = Settings {
            wb_temp: t,
            wb_tint: n,
            ..s
        };
        let lut = CurveLut::new(&l, &s);
        let y: Vec<f32> = (0..3).map(|c| lut.apply(c, d[c] as f32)).collect();
        assert!(
            (y[0] - y[1]).abs() < 2e-3 && (y[0] - y[2]).abs() < 2e-3,
            "{y:?}"
        );
    }

    #[test]
    fn extreme_tone_settings_stay_monotonic() {
        let l = levels();
        let s = Settings {
            highlights: 50.0,
            shadows: -50.0,
            whites: 50.0,
            blacks: -50.0,
            contrast: 50.0,
            high_cyan: 75.0,
            shadow_temp: -75.0,
            ..Settings::default()
        };
        let lut = CurveLut::new(&l, &s);
        for c in 0..3 {
            assert!(lut.lut[c].windows(2).all(|w| w[1] >= w[0]));
        }
    }

    /// Lights, Darks, Glow, Fade and toning at full travel never fold the
    /// curve: the raw tone keeps rising, before `monotonic` would flatten it.
    #[test]
    fn tone_bumps_never_fold_the_curve() {
        let l = levels();
        for v in [-1.0f32, 1.0] {
            for range in [1.0, 5.0, 10.0] {
                let s = Settings {
                    highlights: 50.0 * v,
                    shadows: -50.0 * v,
                    glow: 50.0 * v,
                    fade: 50.0 * v,
                    high_cyan: 75.0 * v,
                    high_temp: -75.0 * v,
                    shadow_tint: 75.0 * v,
                    high_range: range,
                    shadow_range: range,
                    ..Settings::default()
                };
                for c in 0..3 {
                    let cc = ChannelCurve::new(&l, &s, c);
                    let (lo, hi) = (l[c].lo, l[c].hi);
                    let mut prev = f64::MIN;
                    for i in 0..=1000 {
                        let t = cc.tone(lo + (hi - lo) * i as f64 / 1000.0);
                        assert!(t > prev, "{v} range {range} channel {c} at {i}");
                        prev = t;
                    }
                }
            }
        }
    }

    /// Lights and Darks move the upper and lower midtones by about a tenth
    /// and leave black and white where they are.
    #[test]
    fn lights_and_darks_move_their_midtones() {
        let l = levels();
        let tone = |s: &Settings, x: f64| ChannelCurve::new(&l, s, 1).tone(l[1].density(x));
        let s0 = Settings::default();
        let lights = Settings {
            highlights: 50.0,
            ..s0.clone()
        };
        let darks = Settings {
            shadows: 50.0,
            ..s0.clone()
        };
        let lift = |s: &Settings, x: f64| tone(s, x) - tone(&s0, x);
        assert!((lift(&lights, 0.77) - 0.10).abs() < 0.01);
        assert!((lift(&darks, 0.23) - 0.10).abs() < 0.01);
        assert!(lift(&lights, 0.25).abs() < 0.01 && lift(&darks, 0.75).abs() < 0.01);
        for s in [&lights, &darks] {
            assert!(lift(s, 0.0).abs() < 1e-9 && lift(s, 1.0).abs() < 1e-9);
        }
    }

    /// Contrast ±50 steepen and flatten the midtones by the same factor.
    #[test]
    fn contrast_is_symmetric_in_slope() {
        let l = levels();
        let slope = |c: f32| {
            let s = Settings {
                contrast: c,
                ..Settings::default()
            };
            let cc = ChannelCurve::new(&l, &s, 1);
            cc.tone(l[1].density(0.55)) - cc.tone(l[1].density(0.45))
        };
        let (up, flat, down) = (slope(50.0), slope(0.0), slope(-50.0));
        assert!((up / flat - flat / down).abs() < 1e-6, "{up} {flat} {down}");
        assert!((up / flat - 1.75).abs() < 1e-6);
    }
}
