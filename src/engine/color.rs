//! Colour science helpers (NLPColorV3.lua): RGB working spaces, transfer
//! functions, HSV/HSL and minimal ICC profile generation.
//!
//! The working space is **MelissaRGB**: ProPhoto primaries and D50 white point
//! (NLPColorV3's MelissaRGB `toXYZ` is the D50 ProPhoto matrix) with the sRGB
//! tone response — the space Lightroom's tone curves operate in.

pub type Mat3 = [[f64; 3]; 3];

pub const D65: [f64; 2] = [0.3127, 0.3290];
pub const D50: [f64; 2] = [0.3457, 0.3585];

#[derive(Clone, Copy, Debug)]
pub struct Primaries {
    pub r: [f64; 2],
    pub g: [f64; 2],
    pub b: [f64; 2],
    pub white: [f64; 2],
}

pub const SRGB: Primaries = Primaries {
    r: [0.64, 0.33],
    g: [0.30, 0.60],
    b: [0.15, 0.06],
    white: D65,
};
pub const ADOBE_RGB: Primaries = Primaries {
    r: [0.64, 0.33],
    g: [0.21, 0.71],
    b: [0.15, 0.06],
    white: D65,
};
pub const PROPHOTO: Primaries = Primaries {
    r: [0.7347, 0.2653],
    g: [0.1596, 0.8404],
    b: [0.0366, 0.0001],
    white: D50,
};
pub const MELISSA: Primaries = PROPHOTO;

pub fn xy_to_xyz(xy: [f64; 2]) -> [f64; 3] {
    [xy[0] / xy[1], 1.0, (1.0 - xy[0] - xy[1]) / xy[1]]
}

pub fn mat_mul(a: &Mat3, b: &Mat3) -> Mat3 {
    let mut m = [[0.0; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            m[i][j] = (0..3).map(|k| a[i][k] * b[k][j]).sum();
        }
    }
    m
}

pub fn mat_vec(m: &Mat3, v: [f64; 3]) -> [f64; 3] {
    [
        m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
        m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
        m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2],
    ]
}

pub fn mat_inv(m: &Mat3) -> Mat3 {
    let det = m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
        - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
        + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0]);
    let d = if det.abs() < 1e-12 { 1e-12 } else { det };
    [
        [
            (m[1][1] * m[2][2] - m[1][2] * m[2][1]) / d,
            (m[0][2] * m[2][1] - m[0][1] * m[2][2]) / d,
            (m[0][1] * m[1][2] - m[0][2] * m[1][1]) / d,
        ],
        [
            (m[1][2] * m[2][0] - m[1][0] * m[2][2]) / d,
            (m[0][0] * m[2][2] - m[0][2] * m[2][0]) / d,
            (m[0][2] * m[1][0] - m[0][0] * m[1][2]) / d,
        ],
        [
            (m[1][0] * m[2][1] - m[1][1] * m[2][0]) / d,
            (m[0][1] * m[2][0] - m[0][0] * m[2][1]) / d,
            (m[0][0] * m[1][1] - m[0][1] * m[1][0]) / d,
        ],
    ]
}

/// RGB → XYZ matrix for a set of primaries (relative to its own white).
pub fn rgb_to_xyz(p: &Primaries) -> Mat3 {
    let (r, g, b) = (xy_to_xyz(p.r), xy_to_xyz(p.g), xy_to_xyz(p.b));
    let m = [[r[0], g[0], b[0]], [r[1], g[1], b[1]], [r[2], g[2], b[2]]];
    let s = mat_vec(&mat_inv(&m), xy_to_xyz(p.white));
    [
        [m[0][0] * s[0], m[0][1] * s[1], m[0][2] * s[2]],
        [m[1][0] * s[0], m[1][1] * s[1], m[1][2] * s[2]],
        [m[2][0] * s[0], m[2][1] * s[1], m[2][2] * s[2]],
    ]
}

/// Bradford chromatic adaptation matrix from white `src` to `dst`.
pub fn bradford(src: [f64; 2], dst: [f64; 2]) -> Mat3 {
    const B: Mat3 = [
        [0.8951, 0.2664, -0.1614],
        [-0.7502, 1.7135, 0.0367],
        [0.0389, -0.0685, 1.0296],
    ];
    let s = mat_vec(&B, xy_to_xyz(src));
    let d = mat_vec(&B, xy_to_xyz(dst));
    let diag = [
        [d[0] / s[0], 0.0, 0.0],
        [0.0, d[1] / s[1], 0.0],
        [0.0, 0.0, d[2] / s[2]],
    ];
    mat_mul(&mat_inv(&B), &mat_mul(&diag, &B))
}

/// Matrix converting linear RGB in `from` to linear RGB in `to` (with Bradford adaptation).
pub fn rgb_to_rgb(from: &Primaries, to: &Primaries) -> Mat3 {
    let m = rgb_to_xyz(from);
    let adapt = bradford(from.white, to.white);
    mat_mul(&mat_inv(&rgb_to_xyz(to)), &mat_mul(&adapt, &m))
}

pub fn to_f32(m: &Mat3) -> [[f32; 3]; 3] {
    m.map(|r| r.map(|v| v as f32))
}

#[inline]
pub fn mul3(m: &[[f32; 3]; 3], v: [f32; 3]) -> [f32; 3] {
    [
        m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
        m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
        m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2],
    ]
}

// ---- Transfer functions ------------------------------------------------------

#[inline]
pub fn srgb_encode(v: f32) -> f32 {
    if v <= 0.003_130_8 {
        12.92 * v.max(0.0)
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    }
}

#[inline]
pub fn srgb_decode(v: f32) -> f32 {
    if v <= 0.040_45 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

pub const ADOBE_GAMMA: f32 = 563.0 / 256.0;

#[inline]
pub fn prophoto_encode(v: f32) -> f32 {
    if v < 1.0 / 512.0 {
        16.0 * v.max(0.0)
    } else {
        v.powf(1.0 / 1.8)
    }
}

/// Fast encoder via lookup table (input is linear 0..=1, clamped).
pub struct EncodeLut {
    table: Vec<f32>,
}

impl EncodeLut {
    const N: usize = 16384;
    pub fn new(f: impl Fn(f32) -> f32) -> Self {
        let table = (0..=Self::N)
            .map(|i| f(i as f32 / Self::N as f32))
            .collect();
        Self { table }
    }
    #[inline]
    pub fn get(&self, v: f32) -> f32 {
        let x = v.clamp(0.0, 1.0) * Self::N as f32;
        let i = x as usize;
        if i >= Self::N {
            return self.table[Self::N];
        }
        let t = x - i as f32;
        self.table[i] + (self.table[i + 1] - self.table[i]) * t
    }
}

pub fn srgb_encode_lut() -> &'static EncodeLut {
    use std::sync::OnceLock;
    static L: OnceLock<EncodeLut> = OnceLock::new();
    L.get_or_init(|| EncodeLut::new(srgb_encode))
}

pub fn srgb_decode_lut() -> &'static EncodeLut {
    use std::sync::OnceLock;
    static L: OnceLock<EncodeLut> = OnceLock::new();
    L.get_or_init(|| EncodeLut::new(srgb_decode))
}

// ---- HSV / HSL -----------------------------------------------------------------

/// Returns (hue degrees 0..360, saturation 0..1, value 0..1). Gray → hue 0.
#[inline]
pub fn rgb_to_hsv(c: [f32; 3]) -> (f32, f32, f32) {
    let mx = c[0].max(c[1]).max(c[2]);
    let mn = c[0].min(c[1]).min(c[2]);
    let d = mx - mn;
    let s = if mx > 0.0 { d / mx } else { 0.0 };
    let h = if d <= 1e-9 {
        0.0
    } else if mx == c[0] {
        60.0 * (((c[1] - c[2]) / d).rem_euclid(6.0))
    } else if mx == c[1] {
        60.0 * ((c[2] - c[0]) / d + 2.0)
    } else {
        60.0 * ((c[0] - c[1]) / d + 4.0)
    };
    (h, s, mx)
}

#[inline]
pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> [f32; 3] {
    if s <= 0.0 {
        return [v, v, v];
    }
    let h = h.rem_euclid(360.0) / 60.0;
    let i = h.floor();
    let f = h - i;
    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));
    match i as i32 {
        0 => [v, t, p],
        1 => [q, v, p],
        2 => [p, v, t],
        3 => [p, q, v],
        4 => [t, p, v],
        _ => [v, p, q],
    }
}

/// HSL lightness.
#[inline]
pub fn hsl_lightness(c: [f32; 3]) -> f32 {
    (c[0].max(c[1]).max(c[2]) + c[0].min(c[1]).min(c[2])) * 0.5
}

/// Rec.709-ish luminance on encoded values (used for display helpers / sharpening).
#[inline]
pub fn luma(c: [f32; 3]) -> f32 {
    0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
}

/// Relative luminance (Y) of linear MelissaRGB.
pub fn melissa_luminance_weights() -> [f32; 3] {
    let m = rgb_to_xyz(&MELISSA);
    [m[1][0] as f32, m[1][1] as f32, m[1][2] as f32]
}

// ---- ICC profiles --------------------------------------------------------------

pub enum Trc {
    Gamma(f64),
    Table(Vec<u16>),
}

fn s15(v: f64) -> [u8; 4] {
    ((v * 65536.0).round() as i32).to_be_bytes()
}

fn tag_xyz(v: [f64; 3]) -> Vec<u8> {
    let mut d = b"XYZ \0\0\0\0".to_vec();
    for c in v {
        d.extend_from_slice(&s15(c));
    }
    d
}

fn tag_curv(t: &Trc) -> Vec<u8> {
    let mut d = b"curv\0\0\0\0".to_vec();
    match t {
        Trc::Gamma(g) => {
            d.extend_from_slice(&1u32.to_be_bytes());
            d.extend_from_slice(&((g * 256.0).round() as u16).to_be_bytes());
        }
        Trc::Table(t) => {
            d.extend_from_slice(&(t.len() as u32).to_be_bytes());
            for v in t {
                d.extend_from_slice(&v.to_be_bytes());
            }
        }
    }
    d
}

fn tag_desc(s: &str) -> Vec<u8> {
    let mut d = b"desc\0\0\0\0".to_vec();
    d.extend_from_slice(&(s.len() as u32 + 1).to_be_bytes());
    d.extend_from_slice(s.as_bytes());
    d.push(0);
    d.extend_from_slice(&[0u8; 8]); // unicode lang + count
    d.extend_from_slice(&[0u8; 3]); // scriptcode code + count
    d.extend_from_slice(&[0u8; 67]);
    d
}

fn tag_text(s: &str) -> Vec<u8> {
    let mut d = b"text\0\0\0\0".to_vec();
    d.extend_from_slice(s.as_bytes());
    d.push(0);
    d
}

fn tag_sf32(m: &Mat3) -> Vec<u8> {
    let mut d = b"sf32\0\0\0\0".to_vec();
    for r in m {
        for v in r {
            d.extend_from_slice(&s15(*v));
        }
    }
    d
}

/// Build a v2 matrix/TRC display profile.
pub fn make_icc(name: &str, p: &Primaries, trc: Trc) -> Vec<u8> {
    let adapt = bradford(p.white, D50);
    let m = mat_mul(&adapt, &rgb_to_xyz(p));
    let col = |i: usize| [m[0][i], m[1][i], m[2][i]];
    let d50 = xy_to_xyz(D50);
    let curv = tag_curv(&trc);
    let tags: Vec<([u8; 4], Vec<u8>)> = vec![
        (*b"desc", tag_desc(name)),
        (*b"cprt", tag_text("No copyright, use freely")),
        (*b"wtpt", tag_xyz([0.9642, 1.0, 0.8249])),
        (*b"chad", tag_sf32(&adapt)),
        (*b"rXYZ", tag_xyz(col(0))),
        (*b"gXYZ", tag_xyz(col(1))),
        (*b"bXYZ", tag_xyz(col(2))),
        (*b"rTRC", curv.clone()),
        (*b"gTRC", curv.clone()),
        (*b"bTRC", curv),
    ];
    let table_len = 4 + 12 * tags.len();
    let mut offset = 128 + table_len;
    let mut table = (tags.len() as u32).to_be_bytes().to_vec();
    let mut data = Vec::new();
    for (sig, body) in &tags {
        while (offset + data.len()) % 4 != 0 {
            data.push(0);
        }
        let off = offset + data.len();
        table.extend_from_slice(sig);
        table.extend_from_slice(&(off as u32).to_be_bytes());
        table.extend_from_slice(&(body.len() as u32).to_be_bytes());
        data.extend_from_slice(body);
    }
    offset = 128 + table.len() + data.len();
    let mut h = Vec::with_capacity(offset);
    h.extend_from_slice(&(offset as u32).to_be_bytes());
    h.extend_from_slice(b"none");
    h.extend_from_slice(&0x0210_0000u32.to_be_bytes());
    h.extend_from_slice(b"mntrRGB XYZ ");
    h.extend_from_slice(&[0x07, 0xEA, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0]); // 2026-01-01
    h.extend_from_slice(b"acspAPPL");
    h.extend_from_slice(&[0u8; 4]); // flags
    h.extend_from_slice(&[0u8; 8]); // manufacturer + model
    h.extend_from_slice(&[0u8; 8]); // attributes
    h.extend_from_slice(&[0u8; 4]); // intent
    for c in d50 {
        h.extend_from_slice(&s15(c));
    }
    h.extend_from_slice(b"none");
    h.resize(128, 0);
    h.extend_from_slice(&table);
    h.extend_from_slice(&data);
    h
}

fn table_trc(f: impl Fn(f64) -> f64) -> Trc {
    // ICC TRC tables map encoded -> linear.
    Trc::Table(
        (0..1024)
            .map(|i| (f(i as f64 / 1023.0).clamp(0.0, 1.0) * 65535.0).round() as u16)
            .collect(),
    )
}

pub fn icc_for(space: super::settings::OutputSpace) -> Vec<u8> {
    use super::settings::OutputSpace::*;
    match space {
        Srgb => make_icc(
            "sRGB (NLP Studio)",
            &SRGB,
            table_trc(|v| srgb_decode(v as f32) as f64),
        ),
        AdobeRgb => make_icc(
            "Adobe RGB (1998) compatible (NLP Studio)",
            &ADOBE_RGB,
            Trc::Gamma(ADOBE_GAMMA as f64),
        ),
        ProPhoto => make_icc(
            "ProPhoto RGB (NLP Studio)",
            &PROPHOTO,
            table_trc(|v| {
                if v < 16.0 / 512.0 {
                    v / 16.0
                } else {
                    v.powf(1.8)
                }
            }),
        ),
    }
}

/// Linear MelissaRGB → linear output space matrix + encoder.
pub fn output_transform(space: super::settings::OutputSpace) -> ([[f32; 3]; 3], fn(f32) -> f32) {
    use super::settings::OutputSpace::*;
    match space {
        Srgb => (to_f32(&rgb_to_rgb(&MELISSA, &SRGB)), srgb_encode),
        AdobeRgb => (to_f32(&rgb_to_rgb(&MELISSA, &ADOBE_RGB)), |v| {
            v.max(0.0).powf(1.0 / ADOBE_GAMMA)
        }),
        ProPhoto => (to_f32(&rgb_to_rgb(&MELISSA, &PROPHOTO)), prophoto_encode),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn srgb_matrix_matches_reference() {
        let m = rgb_to_xyz(&SRGB);
        assert!((m[0][0] - 0.4124).abs() < 1e-3);
        assert!((m[1][1] - 0.7152).abs() < 1e-3);
        assert!((m[2][2] - 0.9505).abs() < 1e-3);
    }

    #[test]
    fn white_maps_to_white() {
        let m = rgb_to_rgb(&MELISSA, &SRGB);
        let w = mat_vec(&m, [1.0, 1.0, 1.0]);
        for c in w {
            assert!((c - 1.0).abs() < 1e-6);
        }
        let m = rgb_to_rgb(&MELISSA, &PROPHOTO);
        let w = mat_vec(&m, [1.0, 1.0, 1.0]);
        for c in w {
            assert!((c - 1.0).abs() < 1e-3, "{w:?}");
        }
    }

    #[test]
    fn hsv_roundtrip() {
        let c = [0.2, 0.5, 0.9];
        let (h, s, v) = rgb_to_hsv(c);
        let r = hsv_to_rgb(h, s, v);
        for i in 0..3 {
            assert!((c[i] - r[i]).abs() < 1e-5);
        }
    }
}
