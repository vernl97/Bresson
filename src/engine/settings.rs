//! Conversion / editing settings. Field names and defaults mirror
//! `NegativePresets.defaultSettings` from the Lightroom plugin.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

macro_rules! choice_enum {
    ($(#[$m:meta])* $name:ident { $($var:ident => $label:expr),+ $(,)? } default $def:ident) => {
        $(#[$m])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
        pub enum $name { $($var),+ }
        /// Unknown values (choices that no longer exist) load as the default.
        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
                let v = String::deserialize(d)?;
                Ok(match v.as_str() {
                    $(stringify!($var) => $name::$var,)+
                    _ => $name::default(),
                })
            }
        }
        impl $name {
            #[allow(dead_code)]
            pub const ALL: &'static [$name] = &[$($name::$var),+];
            #[allow(dead_code)]
            pub fn label(self) -> &'static str { match self { $($name::$var => $label),+ } }
        }
        impl Default for $name { fn default() -> Self { $name::$def } }
    };
}

choice_enum!(
    /// Scan source (Convert tab). Only affects defaults / instructions.
    Source {
        DigitalCamera => "Digital Camera",
        VueScanRaw => "VueScan / SilverFast DNG",
        TifScan => "TIF Scan (gamma 1.8 / 2.2)",
        LinearScan => "Linear Scan (gamma 1.0)",
    } default DigitalCamera
);

choice_enum!(
    /// What the frame is.
    ImageType {
        Negative => "Negative (C-41 / B&W)",
        Positive => "Positive (Slide)",
        EditOnly => "Edit Only (already positive)",
    } default Negative
);

choice_enum!(
    ColorModel {
        Basic => "Basic",
        Frontier => "Frontier",
        Noritsu => "Noritsu",
        BlackAndWhite => "B+W",
        None => "None",
    } default Basic
);

choice_enum!(
    /// How the orange mask / film base is neutralised before analysis.
    /// Mirrors sampling the border with Lightroom's WB eyedropper.
    FilmBase {
        Auto => "Auto (brightest area)",
        Sampled => "Sampled border",
        AsShot => "As shot",
        Off => "Off",
    } default Auto
);

choice_enum!(
    /// Tone curve the negative is printed with (the plugin's profiles).
    ToneProfile {
        Logarithmic => "Cinematic - Log",
        LogarithmicRich => "Cinematic - Rich",
        LogarithmicFlat => "Cinematic - Flat",
        Standard => "Print - Standard",
        AllSoft => "Print - Soft",
        AllHard => "Print - Hard",
        HighlightHard => "Print - Highlight Hard",
        HighlightSoft => "Print - Highlight Soft",
        ShadowHard => "Print - Shadow Hard",
        ShadowSoft => "Print - Shadow Soft",
        Linear => "Linear",
        LinearGamma => "Linear + Gamma",
        LinearDeep => "Linear + Deep",
        LinearFlat => "Linear + Flat",
    } default Standard
);

choice_enum!(
    /// The plugin's `film` setting (WB dropdown).
    WbMode {
        AutoNeutral => "Auto-Neutral",
        AutoWarm => "Auto-Warm",
        AutoCool => "Auto-Cool",
        AutoMix => "Auto-Mix",
        AutoAvg => "Auto-AVG",
        Standard => "Standard",
        Kodak => "Kodak",
        Fuji => "Fuji",
        CineT => "Cine-T",
        CineD => "Cine-D",
        None => "None",
        Custom => "Custom",
    } default AutoNeutral
);

choice_enum!(
    Lut {
        None => "None",
        Natural => "Natural",
        Frontier => "Frontier",
        Crystal => "Crystal",
        Pakon => "Pakon",
    } default Frontier
);

choice_enum!(
    Sharpening {
        LeaveAsSet => "Leave As Set",
        Off => "Off",
        Lab => "Lab",
        Scanner => "Scanner",
    } default LeaveAsSet
);

choice_enum!(
    /// How a roll's frames are printed. Both use the film's channel slopes
    /// from the roll; Lab sets each frame's colour and density from the
    /// frame, Darkroom prints the whole roll at one balance and exposure.
    RollMode {
        Lab => "Lab (per frame)",
        Darkroom => "Darkroom (one setting)",
    } default Lab
);

choice_enum!(
    /// Shape of the crop rectangle. Fixed ratios follow the frame's
    /// orientation (a 2:3 crop of a portrait frame is portrait).
    CropAspect {
        Original => "Original",
        Free => "Free",
        Square => "1:1",
        OneTwo => "1:2",
        TwoThree => "2:3",
        ThreeFour => "3:4",
        FourFive => "4:5",
        FiveSeven => "5:7",
        LetterSheet => "8.5:11",
        NineSixteen => "9:16",
    } default Original
);

impl CropAspect {
    /// Short side : long side, for the fixed ratios.
    pub fn short_long(self) -> Option<f32> {
        Some(match self {
            Self::Original | Self::Free => return None,
            Self::Square => 1.0,
            Self::OneTwo => 1.0 / 2.0,
            Self::TwoThree => 2.0 / 3.0,
            Self::ThreeFour => 3.0 / 4.0,
            Self::FourFive => 4.0 / 5.0,
            Self::FiveSeven => 5.0 / 7.0,
            Self::LetterSheet => 8.5 / 11.0,
            Self::NineSixteen => 9.0 / 16.0,
        })
    }

    /// Width : height of the crop for a frame of aspect `frame` (w / h), or
    /// `None` when the shape is free.
    pub fn ratio(self, frame: f32) -> Option<f32> {
        match self {
            Self::Free => None,
            Self::Original => Some(frame),
            _ => self
                .short_long()
                .map(|r| if frame >= 1.0 { 1.0 / r } else { r }),
        }
    }
}

choice_enum!(
    ExportFormat {
        Tiff => "TIF",
        Jpeg => "JPG",
    } default Tiff
);

choice_enum!(
    OutputSpace {
        Srgb => "sRGB",
        AdobeRgb => "AdobeRGB",
        ProPhoto => "ProPhotoRGB",
    } default Srgb
);

choice_enum!(
    Placement {
        Subfolder => "Subfolder",
        SameFolder => "Same folder (stack)",
    } default Subfolder
);

/// Version of the conversion engine; part of [`AnalysisKey`].
pub const ENGINE_VERSION: u32 = 6;

/// How a spot uses its source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum HealMode {
    /// A plain copy of the source, its tone and colour included.
    Clone,
    /// The source's texture, matched to the tone and colour around the spot.
    #[default]
    Heal,
}

/// One healed spot, in the source image before any geometry: `at` and `from` are
/// normalised to its width and height, `radius` to its long edge. So a spot keeps its
/// place through rotation and cropping, and at any decode size.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct HealSpot {
    /// The defect's centre.
    pub at: [f32; 2],
    /// The centre of the area copied over it.
    pub from: [f32; 2],
    pub radius: f32,
    pub mode: HealMode,
    /// 0..=100 %
    pub opacity: f32,
}

impl Default for HealSpot {
    fn default() -> Self {
        Self {
            at: [0.5; 2],
            from: [0.5; 2],
            radius: 0.006,
            mode: HealMode::Heal,
            opacity: 100.0,
        }
    }
}

/// One dodge & burn brush stroke, in the source image before any geometry like a
/// [`HealSpot`]: `points` normalised to its width and height, `radius` to its long edge.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct BurnStroke {
    /// The brush centre's path; a single point is one dab.
    pub points: Vec<[f32; 2]>,
    pub radius: f32,
    /// 0..=100: share of the radius that fades out.
    pub feather: f32,
    /// 1..=100: how much one dab covers; overlapping dabs build up to full.
    pub flow: f32,
    /// Print time in stops where the stroke covers fully: + dodges (lighter),
    /// − burns, as `print_exposure`.
    pub ev: f32,
    /// Takes away the strokes before it instead of adding exposure.
    pub erase: bool,
}

impl Default for BurnStroke {
    fn default() -> Self {
        Self {
            points: Vec::new(),
            radius: 0.03,
            feather: 50.0,
            flow: 50.0,
            ev: 0.5,
            erase: false,
        }
    }
}

/// Every parameter that influences how a frame is rendered.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    // ---- Convert tab -------------------------------------------------------
    pub source: Source,
    pub image_type: ImageType,
    pub color_model: ColorModel,
    /// Dye crosstalk matrix: a bundled film's id (`presets::crosstalk_profile`),
    /// or empty for the generic C-41 crosstalk.
    pub crosstalk: String,
    /// How much of the crosstalk correction applies, in percent (100 = all).
    pub crosstalk_strength: f32,
    pub film_base: FilmBase,
    /// Linear camera-space multipliers when `film_base == Sampled`.
    pub film_base_sample: [f32; 3],
    /// Detect the frame (crop and straighten) when converting an uncropped frame.
    pub auto_crop: bool,
    /// A frame of the bare light source (or clear film leader) shot like the
    /// negatives; divides out uneven illumination and lens falloff.
    pub flat_field: Option<PathBuf>,

    // ---- Geometry (Lightroom would provide these) ---------------------------
    /// Quarter turns clockwise.
    pub rotation: u8,
    pub flip_h: bool,
    /// Fine rotation (straighten) in degrees, clockwise.
    pub straighten: f32,
    /// Crop insets in percent of the (rotated) frame: left, top, right, bottom.
    pub crop: [f32; 4],
    /// Shape the crop (and auto-crop) keeps.
    pub crop_aspect: CropAspect,

    // ---- Tone ----------------------------------------------------------------
    pub tone_profile: ToneProfile,
    /// Darkroom print exposure in stops (EV): + = shorter print time, lighter.
    pub print_exposure: f32,
    pub brightness: f32,
    pub contrast: f32,
    /// "Lights"
    pub highlights: f32,
    /// "Darks"
    pub shadows: f32,
    pub whites: f32,
    pub blacks: f32,
    /// "Lab Glow"
    pub glow: f32,
    /// "Lab Fade"
    pub fade: f32,
    /// Chroma noise reduction (0..100): smooths the colour of grain and
    /// sensor noise on the prepared negative, leaving luma alone. Spatial,
    /// like the healed spots, so it stays out of [`AnalysisKey`].
    pub chroma_denoise: f32,

    // ---- Color ---------------------------------------------------------------
    pub wb_mode: WbMode,
    /// Yellow printing filtration from white balance, in CC (+ warmer).
    pub wb_temp: f32,
    /// Magenta printing filtration from white balance, in CC (+ magenta).
    pub wb_tint: f32,
    /// Lab correction as printing filtration in CC:
    /// + takes away red / green / blue.
    pub filtration: [f32; 3],
    /// Mids: red <-> cyan (`cyan`), green <-> magenta (`tint`), blue <-> yellow (`temp`).
    pub mid_cyan: f32,
    pub mid_tint: f32,
    pub mid_temp: f32,
    pub high_cyan: f32,
    pub high_tint: f32,
    pub high_temp: f32,
    pub high_range: f32,
    pub shadow_cyan: f32,
    pub shadow_tint: f32,
    pub shadow_temp: f32,
    pub shadow_range: f32,
    pub lut: Lut,
    /// 0..=200 %
    pub lut_strength: f32,
    /// 1..=10, 5 = neutral (`postSaturation`).
    pub post_saturation: u8,
    pub sharpening: Sharpening,

    // ---- Roll ----------------------------------------------------------------
    /// Use the roll's aggregated analysis instead of the frame's own.
    pub use_roll: bool,
    pub roll_mode: RollMode,

    // ---- Retouching ------------------------------------------------------------
    /// Healed spots, in the order they apply. Specks are too small to matter to the
    /// analysis, so this stays out of [`AnalysisKey`] and a spot needs no Convert.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub heal: Vec<HealSpot>,
    /// Dodge & burn strokes, in the order they apply: local print time, after
    /// the analysis, so it stays out of [`AnalysisKey`].
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub burn: Vec<BurnStroke>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            source: Source::DigitalCamera,
            image_type: ImageType::Negative,
            color_model: ColorModel::Basic,
            crosstalk: String::new(),
            crosstalk_strength: 100.0,
            film_base: FilmBase::Auto,
            film_base_sample: [1.0, 1.0, 1.0],
            auto_crop: false,
            flat_field: None,
            rotation: 0,
            flip_h: false,
            straighten: 0.0,
            crop: [0.0; 4],
            crop_aspect: CropAspect::Original,
            tone_profile: ToneProfile::Standard,
            print_exposure: 0.0,
            brightness: 0.0,
            contrast: 0.0,
            highlights: 0.0,
            shadows: 0.0,
            whites: 0.0,
            blacks: 0.0,
            glow: 0.0,
            fade: 0.0,
            chroma_denoise: 25.0,
            wb_mode: WbMode::AutoNeutral,
            wb_temp: 0.0,
            wb_tint: 0.0,
            filtration: [0.0; 3],
            mid_cyan: 0.0,
            mid_tint: 0.0,
            mid_temp: 0.0,
            high_cyan: 0.0,
            high_tint: 0.0,
            high_temp: 0.0,
            high_range: 5.0,
            shadow_cyan: 0.0,
            shadow_tint: 0.0,
            shadow_temp: 0.0,
            shadow_range: 5.0,
            lut: Lut::Frontier,
            lut_strength: 100.0,
            post_saturation: 5,
            sharpening: Sharpening::LeaveAsSet,
            use_roll: false,
            roll_mode: RollMode::Lab,
            heal: Vec::new(),
            burn: Vec::new(),
        }
    }
}

impl Settings {
    /// Defaults for the B+W colour model.
    pub fn apply_mono_defaults(&mut self) {
        self.tone_profile = ToneProfile::LinearGamma;
        self.wb_mode = WbMode::None;
        (self.wb_temp, self.wb_tint) = (0.0, 0.0);
    }

    /// Defaults for slides and already positive scans.
    pub fn apply_positive_defaults(&mut self) {
        self.tone_profile = ToneProfile::Linear;
        self.lut = Lut::None;
        self.wb_mode = WbMode::None;
        (self.wb_temp, self.wb_tint) = (0.0, 0.0);
    }

    /// Settings defaults when the B+W colour model is chosen (`defaultMonoSettings`).
    pub fn is_monochrome(&self) -> bool {
        self.color_model == ColorModel::BlackAndWhite
    }

    /// Fields that change the analysed negative; changing them requires re-analysis.
    pub fn analysis_key(&self) -> AnalysisKey {
        AnalysisKey {
            engine: ENGINE_VERSION,
            source: self.source,
            image_type: self.image_type,
            color_model: self.color_model,
            crosstalk: self.crosstalk.clone(),
            crosstalk_strength: (self.crosstalk_strength * 10.0).round() as i32,
            film_base: self.film_base,
            film_base_sample: self.film_base_sample.map(|v| (v * 10000.0).round() as i32),
            flat_field: self.flat_field.clone(),
        }
    }

    /// Restore the per-frame fields (capture, analysis inputs and geometry)
    /// from `k`, so applying defaults or resetting keeps the conversion.
    pub fn keep_frame_fields(&mut self, k: &Settings) {
        (
            self.source,
            self.image_type,
            self.color_model,
            self.film_base,
            self.film_base_sample,
            self.auto_crop,
            self.use_roll,
        ) = (
            k.source,
            k.image_type,
            k.color_model,
            k.film_base,
            k.film_base_sample,
            k.auto_crop,
            k.use_roll,
        );
        (self.rotation, self.flip_h, self.straighten, self.crop) =
            (k.rotation, k.flip_h, k.straighten, k.crop);
        self.crop_aspect = k.crop_aspect;
        self.heal = k.heal.clone();
        self.burn = k.burn.clone();
        self.flat_field = k.flat_field.clone();
        (self.crosstalk, self.crosstalk_strength) = (k.crosstalk.clone(), k.crosstalk_strength);
    }

    /// Copy only the adjustment parameters (used by paste/sync). Like the
    /// plugin's sync list, the conversion inputs (source, image type, colour
    /// model, crosstalk) stay with the frame, as do its film
    /// base, roll choice and geometry.
    pub fn copy_edit_from(&mut self, o: &Settings) {
        let keep = self.clone();
        *self = o.clone();
        self.keep_frame_fields(&keep);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AnalysisKey {
    /// Analyses from another engine version need a new Convert.
    #[serde(default)]
    pub engine: u32,
    /// The source selects the colour model's calibration, which the analysis sees.
    #[serde(default)]
    pub source: Source,
    pub image_type: ImageType,
    pub color_model: ColorModel,
    /// Crosstalk matrix and strength (tenths of a percent); sidecars from
    /// before them used the generic matrix at full strength.
    #[serde(default)]
    pub crosstalk: String,
    #[serde(default = "full_strength")]
    pub crosstalk_strength: i32,
    pub film_base: FilmBase,
    pub film_base_sample: [i32; 3],
    #[serde(default)]
    pub flat_field: Option<PathBuf>,
}

fn full_strength() -> i32 {
    1000
}

/// Positive copy settings (Advanced tab).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ExportSettings {
    pub format: ExportFormat,
    pub sixteen_bit: bool,
    pub zip_compression: bool,
    pub jpeg_quality: u8,
    pub space: OutputSpace,
    pub placement: Placement,
    pub subfolder: String,
    pub suffix: String,
    /// File name, see `export::file_name` for the tokens.
    pub name_template: String,
    /// 0 = full resolution.
    pub long_edge: u32,
    pub output_dir: Option<PathBuf>,
    /// Export sharpening; Leave As Set keeps the frame's own setting.
    pub sharpening: Sharpening,
    /// JPEG with an HDR gain map (`engine::gainmap`).
    pub hdr: bool,
}

impl Default for ExportSettings {
    fn default() -> Self {
        Self {
            format: ExportFormat::Jpeg,
            sixteen_bit: true,
            zip_compression: true,
            jpeg_quality: 95,
            space: OutputSpace::Srgb,
            placement: Placement::Subfolder,
            subfolder: "Positives".into(),
            suffix: "-positive".into(),
            name_template: "{name}{suffix}".into(),
            long_edge: 0,
            output_dir: None,
            sharpening: Sharpening::LeaveAsSet,
            hdr: false,
        }
    }
}

/// A named set of export settings. The destination is not part of it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportPreset {
    pub name: String,
    pub export: ExportSettings,
}

impl ExportPreset {
    /// Presets that ship with the app: lab-style JPEG sizes and an archive TIFF.
    pub fn builtin() -> Vec<ExportPreset> {
        let jpeg = |name: &str, long_edge| ExportPreset {
            name: name.into(),
            export: ExportSettings {
                format: ExportFormat::Jpeg,
                jpeg_quality: 95,
                space: OutputSpace::Srgb,
                long_edge,
                ..Default::default()
            },
        };
        vec![
            jpeg("Lab JPEG M (3000 px)", 3000),
            jpeg("Lab JPEG L (6000 px)", 6000),
            ExportPreset {
                name: "Archive TIFF (16-bit ProPhoto)".into(),
                export: ExportSettings {
                    format: ExportFormat::Tiff,
                    sixteen_bit: true,
                    zip_compression: true,
                    space: OutputSpace::ProPhoto,
                    ..Default::default()
                },
            },
        ]
    }

    fn without_destination(ex: &ExportSettings) -> ExportSettings {
        let d = ExportSettings::default();
        ExportSettings {
            placement: d.placement,
            subfolder: d.subfolder,
            output_dir: None,
            sharpening: d.sharpening,
            ..ex.clone()
        }
    }

    pub fn new(name: &str, ex: &ExportSettings) -> Self {
        Self {
            name: name.trim().into(),
            export: Self::without_destination(ex),
        }
    }

    pub fn matches(&self, ex: &ExportSettings) -> bool {
        Self::without_destination(ex) == self.export
    }

    /// Apply to `ex`, keeping its destination.
    pub fn apply_to(&self, ex: &mut ExportSettings) {
        *ex = ExportSettings {
            placement: ex.placement,
            subfolder: ex.subfolder.clone(),
            output_dir: ex.output_dir.clone(),
            sharpening: ex.sharpening,
            ..self.export.clone()
        };
    }
}

/// Analogue capture metadata written into positive copies (ExifV3.lua).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct FilmMetadata {
    pub camera_make: String,
    pub camera_model: String,
    pub lens: String,
    pub film_stock: String,
    pub film_iso: String,
    pub shot_at_iso: String,
    pub shutter_speed: String,
    pub aperture: String,
    pub developer: String,
    pub development_notes: String,
}

impl FilmMetadata {}

/// Folds the retired White point / Black point of stored settings JSON into
/// Whites and Blacks. Both pairs are straight lines on the normalised tone,
/// so the print stays exactly the same. Glow and Fade add to Whites and Blacks
/// in the curve and are taken back out here. The result can lie beyond the
/// sliders' travel, which is kept until the slider is moved.
pub fn fold_point_sliders(settings: &mut serde_json::Value) {
    let Some(o) = settings.as_object_mut() else {
        return;
    };
    let mut take = |k: &str| o.remove(k).and_then(|v| v.as_f64()).unwrap_or(0.0);
    let (wc, bc) = (take("white_clip"), take("black_clip"));
    if wc == 0.0 && bc == 0.0 {
        return;
    }
    let get = |k: &str| o.get(k).and_then(|v| v.as_f64()).unwrap_or(0.0);
    let (glow, fade) = (get("glow"), get("fade"));
    let (whites, blacks) = (get("whites") - glow, get("blacks") + fade);
    let (b, w) = (bc * 0.006, wc * 0.006);
    let d = (1.0 - b - w).max(1e-3);
    // The old curve: x -> (x - b) / d, then slope * x + 0.004 * blacks.
    let slope = 1.0 + whites * 0.006 - blacks * 0.004;
    let new_blacks = (0.004 * blacks - slope * b / d) / 0.004;
    let new_whites = (slope / d - 1.0 + 0.004 * new_blacks) / 0.006;
    o.insert("whites".into(), (new_whites + glow).into());
    o.insert("blacks".into(), (new_blacks - fade).into());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paste_keeps_conversion_inputs() {
        let mut dst = Settings {
            source: Source::VueScanRaw,
            color_model: ColorModel::Noritsu,
            crop: [1.0, 2.0, 3.0, 4.0],
            ..Settings::default()
        };
        let before = dst.analysis_key();
        let src = Settings {
            brightness: 12.0,
            color_model: ColorModel::Frontier,
            ..Settings::default()
        };
        dst.copy_edit_from(&src);
        assert_eq!(dst.brightness, 12.0);
        assert_eq!(dst.analysis_key(), before);
        assert_eq!(dst.crop, [1.0, 2.0, 3.0, 4.0]);
    }

    #[test]
    fn export_presets_keep_the_destination() {
        let mut ex = ExportSettings {
            output_dir: Some("/out".into()),
            ..Default::default()
        };
        let l = &ExportPreset::builtin()[1];
        assert!(!l.matches(&ex));
        l.apply_to(&mut ex);
        assert!(l.matches(&ex));
        assert_eq!(ex.long_edge, 6000);
        assert_eq!(ex.output_dir, Some("/out".into()));
        ex.jpeg_quality = 80;
        let mine = ExportPreset::new(" Web ", &ex);
        assert_eq!(mine.name, "Web");
        assert_eq!(mine.export.output_dir, None);
        assert!(mine.matches(&ex) && !l.matches(&ex));
    }

    #[test]
    fn crop_and_geometry_do_not_change_analysis_key() {
        let mut s = Settings::default();
        let initial_key = s.analysis_key();

        s.crop = [5.0, 10.0, 5.0, 10.0];
        assert_eq!(
            s.analysis_key(),
            initial_key,
            "crop must not alter analysis_key"
        );

        s.straighten = 2.5;
        assert_eq!(
            s.analysis_key(),
            initial_key,
            "straighten must not alter analysis_key"
        );

        s.rotation = 1;
        assert_eq!(
            s.analysis_key(),
            initial_key,
            "rotation must not alter analysis_key"
        );

        s.flip_h = true;
        assert_eq!(
            s.analysis_key(),
            initial_key,
            "flip_h must not alter analysis_key"
        );

        // Convert settings MUST alter analysis_key
        s.color_model = ColorModel::Frontier;
        assert_ne!(
            s.analysis_key(),
            initial_key,
            "color_model must alter analysis_key"
        );
    }

    /// The retired point sliders print the same through Whites and Blacks.
    #[test]
    fn point_sliders_fold_into_whites_and_blacks() {
        let line = |x: f64, whites: f64, blacks: f64| {
            x * (1.0 + whites * 0.006) + blacks * 0.004 * (1.0 - x)
        };
        for (wc, bc, whites, blacks, glow, fade) in [
            (50.0, 0.0, 0.0, 0.0, 0.0, 0.0),
            (-50.0, 0.0, 0.0, 0.0, 0.0, 0.0),
            (0.0, 50.0, 0.0, 0.0, 0.0, 0.0),
            (0.0, -50.0, 0.0, 0.0, 0.0, 0.0),
            (20.0, -30.0, 15.0, -10.0, 25.0, -40.0),
        ] {
            let mut v = serde_json::json!({
                "white_clip": wc, "black_clip": bc, "whites": whites,
                "blacks": blacks, "glow": glow, "fade": fade,
            });
            fold_point_sliders(&mut v);
            assert!(v.get("white_clip").is_none() && v.get("black_clip").is_none());
            let s: Settings = serde_json::from_value(v).unwrap();
            for i in 0..=10 {
                let x = i as f64 / 10.0;
                let (b, w) = (bc * 0.006, wc * 0.006);
                let clipped = (x - b) / (1.0 - b - w);
                let old = line(clipped, whites - glow, blacks + fade);
                let new = line(x, s.whites as f64 - glow, s.blacks as f64 + fade);
                assert!((old - new).abs() < 1e-4, "{wc} {bc} at {x}: {old} {new}");
            }
        }
    }

    /// Settings without the retired sliders are left alone.
    #[test]
    fn folding_leaves_current_settings_alone() {
        let mut v = serde_json::json!({ "whites": 12.5, "blacks": -7.0 });
        let before = v.clone();
        fold_point_sliders(&mut v);
        assert_eq!(v, before);
    }

    #[test]
    fn unknown_choices_load_as_default() {
        let s: Settings =
            serde_json::from_str(r#"{"tone_profile": "Rich", "contrast": 7}"#).unwrap();
        assert_eq!(s.tone_profile, ToneProfile::default());
        assert_eq!(s.contrast, 7.0);
    }

    #[test]
    fn analysis_key_deserializes_with_legacy_crop_fields() {
        let json = r#"{
            "source": "DigitalCamera",
            "image_type": "Negative",
            "color_model": "Basic",
            "pre_saturation": 3,
            "border_buffer": 500,
            "film_base": "Auto",
            "film_base_sample": [0, 0, 0],
            "rotation": 0,
            "flip_h": false,
            "straighten": 0,
            "crop": [0, 0, 0, 0]
        }"#;
        let key: Result<AnalysisKey, _> = serde_json::from_str(json);
        assert!(
            key.is_ok(),
            "legacy sidecar with crop fields must deserialize cleanly"
        );
    }
}
