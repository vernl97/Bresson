//! Skin-weighted print density, after the Frontier's Auto Setup (Nakamura,
//! "Digital Image Processing for the Frontier350", IS&T PICS 1999): the
//! density a print is judged by is usually a face's, and a lab printer that
//! meters the face instead of the frame's average gets far fewer frames
//! rejected for density. Faces are found the same broad way: skin-coloured
//! areas of the grey-balanced image, opened to separate them from
//! neighbours, then kept by shape: compact blobs that do not touch the
//! frame edge (a lit wall, sand or wood fills the frame and touches it).

use serde::{Deserialize, Serialize};

use super::curve::Levels;
use super::pipeline::Negative;

/// The skin found in a frame.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SkinTone {
    /// Normalised green of the skin (0 prints black … 1 white).
    pub tone: f64,
    /// 0..1: how much compact, face-like skin the tone rests on.
    pub weight: f64,
}

/// Printed tone (normalised, after the auto midtone) skin is placed on:
/// the chart's light and dark skin patches print at 0.67 and 0.31 when the
/// chart is exposed correctly; most faces lie in between, nearer the light.
const SKIN_TARGET: f64 = 0.65;
/// Largest share skin takes of the auto midtone; the frame's median keeps
/// the rest (complexions differ, so skin alone is no exposure meter).
const SKIN_SHARE: f64 = 0.6;

/// Whether normalised channels `x` (grey balanced) are skin coloured:
/// red over green over blue with red holding about half of the red-blue
/// difference (orange is about a quarter, yellow less), not too pale or
/// saturated, not in the deep shadows. Skin past the white end counts: a
/// flashed face at night is far denser than the dark scene it is in.
fn is_skin(x: [f64; 3]) -> bool {
    let rb = (x[0] - x[2]) / x[1].max(1.0);
    let share = (x[0] - x[1]) / (x[0] - x[2]).max(1e-6);
    (0.05..0.3).contains(&rb) && (0.33..0.65).contains(&share) && (0.15..2.0).contains(&x[1])
}

/// Erode (`grow == false`) or dilate a mask with a 3×3 square.
fn morph(m: &[bool], w: usize, h: usize, grow: bool) -> Vec<bool> {
    let mut out = vec![false; m.len()];
    for y in 0..h {
        for x in 0..w {
            let mut any = false;
            let mut all = true;
            for dy in -1i32..=1 {
                for dx in -1i32..=1 {
                    let (xx, yy) = (x as i32 + dx, y as i32 + dy);
                    let v = xx >= 0
                        && yy >= 0
                        && (xx as usize) < w
                        && (yy as usize) < h
                        && m[yy as usize * w + xx as usize];
                    any |= v;
                    all &= v;
                }
            }
            out[y * w + x] = if grow { any } else { all };
        }
    }
    out
}

/// The frame's skin, from the analysis buffer under its final levels.
/// `None` without a compact skin area away from the frame edge.
pub fn find(buf: &Negative, l: &Levels) -> Option<SkinTone> {
    let (w, h) = (buf.width, buf.height);
    if w < 16 || h < 16 {
        return None;
    }
    let x: Vec<[f64; 3]> = buf
        .data
        .iter()
        .map(|p| std::array::from_fn(|c| l[c].normalise(p[c] as f64)))
        .collect();
    let mask: Vec<bool> = x.iter().map(|&v| is_skin(v)).collect();
    let mask = morph(&morph(&mask, w, h, false), w, h, true);
    let mut seen = vec![false; mask.len()];
    let (mut tones, mut weight) = (Vec::new(), 0.0);
    for start in 0..mask.len() {
        if !mask[start] || seen[start] {
            continue;
        }
        // One connected area (4-neighbours).
        let mut area = Vec::new();
        let mut stack = vec![start];
        seen[start] = true;
        while let Some(i) = stack.pop() {
            area.push(i);
            let (px, py) = (i % w, i / w);
            let mut visit = |j: usize| {
                if mask[j] && !seen[j] {
                    seen[j] = true;
                    stack.push(j);
                }
            };
            if px > 0 {
                visit(i - 1);
            }
            if px + 1 < w {
                visit(i + 1);
            }
            if py > 0 {
                visit(i - w);
            }
            if py + 1 < h {
                visit(i + w);
            }
        }
        let (mut x0, mut x1, mut y0, mut y1) = (w, 0, h, 0);
        for &i in &area {
            (x0, x1) = (x0.min(i % w), x1.max(i % w));
            (y0, y1) = (y0.min(i / w), y1.max(i / w));
        }
        let share = area.len() as f64 / mask.len() as f64;
        let (bw, bh) = ((x1 - x0 + 1) as f64, (y1 - y0 + 1) as f64);
        let fill = area.len() as f64 / (bw * bh);
        let touches = x0 == 0 || y0 == 0 || x1 + 1 == w || y1 + 1 == h;
        // A face: at least 0.2 % of the frame, fills its box like an
        // ellipse does (0.79) rather than a ragged texture, upright or
        // somewhat wide, not cut by the frame edge.
        if touches || share < 0.002 || fill < 0.5 || !(0.6..2.5).contains(&(bh / bw)) {
            continue;
        }
        let score = (share / 0.01).min(1.0);
        weight += score;
        tones.extend(area.iter().map(|&i| (x[i][1], score)));
    }
    if tones.is_empty() {
        return None;
    }
    // Weighted median tone of the skin.
    tones.sort_by(|a, b| a.0.total_cmp(&b.0));
    let half = tones.iter().map(|t| t.1).sum::<f64>() / 2.0;
    let mut acc = 0.0;
    let tone = tones
        .iter()
        .find(|t| {
            acc += t.1;
            acc >= half
        })
        .map_or(tones[0].0, |t| t.0);
    Some(SkinTone {
        tone,
        weight: weight.min(1.0),
    })
}

/// Normalised tone above which skin is beyond what the levels print, and
/// where moving the white end puts it (see [`keyed`]).
const SKIN_WHITE: f64 = 0.9;
const SKIN_FIT: f64 = 0.75;

/// Levels whose midtone key (the `mid` the auto midtone places, see
/// [`super::curve::auto_gamma`]) moves from the frame's median towards
/// the tone that prints the skin at [`SKIN_TARGET`]: the two midtone
/// gammas are blended in log, skin taking up to [`SKIN_SHARE`]. Skin
/// beyond [`SKIN_WHITE`] first moves the white end out (every channel by
/// the same share, so greys stay grey) until it sits at [`SKIN_FIT`]; a
/// frame whose median is at its black end then has only the skin to go by.
pub fn keyed(l: &Levels, skin: &SkinTone) -> Levels {
    let mut l = *l;
    let mut s = skin.tone;
    if s > SKIN_WHITE {
        let f = 1.0 + (s / SKIN_FIT - 1.0) * skin.weight;
        for c in l.iter_mut() {
            c.hi = c.lo + (c.hi - c.lo) * f;
        }
        s /= f;
    }
    let m = (0..3).map(|c| l[c].normalise(l[c].mid)).sum::<f64>() / 3.0;
    if !(s > 0.0 && s < 1.0) {
        return l;
    }
    // Both within the auto midtone's own limits (see auto_gamma).
    let clamp = |g: f64| g.clamp(0.8, 1.6);
    let g_mid = if m > 0.0 && m < 1.0 {
        clamp(0.5f64.ln() / m.ln())
    } else {
        1.0
    };
    let g_skin = clamp(SKIN_TARGET.ln() / s.ln());
    let k = SKIN_SHARE * skin.weight;
    let gamma = (g_mid.ln() * (1.0 - k) + g_skin.ln() * k).exp();
    // The normalised key that auto_gamma turns into this gamma.
    let key = 0.5f64.powf(1.0 / gamma);
    l.map(|c| super::curve::ChannelLevels {
        mid: c.density(key),
        ..c
    })
}
