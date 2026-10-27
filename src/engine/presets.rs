//! Preset tables ported from `NegativePresets.lua`.

use super::settings::{ColorModel, Source, WbMode};

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
        ColorModel::None | ColorModel::BlackAndWhite => Calibration::default(),
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

/// Static WB presets (film character) as (magenta, yellow) filtration in CC.
pub fn film_character(mode: WbMode) -> Option<(f32, f32)> {
    match mode {
        WbMode::Standard => Some((3.0, 5.5)),
        WbMode::Kodak => Some((2.0, 5.5)),
        WbMode::Fuji => Some((1.0, 5.5)),
        WbMode::CineD => Some((3.0, 5.5)),
        WbMode::CineT => Some((2.0, 7.5)),
        WbMode::None => Some((0.0, 0.0)),
        _ => None,
    }
}

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

/// Fractions of the image allowed to clip (white, black) per image type.
pub fn clip_thresholds(t: super::settings::ImageType) -> (f64, f64) {
    use super::settings::ImageType::*;
    match t {
        Negative => (1.0e-4, 5.0e-4),
        Positive => (1.0e-4, 1.0e-4),
        EditOnly => (0.0, 0.0),
    }
}
