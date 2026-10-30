//! Reading embedded ICC profiles of bitmap scans. Only RGB matrix/TRC
//! profiles (rXYZ/gXYZ/bXYZ + rTRC/gTRC/bTRC) are understood, which covers
//! sRGB, Adobe RGB, ProPhoto, linear working spaces and most scanner
//! profiles; LUT-based profiles fall back to sRGB.

use super::color::{self, Mat3};

/// One channel's tone response: encoded value → linear.
#[derive(Clone, Debug)]
pub enum Curve {
    Gamma(f64),
    Table(Vec<f64>),
    /// ICC `para`: function type and its parameters (g, a, b, c, d, e, f).
    Para(u16, [f64; 7]),
}

impl Curve {
    pub fn eval(&self, x: f64) -> f64 {
        let x = x.clamp(0.0, 1.0);
        match self {
            Curve::Gamma(g) => x.powf(*g),
            Curve::Table(t) => {
                let p = x * (t.len() - 1) as f64;
                let i = (p as usize).min(t.len() - 2);
                t[i] + (t[i + 1] - t[i]) * (p - i as f64)
            }
            Curve::Para(kind, [g, a, b, c, d, e, f]) => {
                let pow = |v: f64| v.max(0.0).powf(*g);
                match kind {
                    0 => pow(x),
                    1 => if x >= -b / a { pow(a * x + b) } else { 0.0 },
                    2 => if x >= -b / a { pow(a * x + b) + c } else { *c },
                    3 => if x >= *d { pow(a * x + b) } else { c * x },
                    _ => if x >= *d { pow(a * x + b) + e } else { c * x + f },
                }
            }
        }
    }
}

/// An RGB matrix/TRC profile.
#[derive(Clone, Debug)]
pub struct Profile {
    /// Linear RGB → XYZ (D50 PCS); the columns are rXYZ, gXYZ, bXYZ.
    pub to_xyz: Mat3,
    pub trc: [Curve; 3],
}

impl Profile {
    /// Linear profile RGB → linear MelissaRGB, rows normalised so the
    /// profile's white stays neutral.
    pub fn to_melissa(&self) -> Mat3 {
        let m = color::mat_mul(&color::mat_inv(&color::rgb_to_xyz(&color::MELISSA)), &self.to_xyz);
        m.map(|r| {
            let s: f64 = r.iter().sum();
            r.map(|v| v / s)
        })
    }

    /// The primaries are sRGB's (within the precision profiles store them).
    pub fn is_srgb_primaries(&self) -> bool {
        let srgb = color::rgb_to_rgb(&color::SRGB, &color::MELISSA);
        let m = self.to_melissa();
        (0..3).all(|i| (0..3).all(|j| (m[i][j] - srgb[i][j]).abs() < 0.02))
    }

    /// Every channel follows the sRGB tone curve.
    pub fn is_srgb_trc(&self) -> bool {
        self.trc.iter().all(|c| {
            (1..20).all(|i| {
                let x = i as f64 / 20.0;
                (c.eval(x) - color::srgb_decode(x as f32) as f64).abs() < 0.005
            })
        })
    }
}

fn be_u16(b: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_be_bytes(b.get(at..at + 2)?.try_into().ok()?))
}

fn be_u32(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_be_bytes(b.get(at..at + 4)?.try_into().ok()?))
}

fn s15(b: &[u8], at: usize) -> Option<f64> {
    Some(be_u32(b, at)? as i32 as f64 / 65536.0)
}

fn xyz(t: &[u8]) -> Option<[f64; 3]> {
    if t.get(..4)? != b"XYZ " {
        return None;
    }
    Some([s15(t, 8)?, s15(t, 12)?, s15(t, 16)?])
}

fn curve(t: &[u8]) -> Option<Curve> {
    match t.get(..4)? {
        b"curv" => {
            let n = be_u32(t, 8)? as usize;
            match n {
                0 => Some(Curve::Gamma(1.0)),
                1 => Some(Curve::Gamma(be_u16(t, 12)? as f64 / 256.0)),
                _ => (0..n)
                    .map(|i| Some(be_u16(t, 12 + 2 * i)? as f64 / 65535.0))
                    .collect::<Option<Vec<_>>>()
                    .map(Curve::Table),
            }
        }
        b"para" => {
            let kind = be_u16(t, 8)?;
            let count = *[1usize, 3, 4, 5, 7].get(kind as usize)?;
            let mut p = [0.0; 7];
            for (i, v) in p.iter_mut().enumerate().take(count) {
                *v = s15(t, 12 + 4 * i)?;
            }
            if kind > 0 && p[1] == 0.0 {
                return None;
            }
            Some(Curve::Para(kind, p))
        }
        _ => None,
    }
}

/// Parse an RGB matrix/TRC profile; `None` for anything else.
pub fn parse(b: &[u8]) -> Option<Profile> {
    if b.get(16..20)? != b"RGB " || b.get(20..24)? != b"XYZ " {
        return None;
    }
    let count = be_u32(b, 128)? as usize;
    let tag = |sig: &[u8; 4]| -> Option<&[u8]> {
        (0..count.min(256)).find_map(|i| {
            let e = 132 + 12 * i;
            if b.get(e..e + 4)? != sig {
                return None;
            }
            let (off, len) = (be_u32(b, e + 4)? as usize, be_u32(b, e + 8)? as usize);
            b.get(off..off.checked_add(len)?)
        })
    };
    let (r, g, bl) = (xyz(tag(b"rXYZ")?)?, xyz(tag(b"gXYZ")?)?, xyz(tag(b"bXYZ")?)?);
    let to_xyz = [
        [r[0], g[0], bl[0]],
        [r[1], g[1], bl[1]],
        [r[2], g[2], bl[2]],
    ];
    // A singular matrix is not a usable profile.
    let det = color::mat_mul(&to_xyz, &color::mat_inv(&to_xyz));
    if (det[0][0] - 1.0).abs() > 1e-3 || (det[1][1] - 1.0).abs() > 1e-3 {
        return None;
    }
    let trc = [
        curve(tag(b"rTRC")?)?,
        curve(tag(b"gTRC")?)?,
        curve(tag(b"bTRC")?)?,
    ];
    Some(Profile { to_xyz, trc })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::settings::OutputSpace;

    #[test]
    fn reads_the_profiles_the_app_writes() {
        let srgb = parse(&color::icc_for(OutputSpace::Srgb)).unwrap();
        assert!(srgb.is_srgb_primaries() && srgb.is_srgb_trc());

        let adobe = parse(&color::icc_for(OutputSpace::AdobeRgb)).unwrap();
        assert!(!adobe.is_srgb_primaries() && !adobe.is_srgb_trc());
        let want = color::rgb_to_rgb(&color::ADOBE_RGB, &color::MELISSA);
        let m = adobe.to_melissa();
        for i in 0..3 {
            for j in 0..3 {
                assert!((m[i][j] - want[i][j]).abs() < 2e-3, "{m:?}");
            }
        }
        assert!((adobe.trc[1].eval(0.5) - 0.5f64.powf(563.0 / 256.0)).abs() < 1e-6);

        let pro = parse(&color::icc_for(OutputSpace::ProPhoto)).unwrap();
        let m = pro.to_melissa();
        for i in 0..3 {
            for j in 0..3 {
                let id = if i == j { 1.0 } else { 0.0 };
                assert!((m[i][j] - id).abs() < 2e-3, "{m:?}");
            }
        }
        assert!((pro.trc[0].eval(0.5) - 0.5f64.powf(1.8)).abs() < 1e-3);
    }

    #[test]
    fn parametric_curves() {
        // sRGB as an ICC type-3 parametric curve.
        let c = Curve::Para(3, [2.4, 1.0 / 1.055, 0.055 / 1.055, 1.0 / 12.92, 0.04045, 0.0, 0.0]);
        for i in 0..=10 {
            let x = i as f64 / 10.0;
            assert!((c.eval(x) - color::srgb_decode(x as f32) as f64).abs() < 1e-4);
        }
    }

    #[test]
    fn rejects_non_rgb_and_truncated_profiles() {
        let mut p = color::icc_for(OutputSpace::Srgb);
        assert!(parse(&p[..200]).is_none());
        p[16..20].copy_from_slice(b"GRAY");
        assert!(parse(&p).is_none());
        assert!(parse(&[]).is_none());
    }
}
