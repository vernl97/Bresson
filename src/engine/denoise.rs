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

use rayon::prelude::*;

use super::pipeline::{Negative, DENSITY_WEIGHTS};

/// Box radius in full-resolution pixels at 0 and at 100.
const RADIUS_MIN: f32 = 1.0;
const RADIUS_MAX: f32 = 8.0;
/// Luma step (D) the filter counts as an edge, at 0 and at 100: larger
/// amounts smooth colour across fainter luma detail.
const EDGE_MIN: f32 = 0.03;
const EDGE_MAX: f32 = 0.06;
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
    let r = ((RADIUS_MIN + (RADIUS_MAX - RADIUS_MIN) * t) * scale).round() as usize;
    if r == 0 {
        return;
    }
    let r = r.min(w.max(h) / 2).max(1);
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

/// Mean over a (2r+1)² box, edges replicated, of every channel.
fn box_blur<const N: usize>(data: &[[f32; N]], w: usize, h: usize, r: usize) -> Vec<[f32; N]> {
    let n = (2 * r + 1) as f32;
    let at = |i: isize, len: usize| i.clamp(0, len as isize - 1) as usize;
    // Rows: running sum.
    let mut tmp = vec![[0.0f32; N]; w * h];
    tmp.par_chunks_mut(w).enumerate().for_each(|(y, out)| {
        let row = &data[y * w..(y + 1) * w];
        let mut acc = [0.0f32; N];
        for k in -(r as isize)..=r as isize {
            let v = row[at(k, w)];
            for c in 0..N {
                acc[c] += v[c];
            }
        }
        for x in 0..w {
            for c in 0..N {
                out[x][c] = acc[c] / n;
            }
            let (add, sub) = (row[at(x as isize + r as isize + 1, w)], row[at(x as isize - r as isize, w)]);
            for c in 0..N {
                acc[c] += add[c] - sub[c];
            }
        }
    });
    // Columns: sum of the rows in the window, row by row (contiguous).
    let mut out = vec![[0.0f32; N]; w * h];
    out.par_chunks_mut(w).enumerate().for_each(|(y, out)| {
        for k in -(r as isize)..=r as isize {
            let yy = at(y as isize + k, h);
            for (o, v) in out.iter_mut().zip(&tmp[yy * w..(yy + 1) * w]) {
                for c in 0..N {
                    o[c] += v[c];
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
                let base = if right { [0.9, 0.6, 0.6] } else { [0.5, 0.5, 0.5] };
                let grain = 0.02 * noise(i, 1);
                let (cr, cb) = (0.04 * noise(i, 2), 0.04 * noise(i, 3));
                [base[0] + grain + cr, base[1] + grain, base[2] + grain + cb]
            })
            .collect();
        Negative { width: w, height: h, data }
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
        assert!(mean(w / 2 - 3).abs() < 0.05, "grey side {}", mean(w / 2 - 3));
        assert!((mean(w / 2 + 2) - 0.3).abs() < 0.05, "red side {}", mean(w / 2 + 2));
    }

    #[test]
    fn nothing_at_zero_or_tiny_scale() {
        let before = patch();
        let mut a = before.clone();
        chroma(&mut a, 0.0, 1.0);
        let mut b = before.clone();
        chroma(&mut b, 100.0, 0.05);
        assert!(a.data == before.data && b.data == before.data);
    }
}
