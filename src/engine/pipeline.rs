//! Negative preparation, analysis (NLPImageProcessorV3.lua) and rendering.

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use super::color::{self, hsl_lightness, rgb_to_hsv};
use super::curve::{ChannelOrigin, CurveLut, Origins, RollClip, update_tone_curve_roll};
use super::dcp;
use super::decode::{self, SourceImage};
use super::look::{self, Look};
use super::presets::{self, TEMP_MULTIPLIER, WbZone, ZONE_COOL, ZONE_NEUTRAL, ZONE_WARM};
use super::settings::{AnalysisKey, FilmBase, ImageType, OutputSpace, RollMode, Settings, WbMode};
use super::sidecar::RollData;

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
    /// Smart-colour zone averages (neutral, warm, cool) as RGB, after the
    /// plugin's integer HSB rounding; roll analysis averages these.
    #[serde(default)]
    pub colors: [[f32; 3]; 3],
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
/// The Lua histogram is 1-based (`L51_1` stores level v at index v+1), so its
/// `index - 1` is the level itself, i.e. `i` here.
pub fn cdf_origins(hist: &[u64; 256], total: u64, t: ImageType) -> ChannelOrigin {
    let (white_t, black_t) = presets::clip_thresholds(t);
    let total = total.max(1) as f64;
    let mut white = 0i64;
    let mut acc = 0.0;
    for (i, &v) in hist.iter().enumerate() {
        acc += v as f64 / total;
        if acc > white_t {
            white = i as i64;
            break;
        }
    }
    let mut black = 255i64;
    acc = 0.0;
    for i in (0..256).rev() {
        acc += hist[i] as f64 / total;
        if acc > black_t {
            black = i as i64;
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

/// The image the smart-colour zones are measured on (`L3_2`/`L4_2` in
/// NLPImageProcessorV3): the analysis rendition in 8-bit ProPhoto (gamma 1.8),
/// each channel stretched with ImageMagick's `-normalize`, and inverted with
/// `-negate` unless the frame is a positive.
fn zone_source(buf: &Negative, t: ImageType) -> Vec<[f32; 3]> {
    let dec = color::srgb_decode_lut();
    let pp: Vec<[f32; 3]> = buf
        .data
        .iter()
        .map(|p| p.map(|v| (dec.get(v).max(0.0).powf(1.0 / 1.8) * 255.0).round() / 255.0))
        .collect();
    let mut out = pp.clone();
    for c in 0..3 {
        let (lo, hi) = normalize_levels(pp.iter().map(|p| p[c]));
        let span = hi - lo;
        for (o, p) in out.iter_mut().zip(&pp) {
            if span > 0.0 {
                o[c] = ((p[c] - lo) / span).clamp(0.0, 1.0);
            }
        }
    }
    if t != ImageType::Positive {
        for o in &mut out {
            *o = o.map(|v| 1.0 - v);
        }
    }
    out
}

/// Black and white levels of ImageMagick's `-normalize`
/// (`-contrast-stretch 2%x1%`) on 8-bit values.
fn normalize_levels(values: impl Iterator<Item = f32>) -> (f32, f32) {
    let mut hist = [0u64; 256];
    let mut n = 0u64;
    for v in values {
        hist[(v.clamp(0.0, 1.0) * 255.0).round() as usize] += 1;
        n += 1;
    }
    let (black_t, white_t) = (0.02 * n as f64, 0.01 * n as f64);
    let mut acc = 0u64;
    let mut black = 0usize;
    while black < 255 {
        acc += hist[black];
        if acc as f64 >= black_t {
            break;
        }
        black += 1;
    }
    acc = 0;
    let mut white = 255usize;
    while white > 0 {
        acc += hist[white];
        if acc as f64 >= white_t {
            break;
        }
        white -= 1;
    }
    (black as f32 / 255.0, white as f32 / 255.0)
}

/// Binary zone mask (`L4_2`): HSB hue in the zone (`-black-threshold`,
/// `-white-threshold`, `-opaque white`, `-threshold 0`, then `-negate` for
/// inverted zones, which therefore include greys), saturation below
/// `100 − satThreshold` and HSL lightness in range. All plugin zones set
/// `isWeightedLuminosity = false`, so the lightness mask is thresholded too.
fn zone_mask(p: [f32; 3], z: &WbZone) -> bool {
    let (h, s, _) = rgb_to_hsv(p);
    let hue = h / 3.6;
    let in_hue = hue > 0.0 && hue >= z.hue_low && hue <= z.hue_high && hue < 100.0;
    if in_hue == z.invert_hue {
        return false;
    }
    if s * 100.0 >= 100.0 - z.sat_threshold {
        return false;
    }
    let l = hsl_lightness(p) * 100.0;
    l > 0.0 && l >= z.lum_low && l <= z.lum_high && l < 100.0
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

/// One "smart colour" zone: average the zone image under the mask
/// (`-compose CopyOpacity -scale 1x1!`) and round to integer HSB like the
/// plugin's metadata. An empty mask gives black.
fn zone_color(img: &[[f32; 3]], z: &WbZone) -> [f32; 3] {
    let (mut acc, mut n) = ([0.0f64; 3], 0u64);
    for p in img {
        if zone_mask(*p, z) {
            for c in 0..3 {
                acc[c] += p[c] as f64;
            }
            n += 1;
        }
    }
    if n == 0 {
        return [0.0; 3];
    }
    round_hsb(acc.map(|v| (v / n as f64) as f32))
}

/// Integer HSB as stored in the plugin's metadata (`formatHSV`).
fn round_hsb(c: [f32; 3]) -> [f32; 3] {
    let (h, s, v) = rgb_to_hsv(c);
    color::hsv_to_rgb(
        h.round(),
        (s * 100.0).round() / 100.0,
        (v * 100.0).round() / 100.0,
    )
}

/// `L45_1` (NLPViewControllerV3): no saturation means no correction,
/// otherwise `linearBalance("r")`.
fn smart_color_wb(c: [f32; 3]) -> (f32, f32) {
    let (_, s, _) = rgb_to_hsv(c);
    if s == 0.0 {
        return (0.0, 0.0);
    }
    linear_balance_wb([c[0] as f64, c[1] as f64, c[2] as f64])
}

/// "Auto-AVG" (autoColor): linear balance of the channel means.
fn avg_wb(o: &Origins) -> (f32, f32) {
    linear_balance_wb([o[0].mean, o[1].mean, o[2].mean])
}

/// Origins and auto WB that Edit Only frames get instead of an analysis
/// (`L56_1`): full 0..255 range, mean 0.5, grey smart-colour samples.
pub fn edit_only_origins() -> Origins {
    [ChannelOrigin {
        white: 0.0,
        black: 255.0,
        mean: 0.5,
    }; 3]
}

/// Full conversion analysis of a prepared negative. Edit Only frames skip
/// the analysis, as `processPhotos` hands them straight to `L56_1`.
pub fn analyze(neg: &Negative, s: &Settings, base: [f32; 3]) -> Analysis {
    let (origins, auto_wb) = if s.image_type == ImageType::EditOnly {
        (edit_only_origins(), AutoWb::default())
    } else {
        let buf = analysis_buffer(neg, s);
        let origins = origins_from_buffer(&buf, s.image_type);
        let wb = auto_wb(&buf, &origins, s.image_type);
        (origins, wb)
    };
    Analysis {
        origins,
        auto_wb,
        base,
        key: s.analysis_key(),
    }
}

pub fn auto_wb(buf: &Negative, origins: &Origins, t: ImageType) -> AutoWb {
    let img = zone_source(buf, t);
    auto_wb_from(
        [ZONE_NEUTRAL, ZONE_WARM, ZONE_COOL].map(|z| zone_color(&img, &z)),
        origins,
    )
}

/// Auto WB candidates from the zone colours and the channel means.
fn auto_wb_from(colors: [[f32; 3]; 3], origins: &Origins) -> AutoWb {
    AutoWb {
        neutral: smart_color_wb(colors[0]),
        warm: smart_color_wb(colors[1]),
        cool: smart_color_wb(colors[2]),
        avg: avg_wb(origins),
        colors,
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

/// A roll built by "Convert as roll" (`createRoll` in NLPRolls).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RollAggregate {
    /// Minimum white points, maximum black points and the 2.2 power mean
    /// (`rgbAvg`) of the frames' mean points.
    pub origins: Origins,
    /// Roll colour ratios `[green, blue]` relative to red (`L23_1`).
    pub multipliers: [f64; 2],
    pub auto_wb: AutoWb,
}

/// `UtilV3.rgbAvg`: power mean with exponent 2.2.
fn rgb_avg(v: impl Iterator<Item = f64>) -> f64 {
    let (mut sum, mut n) = (0.0, 0usize);
    for x in v {
        sum += x.max(0.0).powf(2.2);
        n += 1;
    }
    if n == 0 {
        0.0
    } else {
        (sum / n as f64).powf(1.0 / 2.2)
    }
}

/// `UtilV3.standardDeviation` (sample, n − 1) and mean.
fn std_dev(v: &[f64]) -> (f64, f64) {
    let n = v.len();
    let mean = v.iter().sum::<f64>() / n.max(1) as f64;
    if n < 2 {
        return (0.0, mean);
    }
    let ss: f64 = v.iter().map(|x| (x - mean).powi(2)).sum();
    ((ss / (n - 1) as f64).sqrt(), mean)
}

/// Roll aggregation (`createRoll`, NLPRolls.lua). The origins use every
/// frame; the colour multipliers (`L23_1`) drop frames whose green or blue
/// ratio is more than 2σ above the mean. Returns the roll and the indices of
/// the frames left out of the multipliers.
pub fn aggregate_roll(frames: &[Analysis]) -> (RollAggregate, Vec<usize>) {
    let mut origins = frames[0].origins;
    for c in 0..3 {
        origins[c].white = frames
            .iter()
            .map(|a| a.origins[c].white)
            .fold(f64::MAX, f64::min);
        origins[c].black = frames
            .iter()
            .map(|a| a.origins[c].black)
            .fold(f64::MIN, f64::max);
        origins[c].mean = rgb_avg(frames.iter().map(|a| a.origins[c].mean));
    }
    let ratio = |a: &Analysis, c: usize| {
        let r = origins[0].black - a.origins[0].white;
        (origins[c].black - a.origins[c].white) / if r.abs() < 1e-9 { 1e-9 } else { r }
    };
    let g: Vec<f64> = frames.iter().map(|a| ratio(a, 1)).collect();
    let b: Vec<f64> = frames.iter().map(|a| ratio(a, 2)).collect();
    let ((g_sd, g_mean), (b_sd, b_mean)) = (std_dev(&g), std_dev(&b));
    let excluded: Vec<usize> = (0..frames.len())
        .filter(|&i| b[i] > b_mean + 2.0 * b_sd || g[i] > g_mean + 2.0 * g_sd)
        .collect();
    let kept = |v: &[f64]| {
        let k: Vec<f64> = (0..v.len())
            .filter(|i| !excluded.contains(i))
            .map(|i| v[i])
            .collect();
        if k.is_empty() {
            v.iter().sum::<f64>() / v.len() as f64
        } else {
            k.iter().sum::<f64>() / k.len() as f64
        }
    };
    let multipliers = [kept(&g), kept(&b)];
    // `L24_1`: average each zone's colour over the roll (2.2 power mean).
    // Unlike the per-frame samples, the roll's HSB is stored unrounded.
    let colors = std::array::from_fn(|z| {
        std::array::from_fn(|c| {
            rgb_avg(frames.iter().map(|a| a.auto_wb.colors[z][c] as f64)) as f32
        })
    });
    let auto_wb = auto_wb_from(colors, &origins);
    (
        RollAggregate {
            origins,
            multipliers,
            auto_wb,
        },
        excluded,
    )
}

/// Pro Lab roll origins for one frame (`L34_1`, process version 3): keep the
/// frame's own density range in one anchor channel and place the other two
/// with the roll's colour ratios, so the roll's balance fits the frame.
pub fn lab_origins(roll: &Origins, mult: [f64; 2], frame: &Origins) -> Origins {
    let (gm, bm) = (mult[0], mult[1]);
    let span_r = roll[0].black - frame[0].white;
    let span_r = if span_r.abs() < 1e-9 { 1e-9 } else { span_r };
    let fg = (roll[1].black - frame[1].white) / span_r;
    let fb = (roll[2].black - frame[2].white) / span_r;
    let (dg, db) = (gm - fg, bm - fb);
    // Factors converting a span in the anchor channel to the other channels.
    let (anchor, k) = if dg < 0.0 && dg < db {
        (1, [1.0 / gm, 1.0, bm / gm])
    } else if db < 0.0 && dg > db {
        (2, [1.0 / bm, gm / bm, 1.0])
    } else {
        (0, [1.0, gm, bm])
    };
    let span = roll[anchor].black - frame[anchor].white;
    let range = frame[anchor].black - frame[anchor].white;
    std::array::from_fn(|c| {
        let white = if c == anchor {
            frame[c].white
        } else {
            roll[c].black - span * k[c]
        };
        ChannelOrigin {
            white,
            black: white + range * k[c],
            mean: roll[c].mean,
        }
    })
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
    pub fn new(s: &Settings, origins: Option<&FrameOrigins>, space: OutputSpace) -> Self {
        let lut = origins.map(|o| {
            CurveLut::from_curves(&update_tone_curve_roll(&o.origins, s, o.roll_clip))
                .with_print_exposure(s.print_exposure)
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

/// The origins (and roll clip offsets) a frame renders with.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrameOrigins {
    pub origins: Origins,
    pub roll_clip: RollClip,
}

impl From<Origins> for FrameOrigins {
    fn from(origins: Origins) -> Self {
        Self {
            origins,
            roll_clip: RollClip::default(),
        }
    }
}

/// Pick the origins a frame should render with (`getOriginsOfPhotoForRoll`):
/// the frame's own analysis, or the roll's according to the Roll Process.
pub fn origins_for(
    s: &Settings,
    frame: Option<&Analysis>,
    roll: Option<&RollData>,
) -> Option<FrameOrigins> {
    let Some(roll) = roll.filter(|_| s.use_roll) else {
        return frame.map(|a| a.origins.into());
    };
    let lab = match (s.roll_mode, roll.multipliers, frame) {
        (RollMode::DarkroomPaper, _, _) | (_, None, _) | (_, _, None) => {
            return Some(roll.origins.into());
        }
        (_, Some(m), Some(a)) => lab_origins(&roll.origins, m, &a.origins),
    };
    let roll_clip = if s.roll_mode == RollMode::BasicLab {
        RollClip {
            white: (roll.origins[0].white - lab[0].white) / 2.0,
            black: (lab[0].black - roll.origins[0].black) / 2.0,
        }
    } else {
        RollClip::default()
    };
    Some(FrameOrigins {
        origins: lab,
        roll_clip,
    })
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
        assert_eq!(o.white, 40.0, "{o:?}");
        assert_eq!(o.black, 200.0, "{o:?}");
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

    fn frame(w: [f64; 3], b: [f64; 3]) -> Analysis {
        Analysis {
            origins: std::array::from_fn(|c| ChannelOrigin {
                white: w[c],
                black: b[c],
                mean: 0.5,
            }),
            auto_wb: AutoWb::default(),
            base: [1.0; 3],
            key: Settings::default().analysis_key(),
        }
    }

    #[test]
    fn roll_aggregation_extremes() {
        let (r, _) = aggregate_roll(&[
            frame([40.0; 3], [200.0; 3]),
            frame([35.0; 3], [210.0; 3]),
            frame([45.0; 3], [190.0; 3]),
        ]);
        assert_eq!(r.origins[0].white, 35.0);
        assert_eq!(r.origins[0].black, 210.0);
        // rgbAvg of equal means is the mean itself.
        assert!((r.origins[0].mean - 0.5).abs() < 1e-9);
    }

    #[test]
    fn lab_origins_keep_frame_range_and_roll_ratios() {
        let frames = [
            frame([40.0, 60.0, 80.0], [200.0, 210.0, 220.0]),
            frame([50.0, 70.0, 90.0], [190.0, 200.0, 210.0]),
        ];
        let (r, _) = aggregate_roll(&frames);
        let f = &frames[1].origins;
        let o = lab_origins(&r.origins, r.multipliers, f);
        let [gm, bm] = r.multipliers;
        // Red anchor: the frame's own red range survives unchanged.
        assert_eq!(o[0].white, f[0].white);
        assert!((o[0].black - f[0].black).abs() < 1e-9);
        // The other channels follow the roll's colour ratios.
        let span = r.origins[0].black - o[0].white;
        assert!((r.origins[1].black - o[1].white - span * gm).abs() < 1e-9);
        assert!((r.origins[2].black - o[2].white - span * bm).abs() < 1e-9);
    }

    #[test]
    fn roll_process_modes() {
        let frames = [
            frame([40.0, 60.0, 80.0], [200.0, 210.0, 220.0]),
            frame([50.0, 70.0, 90.0], [190.0, 200.0, 210.0]),
        ];
        let (agg, _) = aggregate_roll(&frames);
        let roll = RollData {
            id: "r".into(),
            origins: agg.origins,
            base: [1.0; 3],
            frames: 2,
            excluded: vec![],
            auto_wb: Some(agg.auto_wb),
            multipliers: Some(agg.multipliers),
        };
        let mut s = Settings::default();
        s.use_roll = true;
        let get = |s: &Settings| origins_for(s, Some(&frames[1]), Some(&roll)).unwrap();
        s.roll_mode = RollMode::DarkroomPaper;
        assert_eq!(get(&s), FrameOrigins::from(agg.origins));
        s.roll_mode = RollMode::ProLab;
        let pro = get(&s);
        assert_eq!(pro.roll_clip, RollClip::default());
        assert_eq!(pro.origins[0].white, 50.0);
        s.roll_mode = RollMode::BasicLab;
        let basic = get(&s);
        assert_eq!(basic.origins, pro.origins);
        assert_eq!(basic.roll_clip.white, (40.0 - 50.0) / 2.0);
        assert_eq!(basic.roll_clip.black, (190.0 - 200.0) / 2.0);
        s.use_roll = false;
        assert_eq!(get(&s), FrameOrigins::from(frames[1].origins));
    }

    /// A synthetic colour negative (orange mask, per-channel H&D gamma, so
    /// the plain inversion has a colour cast) of a mostly neutral scene.
    fn synthetic_negative() -> (SourceImage, Vec<usize>) {
        let (w, h) = (300usize, 200usize);
        let base = [0.92f32, 0.62, 0.38];
        let gamma = [0.58f32, 0.66, 0.74];
        let mut data = vec![[0.0f32; 3]; w * h];
        let mut grey = Vec::new();
        for y in 0..h {
            for x in 0..w {
                let border = x < 10 || x >= w - 10 || y < 10 || y >= h - 10;
                let t = (x - 10.min(x)) as f32 / (w - 20) as f32;
                let scene: Option<[f32; 3]> = if border {
                    None
                } else if y < 100 {
                    Some([t * t * 0.8 + 0.02; 3])
                } else if y < 150 {
                    Some(match (x * 4) / w {
                        0 => [0.45, 0.32, 0.25],
                        1 => [0.12, 0.25, 0.55],
                        2 => [0.15, 0.35, 0.10],
                        _ => [0.60, 0.10, 0.08],
                    })
                } else {
                    Some([0.18; 3])
                };
                if !border && y >= 160 && y < 180 && x > 40 && x < w - 40 {
                    grey.push(y * w + x);
                }
                data[y * w + x] = match scene {
                    None => base,
                    Some(e) => std::array::from_fn(|c| {
                        base[c] * ((e[c] + 0.004) / 0.004).powf(-gamma[c] * 0.42)
                    }),
                };
            }
        }
        let src = SourceImage {
            width: w,
            height: h,
            data,
            to_melissa: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            info: Default::default(),
            as_shot_wb: [1.0; 3],
            is_raw: false,
        };
        (src, grey)
    }

    fn cast(px: &[[f32; 3]], idx: &[usize]) -> f32 {
        let mut m = [0.0f32; 3];
        for &i in idx {
            for c in 0..3 {
                m[c] += px[i][c] / idx.len() as f32;
            }
        }
        (m[0] - m[1]).abs() + (m[2] - m[1]).abs()
    }

    #[test]
    fn auto_neutral_reduces_the_cast() {
        let (src, grey) = synthetic_negative();
        let mut s = Settings::default();
        s.lut = crate::engine::settings::Lut::None;
        let base = resolve_base(&src, &s, None);
        let neg = prepare_negative(&src, &s, base);
        let a = analyze(&neg, &s, base);
        let render = |s: &Settings| {
            Renderer::new(s, Some(&a.origins.into()), OutputSpace::Srgb).render(&neg, 0.0)
        };
        let mut plain = s.clone();
        plain.wb_temp = 0.0;
        plain.wb_tint = 0.0;
        let before = cast(&render(&plain), &grey);
        let (t, n) = wb_for_mode(WbMode::AutoNeutral, &a.auto_wb).unwrap();
        let mut wb = s.clone();
        wb.wb_temp = t;
        wb.wb_tint = n;
        let after = cast(&render(&wb), &grey);
        assert!(
            after < before,
            "auto WB {:?}: cast {before} -> {after}",
            (t, n)
        );
    }

    #[test]
    fn edit_only_skips_analysis() {
        let (src, _) = synthetic_negative();
        let mut s = Settings::default();
        s.image_type = ImageType::EditOnly;
        let base = resolve_base(&src, &s, None);
        let neg = prepare_negative(&src, &s, base);
        let a = analyze(&neg, &s, base);
        assert_eq!(a.origins, edit_only_origins());
        assert_eq!(a.auto_wb, AutoWb::default());
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

        assert_eq!(
            base_uncropped, base_cropped,
            "auto_base must look at the entire image even when cropped"
        );
        assert!((base_cropped[0] - 0.9).abs() < 1e-3);
    }
}
