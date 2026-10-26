//! Negative preparation, analysis (NLPImageProcessorV3.lua) and rendering.

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use super::color::{self, hsl_lightness, rgb_to_hsv};
use super::curve::{ChannelOrigin, CurveLut, Origins, update_tone_curve};
use super::dcp;
use super::decode::{self, SourceImage};
use super::look::{self, Look};
use super::presets::{self, TEMP_MULTIPLIER, WbZone, ZONE_COOL, ZONE_NEUTRAL, ZONE_WARM};
use super::settings::{AnalysisKey, FilmBase, ImageType, OutputSpace, Settings, WbMode};

/// Long edge of the analysis rendition (`LR_size_maxHeight = 600`).
pub const ANALYSIS_SIZE: usize = 600;
/// Film base is mapped to this linear level after neutralisation.
const BASE_LEVEL: f32 = 0.97;

/// The prepared negative: encoded MelissaRGB (0..1), i.e. the pixels the
/// plugin's tone curves operate on.
#[derive(Clone)]
pub struct Negative {
    pub width: usize,
    pub height: usize,
    pub data: Vec<[f32; 3]>,
}

/// Auto white-balance candidates as (temp, tint).
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct AutoWb {
    pub neutral: (f32, f32),
    pub warm: (f32, f32),
    pub cool: (f32, f32),
    pub avg: (f32, f32),
}

/// Per-frame result of "Convert".
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Analysis {
    pub origins: Origins,
    pub auto_wb: AutoWb,
    /// Film base colour (source space) used for neutralisation.
    pub base: [f32; 3],
    pub key: AnalysisKey,
}

// ---- film base -------------------------------------------------------------------

/// Estimate the film base (brightest, unexposed area) in source space:
/// the average of pixels between the 98.5th and 99.5th brightness percentile.
/// Always uses the entire image (uncropped) so that film borders remain
/// available for mask neutralisation even when a crop has been set.
pub fn auto_base(src: &SourceImage, _s: &Settings) -> [f32; 3] {
    let d = &src.data;
    let mut sums: Vec<(f32, usize)> = d
        .iter()
        .enumerate()
        .map(|(i, p)| (p[0] + p[1] + p[2], i))
        .collect();
    if sums.is_empty() {
        return [1.0; 3];
    }
    sums.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    let n = sums.len();
    let lo = ((n as f32 * 0.985) as usize).min(n - 1);
    let hi = ((n as f32 * 0.995) as usize).clamp(lo + 1, n);
    let mut acc = [0.0f64; 3];
    for &(_, i) in &sums[lo..hi] {
        for c in 0..3 {
            acc[c] += d[i][c] as f64;
        }
    }
    let k = (hi - lo) as f64;
    acc.map(|v| ((v / k) as f32).max(1e-4))
}

/// Resolve the base colour for a frame (roll base takes precedence for Auto).
pub fn resolve_base(src: &SourceImage, s: &Settings, roll_base: Option<[f32; 3]>) -> [f32; 3] {
    match s.film_base {
        FilmBase::Sampled => s.film_base_sample.map(|v| v.max(1e-4)),
        FilmBase::Auto => roll_base.unwrap_or_else(|| auto_base(src, s)),
        FilmBase::AsShot | FilmBase::Off => auto_base(src, s),
    }
}

/// Source-space multipliers from a base colour.
pub fn base_multipliers(src: &SourceImage, s: &Settings, base: [f32; 3]) -> [f32; 3] {
    match s.film_base {
        FilmBase::Auto | FilmBase::Sampled => base.map(|b| BASE_LEVEL / b.max(1e-4)),
        FilmBase::AsShot => {
            let wb = src.as_shot_wb;
            let m = (0..3)
                .map(|c| base[c] * wb[c])
                .fold(0.0f32, f32::max)
                .max(1e-4);
            wb.map(|v| v * BASE_LEVEL / m)
        }
        FilmBase::Off => [1.0; 3],
    }
}

/// Calibration + pre-saturation matrix (applied to the linear negative, like
/// Lightroom applies Camera Calibration and Saturation before the tone curve).
fn negative_color_matrix(s: &Settings) -> [[f32; 3]; 3] {
    let cal = presets::calibration(s.color_model, s.source);
    let yw = color::melissa_luminance_weights();
    const K: f32 = 0.25 / 100.0;
    // Primary columns.
    let mut cols = [[1.0f32, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    let hue = [cal.red_hue, cal.green_hue, cal.blue_hue];
    let sat = [cal.red_sat, cal.green_sat, cal.blue_sat];
    // (towards for +hue, towards for −hue)
    let dirs = [(1usize, 2usize), (2, 0), (1, 0)];
    for p in 0..3 {
        let h = hue[p];
        if h > 0.0 {
            cols[p][dirs[p].0] += h * K;
        } else if h < 0.0 {
            cols[p][dirs[p].1] += -h * K;
        }
        let y = cols[p][0] * yw[0] + cols[p][1] * yw[1] + cols[p][2] * yw[2];
        let f = 1.0 + sat[p] / 100.0;
        for c in 0..3 {
            cols[p][c] = y + (cols[p][c] - y) * f;
        }
    }
    let mut m = [[0.0f32; 3]; 3];
    for r in 0..3 {
        for p in 0..3 {
            m[r][p] = cols[p][r];
        }
        let sum: f32 = m[r].iter().sum();
        if sum.abs() > 1e-6 {
            for v in m[r].iter_mut() {
                *v /= sum;
            }
        }
    }
    // Saturation (Lightroom −100..100) around luminance.
    let f = 1.0 + presets::pre_saturation(s.pre_saturation) / 100.0;
    let sat_m: [[f32; 3]; 3] = std::array::from_fn(|r| {
        std::array::from_fn(|c| yw[c] * (1.0 - f) + if r == c { f } else { 0.0 })
    });
    let mut out = [[0.0f32; 3]; 3];
    for r in 0..3 {
        for c in 0..3 {
            out[r][c] = (0..3).map(|k| sat_m[r][k] * m[k][c]).sum();
        }
    }
    out
}

/// The plugin's Negative Lab camera profile for this source, if one matches.
pub fn camera_profile(src: &SourceImage) -> Option<std::sync::Arc<dcp::DcpProfile>> {
    if src.is_raw {
        dcp::profile_for(&src.info)
    } else {
        None
    }
}

/// The scan source implied by a file: VueScan / SilverFast DNGs (scanner
/// profile or scanner software), otherwise digital camera; bitmaps are TIF scans.
pub fn detect_source(src: &SourceImage) -> super::settings::Source {
    use super::settings::Source;
    if !src.is_raw {
        return Source::TifScan;
    }
    let scanner_id = |s: &str| {
        let l = s.to_lowercase();
        l.contains("vuescan") || l.contains("silverfast")
    };
    let by_profile = camera_profile(src).map(|p| {
        scanner_id(&p.camera)
            || p.path.components().any(|c| {
                c.as_os_str()
                    .to_string_lossy()
                    .eq_ignore_ascii_case("Vuescan Profiles")
            })
    });
    match by_profile {
        Some(true) => Source::VueScanRaw,
        Some(false) => Source::DigitalCamera,
        None if src.info.camera_ids.iter().any(|s| scanner_id(s)) => Source::VueScanRaw,
        None => Source::DigitalCamera,
    }
}

/// Camera profile transform with the film base as white balance (like
/// sampling the border with Lightroom's WB eyedropper).
fn dcp_transform(src: &SourceImage, s: &Settings, base: [f32; 3]) -> Option<dcp::DcpTransform> {
    let p = camera_profile(src)?;
    let base64 = base.map(|v| v.max(1e-6) as f64);
    let (neutral, level_ref) = match s.film_base {
        FilmBase::Auto | FilmBase::Sampled => (base64, Some(base)),
        FilmBase::AsShot => {
            let wb = src.as_shot_wb;
            (
                [
                    1.0 / wb[0].max(1e-6) as f64,
                    1.0 / wb[1].max(1e-6) as f64,
                    1.0 / wb[2].max(1e-6) as f64,
                ],
                Some(base),
            )
        }
        FilmBase::Off => (p.xy_to_neutral(color::D65), None),
    };
    Some(dcp::DcpTransform::new(p, neutral, level_ref, BASE_LEVEL))
}

/// Build the encoded negative from source pixels.
pub fn prepare_negative(src: &SourceImage, s: &Settings, base: [f32; 3]) -> Negative {
    let (d, w, h) = decode::geometry(
        &src.data,
        src.width,
        src.height,
        s.rotation,
        s.flip_h,
        s.straighten,
        s.crop,
    );
    let mul = base_multipliers(src, s, base);
    let to_mel = src.to_melissa;
    let dcp = dcp_transform(src, s, base);
    let cm = negative_color_matrix(s);
    let mono = presets::calibration(s.color_model, s.source).grayscale;
    let yw = color::melissa_luminance_weights();
    let enc = color::srgb_encode_lut();
    let data = d
        .par_iter()
        .map(|p| {
            let m = match &dcp {
                Some(t) => t.apply(*p),
                None => color::mul3(&to_mel, [p[0] * mul[0], p[1] * mul[1], p[2] * mul[2]]),
            };
            let mut m = color::mul3(&cm, [m[0].max(0.0), m[1].max(0.0), m[2].max(0.0)]);
            if mono {
                let y = m[0] * yw[0] + m[1] * yw[1] + m[2] * yw[2];
                m = [y, y, y];
            }
            [enc.get(m[0]), enc.get(m[1]), enc.get(m[2])]
        })
        .collect();
    Negative {
        width: w,
        height: h,
        data,
    }
}

// ---- analysis ------------------------------------------------------------------

/// The 600px, border-cropped, blurred analysis buffer.
pub fn analysis_buffer(neg: &Negative, s: &Settings) -> Negative {
    let (d, w, h) = decode::downscale(&neg.data, neg.width, neg.height, ANALYSIS_SIZE);
    let b = s.border_buffer.clamp(0.0, 40.0);
    let ins = [b, b, b, b];
    let (d, w, h) = decode::crop(&d, w, h, ins);
    let d = look::gaussian_blur_3(&d, w, h, 2.0);
    Negative {
        width: w,
        height: h,
        data: d,
    }
}

/// CDF clipping (L50_1): returns (whitePointOrigin, blackPointOrigin, meanPoint).
pub fn cdf_origins(hist: &[u64; 256], total: u64, t: ImageType) -> ChannelOrigin {
    let (white_t, black_t) = presets::clip_thresholds(t);
    let total = total.max(1) as f64;
    let mut white = 0i64;
    let mut acc = 0.0;
    for (i, &v) in hist.iter().enumerate() {
        acc += v as f64 / total;
        if acc > white_t {
            white = (i as i64 - 1).max(0);
            break;
        }
    }
    let mut black = 255i64;
    acc = 0.0;
    for i in (0..256).rev() {
        acc += hist[i] as f64 / total;
        if acc > black_t {
            black = (i as i64 - 1).min(255);
            break;
        }
    }
    if black <= white {
        black = (white + 1).min(255);
        white = black - 1;
    }
    let ks = (white as f64 * 1.005).round() as i64;
    let ke = (black as f64 * 0.995).round() as i64;
    let span = (ke - ks).max(1) as f64;
    let step = 256.0 / span;
    let (mut num, mut den) = (0.0f64, 0.0f64);
    for i in ks.max(0)..=ke.min(255) {
        let hv = hist[i as usize] as f64;
        den += hv;
        num += hv * (i - ks) as f64 * step;
    }
    let mut mean = if den > 0.0 { num / den / 256.0 } else { 0.5 };
    if t == ImageType::Negative {
        mean = 1.0 - mean;
    }
    ChannelOrigin {
        white: white as f64,
        black: black as f64,
        mean,
    }
}

pub fn histograms(data: &[[f32; 3]]) -> [[u64; 256]; 3] {
    let mut h = [[0u64; 256]; 3];
    for p in data {
        for c in 0..3 {
            h[c][(p[c].clamp(0.0, 1.0) * 255.0).round() as usize] += 1;
        }
    }
    h
}

pub fn origins_from_buffer(buf: &Negative, t: ImageType) -> Origins {
    let h = histograms(&buf.data);
    let n = buf.data.len() as u64;
    [
        cdf_origins(&h[0], n, t),
        cdf_origins(&h[1], n, t),
        cdf_origins(&h[2], n, t),
    ]
}

/// Zone mask weight for the smart-colour WB analysis (computed on the negative,
/// like the ImageMagick HSB/HSL masks).
fn zone_weight(p: [f32; 3], z: &WbZone) -> f32 {
    let (h, s, _) = rgb_to_hsv(p);
    let hue = h / 3.6;
    let in_hue = hue > z.hue_low.max(0.0) && hue <= z.hue_high;
    let hue_ok = if z.invert_hue {
        hue > 0.0 && !in_hue
    } else {
        in_hue
    };
    if !hue_ok || s * 100.0 >= 100.0 - z.sat_threshold {
        return 0.0;
    }
    let l = hsl_lightness(p) * 100.0;
    if l < z.lum_low || l > z.lum_high || l >= 100.0 {
        return 0.0;
    }
    l / 100.0
}

/// `linearBalance("r")` followed by `updateFilm`'s conversion: the offsets of
/// green and blue from red (in 8-bit units) become Tint and Temp.
pub fn linear_balance_wb(c: [f64; 3]) -> (f32, f32) {
    if c == [0.0; 3] {
        return (0.0, 0.0);
    }
    let v2 = 255.0 * (c[0] - c[1]);
    let v3 = 255.0 * (c[0] - c[2]);
    (
        (-v3 / TEMP_MULTIPLIER) as f32,
        (-v2 / TEMP_MULTIPLIER) as f32,
    )
}

/// One "smart colour" zone (NLPImageProcessorV3): average the negative's
/// pixels under the HSB/HSL zone mask (weighted by HSL lightness, as the
/// ImageMagick alpha mask does), round to integer HSB like the plugin's
/// metadata, then balance.
fn zone_wb(buf: &Negative, z: &WbZone) -> (f32, f32) {
    let (mut acc, mut wsum) = ([0.0f64; 3], 0.0f64);
    for p in &buf.data {
        let w = zone_weight(*p, z) as f64;
        if w > 0.0 {
            for c in 0..3 {
                acc[c] += p[c] as f64 * w;
            }
            wsum += w;
        }
    }
    if wsum <= 0.0 {
        return (0.0, 0.0);
    }
    let mean = acc.map(|v| (v / wsum) as f32);
    let (h, s, v) = rgb_to_hsv(mean);
    let (h, s, v) = (
        h.round(),
        (s * 100.0).round() / 100.0,
        (v * 100.0).round() / 100.0,
    );
    if s == 0.0 {
        return (0.0, 0.0);
    }
    let c = color::hsv_to_rgb(h, s, v);
    linear_balance_wb([c[0] as f64, c[1] as f64, c[2] as f64])
}

/// "Auto-AVG" (autoColor): linear balance of the channel means.
fn avg_wb(o: &Origins) -> (f32, f32) {
    linear_balance_wb([o[0].mean, o[1].mean, o[2].mean])
}

/// Full conversion analysis of a prepared negative.
pub fn analyze(neg: &Negative, s: &Settings, base: [f32; 3]) -> Analysis {
    let buf = analysis_buffer(neg, s);
    let origins = origins_from_buffer(&buf, s.image_type);
    let auto_wb = auto_wb(&buf, &origins);
    Analysis {
        origins,
        auto_wb,
        base,
        key: s.analysis_key(),
    }
}

pub fn auto_wb(buf: &Negative, origins: &Origins) -> AutoWb {
    AutoWb {
        neutral: zone_wb(buf, &ZONE_NEUTRAL),
        warm: zone_wb(buf, &ZONE_WARM),
        cool: zone_wb(buf, &ZONE_COOL),
        avg: avg_wb(origins),
    }
}

/// (temp, tint) for a WB dropdown choice (`updateFilm`); `None` keeps the current values.
pub fn wb_for_mode(mode: WbMode, a: &AutoWb) -> Option<(f32, f32)> {
    if let Some((magenta, yellow)) = presets::film_character(mode) {
        return Some((yellow, magenta));
    }
    match mode {
        WbMode::AutoNeutral => Some(a.neutral),
        WbMode::AutoWarm => Some(a.warm),
        WbMode::AutoCool => Some(a.cool),
        WbMode::AutoAvg => Some(a.avg),
        WbMode::AutoMix => Some(((a.neutral.0 + a.warm.0) / 2.0, a.neutral.1)),
        _ => None,
    }
}

// ---- roll ---------------------------------------------------------------------------

/// Roll aggregation (NLPRolls.lua): min white points, max black points, mean
/// of means, skipping frames whose colour balance is a > 2.5σ outlier.
/// Returns the roll origins and the indices of excluded frames.
pub fn aggregate_roll(frames: &[Origins]) -> (Origins, Vec<usize>) {
    let n = frames.len();
    let feat = |o: &Origins| {
        [
            o[0].white - o[1].white,
            o[2].white - o[1].white,
            o[0].black - o[1].black,
            o[2].black - o[1].black,
        ]
    };
    let mut excluded = Vec::new();
    if n >= 4 {
        let fs: Vec<[f64; 4]> = frames.iter().map(feat).collect();
        for k in 0..4 {
            let mean = fs.iter().map(|f| f[k]).sum::<f64>() / n as f64;
            let sd = (fs.iter().map(|f| (f[k] - mean).powi(2)).sum::<f64>() / n as f64).sqrt();
            if sd > 1e-9 {
                for (i, f) in fs.iter().enumerate() {
                    if (f[k] - mean).abs() > 2.5 * sd && !excluded.contains(&i) {
                        excluded.push(i);
                    }
                }
            }
        }
    }
    let used: Vec<&Origins> = frames
        .iter()
        .enumerate()
        .filter(|(i, _)| !excluded.contains(i))
        .map(|(_, o)| o)
        .collect();
    let used = if used.is_empty() {
        frames.iter().collect()
    } else {
        used
    };
    let mut out = used[0].to_owned();
    for c in 0..3 {
        out[c].white = used.iter().map(|o| o[c].white).fold(f64::MAX, f64::min);
        out[c].black = used.iter().map(|o| o[c].black).fold(f64::MIN, f64::max);
        out[c].mean = used.iter().map(|o| o[c].mean).sum::<f64>() / used.len() as f64;
    }
    excluded.sort();
    (out, excluded)
}

// ---- rendering --------------------------------------------------------------------

/// Everything needed to turn negative pixels into output pixels.
pub struct Renderer {
    lut: Option<CurveLut>,
    look: Look,
    out_m: [[f32; 3]; 3],
    out_enc: color::EncodeLut,
    sharpen: Option<presets::SharpenDef>,
}

impl Renderer {
    /// `origins == None` renders the negative as-is (unconverted).
    pub fn new(s: &Settings, origins: Option<&Origins>, space: OutputSpace) -> Self {
        let lut = origins.map(|o| {
            CurveLut::from_curves(&update_tone_curve(o, s)).with_print_exposure(s.print_exposure)
        });
        let (out_m, f) = color::output_transform(space);
        Self {
            lut,
            look: Look::new(s),
            out_m,
            out_enc: color::EncodeLut::new(f),
            sharpen: presets::sharpening(s.sharpening),
        }
    }

    /// Scale sharpening for the image size like `updateSharpening`
    /// (relative to 24 MP; strength capped at 150 %, radius at 1–3 px).
    pub fn with_megapixels(mut self, mp: f32) -> Self {
        if let Some(sd) = &mut self.sharpen {
            let f = (mp - 24.0) / 2.0 / 24.0 + 1.0;
            let sharpness = (sd.lr_sharpness * f).round().min(150.0);
            sd.amount *= sharpness / sd.lr_sharpness;
            sd.radius = (sd.radius * f).clamp(1.0, 3.0);
        }
        self
    }

    pub fn curve_lut(&self) -> Option<&CurveLut> {
        self.lut.as_ref()
    }

    pub fn uses_lut_table(&self) -> bool {
        self.look.uses_table()
    }

    /// Encoded MelissaRGB positive (before output conversion).
    #[inline]
    pub fn positive(&self, p: [f32; 3]) -> [f32; 3] {
        match &self.lut {
            Some(l) => self
                .look
                .apply([l.apply(0, p[0]), l.apply(1, p[1]), l.apply(2, p[2])]),
            None => p,
        }
    }

    #[inline]
    pub fn output(&self, mel_enc: [f32; 3]) -> [f32; 3] {
        let dec = color::srgb_decode_lut();
        let lin = [
            dec.get(mel_enc[0]),
            dec.get(mel_enc[1]),
            dec.get(mel_enc[2]),
        ];
        let o = color::mul3(&self.out_m, lin);
        [
            self.out_enc.get(o[0]),
            self.out_enc.get(o[1]),
            self.out_enc.get(o[2]),
        ]
    }

    /// Render a whole buffer to encoded output pixels. `sharpen_scale` is the
    /// ratio of this buffer to full resolution (0 disables sharpening).
    pub fn render(&self, neg: &Negative, sharpen_scale: f32) -> Vec<[f32; 3]> {
        let mut out: Vec<[f32; 3]> = neg
            .data
            .par_iter()
            .map(|p| self.output(self.positive(*p)))
            .collect();
        if let (Some(sd), true) = (self.sharpen, sharpen_scale > 0.0 && self.lut.is_some()) {
            look::sharpen(&mut out, neg.width, neg.height, sd, sharpen_scale);
        }
        out
    }
}

/// Convenience: pick the origins a frame should render with.
pub fn origins_for<'a>(
    s: &Settings,
    frame: Option<&'a Analysis>,
    roll: Option<&'a Origins>,
) -> Option<&'a Origins> {
    if s.use_roll {
        if let Some(r) = roll {
            return Some(r);
        }
    }
    frame.map(|a| &a.origins)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cdf_finds_range() {
        let mut h = [0u64; 256];
        for (i, v) in h.iter_mut().enumerate().take(201).skip(40) {
            *v = 100 + i as u64;
        }
        let total: u64 = h.iter().sum();
        let o = cdf_origins(&h, total, ImageType::Negative);
        assert!((39.0..=41.0).contains(&o.white), "{o:?}");
        assert!((198.0..=200.0).contains(&o.black), "{o:?}");
        assert!(o.mean > 0.3 && o.mean < 0.7);
    }

    #[test]
    fn vuescan_dng_is_detected_as_scanner() {
        let p = std::path::Path::new("../testphoto.dng");
        if !p.exists() {
            return;
        }
        let src = decode::load(p, Some(400)).unwrap();
        assert_eq!(
            detect_source(&src),
            crate::engine::settings::Source::VueScanRaw
        );
        let mut cam = src.clone();
        cam.info.camera_ids = vec!["Nikon D750".into()];
        cam.info.make = "Nikon".into();
        cam.info.model = "D750".into();
        assert_eq!(
            detect_source(&cam),
            crate::engine::settings::Source::DigitalCamera
        );
        let mut unknown = src.clone();
        unknown.info.camera_ids = vec!["Mystery Cam 1".into()];
        unknown.info.make = "Mystery".into();
        unknown.info.model = "Cam 1".into();
        assert_eq!(
            detect_source(&unknown),
            crate::engine::settings::Source::DigitalCamera
        );
    }

    #[test]
    fn roll_aggregation_extremes() {
        let f = |w: f64, b: f64| {
            [ChannelOrigin {
                white: w,
                black: b,
                mean: 0.5,
            }; 3]
        };
        let (o, ex) = aggregate_roll(&[f(40.0, 200.0), f(35.0, 210.0), f(45.0, 190.0)]);
        assert!(ex.is_empty());
        assert_eq!(o[0].white, 35.0);
        assert_eq!(o[0].black, 210.0);
    }

    #[test]
    fn auto_base_samples_entire_image_regardless_of_crop() {
        let mut data = vec![[0.1f32, 0.1, 0.1]; 100 * 100];
        // Put unexposed film base in the border (outer 5 rows)
        for y in 0..5 {
            for x in 0..100 {
                data[y * 100 + x] = [0.9f32, 0.6, 0.4];
            }
        }
        let src = SourceImage {
            width: 100,
            height: 100,
            data,
            to_melissa: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            info: Default::default(),
            as_shot_wb: [1.0; 3],
            is_raw: false,
        };

        let uncropped = Settings::default();
        let mut cropped = Settings::default();
        // Crop out the top 20% containing the border
        cropped.crop = [20.0, 10.0, 10.0, 10.0];

        let base_uncropped = auto_base(&src, &uncropped);
        let base_cropped = auto_base(&src, &cropped);

        assert_eq!(base_uncropped, base_cropped, "auto_base must look at the entire image even when cropped");
        assert!((base_cropped[0] - 0.9).abs() < 1e-3);
    }
}
