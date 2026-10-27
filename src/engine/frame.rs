//! Automatic frame detection: finds the exposed image inside the film rebate
//! and the holder mask, and the angle that squares it up. Lab scanners
//! (Frontier, Noritsu) crop every frame this way. The plugin leaves cropping
//! to Lightroom, so there is no Lua counterpart.

use super::decode::{self, SourceImage};
use super::settings::Settings;

/// Long edge of the detection buffer.
const SIZE: usize = 600;
/// A line belongs to the border when at least this share of it is border,
const LINE_BORDER: f32 = 0.5;
/// or this share is holder or light past the film, which no image area
/// looks like (a tilted holder edge only covers part of a line).
const LINE_OUTSIDE: f32 = 0.25;
/// The image starts after this many image lines in a row, so a thin gap
/// between holder and rebate isn't taken for the image.
const IMAGE_RUN: usize = 6;
/// Unexposed rebate is flat: across a line its density varies far less
/// than thin shadow areas of the image, which can be as clear as the base.
const REBATE_MAX_SD: f32 = 0.03;
/// No side may lose more than this (percent).
const MAX_INSET: f32 = 40.0;
/// Extra inset on each detected side so no rebate edge stays visible (percent).
const SAFETY: f32 = 1.0;
const MAX_ANGLE: f32 = 5.0;

/// Proposed geometry, in the units of `Settings::crop` and `Settings::straighten`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Detected {
    pub crop: [f32; 4],
    pub straighten: f32,
}

/// Detect the frame of `src` after its quarter turns and flip. `base` is the
/// film base in source space (`pipeline::resolve_base`). Returns `None` when
/// no plausible frame is found.
pub fn detect(src: &SourceImage, s: &Settings, base: [f32; 3]) -> Option<Detected> {
    let (d, w, h) = decode::downscale(&src.data, src.width, src.height, SIZE);
    let (d, w, h) = decode::rotate_flip(&d, w, h, s.rotation, s.flip_h);
    if w < 32 || h < 32 {
        return None;
    }
    let px: Vec<Px> = d.iter().map(|p| classify(*p, base)).collect();
    let mask: Vec<bool> = px.iter().map(|p| p.class != Class::Image).collect();
    let angle = angle(&mask, w, h);
    let (px, w, h) = if angle != 0.0 {
        let (d, w, h) = decode::straighten(&d, w, h, angle);
        (d.iter().map(|p| classify(*p, base)).collect(), w, h)
    } else {
        (px, w, h)
    };
    let mut crop = insets(&px, w, h)?;
    if let Some(k) = s.crop_aspect.ratio(w as f32 / h as f32) {
        crop = fit_aspect(crop, w as f32 / h as f32, k);
    }
    Some(Detected {
        crop,
        straighten: angle,
    })
}

/// Film rebate (close to the base colour), light through sprocket holes or
/// past the film (brighter than the base) or the holder. The holder is
/// opaque in at least one channel: below 5 % of the base, where even the
/// densest image areas of real scans stay above about 9 %.
fn classify(p: [f32; 3], base: [f32; 3]) -> Px {
    let r: [f32; 3] = std::array::from_fn(|c| p[c] / base[c].max(1e-4));
    let m = (r[0] + r[1] + r[2]) / 3.0;
    let class = if m >= 1.2 || r.iter().fold(f32::MAX, |a, &b| a.min(b)) < 0.05 {
        Class::Outside
    } else if m > 0.85 && r.iter().all(|v| (v - m).abs() < 0.12 * m) {
        Class::Rebate
    } else {
        Class::Image
    };
    Px { class, m }
}

#[derive(Clone, Copy, PartialEq)]
enum Class {
    Image,
    /// Close to the film base colour.
    Rebate,
    /// Holder, or light past the film.
    Outside,
}

#[derive(Clone, Copy)]
struct Px {
    class: Class,
    /// Mean transmission relative to the base.
    m: f32,
}

#[cfg(test)]
fn is_border(p: [f32; 3], base: [f32; 3]) -> bool {
    classify(p, base).class != Class::Image
}

/// Whether row `y` (`horizontal`) or column `y` is border, judged across
/// the middle half of the other axis: mostly border pixels, and when those
/// are mostly rebate-coloured, flat like real rebate.
fn line_border(px: &[Px], w: usize, h: usize, horizontal: bool, y: usize) -> bool {
    let n = if horizontal { w } else { h };
    let at = |x: usize| {
        if horizontal {
            px[y * w + x]
        } else {
            px[x * w + y]
        }
    };
    let (a, b) = (n / 4, n * 3 / 4);
    let (mut border, mut rebate, mut sum, mut sum2) = (0usize, 0usize, 0.0f32, 0.0f32);
    let len = (b - a).max(1) as f32;
    for x in a..b {
        let p = at(x);
        match p.class {
            Class::Image => {}
            Class::Outside => border += 1,
            Class::Rebate => {
                border += 1;
                rebate += 1;
                sum += p.m;
                sum2 += p.m * p.m;
            }
        }
    }
    if (border - rebate) as f32 >= LINE_OUTSIDE * len {
        return true;
    }
    if (border as f32) < LINE_BORDER * len {
        return false;
    }
    if rebate * 2 < border {
        return true;
    }
    let mean = sum / rebate as f32;
    (sum2 / rebate as f32 - mean * mean).max(0.0).sqrt() < REBATE_MAX_SD
}

/// Insets in percent [left, top, right, bottom].
fn insets(px: &[Px], w: usize, h: usize) -> Option<[f32; 4]> {
    // (line count, horizontal lines?, scan from the far side?)
    let sides = [
        (w, false, false),
        (h, true, false),
        (w, false, true),
        (h, true, true),
    ];
    let mut out = [0.0f32; 4];
    for (k, &(n, horizontal, rev)) in sides.iter().enumerate() {
        let line = |i: usize| {
            let i = if rev { n - 1 - i } else { i };
            line_border(px, w, h, horizontal, i)
        };
        // First line where a run of image lines starts.
        let limit = (n as f32 * MAX_INSET / 100.0) as usize;
        let edge = (0..=limit).find(|&i| (i..i + IMAGE_RUN).all(|j| j < n && !line(j)))?;
        out[k] = if edge == 0 {
            0.0
        } else {
            edge as f32 / n as f32 * 100.0 + SAFETY
        };
    }
    let keep = (100.0 - out[0] - out[2]) * (100.0 - out[1] - out[3]) / 1e4;
    (keep >= 0.3).then_some(out)
}

/// Shrink `insets` (percent, [l, t, r, b]) around their centre to the crop
/// ratio `ratio` (width : height in pixels) of a frame of aspect `frame`.
pub fn fit_aspect(insets: [f32; 4], frame: f32, ratio: f32) -> [f32; 4] {
    let [l, t, r, b] = insets;
    let (w, h) = (100.0 - l - r, 100.0 - t - b);
    // Width per height in percent units.
    let k = ratio / frame;
    let (nw, nh) = if w / h > k { (h * k, h) } else { (w, w / k) };
    let (cx, cy) = (l + w / 2.0, t + h / 2.0);
    [
        cx - nw / 2.0,
        cy - nh / 2.0,
        100.0 - cx - nw / 2.0,
        100.0 - cy - nh / 2.0,
    ]
}

/// Straighten angle in degrees, from line fits through the frame's edges.
fn angle(mask: &[bool], w: usize, h: usize) -> f32 {
    let mut angles = Vec::new();
    // Left/right edges: x as a function of y. Top/bottom: y as a function of x.
    for (horizontal_edge, rev) in [(false, false), (false, true), (true, false), (true, true)] {
        let (n, len) = if horizontal_edge { (w, h) } else { (h, w) };
        let at = |i: usize, j: usize| {
            let j = if rev { len - 1 - j } else { j };
            if horizontal_edge {
                mask[j * w + i]
            } else {
                mask[i * w + j]
            }
        };
        let mut pts = Vec::new();
        for i in n * 3 / 10..n * 7 / 10 {
            let lim = len * 2 / 5;
            if let Some(e) = (1..lim).find(|&j| (j..j + 3).all(|k| !at(i, k))) {
                if at(i, 0) {
                    let e = if rev { (len - e) as f32 } else { e as f32 };
                    pts.push((i as f32, e));
                }
            }
        }
        if pts.len() < (n * 4 / 10) / 3 {
            continue;
        }
        let Some((slope, rms)) = fit(&pts) else {
            continue;
        };
        if rms > 1.5 {
            continue;
        }
        let a = slope.atan().to_degrees();
        // `decode::straighten` turns a vertical edge with dx/dy = tan θ upright,
        // and a horizontal one with dy/dx = −tan θ.
        angles.push(if horizontal_edge { -a } else { a });
    }
    if angles.is_empty() {
        return 0.0;
    }
    angles.sort_by(|a, b| a.total_cmp(b));
    let a = angles[angles.len() / 2];
    if a.abs() < 0.1 || a.abs() > MAX_ANGLE {
        0.0
    } else {
        (a * 100.0).round() / 100.0
    }
}

/// Least-squares line through `pts`: (slope, rms residual).
fn fit(pts: &[(f32, f32)]) -> Option<(f32, f32)> {
    let n = pts.len() as f32;
    let (mx, my) = pts
        .iter()
        .fold((0.0, 0.0), |(a, b), p| (a + p.0 / n, b + p.1 / n));
    let sxx: f32 = pts.iter().map(|p| (p.0 - mx).powi(2)).sum();
    if sxx <= 0.0 {
        return None;
    }
    let sxy: f32 = pts.iter().map(|p| (p.0 - mx) * (p.1 - my)).sum();
    let b = sxy / sxx;
    let rms = (pts
        .iter()
        .map(|p| (p.1 - (my + b * (p.0 - mx))).powi(2))
        .sum::<f32>()
        / n)
        .sqrt();
    Some((b, rms))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::settings::CropAspect;

    const BASE: [f32; 3] = [0.6, 0.35, 0.2];

    /// A camera scan of one frame: black holder, `rebate` px of film base,
    /// then a textured image, rotated by `deg` (clockwise) about the centre.
    fn scan(w: usize, h: usize, holder: usize, rebate: usize, deg: f32) -> SourceImage {
        let (c, s) = (deg.to_radians().cos(), deg.to_radians().sin());
        let (cx, cy) = (w as f32 / 2.0, h as f32 / 2.0);
        let data = (0..w * h)
            .map(|i| {
                let (x, y) = ((i % w) as f32 - cx, (i / w) as f32 - cy);
                // Undo the rotation to find the film coordinate.
                let (fx, fy) = (x * c + y * s + cx, -x * s + y * c + cy);
                let (hx, hy) = (holder as f32, holder as f32);
                let inside = |m: f32| fx >= m && fy >= m && fx < w as f32 - m && fy < h as f32 - m;
                if !inside(hx.min(hy)) {
                    [0.004; 3]
                } else if !inside((holder + rebate) as f32) {
                    BASE
                } else {
                    let t = 0.08 + 0.4 * (((fx / 37.0).sin() * (fy / 23.0).cos()) * 0.5 + 0.5);
                    BASE.map(|b| b * t)
                }
            })
            .collect();
        SourceImage {
            width: w,
            height: h,
            data,
            to_melissa: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            as_shot_wb: [1.0; 3],
            is_raw: false,
            info: Default::default(),
        }
    }

    /// Share of border pixels left after applying `d`, and the share of the
    /// frame kept.
    fn leftover(src: &SourceImage, d: Detected) -> (f32, f32) {
        let (px, w, h) = decode::geometry(
            &src.data,
            src.width,
            src.height,
            0,
            false,
            d.straighten,
            d.crop,
        );
        let b = px.iter().filter(|p| is_border(**p, BASE)).count();
        (
            b as f32 / px.len() as f32,
            (w * h) as f32 / (src.width * src.height) as f32,
        )
    }

    fn free() -> Settings {
        Settings {
            crop_aspect: CropAspect::Free,
            ..Settings::default()
        }
    }

    #[test]
    fn auto_crop_keeps_the_original_aspect() {
        let src = scan(300, 200, 12, 10, 0.0);
        let s = Settings::default();
        let d = detect(&src, &s, BASE).expect("frame");
        let (w, h) = (100.0 - d.crop[0] - d.crop[2], 100.0 - d.crop[1] - d.crop[3]);
        let frame = src.width as f32 / src.height as f32;
        assert!((w / h * frame - frame).abs() < 0.01, "{d:?}");
    }

    #[test]
    fn finds_frame_inside_holder_and_rebate() {
        let src = scan(1200, 800, 60, 40, 0.0);
        let d = detect(&src, &free(), BASE).expect("frame");
        // Border is 100 px: 8.33 % of the width, 12.5 % of the height.
        let want = [8.33 + SAFETY, 12.5 + SAFETY, 8.33 + SAFETY, 12.5 + SAFETY];
        for k in 0..4 {
            assert!((d.crop[k] - want[k]).abs() < 1.0, "{:?}", d.crop);
        }
        assert_eq!(d.straighten, 0.0);
        assert!(leftover(&src, d).0 < 0.002);
    }

    #[test]
    fn straightens_a_tilted_frame() {
        for deg in [2.0f32, -1.5] {
            let src = scan(1200, 800, 90, 40, deg);
            let d = detect(&src, &free(), BASE).expect("frame");
            assert!(
                (d.straighten + deg).abs() < 0.3,
                "tilt {deg}: straighten {}",
                d.straighten
            );
            let (left, kept) = leftover(&src, d);
            assert!(left < 0.002, "tilt {deg}: {left} border left, {d:?}");
            assert!(kept > 0.5, "tilt {deg}: kept {kept}, {d:?}");
        }
    }

    #[test]
    fn borderless_scan_is_left_alone() {
        let src = scan(1200, 800, 0, 0, 0.0);
        let d = detect(&src, &free(), BASE).expect("frame");
        assert_eq!(d.crop, [0.0; 4]);
        assert_eq!(d.straighten, 0.0);
    }

    #[test]
    fn thin_shadows_at_the_edge_are_kept() {
        // A dark (on the print) scene area running off the right edge: on
        // the negative it is nearly as clear as the base, but textured
        // (like raw0074: mean 0.83–0.98 of the base, sd 0.05–0.08).
        let mut src = scan(1200, 800, 0, 0, 0.0);
        for (i, p) in src.data.iter_mut().enumerate() {
            let (x, y) = (i % 1200, i / 1200);
            if x > 900 {
                let (xf, yf) = (x as f32, y as f32);
                let t = 0.91 + 0.14 * ((xf / 7.0 + yf / 5.0).sin() * (yf / 11.0 - xf / 13.0).cos());
                *p = BASE.map(|b| b * t);
            }
        }
        let d = detect(&src, &free(), BASE).expect("frame");
        assert!(d.crop[2] < 2.0, "{d:?}");
    }

    #[test]
    fn blank_frame_is_rejected() {
        let mut src = scan(600, 400, 0, 0, 0.0);
        src.data.fill(BASE);
        assert_eq!(detect(&src, &free(), BASE), None);
    }
}
