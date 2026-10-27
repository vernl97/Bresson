//! A physically modelled colour negative of a ColorChecker, used to measure
//! conversion quality (`synth` CLI command and the engine's quality tests).
//!
//! Scene reflectance (linear ProPhoto, from the chart's CIELAB D50 values)
//! is exposed onto a C-41-like film: each layer has its own speed and H&D
//! slope with a soft toe, the dyes have unwanted absorptions (crosstalk),
//! and the base carries an orange mask. The film is then scanned by a
//! camera whose channels overlap. Because the scene is known, the converted
//! positive can be compared with the chart's reference colours.

use super::color::{self, D50, Mat3, PROPHOTO, SRGB};
use super::decode::SourceImage;

/// ColorChecker Classic, CIELAB D50 (X-Rite, post-2014 formulation).
pub const COLORCHECKER: [[f64; 3]; 24] = [
    [37.54, 14.37, 14.92],
    [64.66, 19.27, 17.50],
    [49.32, -3.82, -22.54],
    [43.46, -12.74, 22.72],
    [54.94, 9.61, -24.79],
    [70.48, -32.26, -0.37],
    [62.73, 35.83, 56.50],
    [39.43, 10.75, -45.17],
    [50.57, 48.64, 16.67],
    [30.10, 22.54, -20.87],
    [71.77, -24.13, 58.19],
    [71.51, 18.24, 67.37],
    [28.37, 15.42, -49.80],
    [54.38, -39.72, 32.27],
    [42.43, 51.05, 28.62],
    [81.80, 2.67, 80.41],
    [50.63, 51.28, -14.12],
    [49.57, -29.71, -28.32],
    [95.19, -1.03, 2.93],
    [81.29, -0.57, 0.44],
    [66.89, -0.75, -0.06],
    [50.76, -0.13, 0.14],
    [35.63, -0.46, -0.48],
    [20.64, 0.07, -0.46],
];

/// Per-layer H&D slope (red, green, blue sensitive layers).
pub const FILM_GAMMA: [f64; 3] = [0.58, 0.64, 0.72];
/// Per-layer speed offset in log exposure (+ = slower).
const FILM_SPEED: [f64; 3] = [0.0, -0.06, 0.08];
/// Log exposure (relative to mid grey) where the toe starts.
const FILM_TOE: f64 = -1.3;
/// Toe softness in log exposure.
const TOE_SOFTNESS: f64 = 0.25;
/// Measured density per unit of dye: rows R/G/B channel, columns C/M/Y dye.
pub const DYE_CROSSTALK: Mat3 = [[1.00, 0.08, 0.02], [0.12, 1.00, 0.06], [0.05, 0.18, 1.00]];
/// Base + fog density (orange mask).
pub const BASE_DENSITY: [f64; 3] = [0.24, 0.56, 0.86];
/// Backlight level at the camera.
const LIGHT: f64 = 1.2;
/// Camera channel overlap (camera RGB from per-channel transmitted light).
const CAMERA_MIX: Mat3 = [[0.90, 0.08, 0.02], [0.06, 0.88, 0.06], [0.02, 0.10, 0.88]];

pub const WIDTH: usize = 720;
pub const HEIGHT: usize = 480;
/// Unexposed rebate around the frame, in pixels.
pub const REBATE: usize = 24;

/// One measured area of the chart.
pub struct Patch {
    /// Reference CIELAB D50.
    pub lab: [f64; 3],
    pub neutral: bool,
    /// Pixel indices of the patch centre.
    pub pixels: Vec<usize>,
}

pub struct Chart {
    pub src: SourceImage,
    /// The 24 ColorChecker patches, then the 12-step grey ramp.
    pub patches: Vec<Patch>,
}

/// Linear ProPhoto reflectance of a CIELAB D50 colour.
fn reflectance(lab: [f64; 3]) -> [f64; 3] {
    let to_rgb = color::mat_inv(&color::rgb_to_xyz(&PROPHOTO));
    color::mat_vec(&to_rgb, color::lab_to_xyz(lab, D50)).map(|v| v.max(1e-4))
}

/// Dye amount formed by one layer at relative log exposure `h`.
fn layer_density(c: usize, h: f64) -> f64 {
    let x = (h - FILM_TOE - FILM_SPEED[c]) / TOE_SOFTNESS;
    // Softplus: a toe that becomes a straight line of slope gamma.
    let soft = if x > 30.0 { x } else { x.exp().ln_1p() };
    FILM_GAMMA[c] * TOE_SOFTNESS * soft
}

/// Camera RGB of the film where the scene had `refl`, exposed `ev` stops
/// over normal (mid grey at log exposure 0). `None` is unexposed rebate.
pub fn scan(refl: Option<[f64; 3]>, ev: f64) -> [f64; 3] {
    let dye = match refl {
        Some(r) => std::array::from_fn(|c| {
            layer_density(c, (r[c] / 0.18).log10() + ev * std::f64::consts::LOG10_2)
        }),
        None => [0.0; 3],
    };
    let d = color::mat_vec(&DYE_CROSSTALK, dye);
    let t: [f64; 3] = std::array::from_fn(|c| LIGHT * 10f64.powf(-(BASE_DENSITY[c] + d[c])));
    color::mat_vec(&CAMERA_MIX, t)
}

/// The chart shot `ev` stops over normal exposure.
pub fn negative(ev: f64) -> Chart {
    let ramp: Vec<f64> = (0..12)
        .map(|i| 0.02 * (0.9f64 / 0.02).powf(i as f64 / 11.0))
        .collect();
    let mut data = vec![[0.0f32; 3]; WIDTH * HEIGHT];
    let mut patches: Vec<Patch> = COLORCHECKER
        .iter()
        .enumerate()
        .map(|(i, lab)| Patch {
            lab: *lab,
            neutral: i >= 18,
            pixels: Vec::new(),
        })
        .chain(ramp.iter().map(|&y| Patch {
            lab: color::xyz_to_lab(color::mat_vec(&color::rgb_to_xyz(&PROPHOTO), [y; 3]), D50),
            neutral: true,
            pixels: Vec::new(),
        }))
        .collect();
    let refl: Vec<[f64; 3]> = COLORCHECKER
        .iter()
        .map(|l| reflectance(*l))
        .chain(ramp.iter().map(|&y| [y; 3]))
        .collect();
    let (x0, x1) = (REBATE, WIDTH - REBATE);
    let (y0, y1) = (REBATE, HEIGHT - REBATE);
    let ramp_bottom = y0 + 72;
    let grid_top = ramp_bottom + 12;
    // Which patch a pixel shows, and whether it is in the patch centre.
    let locate = |x: usize, y: usize| -> Option<(usize, bool)> {
        if y < ramp_bottom {
            let w = (x1 - x0) as f64 / 12.0;
            let i = ((x - x0) as f64 / w) as usize;
            let fx = (x - x0) as f64 / w - i as f64;
            let fy = (y - y0) as f64 / (ramp_bottom - y0) as f64;
            Some((
                24 + i.min(11),
                (0.2..0.8).contains(&fx) && (0.2..0.8).contains(&fy),
            ))
        } else if y >= grid_top {
            let (w, h) = ((x1 - x0) as f64 / 6.0, (y1 - grid_top) as f64 / 4.0);
            let (fx, fy) = ((x - x0) as f64 / w, (y - grid_top) as f64 / h);
            let (cx, cy) = ((fx as usize).min(5), (fy as usize).min(3));
            let (ux, uy) = (fx - cx as f64, fy - cy as f64);
            Some((
                cy * 6 + cx,
                (0.2..0.8).contains(&ux) && (0.2..0.8).contains(&uy),
            ))
        } else {
            None
        }
    };
    // Gaps between the ramp and the grid are a dark surround.
    let surround = [0.03f64; 3];
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let i = y * WIDTH + x;
            let inside = (x0..x1).contains(&x) && (y0..y1).contains(&y);
            let r = if !inside {
                None
            } else {
                match locate(x, y) {
                    Some((p, centre)) => {
                        if centre {
                            patches[p].pixels.push(i);
                        }
                        Some(refl[p])
                    }
                    None => Some(surround),
                }
            };
            data[i] = scan(r, ev).map(|v| v as f32);
        }
    }
    let src = SourceImage {
        width: WIDTH,
        height: HEIGHT,
        data,
        to_melissa: color::to_f32(&color::mat_inv(&CAMERA_MIX)),
        as_shot_wb: [1.0; 3],
        is_raw: false,
        info: super::decode::SourceInfo {
            full_width: WIDTH,
            full_height: HEIGHT,
            ..Default::default()
        },
    };
    Chart { src, patches }
}

/// Quality of a converted chart.
#[derive(Debug, Clone, Copy)]
pub struct ChartMetrics {
    /// Mean CIEDE2000 of the 18 colour patches with lightness matched, i.e.
    /// hue and saturation error independent of the tone curve.
    pub color_de: f64,
    /// Mean CIEDE2000 of all 24 patches.
    pub full_de: f64,
    /// Mean chroma (C*ab) of the neutral patches and ramp.
    pub neutral_cast: f64,
    /// Largest chroma of a neutral patch or ramp step.
    pub neutral_cast_max: f64,
}

/// CIELAB D50 of each patch from encoded sRGB output pixels.
pub fn patch_lab(chart: &Chart, srgb: &[[f32; 3]]) -> Vec<[f64; 3]> {
    let to_pp = color::rgb_to_rgb(&SRGB, &PROPHOTO);
    chart
        .patches
        .iter()
        .map(|p| {
            let mut m = [0.0f64; 3];
            for &i in &p.pixels {
                for c in 0..3 {
                    m[c] += color::srgb_decode(srgb[i][c].clamp(0.0, 1.0)) as f64;
                }
            }
            let m = m.map(|v| v / p.pixels.len().max(1) as f64);
            color::rgb_to_lab(&PROPHOTO, color::mat_vec(&to_pp, m))
        })
        .collect()
}

/// CIEDE2000 between two colours ignoring their lightness difference.
pub fn color_de(a: [f64; 3], b: [f64; 3]) -> f64 {
    color::delta_e2000(a, [a[0], b[1], b[2]])
}

pub fn measure(chart: &Chart, srgb: &[[f32; 3]]) -> ChartMetrics {
    let labs = patch_lab(chart, srgb);
    let mut color_sum = 0.0;
    let mut full_sum = 0.0;
    let (mut cast_sum, mut cast_max, mut n_neutral) = (0.0f64, 0.0f64, 0usize);
    for (i, (p, lab)) in chart.patches.iter().zip(&labs).enumerate() {
        if i < 18 {
            color_sum += color_de(p.lab, *lab);
        }
        if i < 24 {
            full_sum += color::delta_e2000(p.lab, *lab);
        }
        if p.neutral {
            let c = lab[1].hypot(lab[2]);
            cast_sum += c;
            cast_max = cast_max.max(c);
            n_neutral += 1;
        }
    }
    ChartMetrics {
        color_de: color_sum / 18.0,
        full_de: full_sum / 24.0,
        neutral_cast: cast_sum / n_neutral as f64,
        neutral_cast_max: cast_max,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chart_is_a_plausible_negative() {
        let chart = negative(0.0);
        let px = &chart.src.data;
        let rebate = px[2 * WIDTH + 2];
        // Orange mask: red transmits most, blue least.
        assert!(rebate[0] > rebate[1] && rebate[1] > rebate[2], "{rebate:?}");
        // Every exposed patch is denser (darker) than the rebate.
        for p in &chart.patches {
            assert!(p.pixels.len() > 100);
            let v = px[p.pixels[0]];
            assert!((0..3).all(|c| v[c] < rebate[c]), "{v:?}");
        }
        // White is denser than black on the negative.
        let (white, black) = (
            px[chart.patches[18].pixels[0]],
            px[chart.patches[23].pixels[0]],
        );
        assert!(white[1] < black[1]);
        // Reference colours round-trip through the reflectance model.
        let r = reflectance(COLORCHECKER[21]);
        let lab = color::rgb_to_lab(&PROPHOTO, r);
        assert!(color::delta_e2000(lab, COLORCHECKER[21]) < 0.05);
    }
}
