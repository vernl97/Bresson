//! Spot healing: dust and scratches copied over from a clean part of the frame.
//!
//! Works on the source pixels, before any geometry, so a spot stays on its speck
//! through rotation, straightening and cropping, and the same spots apply to a
//! proxy and to a full-resolution export. Pixels are blended in log (density), and
//! the source patch is taken whole pixels away so its grain stays sharp.

use std::f32::consts::{PI, TAU};

use rayon::prelude::*;

use super::look;
use super::settings::{HealMode, HealSpot};

/// Where the feather starts, as a fraction of the radius.
const FEATHER: f32 = 0.8;
/// Smallest radius healed, in pixels: a spot on a small proxy still covers something.
const MIN_RADIUS: f32 = 1.0;

fn log_px(p: [f32; 3]) -> [f32; 3] {
    p.map(|v| v.max(1e-5).ln())
}

/// Log of the pixel at (x, y), edges repeated.
fn log_at(d: &[[f32; 3]], w: usize, h: usize, x: isize, y: isize) -> [f32; 3] {
    let (x, y) = (x.clamp(0, w as isize - 1), y.clamp(0, h as isize - 1));
    log_px(d[y as usize * w + x as usize])
}

/// Log of the bilinear sample at (x, y), edges repeated.
fn log_bilinear(d: &[[f32; 3]], w: usize, h: usize, x: f32, y: f32) -> [f32; 3] {
    let (x, y) = (x.clamp(0.0, w as f32 - 1.0), y.clamp(0.0, h as f32 - 1.0));
    let (x0, y0) = (x as usize, y as usize);
    let (x1, y1) = ((x0 + 1).min(w - 1), (y0 + 1).min(h - 1));
    let (fx, fy) = (x - x0 as f32, y - y0 as f32);
    let (a, b, c, e) = (
        d[y0 * w + x0],
        d[y0 * w + x1],
        d[y1 * w + x0],
        d[y1 * w + x1],
    );
    log_px([0, 1, 2].map(|k| {
        (a[k] * (1.0 - fx) + b[k] * fx) * (1.0 - fy) + (c[k] * (1.0 - fx) + e[k] * fx) * fy
    }))
}

/// A copy of `data` with `spots` healed in order, so a spot sees the ones before it.
pub fn apply(data: &[[f32; 3]], w: usize, h: usize, spots: &[HealSpot]) -> Vec<[f32; 3]> {
    let mut out = data.to_vec();
    if w > 0 && h > 0 {
        for spot in spots {
            heal_spot(&mut out, w, h, spot);
        }
    }
    out
}

/// Seamless cloning on a disc: the source patch plus the harmonic function that takes
/// the patch to the frame around its rim. On a disc that function is the Poisson
/// integral of the rim's differences, so no solver is needed. The difference is taken
/// just outside the disc, where the defect isn't.
fn heal_spot(out: &mut [[f32; 3]], w: usize, h: usize, spot: &HealSpot) {
    let r = (spot.radius * w.max(h) as f32).max(MIN_RADIUS);
    let (cx, cy) = (spot.at[0] * w as f32 - 0.5, spot.at[1] * h as f32 - 0.5);
    let ox = ((spot.from[0] - spot.at[0]) * w as f32).round() as isize;
    let oy = ((spot.from[1] - spot.at[1]) * h as f32).round() as isize;
    let opacity = (spot.opacity / 100.0).clamp(0.0, 1.0);
    if (ox == 0 && oy == 0) || opacity <= 0.0 {
        return;
    }
    let matched = spot.mode == HealMode::Heal;
    let n = ((r * PI / 2.0) as usize).clamp(24, 96);
    let rim: Vec<[f32; 2]> = (0..n)
        .map(|k| {
            let a = TAU * k as f32 / n as f32;
            [a.cos(), a.sin()]
        })
        .collect();
    let (fx, fy) = (ox as f32, oy as f32);
    let raw: Vec<[f32; 3]> = rim
        .iter()
        .map(|u| {
            let mut d = [0.0f32; 3];
            for rr in [r + 0.5, r + 1.5] {
                let (x, y) = (cx + u[0] * rr, cy + u[1] * rr);
                let dst = log_bilinear(out, w, h, x, y);
                let src = log_bilinear(out, w, h, x + fx, y + fy);
                for c in 0..3 {
                    d[c] += (dst[c] - src[c]) / 2.0;
                }
            }
            d
        })
        .collect();
    let rim_diff: Vec<[f32; 3]> = (0..n)
        .map(|k| {
            let (a, b, c) = (raw[(k + n - 1) % n], raw[k], raw[(k + 1) % n]);
            [0, 1, 2].map(|i| (a[i] + b[i] + c[i]) / 3.0)
        })
        .collect();

    let x0 = ((cx - r).floor().max(0.0)) as usize;
    let x1 = ((cx + r).ceil() as isize).clamp(0, w as isize - 1) as usize;
    let y0 = ((cy - r).floor().max(0.0)) as usize;
    let y1 = ((cy + r).ceil() as isize).clamp(0, h as isize - 1) as usize;
    if x0 > x1 || y0 > y1 {
        return;
    }
    let frame: &[[f32; 3]] = out;
    let patch: Vec<(usize, [f32; 3])> = (y0..=y1)
        .into_par_iter()
        .flat_map_iter(|y| {
            let (rim, rim_diff) = (&rim, &rim_diff);
            (x0..=x1).filter_map(move |x| {
                let p = [(x as f32 - cx) / r, (y as f32 - cy) / r];
                let rho2 = p[0] * p[0] + p[1] * p[1];
                if rho2 >= 1.0 {
                    return None;
                }
                let t = ((1.0 - rho2.sqrt()) / (1.0 - FEATHER)).clamp(0.0, 1.0);
                let weight = t * t * (3.0 - 2.0 * t) * opacity;
                let mut diff = [0.0f32; 3];
                let mut sum = 1.0f32;
                if matched {
                    sum = 0.0;
                    for (u, d) in rim.iter().zip(rim_diff) {
                        let (ex, ey) = (u[0] - p[0], u[1] - p[1]);
                        let kernel = (1.0 - rho2) / (ex * ex + ey * ey + 1e-6);
                        sum += kernel;
                        for c in 0..3 {
                            diff[c] += kernel * d[c];
                        }
                    }
                }
                let dst = log_px(frame[y * w + x]);
                let src = log_at(frame, w, h, x as isize + ox, y as isize + oy);
                let v = [0, 1, 2].map(|c| {
                    (dst[c] * (1.0 - weight) + (src[c] + diff[c] / sum.max(1e-6)) * weight).exp()
                });
                Some((y * w + x, v))
            })
        })
        .collect();
    for (k, v) in patch {
        out[k] = v;
    }
}

/// The best place to copy a spot from: of the discs around it that lie fully inside the
/// frame and in `allowed` (normalised source coordinates), the one whose surroundings
/// match the spot's in shape and that holds no defect of its own.
pub fn find_source(
    data: &[[f32; 3]],
    w: usize,
    h: usize,
    at: [f32; 2],
    radius: f32,
    allowed: &dyn Fn([f32; 2]) -> bool,
) -> [f32; 2] {
    let r = (radius * w.max(h) as f32).max(1.5);
    let (cx, cy) = (at[0] * w as f32 - 0.5, at[1] * h as f32 - 0.5);
    let to_uv = |x: f32, y: f32| [(x + 0.5) / w as f32, (y + 0.5) / h as f32];

    // Sample points: alternating rings around the disc, and a sunflower inside it.
    const POINTS: usize = 32;
    let around: Vec<(f32, f32)> = (0..POINTS)
        .map(|k| {
            let a = TAU * k as f32 / POINTS as f32;
            let rr = r * if k % 2 == 0 { 1.15 } else { 1.5 };
            (a.cos() * rr, a.sin() * rr)
        })
        .collect();
    let inside: Vec<(f32, f32)> = (0..POINTS)
        .map(|k| {
            let a = k as f32 * 2.399_963;
            let rr = r * 0.9 * ((k as f32 + 0.5) / POINTS as f32).sqrt();
            (a.cos() * rr, a.sin() * rr)
        })
        .collect();
    let sample = |x: f32, y: f32, pts: &[(f32, f32)]| -> Vec<[f32; 3]> {
        pts.iter()
            .map(|(dx, dy)| log_bilinear(data, w, h, x + dx, y + dy))
            .collect()
    };
    let spread = |v: &[[f32; 3]]| -> f32 {
        (0..3)
            .map(|c| {
                let m = v.iter().map(|p| p[c]).sum::<f32>() / v.len() as f32;
                v.iter().map(|p| (p[c] - m).powi(2)).sum::<f32>() / v.len() as f32
            })
            .sum()
    };
    let target = sample(cx, cy, &around);

    let in_frame = |x: f32, y: f32| {
        x - r >= 0.0 && y - r >= 0.0 && x + r <= w as f32 - 1.0 && y + r <= h as f32 - 1.0
    };
    let in_allowed = |x: f32, y: f32| {
        [(0.0, 0.0), (r, 0.0), (-r, 0.0), (0.0, r), (0.0, -r)]
            .iter()
            .all(|(dx, dy)| allowed(to_uv(x + dx, y + dy)))
    };
    const ANGLES: usize = 16;
    let mut best: Option<(f32, f32, f32)> = None;
    let mut fallback: Option<(f32, f32)> = None;
    for (ring, dist) in [2.6f32, 3.2, 4.0, 5.5, 7.5, 10.0].iter().enumerate() {
        for k in 0..ANGLES {
            let a = TAU * (k as f32 + ring as f32 * 0.5) / ANGLES as f32;
            let (x, y) = (cx + a.cos() * dist * r, cy + a.sin() * dist * r);
            if !in_frame(x, y) {
                continue;
            }
            fallback.get_or_insert((x, y));
            if !in_allowed(x, y) {
                continue;
            }
            let ring_px = sample(x, y, &around);
            let diff: Vec<[f32; 3]> = ring_px
                .iter()
                .zip(&target)
                .map(|(a, b)| [a[0] - b[0], a[1] - b[1], a[2] - b[2]])
                .collect();
            let level = (0..3)
                .map(|c| (diff.iter().map(|d| d[c]).sum::<f32>() / POINTS as f32).powi(2))
                .sum::<f32>();
            // More variation inside than around it: a defect, or an edge passing through.
            let defect = (spread(&sample(x, y, &inside)) - spread(&ring_px)).max(0.0);
            let score = spread(&diff) + 0.05 * level + defect + 1e-5 * dist;
            if best.is_none_or(|b| score < b.0) {
                best = Some((score, x, y));
            }
        }
    }
    if let Some((_, x, y)) = best {
        return to_uv(x, y);
    }
    if let Some((x, y)) = fallback {
        return to_uv(x, y);
    }
    // A spot too large to fit a disc beside it: as far towards the frame's middle as it goes.
    let toward = |c: f32, size: usize| {
        let mid = (size as f32 - 1.0) / 2.0;
        if c < mid { c + 2.2 * r } else { c - 2.2 * r }.clamp(0.0, size as f32 - 1.0)
    };
    to_uv(toward(cx, w), toward(cy, h))
}

/// How much each pixel stands out from its surroundings: what Visualize Spots shows.
/// Fine detail minus a wider blur, in the negative's density.
pub fn spot_map(density: &[[f32; 3]], w: usize, h: usize) -> Vec<f32> {
    let fine = look::gaussian_blur_3(density, w, h, 1.0);
    let wide = look::gaussian_blur_3(density, w, h, 2.5);
    fine.par_iter()
        .zip(wide.par_iter())
        .map(|(a, b)| ((a[0] - b[0]) + (a[1] - b[1]) + (a[2] - b[2])).abs() / 3.0)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const W: usize = 400;
    const H: usize = 300;

    /// A gentle gradient with fine per-pixel grain.
    fn scene() -> Vec<[f32; 3]> {
        (0..W * H)
            .map(|k| {
                let (x, y) = ((k % W) as f32, (k / W) as f32);
                let grain = 1.0 + 0.02 * ((x * 12.9898 + y * 78.233).sin() * 43758.547).fract();
                let v = (0.2 + 0.3 * x / W as f32) * grain;
                [v, v * 0.8, v * 0.6]
            })
            .collect()
    }

    fn dust(d: &mut [[f32; 3]], x: usize, y: usize, r: isize) {
        for dy in -r..=r {
            for dx in -r..=r {
                if dx * dx + dy * dy <= r * r {
                    let k = (y as isize + dy) as usize * W + (x as isize + dx) as usize;
                    d[k] = [0.9, 0.9, 0.9];
                }
            }
        }
    }

    fn spot(x: f32, y: f32, sx: f32, sy: f32, r: f32) -> HealSpot {
        HealSpot {
            at: [(x + 0.5) / W as f32, (y + 0.5) / H as f32],
            from: [(sx + 0.5) / W as f32, (sy + 0.5) / H as f32],
            radius: r / W as f32,
            ..Default::default()
        }
    }

    #[test]
    fn a_speck_goes_and_the_rest_of_the_frame_stays() {
        let clean = scene();
        let mut dirty = clean.clone();
        dust(&mut dirty, 100, 100, 4);
        let healed = apply(&dirty, W, H, &[spot(100.0, 100.0, 160.0, 100.0, 10.0)]);
        for y in 0..H {
            for x in 0..W {
                let k = y * W + x;
                let dist = ((x as f32 - 100.0).powi(2) + (y as f32 - 100.0).powi(2)).sqrt();
                if dist >= 10.5 {
                    assert_eq!(healed[k], dirty[k], "changed outside the spot at {x},{y}");
                } else {
                    // The source is brighter (the gradient), which must not show.
                    let ratio = healed[k][0] / clean[k][0];
                    assert!((ratio - 1.0).abs() < 0.05, "at {x},{y}: {ratio}");
                }
            }
        }
    }

    #[test]
    fn the_seam_follows_the_frame_when_the_source_differs() {
        let clean = scene();
        // Far to the right the source is much brighter than the frame around the spot.
        let healed = apply(&clean, W, H, &[spot(60.0, 150.0, 340.0, 150.0, 20.0)]);
        for x in 40..=80usize {
            let k = 150 * W + x;
            let ratio = healed[k][1] / clean[k][1];
            assert!((ratio - 1.0).abs() < 0.04, "at x {x}: {ratio}");
        }
    }

    #[test]
    fn clone_copies_the_source_as_it_is_and_opacity_blends() {
        let clean = scene();
        let centre = 150 * W + 60;
        let source = 150 * W + 340;
        let mut s = spot(60.0, 150.0, 340.0, 150.0, 20.0);
        s.mode = HealMode::Clone;
        let cloned = apply(&clean, W, H, &[s]);
        assert!((cloned[centre][0] / clean[source][0] - 1.0).abs() < 1e-4);

        s.opacity = 50.0;
        let half = apply(&clean, W, H, &[s]);
        // Half way in density between the frame and the source.
        let expect = (clean[centre][0] * clean[source][0]).sqrt();
        assert!((half[centre][0] / expect - 1.0).abs() < 1e-3);

        s.opacity = 0.0;
        assert_eq!(apply(&clean, W, H, &[s]), clean);
    }

    #[test]
    fn the_spot_map_lights_up_a_speck() {
        let mut d = vec![[0.5f32; 3]; W * H];
        d[100 * W + 100] = [1.5; 3];
        let map = spot_map(&d, W, H);
        assert!(map[100 * W + 100] > 0.03);
        assert!(map[20 * W + 20] < 1e-5);
    }

    #[test]
    fn the_grain_of_the_source_comes_through() {
        let clean = scene();
        let flat = vec![[0.3, 0.3, 0.3]; W * H];
        // Heal a featureless frame from a copy of it with grain painted in the source area.
        let mut frame = flat.clone();
        for y in 130..=170 {
            for x in 280..=320 {
                frame[y * W + x] = clean[y * W + x];
            }
        }
        let healed = apply(&frame, W, H, &[spot(100.0, 150.0, 300.0, 150.0, 15.0)]);
        let spread = |d: &[[f32; 3]]| {
            let row: Vec<f32> = (95..=105).map(|x| d[150 * W + x][0]).collect();
            row.iter().cloned().fold(f32::MIN, f32::max)
                - row.iter().cloned().fold(f32::MAX, f32::min)
        };
        assert_eq!(spread(&flat), 0.0);
        assert!(spread(&healed) > 0.002, "grain was smoothed away");
    }

    #[test]
    fn spots_land_on_the_same_place_at_every_resolution() {
        // The same speck in the same normalised place, at half the size.
        let mut big = scene();
        dust(&mut big, 200, 150, 6);
        let s = spot(200.0, 150.0, 260.0, 150.0, 12.0);
        let (small, sw, sh) = crate::engine::decode::downscale(&big, W, H, W / 2);
        let healed = apply(&small, sw, sh, std::slice::from_ref(&s));
        let centre = healed[sh / 2 * sw + sw / 2];
        assert!(centre[0] < 0.6, "the speck is still there: {centre:?}");
    }

    #[test]
    fn a_spot_at_the_frame_edge_is_safe() {
        let d = scene();
        for at in [(0.0, 0.0), (399.0, 299.0), (0.0, 299.0)] {
            let healed = apply(&d, W, H, &[spot(at.0, at.1, 200.0, 150.0, 30.0)]);
            assert!(healed.iter().flatten().all(|v| v.is_finite()));
        }
        let off = apply(&d, W, H, &[spot(-100.0, -100.0, 200.0, 150.0, 5.0)]);
        assert_eq!(off, d);
    }

    #[test]
    fn exports_and_previews_get_the_spots_through_the_negative() {
        use crate::engine::pipeline::{prepare_negative, resolve_base};
        use crate::engine::settings::Settings;
        let src = crate::engine::testchart::negative(0.0).src;
        // Chroma noise reduction would spread the change by its radius.
        let mut s = Settings {
            chroma_denoise: 0.0,
            ..Default::default()
        };
        let base = resolve_base(&src, &s, None);
        let plain = prepare_negative(&src, &s, base);
        s.heal.push(HealSpot {
            at: [0.5, 0.5],
            from: [0.3, 0.3],
            radius: 0.02,
            ..Default::default()
        });
        // Geometry doesn't move the spot: the turned frame changes the same place.
        s.rotation = 1;
        let healed = prepare_negative(&src, &s, base);
        s.heal.clear();
        let turned = prepare_negative(&src, &s, base);
        let changed: Vec<usize> = (0..healed.data.len())
            .filter(|&k| healed.data[k] != turned.data[k])
            .collect();
        assert!(!changed.is_empty(), "the spot did nothing");
        let reach = 0.02 * src.width.max(src.height) as f32 + 2.0;
        let (mx, my) = (
            (healed.width as f32 - 1.0) / 2.0,
            (healed.height as f32 - 1.0) / 2.0,
        );
        for k in changed {
            let (x, y) = ((k % healed.width) as f32, (k / healed.width) as f32);
            assert!(
                (x - mx).hypot(y - my) <= reach,
                "changed far off at {x},{y}"
            );
        }
        assert_eq!(plain.width, healed.height);
    }

    #[test]
    fn the_source_search_picks_matching_clean_ground() {
        // Bright on the left, dark on the right: a spot in the bright part stays there.
        let mut d: Vec<[f32; 3]> = (0..W * H)
            .map(|k| if k % W < 200 { [0.6; 3] } else { [0.05; 3] })
            .collect();
        let at = [150.5 / W as f32, 150.5 / H as f32];
        let r = 10.0 / W as f32;
        let all = |_: [f32; 2]| true;
        let from = find_source(&d, W, H, at, r, &all);
        assert!(from[0] * (W as f32) < 190.0, "crossed the edge: {from:?}");

        // Dust where the search looks first is passed over.
        let first = [from[0] * W as f32 - 0.5, from[1] * H as f32 - 0.5];
        for dy in -4..=4isize {
            for dx in -4..=4isize {
                let k = (first[1] as isize + dy) as usize * W + (first[0] as isize + dx) as usize;
                d[k] = [0.01; 3];
            }
        }
        let next = find_source(&d, W, H, at, r, &all);
        assert_ne!(next, from, "chose the dusty place");

        // Only inside what `allowed` lets through.
        let left = |p: [f32; 2]| p[0] < 0.2;
        let kept = find_source(&d, W, H, [0.1, 0.5], r, &left);
        assert!(kept[0] + r <= 0.2, "left the allowed area: {kept:?}");
    }
}
