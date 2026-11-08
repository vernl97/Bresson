//! Chroma noise reduction on the prepared negative.
//!
//! Film grain and sensor noise put blotches of colour into flat areas, worst
//! in thin shadows. The density is split into luma (the weighted sum the
//! monochrome model uses) and two colour differences (R − G, B − G). Only the
//! colour differences are smoothed, with a guided filter (He, Sun & Tang,
//! 2010) that takes luma as its guide: in a flat area the colour becomes its
//! local mean, and across a luma edge it follows the edge, so colour does not
//! bleed from one object into the next. Luma is rebuilt exactly as it was, so
//! grain and detail keep their sharpness.
//!
//! The radius is set in full-resolution pixels and scaled to the buffer, so
//! a downscaled preview smooths the same share of the picture as the export.
//! It stays fractional (the box's outermost taps weighted by the fraction),
//! so a small preview follows the slider smoothly instead of in whole-pixel
//! steps.

use rayon::prelude::*;

use super::pipeline::{DENSITY_WEIGHTS, Negative};

/// Box radius in full-resolution pixels at 0 and at 100.
const RADIUS_MIN: f32 = 1.0;
const RADIUS_MAX: f32 = 8.0;
/// Luma step (D) the filter counts as an edge, at 0 and at 100: larger
/// amounts smooth colour across fainter luma detail.
const EDGE_MIN: f32 = 0.03;
const EDGE_MAX: f32 = 0.06;
/// Buffer radius below which the filter is skipped: its effect is
/// negligible there (it vanishes as the radius goes to 0).
const RADIUS_SKIP: f32 = 0.05;
/// Amount up to which the filtered colour fades in (at the smallest radii);
/// above it the radius grows.
const FADE_IN: f32 = 25.0;

/// Smooth the colour noise of `neg` by `amount` (0..=100). `scale` is the
/// buffer's size relative to full resolution (1 = full size).
pub fn chroma(neg: &mut Negative, amount: f32, scale: f32) {
    let t = (amount / 100.0).clamp(0.0, 1.0);
    let (w, h) = (neg.width, neg.height);
    if t <= 0.0 || w == 0 || h == 0 {
        return;
    }
    let r = ((RADIUS_MIN + (RADIUS_MAX - RADIUS_MIN) * t) * scale).min((w.max(h) / 2) as f32);
    if !(r >= RADIUS_SKIP) {
        return;
    }
    let edge = EDGE_MIN + (EDGE_MAX - EDGE_MIN) * t;
    let eps = edge * edge;
    let mix = (amount / FADE_IN).min(1.0);
    let yw = DENSITY_WEIGHTS;
    let luma = |p: &[f32; 3]| p[0] * yw[0] + p[1] * yw[1] + p[2] * yw[2];

    // Guide I = luma, targets p1 = R − G, p2 = B − G.
    let stats: Vec<[f32; 6]> = neg
        .data
        .par_iter()
        .map(|p| {
            let i = luma(p);
            let (c1, c2) = (p[0] - p[1], p[2] - p[1]);
            [i, i * i, c1, c2, i * c1, i * c2]
        })
        .collect();
    let m = box_blur(&stats, w, h, r);
    let coef: Vec<[f32; 4]> = m
        .par_iter()
        .map(|m| {
            let var = (m[1] - m[0] * m[0]).max(0.0) + eps;
            let a1 = (m[4] - m[0] * m[2]) / var;
            let a2 = (m[5] - m[0] * m[3]) / var;
            [a1, m[2] - a1 * m[0], a2, m[3] - a2 * m[0]]
        })
        .collect();
    let mc = box_blur(&coef, w, h, r);
    neg.data.par_iter_mut().zip(mc).for_each(|(p, c)| {
        let i = luma(p);
        let (c1, c2) = (p[0] - p[1], p[2] - p[1]);
        let c1 = c1 + (c[0] * i + c[1] - c1) * mix;
        let c2 = c2 + (c[2] * i + c[3] - c2) * mix;
        // Back to RGB with luma unchanged: i = yw·(g + c1, g, g + c2).
        let g = i - yw[0] * c1 - yw[2] * c2;
        *p = [g + c1, g, g + c2];
    });
}

/// Mean over a box of radius `r` (fractional), edges replicated, of every
/// channel. Taps within `⌊r⌋` weigh 1 and the two at `⌊r⌋ + 1` weigh the
/// fraction, so the box grows continuously from a single pixel at 0.
fn box_blur<const N: usize>(data: &[[f32; N]], w: usize, h: usize, r: f32) -> Vec<[f32; N]> {
    let (r, f) = (r.floor() as usize, r.fract());
    let n = (2 * r + 1) as f32 + 2.0 * f;
    let at = |i: isize, len: usize| i.clamp(0, len as isize - 1) as usize;
    let (ri, ro) = (r as isize, r as isize + 1);
    // Rows: running sum.
    let mut tmp = vec![[0.0f32; N]; w * h];
    tmp.par_chunks_mut(w).enumerate().for_each(|(y, out)| {
        let row = &data[y * w..(y + 1) * w];
        let mut acc = [0.0f32; N];
        for k in -ri..=ri {
            let v = row[at(k, w)];
            for c in 0..N {
                acc[c] += v[c];
            }
        }
        for x in 0..w {
            let xi = x as isize;
            let (lo, hi) = (row[at(xi - ro, w)], row[at(xi + ro, w)]);
            for c in 0..N {
                out[x][c] = (acc[c] + f * (lo[c] + hi[c])) / n;
            }
            let (add, sub) = (
                row[at(xi + ro, w)],
                row[at(xi - ri, w)],
            );
            for c in 0..N {
                acc[c] += add[c] - sub[c];
            }
        }
    });
    // Columns: sum of the rows in the window, row by row (contiguous).
    let mut out = vec![[0.0f32; N]; w * h];
    out.par_chunks_mut(w).enumerate().for_each(|(y, out)| {
        for k in -ro..=ro {
            let wk = if k.abs() == ro { f } else { 1.0 };
            if wk == 0.0 {
                continue;
            }
            let yy = at(y as isize + k, h);
            for (o, v) in out.iter_mut().zip(&tmp[yy * w..(yy + 1) * w]) {
                for c in 0..N {
                    o[c] += wk * v[c];
                }
            }
        }
        for o in out.iter_mut() {
            for c in 0..N {
                o[c] /= n;
            }
        }
    });
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Deterministic noise in −1..1.
    fn noise(i: usize, salt: u32) -> f32 {
        let mut x = (i as u32).wrapping_mul(0x9E37_79B9) ^ salt.wrapping_mul(0x85EB_CA6B);
        x ^= x >> 15;
        x = x.wrapping_mul(0x2C1B_3C6D);
        x ^= x >> 12;
        x = x.wrapping_mul(0x297A_2D39);
        x ^= x >> 15;
        x as f32 / u32::MAX as f32 * 2.0 - 1.0
    }

    fn luma(p: [f32; 3]) -> f32 {
        let yw = DENSITY_WEIGHTS;
        p[0] * yw[0] + p[1] * yw[1] + p[2] * yw[2]
    }

    /// A 128² patch: left half grey, right half reddish and denser, with
    /// luma grain and independent colour noise.
    fn patch() -> Negative {
        let (w, h) = (128, 128);
        let data = (0..w * h)
            .map(|i| {
                let right = i % w >= w / 2;
                let base = if right {
                    [0.9, 0.6, 0.6]
                } else {
                    [0.5, 0.5, 0.5]
                };
                let grain = 0.02 * noise(i, 1);
                let (cr, cb) = (0.04 * noise(i, 2), 0.04 * noise(i, 3));
                [base[0] + grain + cr, base[1] + grain, base[2] + grain + cb]
            })
            .collect();
        Negative::new(w, h, data)
    }

    fn chroma_var(neg: &Negative, x0: usize, x1: usize) -> f32 {
        let vals: Vec<f32> = (0..neg.height)
            .flat_map(|y| (x0..x1).map(move |x| (y, x)))
            .map(|(y, x)| {
                let p = neg.data[y * neg.width + x];
                p[0] - p[1]
            })
            .collect();
        let mean = vals.iter().sum::<f32>() / vals.len() as f32;
        vals.iter().map(|v| (v - mean).powi(2)).sum::<f32>() / vals.len() as f32
    }

    #[test]
    fn smooths_colour_and_keeps_luma() {
        let before = patch();
        let mut after = before.clone();
        chroma(&mut after, 25.0, 1.0);
        for (a, b) in before.data.iter().zip(&after.data) {
            assert!((luma(*a) - luma(*b)).abs() < 1e-5, "luma changed");
        }
        let (v0, v1) = (chroma_var(&before, 8, 40), chroma_var(&after, 8, 40));
        assert!(v1 < v0 / 4.0, "colour noise {v0} -> {v1}");
        let mut strong = before.clone();
        chroma(&mut strong, 100.0, 1.0);
        assert!(chroma_var(&strong, 8, 40) < v1, "more amount smooths more");
    }

    #[test]
    fn colour_does_not_bleed_across_an_edge() {
        let mut neg = patch();
        chroma(&mut neg, 100.0, 1.0);
        let w = neg.width;
        // Mean R − G a few pixels each side of the edge stays near its own side's.
        let mean = |x: usize| {
            (0..neg.height)
                .map(|y| {
                    let p = neg.data[y * w + x];
                    p[0] - p[1]
                })
                .sum::<f32>()
                / neg.height as f32
        };
        assert!(
            mean(w / 2 - 3).abs() < 0.05,
            "grey side {}",
            mean(w / 2 - 3)
        );
        assert!(
            (mean(w / 2 + 2) - 0.3).abs() < 0.05,
            "red side {}",
            mean(w / 2 + 2)
        );
    }

    #[test]
    fn nothing_at_zero_or_tiny_scale() {
        let before = patch();
        let mut a = before.clone();
        chroma(&mut a, 0.0, 1.0);
        let mut b = before.clone();
        chroma(&mut b, 100.0, 0.005);
        assert!(a.data == before.data && b.data == before.data);
    }

    /// The box blur at a whole radius is the plain (2r+1)² mean, and between
    /// whole radii it moves smoothly from one to the next.
    #[test]
    fn fractional_box_is_continuous() {
        let (w, h) = (23, 17);
        let d: Vec<[f32; 1]> = (0..w * h).map(|i| [noise(i, 7)]).collect();
        let naive = |r: usize| -> Vec<f32> {
            let n = ((2 * r + 1) * (2 * r + 1)) as f32;
            let ri = r as isize;
            (0..w * h)
                .map(|i| {
                    let (x, y) = ((i % w) as isize, (i / w) as isize);
                    let mut s = 0.0;
                    for dy in -ri..=ri {
                        for dx in -ri..=ri {
                            let xx = (x + dx).clamp(0, w as isize - 1) as usize;
                            let yy = (y + dy).clamp(0, h as isize - 1) as usize;
                            s += d[yy * w + xx][0];
                        }
                    }
                    s / n
                })
                .collect()
        };
        let diff = |a: &[[f32; 1]], b: &[f32]| {
            a.iter().zip(b).map(|(a, b)| (a[0] - b).abs()).fold(0.0f32, f32::max)
        };
        for r in 0..4 {
            assert!(diff(&box_blur(&d, w, h, r as f32), &naive(r)) < 1e-5, "r = {r}");
            let just_below = box_blur(&d, w, h, r as f32 + 0.999);
            assert!(diff(&just_below, &naive(r + 1)) < 0.01, "r = {r}.999");
        }
    }

    /// Small buffers (the editor preview) follow the slider: more amount
    /// always smooths more, with no flat runs between whole-pixel radii.
    #[test]
    fn preview_follows_the_amount() {
        let before = patch();
        let mut last = chroma_var(&before, 8, 40);
        for amount in [3.0, 6.0, 10.0, 20.0, 25.0, 30.0, 40.0, 60.0, 80.0, 100.0] {
            let mut n = before.clone();
            chroma(&mut n, amount, 0.34);
            let v = chroma_var(&n, 8, 40);
            assert!(v < last * 0.99, "amount {amount}: {v} after {last}");
            last = v;
        }
    }
}
