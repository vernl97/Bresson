//! The print engine: turns film density into a positive the way an enlarger
//! prints a negative onto paper.
//!
//! The input is per-channel density above the film base (`pipeline::Negative`).
//! Each channel's analysed density range ([`ChannelLevels`]) is normalised to
//! 0 (deepest shadow, printed black) … 1 (brightest highlight, printed white).
//! Normalising every channel to its own range matches the layers' different
//! H&D slopes, so neutrals stay neutral from shadows to highlights. Tone and
//! colour controls shape the normalised density. It then becomes a paper log
//! exposure (plus printing filtration and print time), and a paper
//! characteristic curve with a toe and shoulder gives the print density.
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
}

pub type Levels = [ChannelLevels; 3];

/// Log exposure range of the paper between printed white and printed black
/// (ISO R ≈ 105, a grade-2 paper).
pub const PAPER_RANGE: f64 = 1.05;
/// Maximum (asymptotic) print density.
const PAPER_DMAX: f64 = 2.2;
/// One CC unit of printing filtration, in log exposure.
pub const CC: f64 = 0.01;

/// Density range the lookup tables cover.
const LUT_DMIN: f64 = -0.5;
const LUT_DMAX: f64 = 4.5;
pub const LUT_SIZE: usize = 8192;

/// A paper / scanner tone curve.
#[derive(Debug, Clone, Copy)]
struct Paper {
    /// Print density where normalised density 1 lands (paper white side).
    white: f64,
    /// Print density where normalised density 0 lands (black side).
    black: f64,
    /// Contrast (slope multiplier around the midtone).
    grade: f64,
    /// Midtone gamma (> 1 is denser).
    gamma: f64,
    /// Place the median density on mid grey.
    auto: bool,
    /// No paper curve: the normalised density is the encoded output.
    linear: bool,
}

fn paper(p: ToneProfile) -> Paper {
    let base = Paper {
        white: 0.06,
        black: 2.0,
        grade: 1.0,
        gamma: 1.0,
        auto: true,
        linear: false,
    };
    match p {
        ToneProfile::Standard => base,
        ToneProfile::Soft => Paper {
            grade: 0.85,
            ..base
        },
        ToneProfile::Hard => Paper { grade: 1.2, ..base },
        ToneProfile::Rich => Paper {
            grade: 1.1,
            gamma: 1.08,
            black: 2.1,
            ..base
        },
        ToneProfile::Flat => Paper {
            grade: 0.75,
            white: 0.12,
            black: 1.6,
            auto: false,
            ..base
        },
        ToneProfile::Linear => Paper {
            auto: false,
            linear: true,
            ..base
        },
    }
}

fn logit(p: f64) -> f64 {
    (p / (1.0 - p)).ln()
}

/// Paper characteristic curve: print density at paper log exposure `u`,
/// where `u = 0` prints `white` and `u = PAPER_RANGE` prints `black`. A
/// logistic in log exposure, so the toe and shoulder compress smoothly
/// towards paper white and maximum black instead of clipping.
fn print_density(u: f64, p: &Paper) -> f64 {
    let lw = logit(p.white / PAPER_DMAX);
    let lb = logit(p.black / PAPER_DMAX);
    let l = lw + (lb - lw) * (u / PAPER_RANGE);
    PAPER_DMAX / (1.0 + (-l).exp())
}

/// Print density → linear output, with maximum black at 0 and paper white at 1.
fn print_to_linear(d: f64) -> f64 {
    let r_min = 10f64.powf(-PAPER_DMAX);
    (10f64.powf(-d) - r_min) / (1.0 - r_min)
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

/// Smooth bump on 0..1, zero at both ends and 1 at its peak `a / (a + 1)`.
fn bump(x: f64, a: f64) -> f64 {
    if x <= 0.0 || x >= 1.0 {
        return 0.0;
    }
    let peak = a / (a + 1.0);
    let max = peak.powf(a) * (1.0 - peak);
    x.powf(a) * (1.0 - x) / max
}

/// Highlight bump; `range` 1..=10 widens it towards the midtones.
fn highlight_bump(x: f64, range: f64) -> f64 {
    bump(x, 1.0 + (11.0 - range.clamp(1.0, 10.0)) * 0.4)
}

fn shadow_bump(x: f64, range: f64) -> f64 {
    highlight_bump(1.0 - x, range)
}

/// Auto midtone gamma: puts the mean normalised median density on 0.5.
pub fn auto_gamma(l: &Levels) -> f64 {
    let m: f64 = (0..3)
        .map(|c| (l[c].mid - l[c].lo) / (l[c].hi - l[c].lo).max(1e-6))
        .sum::<f64>()
        / 3.0;
    if !(m > 0.0 && m < 1.0) {
        return 1.0;
    }
    (0.5f64.ln() / m.ln()).clamp(0.8, 1.25)
}

/// Everything one channel's curve depends on.
#[derive(Debug, Clone, Copy)]
struct ChannelCurve {
    kind: ImageType,
    /// Density printing black (negatives) / white (positives), after clipping.
    lo: f64,
    hi: f64,
    paper: Paper,
    gamma: f64,
    /// Mid-tone colour gamma of this channel.
    mid_gamma: f64,
    whites: f64,
    blacks: f64,
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
        let p = paper(s.tone_profile);
        let kind = s.image_type;
        let span = (l[c].hi - l[c].lo).max(1e-3);
        // Clip moves the black / white end into the range (− recovers).
        let (wc, bc) = (s.white_clip as f64 * 0.006, s.black_clip as f64 * 0.006);
        let (lo, hi) = match kind {
            ImageType::Positive => (l[c].lo + wc * span, l[c].hi - bc * span),
            _ => (l[c].lo + bc * span, l[c].hi - wc * span),
        };
        let auto = if p.auto && kind == ImageType::Negative {
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
            lo,
            hi,
            paper: p,
            gamma: p.gamma * auto * brightness,
            mid_gamma: 2f64.powf(f(mid) / 75.0 * 0.5),
            whites: f(s.whites) - f(s.glow),
            blacks: f(s.blacks) + f(s.fade),
            highlights: f(s.highlights) + f(s.glow) / 1.5,
            shadows: f(s.shadows) - f(s.fade) / 1.5,
            tone_high: -f(high) / 75.0 * 0.12,
            tone_shadow: -f(shadow) / 75.0 * 0.12,
            high_range: f(s.high_range),
            shadow_range: f(s.shadow_range),
            contrast: p.grade * (1.0 + f(s.contrast) * 0.015),
            exposure: filtration * CC - f(s.print_exposure) * std::f64::consts::LOG10_2,
        }
    }

    /// Normalised density: 0 prints black, 1 prints white.
    fn normalised(&self, d: f64) -> f64 {
        match self.kind {
            ImageType::Negative => (d - self.lo) / (self.hi - self.lo).max(1e-6),
            ImageType::Positive => (self.hi - d) / (self.hi - self.lo).max(1e-6),
            // Already positive: d is −log10 of the linear value.
            ImageType::EditOnly => color::srgb_encode(10f64.powf(-d) as f32) as f64,
        }
    }

    /// Shaped tone before the paper (1 = white).
    fn tone(&self, d: f64) -> f64 {
        let mut x = self.normalised(d);
        x = x * (1.0 + self.whites * 0.006) + self.blacks * 0.004 * (1.0 - x);
        x = pow_ext(x, self.gamma * self.mid_gamma);
        x += 0.10 * self.highlights / 50.0 * highlight_bump(x, 5.0);
        x += 0.10 * self.shadows / 50.0 * shadow_bump(x, 5.0);
        x += self.tone_high * highlight_bump(x, self.high_range);
        x += self.tone_shadow * shadow_bump(x, self.shadow_range);
        0.5 + (x - 0.5) * self.contrast
    }

    /// Paper log exposure (0 = white … `PAPER_RANGE` = black).
    fn exposure(&self, d: f64) -> f64 {
        (1.0 - self.tone(d)) * PAPER_RANGE + self.exposure
    }

    /// Encoded output (sRGB tone response).
    fn output(&self, d: f64) -> f64 {
        let u = self.exposure(d);
        if self.paper.linear {
            return 1.0 - u / PAPER_RANGE;
        }
        let lin = print_to_linear(print_density(u, &self.paper));
        color::srgb_encode(lin as f32) as f64
    }
}

/// Per-channel lookup table: density above base → encoded positive (0..1).
#[derive(Clone)]
pub struct CurveLut {
    pub lut: [Vec<f32>; 3],
    /// Density span shown by [`CurveLut::sample`].
    view: (f64, f64),
}

impl CurveLut {
    pub fn new(l: &Levels, s: &Settings) -> Self {
        let lut = std::array::from_fn(|c| {
            let cc = ChannelCurve::new(l, s, c);
            let mut v: Vec<f32> = (0..LUT_SIZE)
                .map(|i| cc.output(lut_density(i)).clamp(0.0, 1.0) as f32)
                .collect();
            // Strong tone adjustments must never fold the curve back.
            if v[LUT_SIZE - 1] >= v[0] {
                for i in 1..LUT_SIZE {
                    v[i] = v[i].max(v[i - 1]);
                }
            } else {
                for i in 1..LUT_SIZE {
                    v[i] = v[i].min(v[i - 1]);
                }
            }
            v
        });
        let view = if s.image_type == ImageType::EditOnly {
            (0.0, 2.5)
        } else {
            let lo = l.iter().map(|c| c.lo).fold(f64::MAX, f64::min);
            let hi = l.iter().map(|c| c.hi).fold(f64::MIN, f64::max);
            let pad = (hi - lo) * 0.1;
            (lo - pad, hi + pad)
        };
        Self { lut, view }
    }

    /// The unconverted negative as scanned: base at 0.97 linear.
    pub fn negative_view() -> Self {
        let lut = std::array::from_fn(|_| {
            (0..LUT_SIZE)
                .map(|i| color::srgb_encode(0.97 * 10f32.powf(-lut_density(i) as f32)))
                .collect()
        });
        Self {
            lut,
            view: (0.0, 2.5),
        }
    }

    #[inline]
    pub fn apply(&self, c: usize, d: f32) -> f32 {
        let x = ((d - LUT_DMIN as f32) / (LUT_DMAX - LUT_DMIN) as f32 * (LUT_SIZE - 1) as f32)
            .clamp(0.0, (LUT_SIZE - 1) as f32);
        let i = x as usize;
        let l = &self.lut[c];
        if i >= LUT_SIZE - 1 {
            return l[LUT_SIZE - 1];
        }
        let t = x - i as f32;
        l[i] + (l[i + 1] - l[i]) * t
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
            },
            ChannelLevels {
                lo: 0.12,
                hi: 1.10,
                mid: 0.58,
            },
            ChannelLevels {
                lo: 0.15,
                hi: 1.25,
                mid: 0.66,
            },
        ]
    }

    fn mid_grey(l: &Levels, c: usize) -> f32 {
        (l[c].lo + (l[c].hi - l[c].lo) * 0.5) as f32
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
    fn normalised_mid_prints_mid_grey_in_every_channel() {
        let l = levels();
        let s = Settings {
            tone_profile: ToneProfile::Soft,
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
}
