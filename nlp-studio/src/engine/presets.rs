//! Preset tables ported from `NegativePresets.lua`.

use super::settings::{ColorModel, Source, ToneProfile, WbMode};

/// One entry of `NegativePresets.toneProfiles` / `toneProfiles_v3`.
#[derive(Debug, Clone, Copy)]
pub struct ToneProfileDef {
    pub auto_tone: bool,
    pub default_blacks: f64,
    pub default_whites: f64,
    pub default_shadows: f64,
    pub default_highlights: f64,
    pub default_gamma: f64,
    pub default_contrast: f64,
    pub default_soft_high: f64,
    pub default_soft_low: f64,
}

const fn tp(
    auto_tone: bool,
    blacks: f64,
    whites: f64,
    shadows: f64,
    highlights: f64,
    gamma: f64,
    contrast: f64,
    soft_high: f64,
    soft_low: f64,
) -> ToneProfileDef {
    ToneProfileDef {
        auto_tone,
        default_blacks: blacks,
        default_whites: whites,
        default_shadows: shadows,
        default_highlights: highlights,
        default_gamma: gamma,
        default_contrast: contrast,
        default_soft_high: soft_high,
        default_soft_low: soft_low,
    }
}

/// `NegativePresets.toneProfiles` — used for the tone layers and linear clipping.
pub fn tone_profile(p: ToneProfile) -> ToneProfileDef {
    use ToneProfile::*;
    //                       auto   blacks whites shadows highs gamma contr  sHigh  sLow
    match p {
        AllHard => tp(true, 2.0, -2.0, 0.0, 0.0, 1.0, 25.0, 0.0, 0.0),
        AllSoft => tp(true, 10.0, -10.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0),
        HighlightHard => tp(true, 2.0, -2.0, 0.0, 10.0, 1.0, 10.0, 0.0, 0.0),
        HighlightSoft => tp(true, 2.0, -10.0, -10.0, 0.0, 1.0, 0.0, 0.0, 0.0),
        Linear => tp(false, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, -3.0, -3.0),
        LinearDeep => tp(false, 0.0, 0.0, -12.0, 0.0, 1.0, 0.0, -3.0, -3.0),
        LinearFlat => tp(false, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, -15.0, -10.0),
        LinearGamma => tp(false, 0.0, 0.0, 0.0, 0.0, 0.66, 0.0, -3.0, -3.0),
        Logarithmic => tp(false, -10.0, 0.0, -10.0, -25.0, 1.0, 0.0, -3.0, -3.0),
        LogarithmicFlat => tp(false, -10.0, 0.0, -10.0, -25.0, 1.0, 0.0, -9.0, -6.0),
        LogarithmicRich => tp(false, -20.0, 0.0, -10.0, -25.0, 1.0, 5.0, -3.0, -3.0),
        ShadowHard => tp(true, 2.0, -2.0, -10.0, 0.0, 1.0, 10.0, 0.0, 0.0),
        ShadowSoft => tp(true, 9.0, -2.0, 0.0, 10.0, 1.0, 0.0, 0.0, 0.0),
        Standard => tp(true, 2.0, -2.0, 0.0, 0.0, 1.0, 10.0, 0.0, 0.0),
    }
}

/// `NegativePresets.toneProfiles_v3` — only its soft high/low defaults are
/// consumed (by colour-protected clipping).
pub fn tone_profile_v3(p: ToneProfile) -> ToneProfileDef {
    use ToneProfile::*;
    let mut d = tone_profile(p);
    let (sh, sl) = match p {
        Linear => (-3.0, 0.0),
        LinearDeep => (-2.0, 0.0),
        LinearFlat => (-14.0, -4.0),
        LinearGamma => (-2.0, 0.0),
        Logarithmic => (-2.0, 0.0),
        LogarithmicFlat => (-8.0, -4.0),
        LogarithmicRich => (-2.0, 0.0),
        _ => (d.default_soft_high, d.default_soft_low),
    };
    d.default_soft_high = sh;
    d.default_soft_low = sl;
    d
}

/// Camera-calibration style adjustments of a colour model
/// (`colorModels[*].lightroomSettings`). Values use Lightroom's −100..100 scale.
#[derive(Debug, Clone, Copy, Default)]
pub struct Calibration {
    pub red_hue: f32,
    pub red_sat: f32,
    pub green_hue: f32,
    pub green_sat: f32,
    pub blue_hue: f32,
    pub blue_sat: f32,
    pub grayscale: bool,
}

pub fn calibration(model: ColorModel, source: Source) -> Calibration {
    let camera = source == Source::DigitalCamera;
    let c = |rh, rs, bh| Calibration {
        red_hue: rh,
        red_sat: rs,
        blue_hue: bh,
        ..Default::default()
    };
    match model {
        ColorModel::None => Calibration::default(),
        ColorModel::BlackAndWhite => Calibration {
            grayscale: true,
            ..Default::default()
        },
        ColorModel::Basic if camera => c(0.0, 0.0, -10.0),
        ColorModel::Basic => Calibration::default(),
        ColorModel::Frontier if camera => c(15.0, -10.0, -15.0),
        ColorModel::Frontier => c(5.0, -5.0, -15.0),
        ColorModel::Noritsu if camera => c(15.0, -4.0, -10.0),
        ColorModel::Noritsu => c(0.0, 0.0, -10.0),
    }
}

/// Lightroom `Saturation` applied to the negative for each pre-saturation step.
pub fn pre_saturation(level: u8) -> f32 {
    match level {
        0 | 1 => -20.0,
        2 => -10.0,
        3 => 0.0,
        4 => 5.0,
        _ => 8.0,
    }
}

/// `NegativePresets.filmCharacter` — static WB presets as (magenta, yellow).
pub fn film_character(mode: WbMode) -> Option<(f32, f32)> {
    match mode {
        WbMode::Standard => Some((10.0, 17.0)),
        WbMode::Kodak => Some((7.0, 17.0)),
        WbMode::Fuji => Some((4.0, 17.0)),
        WbMode::CineD => Some((10.0, 17.0)),
        WbMode::CineT => Some((7.0, 24.0)),
        WbMode::None => Some((0.0, 0.0)),
        _ => None,
    }
}

/// HSB zone masks used for the "smart colour" white balance analysis
/// (NLPImageProcessorV3). Hue/sat/lum thresholds in percent, as ImageMagick uses them.
#[derive(Debug, Clone, Copy)]
pub struct WbZone {
    pub hue_low: f32,
    pub hue_high: f32,
    pub invert_hue: bool,
    pub sat_threshold: f32,
    pub lum_low: f32,
    pub lum_high: f32,
}

pub const ZONE_NEUTRAL: WbZone = WbZone {
    hue_low: 0.0,
    hue_high: 100.0,
    invert_hue: false,
    sat_threshold: 70.0,
    lum_low: 0.0,
    lum_high: 100.0,
};
pub const ZONE_WARM: WbZone = WbZone {
    hue_low: 40.28,
    hue_high: 75.0,
    invert_hue: false,
    sat_threshold: 60.0,
    lum_low: 25.0,
    lum_high: 75.0,
};
pub const ZONE_COOL: WbZone = WbZone {
    hue_low: 25.0,
    hue_high: 75.0,
    invert_hue: true,
    sat_threshold: 70.0,
    lum_low: 25.0,
    lum_high: 75.0,
};

/// Unsharp-mask parameters emulating the plugin's sharpening presets
/// (Lightroom Sharpness / Radius / Detail / Masking).
#[derive(Debug, Clone, Copy)]
pub struct SharpenDef {
    pub amount: f32,
    pub radius: f32,
    /// Edge threshold (0..1 of luminance) — larger suppresses grain.
    pub threshold: f32,
    /// Lightroom `Sharpness` of the preset (cap for size scaling is 150).
    pub lr_sharpness: f32,
}

pub fn sharpening(s: super::settings::Sharpening) -> Option<SharpenDef> {
    use super::settings::Sharpening::*;
    match s {
        LeaveAsSet | Off => None,
        Lab => Some(SharpenDef {
            amount: 0.9,
            radius: 1.3,
            threshold: 0.012,
            lr_sharpness: 100.0,
        }),
        Scanner => Some(SharpenDef {
            amount: 0.6,
            radius: 1.0,
            threshold: 0.0,
            lr_sharpness: 50.0,
        }),
    }
}

/// `_G.tempMultiplier`
pub const TEMP_MULTIPLIER: f64 = 0.75;

/// CDF clipping thresholds (white, black) per image type (`L39_1` in NLPImageProcessorV3).
pub fn clip_thresholds(t: super::settings::ImageType) -> (f64, f64) {
    use super::settings::ImageType::*;
    match t {
        Negative => (1.0e-4, 5.0e-4),
        Positive => (1.0e-4, 1.0e-4),
        EditOnly => (0.0, 0.0),
    }
}
