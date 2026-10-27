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

/// Layer response of a film: per layer the toe softness (log exposure)
/// and the dye density from which the layer rolls off into a shoulder.
#[derive(Clone, Copy)]
pub struct Film {
    pub toe: [f64; 3],
    pub shoulder: [f64; 3],
}

/// The chart's film: the same toe in every layer and no shoulder.
pub const C41: Film = Film {
    toe: [TOE_SOFTNESS; 3],
    shoulder: [f64::INFINITY; 3],
};

/// A film whose layers cross over: the blue layer has a long soft toe and
/// the red layer shoulders early, so a grey drifts blue-yellow from
/// shadows to highlights and red-cyan in the highlights.
pub const CROSSOVER: Film = Film {
    toe: [0.2, 0.25, 0.45],
    shoulder: [1.0, f64::INFINITY, f64::INFINITY],
};

/// Dye amount formed by one layer at relative log exposure `h`.
fn layer_density(film: &Film, c: usize, h: f64) -> f64 {
    let t = film.toe[c];
    let x = (h - FILM_TOE - FILM_SPEED[c]) / t;
    // Softplus: a toe that becomes a straight line of slope gamma.
    let soft = if x > 30.0 { x } else { x.exp().ln_1p() };
    let d = FILM_GAMMA[c] * t * soft;
    // Shoulder: half the slope a long way past the knee.
    let k = film.shoulder[c];
    if d > k {
        k + (d - k) / (1.0 + (d - k)).sqrt()
    } else {
        d
    }
}

/// Camera RGB of the film where the scene had `refl`, exposed `ev` stops
/// over normal (mid grey at log exposure 0). `None` is unexposed rebate.
pub fn scan(refl: Option<[f64; 3]>, ev: f64) -> [f64; 3] {
    scan_on(&C41, refl, ev)
}

fn scan_on(film: &Film, refl: Option<[f64; 3]>, ev: f64) -> [f64; 3] {
    let dye = match refl {
        Some(r) => std::array::from_fn(|c| {
            layer_density(
                film,
                c,
                (r[c] / 0.18).log10() + ev * std::f64::consts::LOG10_2,
            )
        }),
        None => [0.0; 3],
    };
    let d = color::mat_vec(&DYE_CROSSTALK, dye);
    let t: [f64; 3] = std::array::from_fn(|c| LIGHT * 10f64.powf(-(BASE_DENSITY[c] + d[c])));
    color::mat_vec(&CAMERA_MIX, t)
}

/// The chart shot `ev` stops over normal exposure.
pub fn negative(ev: f64) -> Chart {
    negative_on(&C41, ev)
}

/// The chart shot on `film`.
pub fn negative_on(film: &Film, ev: f64) -> Chart {
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
            data[i] = scan_on(film, r, ev).map(|v| v as f32);
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

/// A scene dominated by one colour: the grey ramp along the top, the six
/// neutral ColorChecker patches along the bottom, and `field` (CIELAB D50)
/// filling the rest (about 60 % of the frame). Auto colour balance must keep
/// the greys neutral without being pulled towards the field.
pub fn dominated(ev: f64, field: [f64; 3]) -> Chart {
    dominated_shaded(ev, field, 0.0)
}

/// [`dominated`] with the field shaded from `spread` L* darker on the left
/// to `spread` lighter on the right, like a lit surface (skin, sand, wood)
/// that spans many tones and so reaches every tonal step of the analysis.
pub fn dominated_shaded(ev: f64, field: [f64; 3], spread: f64) -> Chart {
    let ramp: Vec<f64> = (0..12)
        .map(|i| 0.02 * (0.9f64 / 0.02).powf(i as f64 / 11.0))
        .collect();
    let neutral_lab =
        |y: f64| color::xyz_to_lab(color::mat_vec(&color::rgb_to_xyz(&PROPHOTO), [y; 3]), D50);
    let mut patches: Vec<Patch> = ramp
        .iter()
        .map(|&y| neutral_lab(y))
        .chain(COLORCHECKER[18..].iter().copied())
        .map(|lab| Patch {
            lab,
            neutral: true,
            pixels: Vec::new(),
        })
        .collect();
    let refl: Vec<[f64; 3]> = ramp
        .iter()
        .map(|&y| [y; 3])
        .chain(COLORCHECKER[18..].iter().map(|l| reflectance(*l)))
        .collect();
    let (x0, x1) = (REBATE, WIDTH - REBATE);
    let shades: Vec<[f64; 3]> = (x0..x1)
        .map(|x| {
            let t = (x - x0) as f64 / (x1 - x0 - 1) as f64 * 2.0 - 1.0;
            reflectance([field[0] + spread * t, field[1], field[2]])
        })
        .collect();
    let (y0, y1) = (REBATE, HEIGHT - REBATE);
    let ramp_bottom = y0 + 56;
    let row_top = y1 - 56;
    let mut data = vec![[0.0f32; 3]; WIDTH * HEIGHT];
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let i = y * WIDTH + x;
            let inside = (x0..x1).contains(&x) && (y0..y1).contains(&y);
            let strip = |n: usize, top: usize, bottom: usize| {
                let w = (x1 - x0) as f64 / n as f64;
                let k = ((x - x0) as f64 / w) as usize;
                let fx = (x - x0) as f64 / w - k as f64;
                let fy = (y - top) as f64 / (bottom - top) as f64;
                (
                    k.min(n - 1),
                    (0.2..0.8).contains(&fx) && (0.2..0.8).contains(&fy),
                )
            };
            let r = if !inside {
                None
            } else if y < ramp_bottom || y >= row_top {
                let (p, centre) = if y < ramp_bottom {
                    strip(12, y0, ramp_bottom)
                } else {
                    let (k, c) = strip(6, row_top, y1);
                    (12 + k, c)
                };
                if centre {
                    patches[p].pixels.push(i);
                }
                Some(refl[p])
            } else {
                Some(shades[x - x0])
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

/// A portrait: a face (an ellipse of `face`, CIELAB D50, shaded 6 L*
/// across) in front of a `background` filling the frame, with the grey
/// ramp along the top. Patch 0 is the centre of the face, then the ramp.
pub fn portrait(ev: f64, background: [f64; 3], face: [f64; 3]) -> Chart {
    let ramp: Vec<f64> = (0..12)
        .map(|i| 0.02 * (0.9f64 / 0.02).powf(i as f64 / 11.0))
        .collect();
    let mut patches: Vec<Patch> = std::iter::once(Patch {
        lab: face,
        neutral: false,
        pixels: Vec::new(),
    })
    .chain(ramp.iter().map(|&y| Patch {
        lab: color::xyz_to_lab(color::mat_vec(&color::rgb_to_xyz(&PROPHOTO), [y; 3]), D50),
        neutral: true,
        pixels: Vec::new(),
    }))
    .collect();
    let (x0, x1) = (REBATE, WIDTH - REBATE);
    let (y0, y1) = (REBATE, HEIGHT - REBATE);
    let ramp_bottom = y0 + 56;
    let back = reflectance(background);
    let (cx, cy) = ((x0 + x1) as f64 / 2.0, (ramp_bottom + y1) as f64 / 2.0);
    let (rx, ry) = (0.11 * WIDTH as f64, 0.17 * HEIGHT as f64);
    let mut data = vec![[0.0f32; 3]; WIDTH * HEIGHT];
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let i = y * WIDTH + x;
            let inside = (x0..x1).contains(&x) && (y0..y1).contains(&y);
            let (u, v) = ((x as f64 - cx) / rx, (y as f64 - cy) / ry);
            let r = if !inside {
                None
            } else if y < ramp_bottom {
                let w = (x1 - x0) as f64 / 12.0;
                let k = (((x - x0) as f64 / w) as usize).min(11);
                let fx = (x - x0) as f64 / w - k as f64;
                let fy = (y - y0) as f64 / (ramp_bottom - y0) as f64;
                if (0.2..0.8).contains(&fx) && (0.2..0.8).contains(&fy) {
                    patches[1 + k].pixels.push(i);
                }
                Some([ramp[k]; 3])
            } else if u * u + v * v <= 1.0 {
                if u * u + v * v <= 0.25 {
                    patches[0].pixels.push(i);
                }
                Some(reflectance([face[0] + 3.0 * u, face[1], face[2]]))
            } else {
                Some(back)
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

/// Mean and largest chroma (C*ab) of a chart's neutral patches.
pub fn neutral_cast(chart: &Chart, srgb: &[[f32; 3]]) -> (f64, f64) {
    let labs = patch_lab(chart, srgb);
    let c: Vec<f64> = chart
        .patches
        .iter()
        .zip(&labs)
        .filter(|(p, _)| p.neutral)
        .map(|(_, l)| l[1].hypot(l[2]))
        .collect();
    (
        c.iter().sum::<f64>() / c.len() as f64,
        c.iter().fold(0.0, |a, &b| a.max(b)),
    )
}

/// Replace the rebate with a scanner holder or mask that reads `density`
/// above the base (many scans can't measure much more than 2–2.5 D).
pub fn with_holder(chart: &mut Chart, density: f64) {
    let holder = scan(None, 0.0).map(|v| (v * 10f64.powf(-density)) as f32);
    let (w, h) = (chart.src.width, chart.src.height);
    for (i, p) in chart.src.data.iter_mut().enumerate() {
        let (x, y) = (i % w, i / w);
        if x < REBATE || x >= w - REBATE || y < REBATE || y >= h - REBATE {
            *p = holder;
        }
    }
}

/// Put a light source (a lamp, 20× brighter than white) in a strip down
/// the left tenth of the frame. Returns the patches it doesn't cover.
pub fn with_light(chart: &mut Chart, ev: f64) -> Vec<usize> {
    let lamp = scan(Some([0.9 * 20.0; 3]), ev).map(|v| v as f32);
    let (w, h) = (chart.src.width, chart.src.height);
    let x1 = REBATE + (w - 2 * REBATE) / 10;
    let inside = |i: usize| {
        let (x, y) = (i % w, i / w);
        (REBATE..x1).contains(&x) && (REBATE..h - REBATE).contains(&y)
    };
    for (i, p) in chart.src.data.iter_mut().enumerate() {
        if inside(i) {
            *p = lamp;
        }
    }
    (0..chart.patches.len())
        .filter(|&k| !chart.patches[k].pixels.iter().any(|&i| inside(i)))
        .collect()
}

/// Uneven illumination (lens falloff plus a light source that is warmer at
/// the edges): camera RGB gain at normalised position `(u, v)`.
pub fn falloff(u: f64, v: f64, strength: f64) -> [f64; 3] {
    let r2 = ((u - 0.5).powi(2) + (v - 0.5).powi(2)) * 2.0;
    [0.80, 0.90, 1.0].map(|k| 1.0 - strength * k * r2)
}

/// Scan the chart through uneven illumination.
pub fn vignette(chart: &mut Chart, strength: f64) {
    let (w, h) = (chart.src.width, chart.src.height);
    for (i, p) in chart.src.data.iter_mut().enumerate() {
        let g = falloff(
            (i % w) as f64 / w as f64,
            (i / w) as f64 / h as f64,
            strength,
        );
        *p = std::array::from_fn(|c| p[c] * g[c] as f32);
    }
}

/// A flat-field frame: the bare light source shot through the same uneven
/// illumination, linear camera RGB.
pub fn flat_frame(strength: f64) -> SourceImage {
    let light = color::mat_vec(&CAMERA_MIX, [LIGHT * 0.7; 3]);
    let data = (0..WIDTH * HEIGHT)
        .map(|i| {
            let g = falloff(
                (i % WIDTH) as f64 / WIDTH as f64,
                (i / WIDTH) as f64 / HEIGHT as f64,
                strength,
            );
            std::array::from_fn(|c| (light[c] * g[c]) as f32)
        })
        .collect();
    SourceImage {
        width: WIDTH,
        height: HEIGHT,
        data,
        to_melissa: color::to_f32(&color::mat_inv(&CAMERA_MIX)),
        as_shot_wb: [1.0; 3],
        is_raw: false,
        info: Default::default(),
    }
}

/// Quality of a converted chart.
#[derive(Debug, Clone, Copy)]
pub struct ChartMetrics {
    /// Mean CIEDE2000 of the 18 colour patches with lightness matched, i.e.
    /// hue and saturation error independent of the tone curve.
    pub color_de: f64,
    /// Mean CIEDE2000 of the 18 colour patches against what a perfect film
    /// would print through the same tone curve: the reference reflectance
    /// of each channel mapped through that channel's measured grey curve.
    /// Isolates the colour errors of the negative process (crosstalk,
    /// crossover, balance) from the choice of tone curve.
    pub tone_matched_de: f64,
    /// Mean CIEDE2000 of all 24 patches.
    pub full_de: f64,
    /// Mean chroma (C*ab) of the neutral patches and ramp.
    pub neutral_cast: f64,
    /// Largest chroma of a neutral patch or ramp step.
    pub neutral_cast_max: f64,
}

/// Mean linear ProPhoto colour of each patch from encoded sRGB output pixels.
fn patch_rgb(chart: &Chart, srgb: &[[f32; 3]]) -> Vec<[f64; 3]> {
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
            color::mat_vec(&to_pp, m.map(|v| v / p.pixels.len().max(1) as f64))
        })
        .collect()
}

/// CIELAB D50 of each patch from encoded sRGB output pixels.
pub fn patch_lab(chart: &Chart, srgb: &[[f32; 3]]) -> Vec<[f64; 3]> {
    patch_rgb(chart, srgb)
        .into_iter()
        .map(|m| color::rgb_to_lab(&PROPHOTO, m))
        .collect()
}

/// Piecewise linear interpolation through sorted `(x, y)` points,
/// continuing the end segments outside them.
fn interp(pts: &[(f64, f64)], x: f64) -> f64 {
    let i = pts
        .windows(2)
        .position(|w| x <= w[1].0)
        .unwrap_or(pts.len() - 2);
    let ((x0, y0), (x1, y1)) = (pts[i], pts[i + 1]);
    if (x1 - x0).abs() < 1e-12 {
        return y0;
    }
    y0 + (y1 - y0) * (x - x0) / (x1 - x0)
}

/// Colour error against a perfect film printed with the same curves.
fn tone_matched(chart: &Chart, rgb: &[[f64; 3]]) -> f64 {
    let to_srgb = color::rgb_to_rgb(&PROPHOTO, &SRGB);
    let to_pp = color::rgb_to_rgb(&SRGB, &PROPHOTO);
    // Each channel's grey curve: log reflectance → encoded output.
    let curves: [Vec<(f64, f64)>; 3] = std::array::from_fn(|c| {
        let mut pts: Vec<(f64, f64)> = chart
            .patches
            .iter()
            .zip(rgb)
            .filter(|(p, _)| p.neutral)
            .map(|(p, o)| {
                let y = color::lab_to_xyz(p.lab, D50)[1];
                (y.log10(), color::srgb_encode(o[c].max(0.0) as f32) as f64)
            })
            .collect();
        pts.sort_by(|a, b| a.0.total_cmp(&b.0));
        pts
    });
    let mut sum = 0.0;
    for (p, o) in chart.patches.iter().zip(rgb).take(18) {
        let r = reflectance(p.lab);
        let ideal: [f64; 3] = std::array::from_fn(|c| {
            let e = interp(&curves[c], r[c].log10()).clamp(0.0, 1.0);
            color::srgb_decode(e as f32) as f64
        });
        // Clip to the sRGB output gamut like the rendered patch.
        let ideal = color::mat_vec(
            &to_pp,
            color::mat_vec(&to_srgb, ideal).map(|v| v.clamp(0.0, 1.0)),
        );
        sum += color::delta_e2000(
            color::rgb_to_lab(&PROPHOTO, ideal),
            color::rgb_to_lab(&PROPHOTO, *o),
        );
    }
    sum / 18.0
}

/// CIEDE2000 between two colours ignoring their lightness difference.
pub fn color_de(a: [f64; 3], b: [f64; 3]) -> f64 {
    color::delta_e2000(a, [a[0], b[1], b[2]])
}

pub fn measure(chart: &Chart, srgb: &[[f32; 3]]) -> ChartMetrics {
    let rgb = patch_rgb(chart, srgb);
    let labs: Vec<[f64; 3]> = rgb
        .iter()
        .map(|m| color::rgb_to_lab(&PROPHOTO, *m))
        .collect();
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
        tone_matched_de: tone_matched(chart, &rgb),
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
