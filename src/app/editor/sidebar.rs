//! The editor's sidebar: histogram, tool strip, Convert and the adjustment sections.

use eframe::egui::{self, RichText, Sense, vec2};

use super::{ColorTab, HistView, Pick};
use crate::app::NlpApp;
use crate::app::widgets::{
    self, Icon, fl_combo, fl_slider, fl_slider_u8, grad, icon_button, segmented, theme,
};
use crate::engine::settings::*;

/// The sidebar's gutter on the right, shared with its scroll bar. With the panel's
/// margin it makes the content's right edge match its left (16 pt).
const SCROLL_GUTTER: f32 = 10.0;

/// Sidebar sections that can be reset on their own.
#[derive(Clone, Copy)]
pub(super) enum Section {
    /// White balance and tone together, like Lightroom's Basic panel.
    Basic,
    Tone,
    WhiteBalance,
    /// Hyper Tone, glow, fade and clipping.
    PrintCharacter,
    /// Colour toning, saturation and LUT.
    Colour,
}

/// Tint ±150 spans the same range as ±50 of magenta filtration.
const TINT_PER_FILTRATION: f32 = 3.0;

macro_rules! differs {
    ($s:expr, $r:expr; $($f:ident),*) => { false $(|| $s.$f != $r.$f)* };
}
macro_rules! take {
    ($s:expr, $r:expr; $($f:ident),*) => {{ $( $s.$f = $r.$f.clone(); )* }};
}

impl Section {
    fn title(self) -> &'static str {
        match self {
            Section::Basic => "Basic",
            Section::Tone => "Tone",
            Section::WhiteBalance => "White balance",
            Section::PrintCharacter => "Finishing",
            Section::Colour => "Colour",
        }
    }

    /// Whether `s` differs from the reference `r` in this section.
    fn modified(self, s: &Settings, r: &Settings) -> bool {
        match self {
            Section::Basic => Section::Tone.modified(s, r) || Section::WhiteBalance.modified(s, r),
            Section::Tone => differs!(s, r; tone_profile, print_exposure, brightness, contrast,
                highlights, shadows, whites, blacks),
            Section::WhiteBalance => s.wb_mode != r.wb_mode,
            Section::PrintCharacter => differs!(s, r; glow, fade, chroma_denoise),
            Section::Colour => differs!(s, r; mid_cyan, mid_tint, mid_temp, high_cyan, high_tint,
                high_temp, high_range, shadow_cyan, shadow_tint, shadow_temp, shadow_range,
                lut, lut_strength, post_saturation),
        }
    }

    pub(super) fn reset(self, s: &mut Settings, r: &Settings) {
        match self {
            Section::Basic => {
                Section::Tone.reset(s, r);
                Section::WhiteBalance.reset(s, r);
            }
            Section::Tone => take!(s, r; tone_profile, print_exposure, brightness, contrast,
                highlights, shadows, whites, blacks),
            Section::WhiteBalance => take!(s, r; wb_mode, wb_temp, wb_tint),
            Section::PrintCharacter => take!(s, r; glow, fade, chroma_denoise),
            Section::Colour => take!(s, r; mid_cyan, mid_tint, mid_temp, high_cyan, high_tint,
                high_temp, high_range, shadow_cyan, shadow_tint, shadow_temp, shadow_range,
                lut, lut_strength, post_saturation),
        }
    }
}

/// A small muted caption over a group of sliders, aligned with their labels.
fn caption(ui: &mut egui::Ui, text: &str) {
    let (cap, _) = ui.allocate_exact_size(vec2(ui.available_width(), 16.0), Sense::hover());
    ui.painter().text(
        cap.left_center() + vec2(widgets::LABEL_W, 0.0),
        egui::Align2::RIGHT_CENTER,
        text,
        egui::FontId::proportional(11.0),
        theme::MUTED,
    );
}

/// A section header that marks edits and resets the section on request;
/// returns whether it is open.
fn section(ui: &mut egui::Ui, sec: Section, s: &mut Settings, r: &Settings, open: bool) -> bool {
    let (open, reset) = widgets::fold_section(ui, sec.title(), open, sec.modified(s, r));
    if reset {
        sec.reset(s, r);
    }
    open
}

impl NlpApp {
    /// The histogram, or the print curves when the ⋯ menu has them on. Like Lightroom's, the
    /// five regions are drag handles for Blacks, Shadows, Exposure, Highlights and Whites, and the corner
    /// triangles toggle the clipping view.
    fn histogram_panel(&mut self, ui: &mut egui::Ui, i: usize, s: &mut Settings) {
        ui.add_space(10.0);
        if self.editor.hist_view == HistView::Curves && !self.editor.curves.is_empty() {
            widgets::curves_plot(ui, &self.editor.curves, vec2(ui.available_width(), 96.0));
            ui.add_space(2.0);
            return;
        }
        use widgets::HistRegion as R;
        let editable = self.frames[i].sc.is_converted() && !self.editor.before;
        let fmt = |v: f32, stops: bool| {
            if stops {
                format!("{v:+.1}")
            } else {
                format!("{v:+.0}")
            }
        };
        let values = editable.then(|| {
            [
                fmt(s.blacks, false),
                fmt(s.shadows, false),
                fmt(s.print_exposure, true),
                fmt(s.highlights, false),
                fmt(s.whites, false),
            ]
        });
        let ev = widgets::histogram(
            ui,
            &self.editor.hist,
            vec2(ui.available_width(), 96.0),
            self.editor.hist_frozen,
            self.editor.clip_view,
            values,
        );
        if ev.clip_toggle {
            self.editor.clip_view = !self.editor.clip_view;
            self.editor.dirty = true;
        }
        // Points of drag per full slider range, and the field each region owns.
        fn field(s: &mut Settings, r: R) -> (&mut f32, f32, f32) {
            match r {
                R::Blacks => (&mut s.blacks, 100.0, 0.4),
                R::Darks => (&mut s.shadows, 50.0, 0.4),
                R::Exposure => (&mut s.print_exposure, 5.0, 0.02),
                R::Lights => (&mut s.highlights, 50.0, 0.4),
                R::Whites => (&mut s.whites, 100.0, 0.4),
            }
        }
        if let Some((r, dx)) = ev.drag {
            let (v, lim, per_px) = field(s, r);
            *v = (*v + dx * per_px).clamp(-lim, lim);
        }
        if let Some(r) = ev.reset {
            *field(s, r).0 = 0.0;
        }
        ui.add_space(2.0);
    }

    /// The tool strip under the histogram: Crop, Heal, Dodge & burn, and a placeholder
    /// for a tool to come.
    fn tool_strip(&mut self, ui: &mut egui::Ui, i: usize) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            let n = 4.0;
            let size = vec2((ui.available_width() - 2.0 * (n - 1.0)) / n, 36.0);
            let cropping = self.editor.crop().is_some();
            if widgets::sized_icon_button(
                ui,
                Icon::Crop,
                "Crop & straighten (C)",
                cropping,
                true,
                None,
                size,
                3.0,
            )
            .clicked()
            {
                if cropping {
                    self.apply_crop(i);
                } else {
                    self.enter_crop(i);
                }
            }
            let healing = self.editor.heal().is_some();
            if widgets::sized_icon_button(
                ui,
                Icon::Heal,
                "Spot healing: remove dust and scratches (Q)",
                healing,
                true,
                None,
                size,
                3.0,
            )
            .clicked()
            {
                self.toggle_heal(i);
            }
            let burning = self.editor.burn().is_some();
            if widgets::sized_icon_button(
                ui,
                Icon::Burn,
                "Dodge & burn: lighten or darken areas by hand (K)",
                burning,
                true,
                None,
                size,
                3.0,
            )
            .clicked()
            {
                self.toggle_burn(i);
            }
            widgets::sized_icon_button(
                ui,
                Icon::Placeholder,
                "Tool 4 (coming soon)",
                false,
                false,
                None,
                size,
                3.0,
            );
        });
        ui.add_space(2.0);
    }

    pub(super) fn sidebar(&mut self, ui: &mut egui::Ui, i: usize, s: &mut Settings) {
        egui::Frame::new()
            .inner_margin(egui::Margin {
                right: SCROLL_GUTTER as i8,
                ..Default::default()
            })
            .show(ui, |ui| {
                self.histogram_panel(ui, i, s);
                self.tool_strip(ui, i);
                self.crop_panel(ui, i);
                self.heal_panel(ui, s);
                self.burn_panel(ui, s);
            });
        ui.add_space(8.0);
        widgets::reset_section_gap(ui);
        self.film_dialog(ui, i, s);
        let needs_convert = self.needs_convert(i, s);
        // The edit's actions, at the bottom: Convert while the frame needs it,
        // otherwise Copy from last and Reset.
        egui::Panel::bottom("sidebar_actions")
            .exact_size(44.0)
            .frame(egui::Frame::new().inner_margin(egui::Margin {
                top: 8,
                right: SCROLL_GUTTER as i8,
                ..Default::default()
            }))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;
                    let w = ui.available_width();
                    if needs_convert {
                        ui.allocate_ui(vec2(w, 32.0), |ui| self.convert_button(ui, i, s));
                    } else {
                        let bw = (w - 6.0) / 2.0;
                        let prev = self.editor.prev_edit.clone();
                        if ui
                            .add_enabled(
                                prev.is_some(),
                                widgets::ghost("Copy from last").min_size(vec2(bw, 32.0)),
                            )
                            .on_hover_text(
                                "Apply the settings of the frame you edited before this one",
                            )
                            .clicked()
                            && let Some(p) = prev
                        {
                            self.paste_edit_to(vec![i], &p);
                        }
                        // An auto WB's Temp / Tint are its result, not an edit.
                        let mut defaults = s.clone();
                        self.reset_edits(i, &mut defaults);
                        if defaults.wb_mode == s.wb_mode {
                            defaults.wb_temp = s.wb_temp;
                            defaults.wb_tint = s.wb_tint;
                        }
                        if ui
                            .add_enabled(
                                defaults != *s,
                                widgets::ghost("Reset frame").min_size(vec2(bw, 32.0)),
                            )
                            .on_hover_text(
                                "Reset all edits to the defaults (Undo brings them back)",
                            )
                            .on_disabled_hover_text("This frame has no edits to reset")
                            .clicked()
                        {
                            self.reset_edits(i, s);
                        }
                    }
                });
            });
        // The handle stays visible while the panel has more below the fold.
        ui.spacing_mut().scroll.dormant_handle_opacity = 0.6;
        let scroll = egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                // Section open states are remembered by id; scoping the ids to
                // this photo makes each one open with its default state.
                ui.push_id(self.editor.epoch, |ui| {
                    ui.set_width(ui.available_width() - SCROLL_GUTTER);
                    if self.frames[i].sc.is_converted() {
                        self.adjust_sections(ui, s);
                    } else {
                        ui.add_space(12.0);
                        ui.label(
                            RichText::new(
                                "This negative is not converted yet. Convert it to edit its \
                                 tone and colour; the film details are kept either way.",
                            )
                            .small()
                            .color(theme::MUTED),
                        );
                    }
                    ui.add_space(16.0);
                });
            });
        // More below the fold: fade the last rows, so it doesn't look like the end.
        let hidden = scroll.content_size.y - scroll.state.offset.y - scroll.inner_rect.height();
        if hidden > 1.0 {
            widgets::scroll_fade(ui, scroll.inner_rect);
        }
    }

    /// The ⋯ menu's "Film & roll…" dialog: film details and roll controls.
    fn film_dialog(&mut self, ui: &mut egui::Ui, i: usize, s: &mut Settings) {
        if !self.editor.film_open {
            return;
        }
        let frame = egui::Frame::new()
            .fill(theme::PANEL)
            .stroke(egui::Stroke::new(1.0, theme::LINE))
            .corner_radius(12.0)
            .inner_margin(egui::Margin::same(22))
            .shadow(ui.ctx().global_style().visuals.window_shadow);
        let r = egui::Modal::new(egui::Id::new("film_modal"))
            .backdrop_color(egui::Color32::from_black_alpha(150))
            .frame(frame)
            .show(ui.ctx(), |ui| {
                ui.set_width(320.0);
                ui.label(
                    RichText::new("Film & roll")
                        .font(theme::medium(16.0))
                        .color(theme::TEXT),
                );
                ui.add_space(12.0);
                self.film_details(ui, i);
                self.roll_controls(ui, i, s);
                ui.add_space(12.0);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add(widgets::primary("Done")).clicked()
                })
                .inner
            });
        if r.inner || r.should_close() {
            self.editor.film_open = false;
        }
    }

    /// Film and capture details: written to the exported EXIF and used by
    /// the export file name tokens.
    fn film_details(&mut self, ui: &mut egui::Ui, i: usize) {
        let changed =
            crate::app::roll::film_fields(ui, "film_meta", &mut self.frames[i].sc.metadata, true);
        if changed {
            self.frames[i].dirty_since = Some(self.time);
            // A stock from the list selects that film's crosstalk matrix.
            let id = crate::engine::presets::crosstalk_id_for_stock(
                &self.frames[i].sc.metadata.film_stock,
            );
            if let Some(id) = id {
                if self.frames[i].sc.settings.crosstalk != id {
                    self.frames[i].sc.settings.crosstalk = id.to_string();
                    self.mark_changed(i);
                    self.editor.invalidate();
                }
            }
        }
        // Copy to the frame's roll, or to the whole folder.
        let roll = self.frames[i].sc.roll.as_ref().map(|r| r.id.clone());
        let others: Vec<usize> = (0..self.frames.len())
            .filter(|&k| k != i)
            .filter(|&k| {
                roll.is_none() || self.frames[k].sc.roll.as_ref().map(|r| &r.id) == roll.as_ref()
            })
            .collect();
        if !others.is_empty() {
            ui.add_space(6.0);
            let scope = if roll.is_some() { "roll" } else { "folder" };
            let label = format!("Apply to {scope} ({} frames)", others.len());
            if ui
                .add(widgets::ghost(&label).min_size(egui::vec2(ui.available_width(), 28.0)))
                .on_hover_text(format!(
                    "Copy these details to {} other frame(s)",
                    others.len()
                ))
                .clicked()
            {
                let m = self.frames[i].sc.metadata.clone();
                let crosstalk = self.frames[i].sc.settings.crosstalk.clone();
                let matched =
                    crate::engine::presets::crosstalk_id_for_stock(&m.film_stock).is_some();
                for &k in &others {
                    self.frames[k].sc.metadata = m.clone();
                    self.frames[k].dirty_since = Some(self.time);
                    if matched {
                        self.frames[k].sc.settings.crosstalk = crosstalk.clone();
                        self.frames[k].thumb_dirty = true;
                    }
                }
                self.toast(
                    format!("Film details copied to {} frame(s)", others.len()),
                    false,
                );
            }
        }
        ui.add_space(8.0);
    }

    fn convert_button(&mut self, ui: &mut egui::Ui, i: usize, s: &Settings) {
        let f = &self.frames[i];
        let converted = f.sc.is_converted();
        let stale =
            f.sc.analysis
                .as_ref()
                .map(|a| a.key != s.analysis_key())
                .unwrap_or(false);
        let label = match (converted, stale) {
            (false, _) => "Convert negative",
            (true, true) => "Re-convert (settings changed)",
            (true, false) => "Re-convert",
        };
        let btn = if !converted || stale {
            widgets::primary(label)
        } else {
            widgets::ghost(label)
        };
        if ui
            .add_sized([ui.available_width(), 32.0], btn)
            .on_hover_text("Choose the conversion options and analyse this frame")
            .clicked()
        {
            self.frames[i].sc.settings = s.clone();
            self.open_convert_dialog(vec![i], false);
        }
    }

    /// Unconverted, or the analysis inputs changed since the last convert.
    fn needs_convert(&self, i: usize, s: &Settings) -> bool {
        let f = &self.frames[i];
        !f.sc.is_converted()
            || f.sc
                .analysis
                .as_ref()
                .map(|a| a.key != s.analysis_key())
                .unwrap_or(false)
    }

    /// A dropdown row with an eyedropper button at its end.
    fn combo_with_picker<T: Copy + PartialEq>(
        &mut self,
        ui: &mut egui::Ui,
        v: &mut T,
        all: &[T],
        name: impl Fn(T) -> &'static str,
        pick: Pick,
        tip: &str,
        enabled: bool,
    ) {
        const PICKER_W: f32 = 32.0;
        let caption = match pick {
            Pick::WhiteBalance => "WB",
            Pick::FilmBase => "Base",
        };
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            widgets::combo_row(ui, caption, v, all, name, PICKER_W);
            let picking = self.editor.pick() == Some(pick);
            let enabled = enabled && self.editor.crop().is_none();
            if icon_button(ui, Icon::Eyedropper, tip, picking, enabled).clicked() {
                self.set_pick(if picking { None } else { Some(pick) });
            }
        });
    }

    fn adjust_sections(&mut self, ui: &mut egui::Ui, s: &mut Settings) {
        // What each section resets to: this frame's defaults.
        let mut reference = s.clone();
        if let Some(i) = self.editor_index() {
            self.reset_edits(i, &mut reference);
        }
        let r = &reference;
        let wb_mode = s.wb_mode;
        let basic_open = section(ui, Section::Basic, s, r, true);
        if s.wb_mode != wb_mode
            && let Some(i) = self.editor_index()
        {
            // Reset to an auto mode: recompute its Temp / Tint.
            self.refresh_wb(i, s);
        }
        if basic_open {
            widgets::combo_row(
                ui,
                "Profile",
                &mut s.tone_profile,
                ToneProfile::ALL,
                ToneProfile::label,
                0.0,
            );
            widgets::group_divider(ui);
            let wb_key = widgets::shortcut(ui.ctx(), egui::Modifiers::NONE, egui::Key::W);
            self.combo_with_picker(
                ui,
                &mut s.wb_mode,
                WbMode::ALL,
                WbMode::label,
                Pick::WhiteBalance,
                &format!("Pick a neutral grey in the image ({wb_key})"),
                true,
            );
            let auto = matches!(
                s.wb_mode,
                WbMode::AutoNeutral | WbMode::AutoWarm | WbMode::AutoCool | WbMode::AutoMix
            );
            // An auto mode's values are its result, not an edit: nothing to mark
            // or reset. Moving either slider switches to Custom.
            let note = if auto {
                "\n\nSet by the WB mode; moving it switches to Custom."
            } else {
                ""
            };
            let mut t = s.wb_temp;
            if widgets::fl_slider_auto(
                ui,
                "Temp",
                &mut t,
                -50.0..=50.0,
                if auto { s.wb_temp } else { 0.0 },
                1,
                &format!("Yellow printing filtration in CC: blue ↔ yellow{note}"),
                Some(grad::TEMP),
                auto,
            ) {
                s.wb_temp = t;
                s.wb_mode = WbMode::Custom;
            }
            // Tint on ±150, as in Lightroom; the setting stays filtration.
            let mut k = s.wb_tint * TINT_PER_FILTRATION;
            let tint_default = if auto { k } else { 0.0 };
            if widgets::fl_slider_auto(
                ui,
                "Tint",
                &mut k,
                -150.0..=150.0,
                tint_default,
                0,
                &format!("Green ↔ magenta{note}"),
                Some(grad::TINT),
                auto,
            ) {
                s.wb_tint = k / TINT_PER_FILTRATION;
                s.wb_mode = WbMode::Custom;
            }
            widgets::group_divider(ui);
            let tone = |ui: &mut egui::Ui, l: &str, v: &mut f32, tip: &str| {
                fl_slider(ui, l, v, -50.0..=50.0, 0.0, 0, tip, None)
            };
            // Shown in stops: 1 EV = 10 plugin units, 0.1 EV steps.
            fl_slider(
                ui,
                "Exposure",
                &mut s.print_exposure,
                -5.0..=5.0,
                0.0,
                1,
                "Print exposure in stops, like the print time in a darkroom: + a shorter time, lighter (highlights fade to paper white); − a longer time, darker (shadows block up)",
                None,
            );
            tone(
                ui,
                "Brightness",
                &mut s.brightness,
                "Midtone gamma: lower = denser, higher = airy",
            );
            tone(ui, "Contrast", &mut s.contrast, "Midtone contrast slope");
            widgets::group_divider(ui);
            tone(ui, "Highlights", &mut s.highlights, "Upper midtones");
            tone(ui, "Shadows", &mut s.shadows, "Lower midtones");
            fl_slider(
                ui,
                "Whites",
                &mut s.whites,
                -100.0..=100.0,
                0.0,
                0,
                "Where the print reaches paper white: up clips highlights, down recovers them",
                None,
            );
            fl_slider(
                ui,
                "Blacks",
                &mut s.blacks,
                -100.0..=100.0,
                0.0,
                0,
                "Where the print reaches full black: up = matte and recovers crushed shadows, down = deeper",
                None,
            );
        }
        // Closed by default, like Finishing; the user opens it when wanted.
        if section(ui, Section::Colour, s, r, false) {
            segmented(
                ui,
                &mut self.editor.color_tab,
                &[
                    (ColorTab::Mids, "Midtones"),
                    (ColorTab::Highs, "Highlights"),
                    (ColorTab::Shadows, "Shadows"),
                ],
            );
            ui.add_space(6.0);
            let (c, m, y, range) = match self.editor.color_tab {
                ColorTab::Mids => (&mut s.mid_cyan, &mut s.mid_tint, &mut s.mid_temp, None),
                ColorTab::Highs => (
                    &mut s.high_cyan,
                    &mut s.high_tint,
                    &mut s.high_temp,
                    Some(&mut s.high_range),
                ),
                ColorTab::Shadows => (
                    &mut s.shadow_cyan,
                    &mut s.shadow_tint,
                    &mut s.shadow_temp,
                    Some(&mut s.shadow_range),
                ),
            };
            fl_slider(
                ui,
                "Cyan",
                c,
                -75.0..=75.0,
                0.0,
                0,
                "Red ↔ cyan in these tones",
                Some(grad::RED_CYAN),
            );
            fl_slider(
                ui,
                "Magenta",
                m,
                -75.0..=75.0,
                0.0,
                0,
                "Green ↔ magenta in these tones",
                Some(grad::GREEN_MAGENTA),
            );
            fl_slider(
                ui,
                "Yellow",
                y,
                -75.0..=75.0,
                0.0,
                0,
                "Blue ↔ yellow in these tones",
                Some(grad::BLUE_YELLOW),
            );
            if let Some(r) = range {
                fl_slider(
                    ui,
                    "Range",
                    r,
                    1.0..=10.0,
                    5.0,
                    0,
                    "Span of tones affected",
                    None,
                );
            }
            widgets::group_divider(ui);
            widgets::combo_row(ui, "Look", &mut s.lut, Lut::ALL, Lut::label, 0.0);
            widgets::fl_slider_unit(
                ui,
                "Amount",
                &mut s.lut_strength,
                0.0..=200.0,
                100.0,
                0,
                "%",
                "Strength of the look",
                None,
            );
            fl_slider_u8(
                ui,
                "Saturation",
                &mut s.post_saturation,
                1..=10,
                5,
                "Look saturation (5 = default)",
            );
        }
        // Closed unless wanted: it keeps its state from frame to frame, and a
        // frame with Finishing edits opens it.
        let finished = Section::PrintCharacter.modified(s, r);
        let finishing = section(
            ui,
            Section::PrintCharacter,
            s,
            r,
            self.editor.finishing_open || finished,
        );
        if !finished {
            self.editor.finishing_open = finishing;
        }
        if finishing {
            let tone = |ui: &mut egui::Ui, l: &str, v: &mut f32, tip: &str| {
                fl_slider(ui, l, v, -50.0..=50.0, 0.0, 0, tip, None)
            };
            tone(ui, "Glow", &mut s.glow, "Silky compressed highlights");
            tone(ui, "Fade", &mut s.fade, "Open, compressed deep shadows");
            widgets::group_divider(ui);
            caption(ui, "Noise");
            fl_slider(
                ui,
                "Chroma",
                &mut s.chroma_denoise,
                0.0..=100.0,
                25.0,
                0,
                "Smooths coloured grain and colour noise; luminance grain and detail are kept",
                None,
            );
        }
    }

    /// Roll analysis: whether the frame is printed from the roll's balance,
    /// and how.
    fn roll_controls(&mut self, ui: &mut egui::Ui, i: usize, s: &mut Settings) {
        if self.frames[i].sc.roll.is_none() || !self.frames[i].sc.is_converted() {
            return;
        }
        ui.add_space(8.0);
        widgets::checkbox(ui, &mut s.use_roll, "Use roll analysis");
        if s.use_roll {
            fl_combo(
                ui,
                "Roll process",
                &mut s.roll_mode,
                RollMode::ALL,
                RollMode::label,
                "",
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sections_mark_and_reset_only_their_own_fields() {
        let r = Settings::default();
        let mut s = r.clone();
        assert!(!Section::Colour.modified(&s, &r));
        s.lut_strength = 60.0;
        s.brightness = 12.0;
        assert!(Section::Colour.modified(&s, &r));
        assert!(Section::Tone.modified(&s, &r));
        Section::Colour.reset(&mut s, &r);
        assert!(!Section::Colour.modified(&s, &r));
        assert_eq!(s.brightness, 12.0, "other sections keep their edits");
        assert!(Section::Tone.modified(&s, &r));
    }
}
