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
    /// How a roll's frames are printed. Both use the film's channel slopes
    /// from the roll; Lab sets each frame's colour and density from the
    /// frame, Darkroom prints the whole roll at one balance and exposure.
    RollMode {
        Lab => "Lab (per frame)",
        Darkroom => "Darkroom (one setting)",
    } default Lab
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

/// Version of the conversion engine; part of [`AnalysisKey`].
pub const ENGINE_VERSION: u32 = 4;

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
    /// Hyper Tone: compression (0..100) of the large-area tones above and
    /// below the frame's key, i.e. automatic burning and dodging.
    pub hyper_tone_highlights: f32,
    pub hyper_tone_shadows: f32,
    /// Moves the white end into the density range (− recovers highlights).
    pub white_clip: f32,
    /// Moves the black end into the density range (− recovers shadows).
    pub black_clip: f32,

    // ---- Color ---------------------------------------------------------------
    pub wb_mode: WbMode,
    /// Yellow printing filtration from white balance, in CC (+ warmer).
    pub wb_temp: f32,
    /// Magenta printing filtration from white balance, in CC (+ magenta).
    pub wb_tint: f32,
    /// Lab correction (the C·M·Y keys) as printing filtration in CC:
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
            auto_crop: false,
            flat_field: None,
            rotation: 0,
            flip_h: false,
            straighten: 0.0,
            crop: [0.0; 4],
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
            hyper_tone_highlights: 0.0,
            hyper_tone_shadows: 0.0,
            white_clip: 0.0,
            black_clip: 0.0,
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
            pre_saturation: self.pre_saturation,
            border_buffer: (self.border_buffer * 100.0).round() as i32,
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
            self.pre_saturation,
            self.border_buffer,
            self.film_base,
            self.film_base_sample,
            self.auto_crop,
            self.use_roll,
        ) = (
            k.source,
            k.image_type,
            k.color_model,
            k.pre_saturation,
            k.border_buffer,
            k.film_base,
            k.film_base_sample,
            k.auto_crop,
            k.use_roll,
        );
        (self.rotation, self.flip_h, self.straighten, self.crop) =
            (k.rotation, k.flip_h, k.straighten, k.crop);
        self.flat_field = k.flat_field.clone();
    }

    /// Copy only the adjustment parameters (used by paste/sync). Like the
    /// plugin's sync list, the conversion inputs (source, image type, colour
    /// model, pre-saturation, border) stay with the frame, as do its film
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
    pub pre_saturation: u8,
    pub border_buffer: i32,
    pub film_base: FilmBase,
    pub film_base_sample: [i32; 3],
    #[serde(default)]
    pub flat_field: Option<PathBuf>,
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

/// Lab-scanner corrections: the Frontier/Noritsu D·C·M·Y keys, in fixed
/// steps of print exposure and printing filtration.
pub mod lab {
    use super::Settings;

    /// One density click: 1/6 EV, about 0.05 D.
    pub const D_STEP: f32 = 1.0 / 6.0;
    /// One colour click, in CC of filtration.
    pub const C_STEP: f32 = 2.5;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Channel {
        /// Print density: + is darker.
        D,
        C,
        M,
        Y,
    }

    fn value(s: &mut Settings, ch: Channel) -> (&mut f32, f32, f32) {
        match ch {
            // Density runs against print exposure (+ EV = lighter).
            Channel::D => (&mut s.print_exposure, -D_STEP, 5.0),
            Channel::C => (&mut s.filtration[0], C_STEP, 50.0),
            Channel::M => (&mut s.filtration[1], C_STEP, 50.0),
            Channel::Y => (&mut s.filtration[2], C_STEP, 50.0),
        }
    }

    /// Move `ch` by `clicks` steps.
    pub fn nudge(s: &mut Settings, ch: Channel, clicks: i32) {
        let (v, step, max) = value(s, ch);
        *v = (*v + step * clicks as f32).clamp(-max, max);
    }

    /// Current offsets in clicks: D, C, M, Y.
    pub fn clicks(s: &Settings) -> [i32; 4] {
        let mut s = s.clone();
        [Channel::D, Channel::C, Channel::M, Channel::Y].map(|ch| {
            let (v, step, _) = value(&mut s, ch);
            (*v / step).round() as i32
        })
    }

    /// Zero density and the filtration.
    pub fn reset(s: &mut Settings) {
        s.print_exposure = 0.0;
        s.filtration = [0.0; 3];
    }

    /// "D+2 C−1 M0 Y+1", or `None` when every offset is zero.
    pub fn readout(s: &Settings) -> Option<String> {
        let c = clicks(s);
        if c == [0; 4] {
            return None;
        }
        let f = |v: i32| match v {
            0 => "0".to_string(),
            v if v > 0 => format!("+{v}"),
            v => format!("−{}", -v),
        };
        Some(format!(
            "D{} C{} M{} Y{}",
            f(c[0]),
            f(c[1]),
            f(c[2]),
            f(c[3])
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paste_keeps_conversion_inputs() {
        let mut dst = Settings {
            source: Source::VueScanRaw,
            color_model: ColorModel::Noritsu,
            pre_saturation: 5,
            border_buffer: 8.0,
            crop: [1.0, 2.0, 3.0, 4.0],
            ..Settings::default()
        };
        let before = dst.analysis_key();
        let src = Settings {
            brightness: 12.0,
            color_model: ColorModel::Frontier,
            border_buffer: 2.0,
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
    fn lab_keys_step_and_read_out() {
        let mut s = Settings::default();
        assert_eq!(lab::readout(&s), None);
        lab::nudge(&mut s, lab::Channel::D, 2);
        lab::nudge(&mut s, lab::Channel::C, -1);
        lab::nudge(&mut s, lab::Channel::Y, 3);
        assert!(s.print_exposure < 0.0, "denser prints are darker");
        assert_eq!(lab::clicks(&s), [2, -1, 0, 3]);
        assert_eq!(lab::readout(&s).unwrap(), "D+2 C−1 M0 Y+3");
        let key = s.analysis_key();
        lab::reset(&mut s);
        assert_eq!(lab::readout(&s), None);
        assert_eq!(s.analysis_key(), key);
        for _ in 0..100 {
            lab::nudge(&mut s, lab::Channel::M, 1);
        }
        assert_eq!(s.filtration[1], 50.0);
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
