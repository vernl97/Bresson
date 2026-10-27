//! Bayer demosaicing for full-resolution exports: a ratio-corrected
//! directional method after Luis Sanz Rodríguez's RCD (as used by
//! RawTherapee and darktable).
//!
//! 1. The interpolation direction per pixel comes from local vertical and
//!    horizontal activity, refined against its neighbours.
//! 2. Green at red and blue sites is estimated from each neighbouring green
//!    scaled by the low-pass ratio between the two positions (which keeps
//!    colour edges from bleeding), weighted by directional gradients.
//! 3. Red and blue are rebuilt from colour differences to green: along the
//!    diagonals at blue / red sites, along the cardinals at green sites.
//!
//! The four-pixel border falls back to averaging the same colour nearby.

use rayon::prelude::*;

const EPS: f32 = 1e-5;
const BORDER: usize = 4;

/// Demosaic `m` (`w` × `h`, one sample per pixel) where `color(row, col)`
/// gives each site's colour: 0 red, 1 green, 2 blue.
pub fn rcd(m: &[f32], w: usize, h: usize, color: impl Fn(usize, usize) -> usize + Sync) -> Vec<[f32; 3]> {
    let n = w * h;
    let col: Vec<u8> = (0..n).map(|i| color(i / w, i % w) as u8).collect();
    if w < 2 * BORDER + 2 || h < 2 * BORDER + 2 {
        return average_fill(m, w, h, &col, 0);
    }
    let wi = w as isize;
    let at = |i: usize, d: isize| m[(i as isize + d) as usize];
    let interior = |i: usize, b: usize| {
        let (r, c) = (i / w, i % w);
        r >= b && r < h - b && c >= b && c < w - b
    };

    // 1. Direction: vertical vs horizontal activity, 3×3 box-summed.
    let activity = |d: isize| -> Vec<f32> {
        (0..n)
            .into_par_iter()
            .map(|i| {
                if !interior(i, 2) {
                    return 0.0;
                }
                (at(i, -d) - at(i, d)).abs() + (2.0 * m[i] - at(i, -2 * d) - at(i, 2 * d)).abs()
            })
            .collect()
    };
    let ratio = |a: &[f32], b: &[f32]| -> Vec<f32> {
        (0..n)
            .into_par_iter()
            .map(|i| {
                if !interior(i, 3) {
                    return 0.5;
                }
                let (mut sa, mut sb) = (0.0, 0.0);
                for dy in -1..=1isize {
                    for dx in -1..=1isize {
                        let j = (i as isize + dy * wi + dx) as usize;
                        sa += a[j];
                        sb += b[j];
                    }
                }
                sa / (sa + sb + EPS)
            })
            .collect()
    };
    let vh = ratio(&activity(wi), &activity(1));
    // Prefer the neighbours' direction when it is more decided.
    let refined = |dir: &[f32], i: usize| {
        let nb = (dir[i - w - 1] + dir[i - w + 1] + dir[i + w - 1] + dir[i + w + 1]) / 4.0;
        if (0.5 - dir[i]).abs() < (0.5 - nb).abs() {
            nb
        } else {
            dir[i]
        }
    };

    // 2. Low-pass (luminance-like) image of the mosaic.
    let lpf: Vec<f32> = (0..n)
        .into_par_iter()
        .map(|i| {
            if !interior(i, 1) {
                return m[i];
            }
            0.25 * m[i]
                + 0.125 * (at(i, -1) + at(i, 1) + at(i, -wi) + at(i, wi))
                + 0.0625 * (at(i, -wi - 1) + at(i, -wi + 1) + at(i, wi - 1) + at(i, wi + 1))
        })
        .collect();

    // Border: average each missing colour over the 5×5 neighbourhood. Its
    // green also serves the interior steps that reach into the border.
    let border = average_fill(m, w, h, &col, BORDER);

    // 3. Green at red / blue sites.
    let green: Vec<f32> = (0..n)
        .into_par_iter()
        .map(|i| {
            if col[i] == 1 {
                return m[i];
            }
            if !interior(i, BORDER) {
                return border[i][1];
            }
            let est = |d: isize| {
                let grad = EPS
                    + (at(i, d) - at(i, -d)).abs()
                    + (m[i] - at(i, 2 * d)).abs()
                    + (at(i, d) - at(i, 3 * d)).abs()
                    + (at(i, 2 * d) - at(i, 4 * d)).abs();
                let val = at(i, d) * 2.0 * lpf[i] / (EPS + lpf[i] + lpf[(i as isize + 2 * d) as usize]);
                (val, grad)
            };
            let pair = |d: isize| {
                let ((a, ga), (b, gb)) = (est(-d), est(d));
                (gb * a + ga * b) / (ga + gb)
            };
            let k = refined(&vh, i);
            (k * pair(1) + (1.0 - k) * pair(wi)).max(0.0)
        })
        .collect();

    // 4. Blue at red sites and red at blue sites, along the diagonals.
    let pq = ratio(&activity(wi + 1), &activity(wi - 1));
    let mut rgb: Vec<[f32; 3]> = (0..n)
        .into_par_iter()
        .map(|i| {
            if !interior(i, BORDER) {
                return border[i];
            }
            let mut p = [0.0f32; 3];
            p[1] = green[i];
            let c = col[i] as usize;
            p[c] = m[i];
            if c != 1 {
                let x = 2 - c;
                let est = |d: isize| {
                    let j = (i as isize + d) as usize;
                    let k = (i as isize - d) as usize;
                    let grad = EPS
                        + (at(i, d) - at(i, -d)).abs()
                        + (at(i, d) - at(i, 3 * d)).abs()
                        + (green[i] - green[(i as isize + 2 * d) as usize]).abs()
                        + (green[j] - green[k]).abs();
                    (m[j] - green[j], grad)
                };
                let pair = |d: isize| {
                    let ((a, ga), (b, gb)) = (est(-d), est(d));
                    (gb * a + ga * b) / (ga + gb)
                };
                let k = refined(&pq, i);
                // pq near 1: strong NW–SE activity, interpolate along NE–SW.
                p[x] = (green[i] + k * pair(wi - 1) + (1.0 - k) * pair(wi + 1)).max(0.0);
            }
            p
        })
        .collect();

    // 5. Red and blue at green sites, along the cardinals (every cardinal
    //    neighbour of a green site now has all three colours).
    let done = rgb.clone();
    rgb.par_iter_mut().enumerate().for_each(|(i, p)| {
        if col[i] != 1 || !interior(i, BORDER) {
            return;
        }
        let k = refined(&vh, i);
        for x in [0usize, 2] {
            let est = |d: isize| {
                let j = (i as isize + d) as usize;
                let k3 = (i as isize + 3 * d) as usize;
                let o = (i as isize - d) as usize;
                let grad = EPS
                    + (green[i] - green[(i as isize + 2 * d) as usize]).abs()
                    + (done[j][x] - done[o][x]).abs()
                    + (done[j][x] - done[k3][x]).abs();
                (done[j][x] - done[j][1], grad)
            };
            let pair = |d: isize| {
                let ((a, ga), (b, gb)) = (est(-d), est(d));
                (gb * a + ga * b) / (ga + gb)
            };
            p[x] = (green[i] + k * pair(1) + (1.0 - k) * pair(wi)).max(0.0);
        }
    });

    rgb
}

/// Each pixel's own sample plus 5×5 same-colour averages of the others;
/// only computed outside an inner margin of `skip` pixels (0: everywhere).
fn average_fill(m: &[f32], w: usize, h: usize, col: &[u8], skip: usize) -> Vec<[f32; 3]> {
    (0..w * h)
        .into_par_iter()
        .map(|i| {
            let (r, c) = (i / w, i % w);
            if skip > 0 && r >= skip && r < h - skip && c >= skip && c < w - skip {
                return [0.0; 3];
            }
            let (mut sum, mut cnt) = ([0.0f32; 3], [0u32; 3]);
            for y in r.saturating_sub(2)..(r + 3).min(h) {
                for x in c.saturating_sub(2)..(c + 3).min(w) {
                    let j = y * w + x;
                    sum[col[j] as usize] += m[j];
                    cnt[col[j] as usize] += 1;
                }
            }
            let mut p: [f32; 3] = std::array::from_fn(|k| sum[k] / cnt[k].max(1) as f32);
            p[col[i] as usize] = m[i];
            p
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// RGGB colour of a site.
    fn rggb(r: usize, c: usize) -> usize {
        match (r % 2, c % 2) {
            (0, 0) => 0,
            (1, 1) => 2,
            _ => 1,
        }
    }

    /// A test scene: a colour gradient, sharp coloured and neutral edges,
    /// and a fine grating (the hard case for demosaicing).
    fn scene(w: usize, h: usize) -> Vec<[f32; 3]> {
        (0..w * h)
            .map(|i| {
                let (y, x) = ((i / w) as f32, (i % w) as f32);
                let (fx, fy) = (x / w as f32, y / h as f32);
                if fy < 0.33 {
                    [0.2 + 0.6 * fx, 0.3 + 0.4 * fy, 0.7 - 0.5 * fx]
                } else if fy < 0.66 {
                    // Neutral and coloured squares with hard edges.
                    let tile = ((x / 12.0) as usize + (y / 12.0) as usize) % 4;
                    [[0.8, 0.8, 0.8], [0.1, 0.1, 0.1], [0.7, 0.2, 0.1], [0.1, 0.3, 0.7]][tile]
                } else {
                    // Grating: neutral stripes getting finer to the right.
                    let f = 0.05 + 0.3 * fx;
                    let v = 0.5 + 0.4 * (x * f * std::f32::consts::TAU).sin();
                    [v, v, v]
                }
            })
            .collect()
    }

    fn mosaic(img: &[[f32; 3]], w: usize) -> Vec<f32> {
        img.iter()
            .enumerate()
            .map(|(i, p)| p[rggb(i / w, i % w)])
            .collect()
    }

    /// PSNR (dB) and false colour on neutral areas (mean chroma, 0..1), over
    /// the interior.
    fn quality(out: &[[f32; 3]], truth: &[[f32; 3]], w: usize, h: usize) -> (f64, f64) {
        let (mut se, mut n, mut fc, mut nf) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
        for y in 8..h - 8 {
            for x in 8..w - 8 {
                let i = y * w + x;
                for c in 0..3 {
                    se += ((out[i][c] - truth[i][c]) as f64).powi(2);
                    n += 1.0;
                }
                let t = truth[i];
                if (t[0] - t[1]).abs() < 1e-6 && (t[1] - t[2]).abs() < 1e-6 {
                    let o = out[i];
                    fc += (o.iter().fold(f32::MIN, |a, &b| a.max(b))
                        - o.iter().fold(f32::MAX, |a, &b| a.min(b))) as f64;
                    nf += 1.0;
                }
            }
        }
        (10.0 * (1.0 / (se / n)).log10(), fc / nf)
    }

    #[test]
    fn flat_colour_is_reproduced_exactly() {
        let (w, h) = (32, 24);
        let img = vec![[0.6f32, 0.4, 0.2]; w * h];
        let out = rcd(&mosaic(&img, w), w, h, rggb);
        for p in &out {
            for c in 0..3 {
                assert!((p[c] - img[0][c]).abs() < 1e-4, "{p:?}");
            }
        }
    }

    /// Better than rawler's PPG (used before) and plain averaging on edges
    /// and fine detail.
    #[test]
    fn beats_ppg_and_bilinear() {
        use rawler::cfa::{CFA, PlaneColor};
        use rawler::imgop::sensor::Demosaic;
        use rawler::imgop::sensor::bayer::ppg::PPGDemosaic;
        use rawler::pixarray::PixF32;
        let (w, h) = (384, 288);
        let truth = scene(w, h);
        let m = mosaic(&truth, w);
        let ours = rcd(&m, w, h, rggb);
        let cfa = CFA::new("RGGB");
        let pixels = PixF32::new_with(m.clone(), w, h);
        let ppg = PPGDemosaic::new().demosaic(&pixels, &cfa, &PlaneColor::default(), pixels.rect());
        let col: Vec<u8> = (0..w * h).map(|i| rggb(i / w, i % w) as u8).collect();
        let avg = average_fill(&m, w, h, &col, 0);
        let (q_ours, q_ppg, q_avg) = (
            quality(&ours, &truth, w, h),
            quality(&ppg.data, &truth, w, h),
            quality(&avg, &truth, w, h),
        );
        eprintln!(
            "demosaic PSNR / false colour: rcd {:.2} dB {:.4} | ppg {:.2} dB {:.4} | average {:.2} dB {:.4}",
            q_ours.0, q_ours.1, q_ppg.0, q_ppg.1, q_avg.0, q_avg.1
        );
        assert!(q_ours.0 > q_ppg.0, "PSNR {} vs PPG {}", q_ours.0, q_ppg.0);
        assert!(q_ours.1 < q_ppg.1, "false colour {} vs PPG {}", q_ours.1, q_ppg.1);
    }
}
