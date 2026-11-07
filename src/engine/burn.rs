//! Dodge & burn: print time changed by hand over parts of the frame, as a printer
//! holds back a shadow with a card or burns in a sky through a hole in one.
//!
//! Strokes are brush paths in the source image (see [`BurnStroke`]); this turns them
//! into a map of stops per output pixel, which the renderer adds to the print time
//! before the tone profile (`CurveLut::apply_rgb_shifted`), exactly as the global
//! print exposure moves it. It comes after the analysis and never changes it.

use rayon::prelude::*;

use super::decode::GeometryMap;
use super::settings::BurnStroke;

/// Dab spacing along a stroke, as a share of its radius.
const SPACING: f32 = 0.25;

/// Coverage of one dab at distance `d` from its centre: full inside the hard core,
/// fading smoothly to nothing at the radius.
#[inline]
fn falloff(d: f32, radius: f32, inner: f32) -> f32 {
    if d >= radius {
        0.0
    } else if d <= inner {
        1.0
    } else {
        let t = (radius - d) / (radius - inner);
        t * t * (3.0 - 2.0 * t)
    }
}

/// Dab centres in output pixels, evenly spaced along the stroke's path.
fn dabs(stroke: &BurnStroke, map: &GeometryMap, w: f32, h: f32, radius: f32) -> Vec<(f32, f32)> {
    let pts: Vec<(f32, f32)> = stroke
        .points
        .iter()
        .map(|p| map.to_view(p[0] * w - 0.5, p[1] * h - 0.5))
        .collect();
    let step = (radius * SPACING).max(0.5);
    let mut out = Vec::new();
    let Some(&first) = pts.first() else {
        return out;
    };
    out.push(first);
    // Distance walked since the last dab.
    let mut carry = 0.0;
    for seg in pts.windows(2) {
        let (a, b) = (seg[0], seg[1]);
        let len = ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt();
        let mut t = step - carry;
        while t <= len {
            let f = t / len;
            out.push((a.0 + (b.0 - a.0) * f, a.1 + (b.1 - a.1) * f));
            t += step;
        }
        carry = (carry + len) % step;
    }
    out
}

/// Print time in stops per pixel of the output `map` describes, for `strokes` drawn
/// on a `src_w` × `src_h` source. `None` without strokes.
pub fn exposure_map(
    strokes: &[BurnStroke],
    map: &GeometryMap,
    src_w: usize,
    src_h: usize,
) -> Option<Vec<f32>> {
    if strokes.is_empty() || src_w == 0 || src_h == 0 {
        return None;
    }
    let (ow, oh) = map.out;
    let long = src_w.max(src_h) as f32;
    let mut local = vec![0.0f32; ow * oh];
    for stroke in strokes {
        let radius = (stroke.radius * long).max(0.5);
        let inner = radius * (1.0 - stroke.feather.clamp(0.0, 100.0) / 100.0);
        let flow = (stroke.flow / 100.0).clamp(0.01, 1.0);
        let centres = dabs(stroke, map, src_w as f32, src_h as f32, radius);
        if centres.is_empty() {
            continue;
        }
        // The stroke's bounding box on the output.
        let (mut x0, mut y0, mut x1, mut y1) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
        for &(x, y) in &centres {
            (x0, y0, x1, y1) = (x0.min(x), y0.min(y), x1.max(x), y1.max(y));
        }
        let bx0 = (x0 - radius).floor().max(0.0) as usize;
        let by0 = (y0 - radius).floor().max(0.0) as usize;
        let bx1 = ((x1 + radius).ceil().max(0.0) as usize).min(ow);
        let by1 = ((y1 + radius).ceil().max(0.0) as usize).min(oh);
        if bx0 >= bx1 || by0 >= by1 {
            continue;
        }
        let (ev, erase) = (stroke.ev, stroke.erase);
        local
            .par_chunks_mut(ow)
            .enumerate()
            .skip(by0)
            .take(by1 - by0)
            .for_each(|(y, row)| {
                let fy = y as f32;
                let near: Vec<(f32, f32)> = centres
                    .iter()
                    .copied()
                    .filter(|c| (c.1 - fy).abs() < radius)
                    .collect();
                if near.is_empty() {
                    return;
                }
                let mut cover = vec![0.0f32; bx1 - bx0];
                for (cx, cy) in near {
                    let dy = cy - fy;
                    let half = (radius * radius - dy * dy).max(0.0).sqrt();
                    let xa = ((cx - half).floor().max(bx0 as f32) as usize).max(bx0);
                    let xb = ((cx + half).ceil().max(0.0) as usize + 1).min(bx1);
                    for x in xa..xb {
                        let dx = x as f32 - cx;
                        let f = flow * falloff((dx * dx + dy * dy).sqrt(), radius, inner);
                        let m = &mut cover[x - bx0];
                        *m += (1.0 - *m) * f;
                    }
                }
                for (v, m) in row[bx0..bx1].iter_mut().zip(cover) {
                    if erase {
                        *v *= 1.0 - m;
                    } else {
                        *v += ev * m;
                    }
                }
            });
    }
    Some(local)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dab(at: [f32; 2], ev: f32) -> BurnStroke {
        BurnStroke {
            points: vec![at],
            radius: 0.05,
            feather: 0.0,
            flow: 100.0,
            ev,
            erase: false,
        }
    }

    #[test]
    fn no_strokes_no_map() {
        let map = GeometryMap::new(100, 60, 0, false, 0.0, [0.0; 4]);
        assert!(exposure_map(&[], &map, 100, 60).is_none());
    }

    #[test]
    fn a_dab_lands_where_the_geometry_puts_its_source_point() {
        let (w, h) = (120usize, 80usize);
        for rotation in 0..4u8 {
            for flip in [false, true] {
                for angle in [0.0f32, 4.0] {
                    for insets in [[0.0; 4], [10.0, 5.0, 15.0, 10.0]] {
                        let map = GeometryMap::new(w, h, rotation, flip, angle, insets);
                        let at = [0.45, 0.55];
                        let m = exposure_map(&[dab(at, 1.0)], &map, w, h).unwrap();
                        let (ow, _) = map.out;
                        let (mut sx, mut sy, mut n) = (0.0, 0.0, 0.0);
                        for (i, v) in m.iter().enumerate() {
                            sx += (i % ow) as f32 * v;
                            sy += (i / ow) as f32 * v;
                            n += v;
                        }
                        let (vx, vy) = map.to_view(at[0] * w as f32 - 0.5, at[1] * h as f32 - 0.5);
                        assert!(
                            (sx / n - vx).abs() < 0.5 && (sy / n - vy).abs() < 0.5,
                            "rot {rotation} flip {flip} angle {angle}"
                        );
                        let peak = m.iter().cloned().fold(0.0, f32::max);
                        assert!((peak - 1.0).abs() < 1e-5, "full flow, no feather: one stop");
                    }
                }
            }
        }
    }

    #[test]
    fn strokes_add_and_erase_takes_away() {
        let map = GeometryMap::new(100, 100, 0, false, 0.0, [0.0; 4]);
        let at = |m: &[f32]| m[50 * 100 + 50];
        let dodge = dab([0.5, 0.5], 0.5);
        let burn = dab([0.5, 0.5], -0.25);
        let m = exposure_map(&[dodge.clone(), burn], &map, 100, 100).unwrap();
        assert!((at(&m) - 0.25).abs() < 1e-5);
        assert_eq!(m[5 * 100 + 5], 0.0, "nothing outside the brush");
        let erase = BurnStroke {
            erase: true,
            ..dab([0.5, 0.5], 0.0)
        };
        let m = exposure_map(&[dodge, erase], &map, 100, 100).unwrap();
        assert!(at(&m).abs() < 1e-5);
    }

    #[test]
    fn feather_and_flow_soften() {
        let map = GeometryMap::new(200, 200, 0, false, 0.0, [0.0; 4]);
        let soft = BurnStroke {
            feather: 100.0,
            ..dab([0.5, 0.5], 1.0)
        };
        let m = exposure_map(&[soft], &map, 200, 200).unwrap();
        // Coverage falls from the centre to the rim.
        let row: Vec<f32> = (100..112).map(|x| m[100 * 200 + x]).collect();
        assert!(row.windows(2).all(|p| p[1] <= p[0] + 1e-6), "{row:?}");
        assert!(row[0] > 0.9 && row[9] < 0.2);
        let light = BurnStroke {
            flow: 25.0,
            ..dab([0.5, 0.5], 1.0)
        };
        let m = exposure_map(&[light], &map, 200, 200).unwrap();
        assert!(
            (m[100 * 200 + 100] - 0.25).abs() < 1e-5,
            "one dab at a quarter flow"
        );
        // A line of dabs builds up beyond one dab's flow.
        let line = BurnStroke {
            points: vec![[0.3, 0.5], [0.7, 0.5]],
            flow: 25.0,
            ..dab([0.5, 0.5], 1.0)
        };
        let m = exposure_map(&[line], &map, 200, 200).unwrap();
        assert!(m[100 * 200 + 100] > 0.5);
    }
}
