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

/// A film's dye crosstalk correction, applied to the negative's density
/// (rows: output R, G, B; columns: input R, G, B; each row sums to 1).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Crosstalk {
    /// The file name in `crosstalk/`, stored in `Settings::crosstalk`.
    pub id: &'static str,
    pub name: &'static str,
    pub matrix: [[f32; 3]; 3],
}

/// The bundled matrices (`crosstalk/*.toml`, from NegPy's gallery: read
/// off published spectral-dye-density sheets for a narrowband scanner).
const CROSSTALK_FILES: &[(&str, &str)] = &[
    (
        "fujifilm_100",
        include_str!("../../crosstalk/fujifilm_100.toml"),
    ),
    (
        "fujifilm_200",
        include_str!("../../crosstalk/fujifilm_200.toml"),
    ),
    (
        "fujifilm_400",
        include_str!("../../crosstalk/fujifilm_400.toml"),
    ),
    (
        "fujifilm_c200",
        include_str!("../../crosstalk/fujifilm_c200.toml"),
    ),
    (
        "fujifilm_natura_1600",
        include_str!("../../crosstalk/fujifilm_natura_1600.toml"),
    ),
    (
        "fujifilm_pro_400h",
        include_str!("../../crosstalk/fujifilm_pro_400h.toml"),
    ),
    (
        "fujifilm_superia_premium_400",
        include_str!("../../crosstalk/fujifilm_superia_premium_400.toml"),
    ),
    (
        "fujifilm_superia_xtra_400",
        include_str!("../../crosstalk/fujifilm_superia_xtra_400.toml"),
    ),
    (
        "harman_phoenix_ii",
        include_str!("../../crosstalk/harman_phoenix_ii.toml"),
    ),
    (
        "kodak_aerocolor_iv_2460",
        include_str!("../../crosstalk/kodak_aerocolor_iv_2460.toml"),
    ),
    (
        "kodak_ektacolor_pro_160",
        include_str!("../../crosstalk/kodak_ektacolor_pro_160.toml"),
    ),
    (
        "kodak_ektacolor_pro_400",
        include_str!("../../crosstalk/kodak_ektacolor_pro_400.toml"),
    ),
    (
        "kodak_ektacolor_pro_800",
        include_str!("../../crosstalk/kodak_ektacolor_pro_800.toml"),
    ),
    (
        "kodak_ektar_100",
        include_str!("../../crosstalk/kodak_ektar_100.toml"),
    ),
    (
        "kodak_gold_200",
        include_str!("../../crosstalk/kodak_gold_200.toml"),
    ),
    (
        "kodak_pro_image_100",
        include_str!("../../crosstalk/kodak_pro_image_100.toml"),
    ),
    (
        "kodak_ultramax_400",
        include_str!("../../crosstalk/kodak_ultramax_400.toml"),
    ),
    (
        "kodak_verita_200d",
        include_str!("../../crosstalk/kodak_verita_200d.toml"),
    ),
    (
        "kodak_vision3_200t",
        include_str!("../../crosstalk/kodak_vision3_200t.toml"),
    ),
    (
        "kodak_vision3_250d",
        include_str!("../../crosstalk/kodak_vision3_250d.toml"),
    ),
    (
        "kodak_vision3_500t",
        include_str!("../../crosstalk/kodak_vision3_500t.toml"),
    ),
    (
        "kodak_vision3_50d",
        include_str!("../../crosstalk/kodak_vision3_50d.toml"),
    ),
];

/// The bundled crosstalk matrices, sorted by name.
pub fn crosstalk_profiles() -> &'static [Crosstalk] {
    static ALL: std::sync::OnceLock<Vec<Crosstalk>> = std::sync::OnceLock::new();
    ALL.get_or_init(|| {
        let mut v: Vec<Crosstalk> = CROSSTALK_FILES
            .iter()
            .filter_map(|&(id, text)| parse_crosstalk(id, text))
            .collect();
        v.sort_by(|a, b| a.name.cmp(b.name));
        v
    })
}

/// The bundled matrix with this id; `None` for the generic C-41 crosstalk.
pub fn crosstalk_profile(id: &str) -> Option<&'static Crosstalk> {
    crosstalk_profiles().iter().find(|p| p.id == id)
}

/// Read a NegPy crosstalk file: `name = "…"` and the nine numbers of
/// `matrix = [[…], […], […]]`. Rows are normalised to sum 1, as NegPy does.
fn parse_crosstalk(id: &'static str, text: &'static str) -> Option<Crosstalk> {
    let line = |key: &str| {
        text.lines()
            .map(str::trim)
            .find(|l| l.starts_with(key) && l[key.len()..].trim_start().starts_with('='))
    };
    let name = line("name").and_then(|l| l.split('"').nth(1)).unwrap_or(id);
    let start = text.find(line("matrix")?)?;
    let nums: Vec<f32> = text[start + "matrix".len()..]
        .lines()
        .map(|l| l.split('#').next().unwrap_or(""))
        .flat_map(|l| {
            l.split(|c: char| !(c.is_ascii_digit() || matches!(c, '.' | '-' | '+' | 'e' | 'E')))
        })
        .filter_map(|t| t.parse().ok())
        .take(9)
        .collect();
    if nums.len() != 9 {
        return None;
    }
    let matrix = std::array::from_fn(|r| {
        let row: [f32; 3] = std::array::from_fn(|c| nums[r * 3 + c]);
        let sum: f32 = row.iter().sum();
        if sum.abs() > 1e-6 {
            row.map(|v| v / sum)
        } else {
            row
        }
    });
    Some(Crosstalk { id, name, matrix })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_bundled_crosstalk_matrix_loads() {
        assert_eq!(crosstalk_profiles().len(), CROSSTALK_FILES.len());
        let gold = crosstalk_profile("kodak_gold_200").unwrap();
        assert_eq!(gold.name, "Kodak Gold 200 (approx)");
        // Off-diagonals unmix: they are small, and rows keep neutrals neutral.
        for p in crosstalk_profiles() {
            for (r, row) in p.matrix.iter().enumerate() {
                assert!((row.iter().sum::<f32>() - 1.0).abs() < 1e-5, "{}", p.id);
                assert!(row[r] > 0.8 && row[r] < 1.4, "{} row {r}", p.id);
            }
        }
    }
}
