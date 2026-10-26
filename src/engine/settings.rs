//! Conversion / editing settings. Field names and defaults mirror
//! `NegativePresets.defaultSettings` from the Lightroom plugin.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

macro_rules! choice_enum {
    ($(#[$m:meta])* $name:ident { $($var:ident => $label:expr),+ $(,)? } default $def:ident) => {
        $(#[$m])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub enum $name { $($var),+ }
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
    ToneProfile {
        Logarithmic => "Cinematic - Log",
        LogarithmicRich => "Cinematic - Rich",
        LogarithmicFlat => "Cinematic - Flat",
        Standard => "LAB - Standard",
        AllSoft => "LAB - Soft",
        AllHard => "LAB - Hard",
        HighlightHard => "LAB - Highlight Hard",
        HighlightSoft => "LAB - Highlight Soft",
        ShadowHard => "LAB - Shadow Hard",
        ShadowSoft => "LAB - Shadow Soft",
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
    EnginePreset {
        V3_1 => "v3.1 (multi-pass)",
        V3_0 => "v3.0",
        V2_3 => "v2.3",
        V2_2 => "v2.2",
        V2_1 => "v2.1",
        Custom => "Custom",
    } default V3_1
);

choice_enum!(
    CurvePoints {
        Auto => "Auto",
        Smooth => "Smooth",
        Precise => "Precise",
        Manual => "Manual",
    } default Auto
);

choice_enum!(
    CorrectionType {
        SinglePass => "Single Pass",
        MultiPass => "Multi Pass",
    } default MultiPass
);

choice_enum!(
    LayerOrder {
        ColorFirst => "Colors First",
        TonesFirst => "Tones First",
    } default ColorFirst
);

choice_enum!(
    WbTonality {
        AddDensity => "Add Density",
        NeutralDensity => "Neutral Density",
        SubtractDensity => "Subtract Density",
    } default AddDensity
);

choice_enum!(
    WbMethod {
        LinearFixed => "Linear Fixed",
        LinearDynamic => "Linear Dynamic",
        MidtoneWeighted => "Midtone Weighted",
        HighlightWeighted => "Highlight Weighted",
        ShadowWeighted => "Shadow Weighted",
    } default ShadowWeighted
);

choice_enum!(
    ToningMethod {
        Standard => "Standard",
        Vintage => "Vintage",
    } default Standard
);

choice_enum!(
    ClipMethod {
        Linear => "Linear Clipping",
        ColorProtected => "Color Protected",
    } default ColorProtected
);

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

/// Every parameter that influences how a frame is rendered.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    // ---- Convert tab -------------------------------------------------------
    pub source: Source,
    pub image_type: ImageType,
    pub color_model: ColorModel,
    /// 1..=5, "3 - Default".
    pub pre_saturation: u8,
    /// Border buffer in percent (`filmBorder`).
    pub border_buffer: f32,
    pub film_base: FilmBase,
    /// Linear camera-space multipliers when `film_base == Sampled`.
    pub film_base_sample: [f32; 3],

    // ---- Geometry (Lightroom would provide these) ---------------------------
    /// Quarter turns clockwise.
    pub rotation: u8,
    pub flip_h: bool,
    /// Fine rotation (straighten) in degrees, clockwise.
    pub straighten: f32,
    /// Crop insets in percent of the (rotated) frame: left, top, right, bottom.
    pub crop: [f32; 4],

    // ---- Tone ----------------------------------------------------------------
    pub tone_profile: ToneProfile,
    /// The plugin's Exposure (curve units). Superseded by `print_exposure`;
    /// old values are migrated when a sidecar is loaded.
    pub exposure: f32,
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
    pub soft_highlights: bool,
    pub soft_shadows: bool,
    /// "WhiteClip" (`softHigh` internally).
    pub white_clip: f32,
    /// "BlackClip" (`softLow` internally).
    pub black_clip: f32,

    // ---- Color ---------------------------------------------------------------
    pub wb_mode: WbMode,
    pub wb_temp: f32,
    pub wb_tint: f32,
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

    // ---- Advanced / engine ---------------------------------------------------
    pub engine: EnginePreset,
    pub curve_points: CurvePoints,
    pub curve_resolution: u8,
    pub correction_type: CorrectionType,
    pub layer_order: LayerOrder,
    pub wb_tonality: WbTonality,
    pub wb_method: WbMethod,
    pub toning_method: ToningMethod,
    pub clip_method: ClipMethod,
    /// Use the roll's aggregated analysis instead of the frame's own.
    pub use_roll: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            source: Source::DigitalCamera,
            image_type: ImageType::Negative,
            color_model: ColorModel::Basic,
            pre_saturation: 3,
            border_buffer: 5.0,
            film_base: FilmBase::Auto,
            film_base_sample: [1.0, 1.0, 1.0],
            rotation: 0,
            flip_h: false,
            straighten: 0.0,
            crop: [0.0; 4],
            tone_profile: ToneProfile::Standard,
            exposure: 0.0,
            print_exposure: 0.0,
            brightness: 0.0,
            contrast: 0.0,
            highlights: 0.0,
            shadows: 0.0,
            whites: 0.0,
            blacks: 0.0,
            glow: 0.0,
            fade: 0.0,
            soft_highlights: false,
            soft_shadows: false,
            white_clip: 0.0,
            black_clip: 0.0,
            wb_mode: WbMode::AutoNeutral,
            wb_temp: 0.0,
            wb_tint: 0.0,
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
            engine: EnginePreset::V3_1,
            curve_points: CurvePoints::Auto,
            curve_resolution: 7,
            correction_type: CorrectionType::MultiPass,
            layer_order: LayerOrder::ColorFirst,
            wb_tonality: WbTonality::AddDensity,
            wb_method: WbMethod::ShadowWeighted,
            toning_method: ToningMethod::Standard,
            clip_method: ClipMethod::ColorProtected,
            use_roll: false,
        }
    }
}

impl Settings {
    /// Apply one of the plugin's engine presets (`NegativePresets.engine`).
    pub fn apply_engine_preset(&mut self, preset: EnginePreset) {
        use CorrectionType::*;
        self.engine = preset;
        let (pts, corr, order, ton, meth, clip) = match preset {
            EnginePreset::V3_1 => (
                CurvePoints::Auto,
                MultiPass,
                LayerOrder::ColorFirst,
                WbTonality::AddDensity,
                WbMethod::ShadowWeighted,
                ClipMethod::ColorProtected,
            ),
            EnginePreset::V3_0 => (
                CurvePoints::Auto,
                SinglePass,
                LayerOrder::ColorFirst,
                WbTonality::AddDensity,
                WbMethod::ShadowWeighted,
                ClipMethod::ColorProtected,
            ),
            EnginePreset::V2_3 => (
                CurvePoints::Auto,
                SinglePass,
                LayerOrder::ColorFirst,
                WbTonality::AddDensity,
                WbMethod::LinearFixed,
                ClipMethod::Linear,
            ),
            EnginePreset::V2_2 => (
                CurvePoints::Precise,
                SinglePass,
                LayerOrder::ColorFirst,
                WbTonality::AddDensity,
                WbMethod::LinearFixed,
                ClipMethod::Linear,
            ),
            EnginePreset::V2_1 => {
                self.curve_resolution = 4;
                (
                    CurvePoints::Smooth,
                    SinglePass,
                    LayerOrder::TonesFirst,
                    WbTonality::NeutralDensity,
                    WbMethod::LinearFixed,
                    ClipMethod::Linear,
                )
            }
            EnginePreset::Custom => return,
        };
        self.curve_points = pts;
        self.correction_type = corr;
        self.layer_order = order;
        self.wb_tonality = ton;
        self.wb_method = meth;
        self.clip_method = clip;
    }

    /// Apply `defaultMonoSettings` (B+W colour model).
    pub fn apply_mono_defaults(&mut self) {
        self.tone_profile = ToneProfile::LinearGamma;
        self.wb_mode = WbMode::None;
        (self.wb_temp, self.wb_tint) = (0.0, 0.0);
        self.white_clip = -5.0;
        self.black_clip = -5.0;
        self.correction_type = CorrectionType::SinglePass;
    }

    /// Apply `defaultPositiveSettings` (slides / already positive scans).
    pub fn apply_positive_defaults(&mut self) {
        self.tone_profile = ToneProfile::Linear;
        self.wb_mode = WbMode::None;
        (self.wb_temp, self.wb_tint) = (0.0, 0.0);
        self.correction_type = CorrectionType::SinglePass;
    }

    /// Settings defaults when the B+W colour model is chosen (`defaultMonoSettings`).
    pub fn is_monochrome(&self) -> bool {
        self.color_model == ColorModel::BlackAndWhite
    }

    /// Fields that change the analysed negative; changing them requires re-analysis.
    pub fn analysis_key(&self) -> AnalysisKey {
        AnalysisKey {
            source: self.source,
            image_type: self.image_type,
            color_model: self.color_model,
            pre_saturation: self.pre_saturation,
            border_buffer: (self.border_buffer * 100.0).round() as i32,
            film_base: self.film_base,
            film_base_sample: self.film_base_sample.map(|v| (v * 10000.0).round() as i32),
        }
    }

    /// Copy only the Edit/Advanced-tab parameters (used by paste/sync so a
    /// frame keeps its own crop and analysis-relevant convert settings).
    pub fn copy_edit_from(&mut self, o: &Settings) {
        let keep = (
            self.rotation,
            self.flip_h,
            self.straighten,
            self.crop,
            self.film_base,
            self.film_base_sample,
        );
        *self = o.clone();
        (
            self.rotation,
            self.flip_h,
            self.straighten,
            self.crop,
            self.film_base,
            self.film_base_sample,
        ) = keep;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AnalysisKey {
    /// The source selects the colour model's calibration, which the analysis sees.
    #[serde(default)]
    pub source: Source,
    pub image_type: ImageType,
    pub color_model: ColorModel,
    pub pre_saturation: u8,
    pub border_buffer: i32,
    pub film_base: FilmBase,
    pub film_base_sample: [i32; 3],
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
    /// 0 = full resolution.
    pub long_edge: u32,
    pub output_dir: Option<PathBuf>,
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
            long_edge: 0,
            output_dir: None,
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crop_and_geometry_do_not_change_analysis_key() {
        let mut s = Settings::default();
        let initial_key = s.analysis_key();

        s.crop = [5.0, 10.0, 5.0, 10.0];
        assert_eq!(s.analysis_key(), initial_key, "crop must not alter analysis_key");

        s.straighten = 2.5;
        assert_eq!(s.analysis_key(), initial_key, "straighten must not alter analysis_key");

        s.rotation = 1;
        assert_eq!(s.analysis_key(), initial_key, "rotation must not alter analysis_key");

        s.flip_h = true;
        assert_eq!(s.analysis_key(), initial_key, "flip_h must not alter analysis_key");

        // Convert settings MUST alter analysis_key
        s.color_model = ColorModel::Frontier;
        assert_ne!(s.analysis_key(), initial_key, "color_model must alter analysis_key");
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
        assert!(key.is_ok(), "legacy sidecar with crop fields must deserialize cleanly");
    }
}
