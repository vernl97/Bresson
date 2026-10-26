//! Editor View: fine-tune one frame. Layout: a light toolbar (navigation,
//! zoom, history on the left; title centred; geometry and export on the
//! right), a grey canvas with zoom / pan and an interactive crop mode, and a
//! sidebar holding the plugin's Convert / Edit / Advanced controls.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use eframe::egui::{self, Color32, Rect, RichText, Sense, Stroke, pos2, vec2};

use super::folder::{lut_folder_ui, source_instructions};
use super::widgets::{
    self, Icon, fl_combo, fl_slider, fl_slider_u8, grad, header, icon_button, segmented, theme,
};
use super::{NlpApp, View, to_color_image};
use crate::engine::decode::{self, SourceImage};
use crate::engine::pipeline::{self, Negative, Renderer};
use crate::engine::settings::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Convert,
    Edit,
    Advanced,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ColorTab {
    Mids,
    Highs,
    Shadows,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Pick {
    WhiteBalance,
    FilmBase,
}

/// Which part of the crop rectangle is being dragged.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Handle {
    Move,
    L,
    R,
    T,
    B,
    TL,
    TR,
    BL,
    BR,
    Dial,
}

/// Uncommitted crop / straighten state while crop mode is open.
#[derive(Clone, Copy)]
struct CropEdit {
    insets: [f32; 4],
    angle: f32,
    drag: Option<Handle>,
}

/// A requested zoom change, resolved in the canvas where the geometry is known.
#[derive(Clone, Copy)]
enum ZoomReq {
    /// Multiply the zoom around the canvas centre.
    Factor(f32),
    /// Double-click: zoom into this point, or back to fit when zoomed.
    Toggle(egui::Pos2),
}

/// A running zoom/pan animation.
#[derive(Clone, Copy)]
struct ZoomAnim {
    from_z: f32,
    to_z: f32,
    to_fit: bool,
    from_pan: egui::Vec2,
    to_pan: egui::Vec2,
    start: f64,
}

const ZOOM_ANIM_SECS: f64 = 0.25;

pub struct Editor {
    pub path: Option<PathBuf>,
    pub src: Option<Arc<SourceImage>>,
    pub loading: bool,
    neg: Option<Negative>,
    neg_small: Option<Negative>,
    neg_key: Option<(AnalysisKey, [i32; 4], i32, u8, bool, [i32; 3])>,
    tex: Option<egui::TextureHandle>,
    dirty: bool,
    pub before: bool,
    tab: Tab,
    color_tab: ColorTab,
    pick: Option<Pick>,
    undo: Vec<Settings>,
    redo: Vec<Settings>,
    committed: Option<Settings>,
    hist: [[u32; 256]; 3],
    curves: Vec<[f32; 4]>,
    uses_table: bool,
    render_ms: f32,
    /// Display scale (screen points per preview pixel); `None` = fit.
    zoom: Option<f32>,
    pan: egui::Vec2,
    crop: Option<CropEdit>,
    pub export_open: bool,
    /// Conversion state seen last frame (to switch to Edit after converting).
    was_converted: Option<bool>,
    zoom_req: Option<ZoomReq>,
    zoom_anim: Option<ZoomAnim>,
}

impl Editor {
    #[cfg(test)]
    pub fn set_tab(&mut self, t: Tab) {
        self.tab = t;
    }

    /// Test hook: adjust the pending crop.
    #[cfg(test)]
    pub fn set_pending_crop(&mut self, insets: [f32; 4], angle: f32) {
        if let Some(c) = &mut self.crop {
            c.insets = insets;
            c.angle = angle;
            self.dirty = true;
        }
    }
}

impl Default for Editor {
    fn default() -> Self {
        Self {
            path: None,
            src: None,
            loading: false,
            neg: None,
            neg_small: None,
            neg_key: None,
            tex: None,
            dirty: true,
            before: false,
            tab: Tab::Edit,
            color_tab: ColorTab::Mids,
            pick: None,
            undo: Vec::new(),
            redo: Vec::new(),
            committed: None,
            hist: [[0; 256]; 3],
            curves: Vec::new(),
            uses_table: false,
            render_ms: 0.0,
            zoom: None,
            pan: egui::Vec2::ZERO,
            crop: None,
            export_open: false,
            was_converted: None,
            zoom_req: None,
            zoom_anim: None,
        }
    }
}

impl Editor {
    #[cfg(test)]
    pub fn is_zoomed(&self) -> bool {
        self.zoom.is_some()
    }

    /// Test hook: what a double-click at `pos` requests.
    #[cfg(test)]
    pub fn double_click_at(&mut self, pos: egui::Pos2) {
        self.zoom_req = Some(ZoomReq::Toggle(pos));
    }

    pub fn load(&mut self, path: &Path, settings: Settings) {
        let keep = (self.tab, self.color_tab);
        *self = Editor {
            path: Some(path.to_path_buf()),
            loading: true,
            committed: Some(settings),
            ..Default::default()
        };
        (self.tab, self.color_tab) = keep;
    }

    /// Force the negative to be re-prepared and the preview re-rendered.
    pub fn invalidate(&mut self) {
        self.neg_key = None;
        self.dirty = true;
    }
}

/// Rotate crop insets [l, t, r, b] with the image.
fn rotate_insets(c: [f32; 4], clockwise: bool) -> [f32; 4] {
    if clockwise {
        [c[3], c[0], c[1], c[2]]
    } else {
        [c[1], c[2], c[3], c[0]]
    }
}

impl NlpApp {
    fn editor_index(&self) -> Option<usize> {
        let p = self.editor.path.as_ref()?;
        self.index_of(p)
    }

    /// Commit a settings change made in the editor.
    fn apply_change(&mut self, i: usize, before: &Settings, mut s: Settings) {
        if s == *before {
            return;
        }
        if s.engine != before.engine {
            let p = s.engine;
            s.apply_engine_preset(p);
        } else if s.curve_points != before.curve_points
            || s.correction_type != before.correction_type
            || s.layer_order != before.layer_order
            || s.wb_tonality != before.wb_tonality
            || s.wb_method != before.wb_method
            || s.clip_method != before.clip_method
        {
            s.engine = EnginePreset::Custom;
        }
        let wb_changed = s.wb_mode != before.wb_mode;
        self.frames[i].sc.settings = s;
        if wb_changed {
            self.frames[i].apply_wb_mode();
        }
        self.mark_changed(i);
        self.editor.dirty = true;
    }

    fn history_step(&mut self, i: usize, undo: bool) {
        let target = if undo {
            self.editor.undo.pop()
        } else {
            self.editor.redo.pop()
        };
        let Some(t) = target else { return };
        let cur = self.frames[i].sc.settings.clone();
        if undo {
            self.editor.redo.push(cur.clone());
        } else {
            self.editor.undo.push(cur.clone());
        }
        self.editor.committed = Some(t.clone());
        self.frames[i].sc.settings = t;
        self.mark_changed(i);
        self.editor.dirty = true;
    }

    fn track_history(&mut self, i: usize, pointer_down: bool) {
        if pointer_down {
            return;
        }
        let cur = &self.frames[i].sc.settings;
        match &self.editor.committed {
            Some(c) if c != cur => {
                self.editor.undo.push(c.clone());
                if self.editor.undo.len() > 200 {
                    self.editor.undo.remove(0);
                }
                self.editor.redo.clear();
                self.editor.committed = Some(cur.clone());
            }
            None => self.editor.committed = Some(cur.clone()),
            _ => {}
        }
    }

    fn leave_editor(&mut self) {
        if let Some(i) = self.editor_index() {
            self.save_frame(i);
        }
        self.editor.pick = None;
        self.editor.crop = None;
        self.view = View::Folder;
        if self.single_mode {
            // Lone image: return to the welcome screen.
            self.save_all();
            self.frames.clear();
            self.selected.clear();
            self.active = None;
            self.single_mode = false;
            self.editor = super::editor::Editor::default();
        }
    }

    fn goto(&mut self, i: usize, delta: isize) {
        let j = i as isize + delta;
        if (0..self.frames.len() as isize).contains(&j) {
            self.save_frame(i);
            self.selected.clear();
            self.selected.insert(j as usize);
            self.open_editor(j as usize);
        }
    }

    /// Settings the preview renders with (crop mode shows the uncropped,
    /// live-straightened frame).
    fn preview_settings(&self, i: usize) -> Settings {
        let mut s = self.frames[i].sc.settings.clone();
        if let Some(c) = &self.editor.crop {
            s.crop = [0.0; 4];
            s.straighten = c.angle;
        }
        s
    }

    /// (Re)render the preview texture if needed.
    fn update_preview(&mut self, ctx: &egui::Context, i: usize, dragging: bool) {
        let Some(src) = self.editor.src.clone() else {
            return;
        };
        if !self.editor.dirty && self.editor.tex.is_some() {
            return;
        }
        let t0 = std::time::Instant::now();
        let s = self.preview_settings(i);
        let f = &self.frames[i];
        let base = f.base_for(&src);
        let key = (
            s.analysis_key(),
            s.crop.map(|v| (v * 100.0).round() as i32),
            (s.straighten * 100.0).round() as i32,
            s.rotation,
            s.flip_h,
            base.map(|v| (v * 1e5) as i32),
        );
        if self.editor.neg_key.as_ref() != Some(&key) || self.editor.neg.is_none() {
            let neg = pipeline::prepare_negative(&src, &s, base);
            let (d, w, h) = decode::downscale(&neg.data, neg.width, neg.height, 1024);
            self.editor.neg_small = Some(Negative {
                width: w,
                height: h,
                data: d,
            });
            self.editor.neg = Some(neg);
            self.editor.neg_key = Some(key);
        }
        let use_small = dragging
            && self
                .editor
                .neg
                .as_ref()
                .map(|n| n.width.max(n.height) > 1100)
                .unwrap_or(false);
        let neg = if use_small {
            self.editor.neg_small.as_ref().unwrap()
        } else {
            self.editor.neg.as_ref().unwrap()
        };
        let roll = f.sc.roll.as_ref().map(|r| r.origins);
        let origins = if self.editor.before {
            None
        } else {
            pipeline::origins_for(&s, f.sc.analysis.as_ref(), roll.as_ref())
        };
        let mp = (src.info.full_width * src.info.full_height) as f32 / 1e6;
        let r = Renderer::new(&s, origins, OutputSpace::Srgb).with_megapixels(mp);
        let full_long = src.info.full_width.max(src.info.full_height).max(1) as f32;
        let crop_frac = 1.0 - (s.crop[0] + s.crop[2]).max(s.crop[1] + s.crop[3]) / 100.0;
        let scale = neg.width.max(neg.height) as f32 / (full_long * crop_frac.max(0.1));
        let px = r.render(neg, scale.min(1.0));
        let mut hist = [[0u32; 256]; 3];
        for p in px.iter().step_by(3) {
            for c in 0..3 {
                hist[c][(p[c].clamp(0.0, 1.0) * 255.0) as usize] += 1;
            }
        }
        self.editor.hist = hist;
        self.editor.curves = r.curve_lut().map(|l| l.sample(96)).unwrap_or_default();
        self.editor.uses_table = r.uses_lut_table();
        let img = to_color_image(&px, neg.width, neg.height);
        match &mut self.editor.tex {
            Some(t) => t.set(img, egui::TextureOptions::LINEAR),
            None => {
                self.editor.tex =
                    Some(ctx.load_texture("editor-preview", img, egui::TextureOptions::LINEAR))
            }
        }
        self.editor.dirty = use_small;
        self.editor.render_ms = t0.elapsed().as_secs_f32() * 1000.0;
    }

    fn zoom_by(&mut self, factor: f32, fit: f32) {
        self.editor.zoom_anim = None;
        let cur = self.editor.zoom.unwrap_or(fit);
        let next = (cur * factor).clamp(fit * 0.5, 8.0);
        if next <= fit * 1.001 && factor < 1.0 {
            self.editor.zoom = None;
            self.editor.pan = egui::Vec2::ZERO;
        } else {
            self.editor.zoom = Some(next.max(fit));
        }
    }

    pub(crate) fn enter_crop(&mut self, i: usize) {
        let s = &self.frames[i].sc.settings;
        self.editor.crop = Some(CropEdit {
            insets: s.crop,
            angle: s.straighten,
            drag: None,
        });
        self.editor.pick = None;
        self.editor.zoom = None;
        self.editor.pan = egui::Vec2::ZERO;
        self.editor.dirty = true;
    }

    fn apply_crop(&mut self, i: usize) {
        if let Some(c) = self.editor.crop.take() {
            let before = self.frames[i].sc.settings.clone();
            let mut s = before.clone();
            s.crop = c.insets;
            s.straighten = c.angle;
            self.apply_change(i, &before, s);
        }
        self.editor.dirty = true;
    }

    fn cancel_crop(&mut self) {
        self.editor.crop = None;
        self.editor.dirty = true;
    }

    pub fn editor_ui(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        let Some(i) = self.editor_index() else {
            self.view = View::Folder;
            return;
        };
        let pointer_down = ctx.input(|inp| inp.pointer.any_down());
        if self.editor_shortcuts(&ctx, i) {
            return;
        }
        self.track_history(i, pointer_down);
        self.update_preview(&ctx, i, pointer_down);
        if self.editor.dirty {
            ctx.request_repaint();
        }

        let before = self.frames[i].sc.settings.clone();
        let mut s = before.clone();

        egui::Panel::top("editor_toolbar")
            .exact_size(54.0)
            .frame(
                egui::Frame::new()
                    .fill(theme::TOOLBAR)
                    .stroke(Stroke::new(1.0, theme::LINE))
                    .inner_margin(egui::Margin::symmetric(12, 0)),
            )
            .show(ui, |ui| self.toolbar(ui, i, &mut s));

        // The toolbar may have left the editor (back) or switched frames.
        if self.view != View::Editor || self.editor_index() != Some(i) {
            return;
        }
        egui::Panel::right("editor_sidebar")
            .resizable(true)
            .default_size(360.0)
            .min_size(320.0)
            .max_size(560.0)
            .frame(
                egui::Frame::new()
                    .fill(theme::SIDEBAR)
                    .stroke(Stroke::new(1.0, theme::LINE))
                    .inner_margin(egui::Margin {
                        left: 20,
                        right: 16,
                        top: 14,
                        bottom: 8,
                    }),
            )
            .show(ui, |ui| self.sidebar(ui, i, &mut s));

        if self.editor_index() == Some(i) && self.editor.crop.is_none() {
            self.apply_change(i, &before, s);
        } else if self.editor_index() == Some(i) {
            // In crop mode only geometry-independent edits apply directly.
            let mut s2 = s;
            s2.crop = before.crop;
            s2.straighten = before.straighten;
            self.apply_change(i, &before, s2);
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(theme::CANVAS))
            .show(ui, |ui| self.canvas(ui, i));
    }

    /// Returns true when the view changed and drawing should stop.
    fn editor_shortcuts(&mut self, ctx: &egui::Context, i: usize) -> bool {
        if ctx.egui_wants_keyboard_input() {
            return false;
        }
        let k = ctx.input(|inp| {
            let cmd = inp.modifiers.command;
            [
                inp.key_pressed(egui::Key::B),
                cmd && !inp.modifiers.shift && inp.key_pressed(egui::Key::Z),
                cmd && inp.modifiers.shift && inp.key_pressed(egui::Key::Z),
                inp.key_pressed(egui::Key::ArrowLeft),
                inp.key_pressed(egui::Key::ArrowRight),
                inp.key_pressed(egui::Key::Escape),
                inp.key_pressed(egui::Key::W),
                inp.key_pressed(egui::Key::Space),
                inp.key_pressed(egui::Key::Plus) || inp.key_pressed(egui::Key::Equals),
                inp.key_pressed(egui::Key::Minus),
                inp.key_pressed(egui::Key::Enter),
                inp.key_pressed(egui::Key::C) && !cmd,
            ]
        });
        let [
            b,
            undo,
            redo,
            left,
            right,
            esc,
            w,
            space,
            plus,
            minus,
            enter,
            c,
        ] = k;
        if b {
            self.editor.before = !self.editor.before;
            self.editor.dirty = true;
        }
        if undo {
            self.history_step(i, true);
        }
        if redo {
            self.history_step(i, false);
        }
        if plus {
            self.editor.zoom_req = Some(ZoomReq::Factor(1.5));
        }
        if minus {
            self.editor.zoom_req = Some(ZoomReq::Factor(1.0 / 1.5));
        }
        if c {
            if self.editor.crop.is_some() {
                self.cancel_crop();
            } else {
                self.enter_crop(i);
            }
        }
        if enter && self.editor.crop.is_some() {
            self.apply_crop(i);
        }
        if w {
            self.editor.pick = if self.editor.pick == Some(Pick::WhiteBalance) {
                None
            } else {
                Some(Pick::WhiteBalance)
            };
        }
        if esc {
            if self.editor.pick.is_some() {
                self.editor.pick = None;
            } else if self.editor.crop.is_some() {
                self.cancel_crop();
            } else {
                self.leave_editor();
                return true;
            }
        }
        if space && self.editor.crop.is_none() {
            self.leave_editor();
            return true;
        }
        if left && self.editor.crop.is_none() {
            self.goto(i, -1);
            return true;
        }
        if right && self.editor.crop.is_none() {
            self.goto(i, 1);
            return true;
        }
        false
    }

    // ---- toolbar ----------------------------------------------------------------

    fn toolbar(&mut self, ui: &mut egui::Ui, i: usize, s: &mut Settings) {
        let rect = ui.max_rect();
        // Centred title.
        let f = &self.frames[i];
        let sub = if self.editor.crop.is_some() {
            "Crop / Rotate".to_string()
        } else if self.editor.before {
            "Before — negative".to_string()
        } else if f.sc.is_converted() {
            format!(
                "{}{}",
                s.tone_profile.label(),
                if f.is_roll() { " · roll" } else { "" }
            )
        } else {
            "Not converted".to_string()
        };
        let p = ui.painter();
        p.text(
            pos2(rect.center().x, rect.center().y - 8.0),
            egui::Align2::CENTER_CENTER,
            &f.name,
            egui::FontId::proportional(14.5),
            theme::TEXT,
        );
        p.text(
            pos2(rect.center().x, rect.center().y + 9.0),
            egui::Align2::CENTER_CENTER,
            sub,
            egui::FontId::proportional(12.0),
            theme::MUTED,
        );

        let n = self.frames.len();
        ui.horizontal_centered(|ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            if icon_button(ui, Icon::Back, "Back to folder (Space)", false, true).clicked() {
                self.leave_editor();
                return;
            }
            ui.add_space(14.0);
            if icon_button(ui, Icon::Prev, "Previous image (←)", false, i > 0).clicked() {
                self.goto(i, -1);
                return;
            }
            if icon_button(ui, Icon::Next, "Next image (→)", false, i + 1 < n).clicked() {
                self.goto(i, 1);
                return;
            }
            ui.add_space(14.0);
            if icon_button(
                ui,
                Icon::ZoomOut,
                "Zoom out (−)",
                false,
                self.editor.zoom.is_some(),
            )
            .clicked()
            {
                self.editor.zoom_req = Some(ZoomReq::Factor(1.0 / 1.5));
            }
            if icon_button(ui, Icon::ZoomIn, "Zoom in (+)", false, true).clicked() {
                self.editor.zoom_req = Some(ZoomReq::Factor(1.5));
            }
            ui.add_space(14.0);
            if icon_button(
                ui,
                Icon::Undo,
                "Undo (⌘Z)",
                false,
                !self.editor.undo.is_empty(),
            )
            .clicked()
            {
                self.history_step(i, true);
            }
            if icon_button(
                ui,
                Icon::Redo,
                "Redo (⇧⌘Z)",
                false,
                !self.editor.redo.is_empty(),
            )
            .clicked()
            {
                self.history_step(i, false);
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 2.0;
                if icon_button(
                    ui,
                    Icon::Export,
                    "Export positive copy",
                    self.export_targets.is_some() || self.editor.export_open,
                    true,
                )
                .clicked()
                {
                    self.open_export_dialog(vec![i]);
                }
                ui.add_space(14.0);
                let cropping = self.editor.crop.is_some();
                if icon_button(ui, Icon::Crop, "Crop / straighten (C)", cropping, true).clicked() {
                    if cropping {
                        self.apply_crop(i);
                    } else {
                        self.enter_crop(i);
                    }
                }
                ui.add_space(14.0);
                if icon_button(ui, Icon::Flip, "Flip horizontally", s.flip_h, !cropping).clicked() {
                    s.flip_h = !s.flip_h;
                    s.crop = [s.crop[2], s.crop[1], s.crop[0], s.crop[3]];
                }
                if icon_button(ui, Icon::RotateRight, "Rotate right (CW)", false, !cropping)
                    .clicked()
                {
                    s.rotation = (s.rotation + 1) % 4;
                    s.crop = rotate_insets(s.crop, true);
                }
                if icon_button(ui, Icon::RotateLeft, "Rotate left (CCW)", false, !cropping)
                    .clicked()
                {
                    s.rotation = (s.rotation + 3) % 4;
                    s.crop = rotate_insets(s.crop, false);
                }
                ui.add_space(14.0);
                if icon_button(
                    ui,
                    Icon::Compare,
                    "Before / after (B)",
                    self.editor.before,
                    true,
                )
                .clicked()
                {
                    self.editor.before = !self.editor.before;
                    self.editor.dirty = true;
                }
                if self.editor.loading || self.jobs.busy() > 0 {
                    ui.add_space(8.0);
                    ui.add(egui::Spinner::new().color(theme::MUTED));
                }
            });
        });
    }

    // ---- sidebar ----------------------------------------------------------------

    fn sidebar(&mut self, ui: &mut egui::Ui, i: usize, s: &mut Settings) {
        if self.editor.crop.is_some() {
            self.crop_panel(ui, i);
        }
        widgets::histogram(
            ui,
            &self.editor.hist,
            vec2(ui.available_width() - 4.0, 64.0),
        );
        ui.add_space(10.0);
        // Unconverted frames: only the Convert tab is available. Once the
        // conversion arrives, unlock and move on to Edit.
        let converted = self.frames[i].sc.is_converted();
        match (self.editor.was_converted, converted) {
            (_, false) => self.editor.tab = Tab::Convert,
            (Some(false), true) => self.editor.tab = Tab::Edit,
            _ => {}
        }
        self.editor.was_converted = Some(converted);
        let locked: &[Tab] = if converted {
            &[]
        } else {
            &[Tab::Edit, Tab::Advanced]
        };
        widgets::segmented_locked(
            ui,
            &mut self.editor.tab,
            &[
                (Tab::Convert, "Convert"),
                (Tab::Edit, "Edit"),
                (Tab::Advanced, "Advanced"),
            ],
            locked,
            "Convert the negative first",
        );
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.set_width(ui.available_width() - 6.0);
                match self.editor.tab {
                    Tab::Convert => self.convert_tab(ui, i, s),
                    Tab::Edit => self.edit_tab(ui, i, s),
                    Tab::Advanced => self.advanced_tab(ui, i, s),
                }
                ui.add_space(16.0);
            });
    }

    fn crop_panel(&mut self, ui: &mut egui::Ui, i: usize) {
        header(ui, "CROP");
        egui::Frame::new()
            .fill(Color32::from_rgb(234, 234, 236))
            .corner_radius(8.0)
            .inner_margin(8.0)
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    if ui.button("Reset Crop").clicked() {
                        if let Some(c) = &mut self.editor.crop {
                            c.insets = [0.0; 4];
                            c.angle = 0.0;
                            self.editor.dirty = true;
                        }
                    }
                    if ui.button("Cancel").on_hover_text("Esc").clicked() {
                        self.cancel_crop();
                    }
                    if ui
                        .add(
                            egui::Button::new(RichText::new("Apply").color(Color32::WHITE))
                                .fill(theme::ACCENT),
                        )
                        .on_hover_text("Enter")
                        .clicked()
                    {
                        self.apply_crop(i);
                    }
                });
            });
        if let Some(c) = &mut self.editor.crop {
            ui.add_space(6.0);
            let mut a = c.angle;
            if fl_slider(
                ui,
                "Straighten",
                &mut a,
                -45.0..=45.0,
                0.0,
                1,
                "Fine rotation in degrees",
                None,
            ) {
                c.angle = a;
                self.editor.dirty = true;
            }
            ui.label(
                RichText::new(
                    "Drag the frame edges or corners on the image; drag inside to move it.",
                )
                .small()
                .color(theme::MUTED),
            );
        }
        ui.add_space(6.0);
        ui.separator();
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
            (false, _) => "Convert Negative",
            (true, true) => "Re-convert (settings changed)",
            (true, false) => "Re-convert",
        };
        let primary = !converted || stale;
        let btn = if primary {
            egui::Button::new(RichText::new(label).strong().color(Color32::WHITE))
                .fill(theme::ACCENT)
        } else {
            egui::Button::new(label)
        };
        if ui
            .add_sized([ui.available_width(), 32.0], btn)
            .on_hover_text(
                "Analyse this frame on its own (rolls are converted from the Folder View)",
            )
            .clicked()
        {
            self.frames[i].sc.settings = s.clone();
            self.convert(vec![i], false);
        }
    }

    fn convert_tab(&mut self, ui: &mut egui::Ui, i: usize, s: &mut Settings) {
        if widgets::fold(ui, "PROCESS") {
            if fl_combo(
                ui,
                "",
                &mut s.image_type,
                ImageType::ALL,
                ImageType::label,
                "",
            ) {
                if s.image_type == ImageType::Negative {
                    let d = Settings::default();
                    (s.tone_profile, s.wb_mode, s.correction_type) =
                        (d.tone_profile, d.wb_mode, d.correction_type);
                } else {
                    s.apply_positive_defaults();
                }
            }
        }
        if widgets::fold(ui, "SOURCE") {
            if fl_combo(
                ui,
                "Scan source",
                &mut s.source,
                Source::ALL,
                Source::label,
                "",
            ) {
                s.film_base = match s.source {
                    Source::TifScan | Source::LinearScan => FilmBase::Off,
                    _ => FilmBase::Auto,
                };
            }
            if fl_combo(
                ui,
                "Color Model",
                &mut s.color_model,
                ColorModel::ALL,
                ColorModel::label,
                "Basic: neutral · Frontier: teal-blues, golden yellows, warm skin · Noritsu: cleaner and cooler · B+W: monochrome · None: no calibration",
            ) && s.color_model == ColorModel::BlackAndWhite
            {
                s.apply_mono_defaults();
            }
            ui.label(RichText::new("Camera profile").color(theme::TEXT));
            match self
                .editor
                .src
                .as_ref()
                .map(|src| (src.is_raw, pipeline::camera_profile(src)))
            {
                Some((_, Some(p))) => {
                    ui.label(RichText::new(format!("{} · {}", p.name, p.camera)).color(theme::OK))
                        .on_hover_text(p.path.display().to_string());
                }
                Some((true, None)) => {
                    ui.label(
                        RichText::new(
                            "No Negative Lab profile for this camera — using the camera matrix",
                        )
                        .color(theme::WARN),
                    );
                }
                Some((false, None)) => {
                    ui.label(RichText::new("Not used for TIF / JPEG scans").color(theme::MUTED));
                }
                None => {
                    ui.label(RichText::new("…").color(theme::MUTED));
                }
            }
            ui.add_space(4.0);
            fl_combo(
                ui,
                "Film base",
                &mut s.film_base,
                FilmBase::ALL,
                FilmBase::label,
                "Neutralises the orange mask before analysis",
            );
            let picking = self.editor.pick == Some(Pick::FilmBase);
            ui.horizontal(|ui| {
                if icon_button(
                    ui,
                    Icon::Eyedropper,
                    "Sample the unexposed film border",
                    picking,
                    true,
                )
                .clicked()
                {
                    self.editor.pick = if picking { None } else { Some(Pick::FilmBase) };
                }
                ui.label(RichText::new("Sample film border").color(theme::MUTED));
            });
        }
        if widgets::fold(ui, "ANALYSIS") {
            fl_slider_u8(
                ui,
                "Pre-Saturation",
                &mut s.pre_saturation,
                1..=5,
                3,
                "1–2: fewer distinct hues, warmer · 4–5: more separation, cooler",
            );
            fl_slider(
                ui,
                "Border Buffer",
                &mut s.border_buffer,
                0.0..=25.0,
                5.0,
                0,
                "Percent of the frame edge ignored by the analysis",
                None,
            );
            ui.add_space(8.0);
            self.convert_button(ui, i, s);
            if self.frames[i].sc.is_converted()
                && ui
                    .add_sized([ui.available_width(), 26.0], egui::Button::new("Unconvert"))
                    .clicked()
            {
                self.unconvert(&[i]);
            }
            ui.add_space(10.0);
            ui.label(
                RichText::new(source_instructions(s.source))
                    .small()
                    .color(theme::MUTED),
            );
        }
    }

    fn edit_tab(&mut self, ui: &mut egui::Ui, i: usize, s: &mut Settings) {
        if !self.frames[i].sc.is_converted() {
            ui.add_space(10.0);
            ui.label(RichText::new("This frame is not converted yet.").color(theme::WARN));
            self.convert_button(ui, i, s);
        }
        if widgets::fold(ui, "TONES") {
            fl_combo(
                ui,
                "",
                &mut s.tone_profile,
                ToneProfile::ALL,
                ToneProfile::label,
                "",
            );
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
                "Print exposure in stops, like changing the print time in a darkroom: + lighter (highlights fade to paper white), − darker (shadows block up)",
                None,
            );
            tone(
                ui,
                "Brightness",
                &mut s.brightness,
                "Midtone gamma: lower = denser, higher = airy",
            );
            tone(ui, "Contrast", &mut s.contrast, "Midtone contrast slope");
            tone(ui, "Lights", &mut s.highlights, "Upper midtones");
            tone(ui, "Darks", &mut s.shadows, "Lower midtones");
            tone(ui, "Whites", &mut s.whites, "White point roll-off");
            tone(
                ui,
                "Blacks",
                &mut s.blacks,
                "Black point: up = matte, down = deeper",
            );
            tone(ui, "Lab Glow", &mut s.glow, "Silky compressed highlights");
            tone(ui, "Lab Fade", &mut s.fade, "Open, compressed deep shadows");
        }
        if widgets::fold(ui, "DYNAMIC RANGE") {
            fl_slider(
                ui,
                "WhiteClip",
                &mut s.white_clip,
                -50.0..=50.0,
                0.0,
                0,
                "Negative values recover clipped highlights",
                None,
            );
            fl_slider(
                ui,
                "BlackClip",
                &mut s.black_clip,
                -50.0..=50.0,
                0.0,
                0,
                "Negative values recover crushed shadows",
                None,
            );
            ui.horizontal(|ui| {
                ui.checkbox(&mut s.soft_highlights, "Soft Highs");
                ui.add_space(16.0);
                ui.checkbox(&mut s.soft_shadows, "Soft Lows");
            });
        }
        if widgets::fold(ui, "WHITE BALANCE") {
            let converted = self.frames[i].sc.is_converted();
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                let picking = self.editor.pick == Some(Pick::WhiteBalance);
                if icon_button(
                    ui,
                    Icon::Eyedropper,
                    "Pick a neutral grey in the image (W)",
                    picking,
                    converted,
                )
                .clicked()
                {
                    self.editor.pick = if picking {
                        None
                    } else {
                        Some(Pick::WhiteBalance)
                    };
                }
                ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                    fl_combo(ui, "", &mut s.wb_mode, WbMode::ALL, WbMode::label, "");
                });
            });
            let mut t = s.wb_temp;
            if fl_slider(
                ui,
                "Temp",
                &mut t,
                -75.0..=75.0,
                0.0,
                2,
                "Blue ↔ Yellow — most film conversions need some warmth",
                Some(grad::TEMP),
            ) {
                s.wb_temp = t;
                s.wb_mode = WbMode::Custom;
            }
            let mut k = s.wb_tint;
            if fl_slider(
                ui,
                "Tint",
                &mut k,
                -75.0..=75.0,
                0.0,
                2,
                "Green ↔ Magenta",
                Some(grad::TINT),
            ) {
                s.wb_tint = k;
                s.wb_mode = WbMode::Custom;
            }
        }
        if widgets::fold(ui, "COLOR TONING") {
            segmented(
                ui,
                &mut self.editor.color_tab,
                &[
                    (ColorTab::Mids, "Mids"),
                    (ColorTab::Highs, "Highs"),
                    (ColorTab::Shadows, "Shadows"),
                ],
            );
            ui.add_space(4.0);
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
                "Red ↔ Cyan",
                c,
                -75.0..=75.0,
                0.0,
                0,
                "",
                Some(grad::RED_CYAN),
            );
            fl_slider(
                ui,
                "Green ↔ Magenta",
                m,
                -75.0..=75.0,
                0.0,
                0,
                "",
                Some(grad::GREEN_MAGENTA),
            );
            fl_slider(
                ui,
                "Blue ↔ Yellow",
                y,
                -75.0..=75.0,
                0.0,
                0,
                "",
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
        }
        if widgets::fold(ui, "LUT") {
            fl_combo(ui, "", &mut s.lut, Lut::ALL, Lut::label, "");
            fl_slider(
                ui,
                "Amount",
                &mut s.lut_strength,
                0.0..=200.0,
                100.0,
                0,
                "LUT strength in percent",
                None,
            );
            fl_slider_u8(
                ui,
                "Saturation",
                &mut s.post_saturation,
                1..=10,
                5,
                "LUT saturation variant (5 = default)",
            );
            if s.lut != Lut::None && !self.editor.uses_table && !s.is_monochrome() {
                ui.label(
                    RichText::new(
                        "Plugin LUT folder not found — using approximations (see Advanced).",
                    )
                    .small()
                    .color(theme::WARN),
                );
            }
        }
        if widgets::fold(ui, "SHARPENING") {
            fl_combo(
                ui,
                "",
                &mut s.sharpening,
                Sharpening::ALL,
                Sharpening::label,
                "",
            );
        }
        if widgets::fold(ui, "DEFAULTS") {
            ui.horizontal(|ui| {
                if ui
                    .button("Save as Default")
                    .on_hover_text("New frames start with these settings")
                    .clicked()
                {
                    let mut d = s.clone();
                    (d.crop, d.rotation, d.flip_h, d.straighten, d.use_roll) =
                        ([0.0; 4], 0, false, 0.0, false);
                    self.defaults.settings = d;
                    let _ = crate::engine::sidecar::save_defaults(&self.defaults);
                    self.toast("Saved as default settings", false);
                }
                if ui.button("Load Default").clicked() {
                    let keep = s.clone();
                    s.copy_edit_from(&self.defaults.settings);
                    (
                        s.source,
                        s.image_type,
                        s.color_model,
                        s.pre_saturation,
                        s.border_buffer,
                        s.use_roll,
                    ) = (
                        keep.source,
                        keep.image_type,
                        keep.color_model,
                        keep.pre_saturation,
                        keep.border_buffer,
                        keep.use_roll,
                    );
                }
                if ui.button("Reset").clicked() {
                    let keep = s.clone();
                    *s = Settings::default();
                    (
                        s.source,
                        s.image_type,
                        s.color_model,
                        s.pre_saturation,
                        s.border_buffer,
                        s.film_base,
                        s.film_base_sample,
                    ) = (
                        keep.source,
                        keep.image_type,
                        keep.color_model,
                        keep.pre_saturation,
                        keep.border_buffer,
                        keep.film_base,
                        keep.film_base_sample,
                    );
                    (s.rotation, s.flip_h, s.straighten, s.crop, s.use_roll) = (
                        keep.rotation,
                        keep.flip_h,
                        keep.straighten,
                        keep.crop,
                        keep.use_roll,
                    );
                    s.apply_engine_preset(keep.engine);
                }
            });
        }
    }

    fn advanced_tab(&mut self, ui: &mut egui::Ui, i: usize, s: &mut Settings) {
        if widgets::fold(ui, "ENGINE") {
            fl_combo(
                ui,
                "Engine preset",
                &mut s.engine,
                EnginePreset::ALL,
                EnginePreset::label,
                "",
            );
            fl_combo(
                ui,
                "Curve points",
                &mut s.curve_points,
                CurvePoints::ALL,
                CurvePoints::label,
                "Auto: by tonal range · Smooth: 4 · Precise: 14 · Manual: 2–14",
            );
            if s.curve_points == CurvePoints::Manual {
                fl_slider_u8(ui, "Points", &mut s.curve_resolution, 2..=14, 7, "");
            }
            fl_combo(
                ui,
                "Processing",
                &mut s.correction_type,
                CorrectionType::ALL,
                CorrectionType::label,
                "",
            );
            fl_combo(
                ui,
                "Order",
                &mut s.layer_order,
                LayerOrder::ALL,
                LayerOrder::label,
                "Colors First keeps colour stable while editing tones",
            );
            fl_combo(
                ui,
                "WB density",
                &mut s.wb_tonality,
                WbTonality::ALL,
                WbTonality::label,
                "",
            );
            fl_combo(
                ui,
                "WB type",
                &mut s.wb_method,
                WbMethod::ALL,
                WbMethod::label,
                "",
            );
            fl_combo(
                ui,
                "Toning",
                &mut s.toning_method,
                ToningMethod::ALL,
                ToningMethod::label,
                "",
            );
            fl_combo(
                ui,
                "Clipping",
                &mut s.clip_method,
                ClipMethod::ALL,
                ClipMethod::label,
                "",
            );
        }
        if widgets::fold(ui, "CURVES") {
            if self.editor.curves.is_empty() {
                ui.label(RichText::new("Convert the frame to see its curves.").color(theme::MUTED));
            } else {
                widgets::curves_plot(ui, &self.editor.curves, vec2(ui.available_width(), 150.0));
            }
            let f = &self.frames[i];
            let o = if s.use_roll {
                f.roll_origins().copied()
            } else {
                f.sc.analysis.as_ref().map(|a| a.origins)
            };
            if let Some(o) = o {
                ui.add_space(6.0);
                egui::Grid::new("origins")
                    .num_columns(4)
                    .spacing([18.0, 2.0])
                    .show(ui, |ui| {
                        for h in ["", "White", "Black", "Mean"] {
                            ui.label(RichText::new(h).small().color(theme::MUTED));
                        }
                        ui.end_row();
                        for (c, n) in ["R", "G", "B"].iter().enumerate() {
                            ui.label(*n);
                            ui.label(format!("{:.0}", o[c].white));
                            ui.label(format!("{:.0}", o[c].black));
                            ui.label(format!("{:.3}", o[c].mean));
                            ui.end_row();
                        }
                    });
                if f.sc.roll.is_some() {
                    ui.checkbox(&mut s.use_roll, "Use roll analysis");
                }
            }
        }
        if widgets::fold(ui, "LUT FOLDER") {
            lut_folder_ui(self, ui);
        }
    }

    // ---- canvas -----------------------------------------------------------------

    fn canvas(&mut self, ui: &mut egui::Ui, i: usize) {
        let avail = ui.max_rect();
        let Some(tex) = self.editor.tex.clone() else {
            ui.centered_and_justified(|ui| {
                ui.add(egui::Spinner::new().size(28.0).color(Color32::WHITE));
            });
            return;
        };
        let cropping = self.editor.crop.is_some();
        let margin = if cropping {
            vec2(48.0, 110.0)
        } else {
            vec2(32.0, 32.0)
        };
        let ts = tex.size_vec2();
        let fit = ((avail.width() - margin.x) / ts.x)
            .min((avail.height() - margin.y) / ts.y)
            .max(0.01);
        let center = if cropping {
            avail.center() - vec2(0.0, 30.0)
        } else {
            avail.center()
        };
        let now = ui.input(|inp| inp.time);
        let max_pan = |z: f32| ((ts * z - avail.size()) * 0.5).max(egui::Vec2::ZERO);
        // Turn a zoom request into an animation from the current view.
        if let Some(req) = self.editor.zoom_req.take() {
            let cur_z = self.editor.zoom.unwrap_or(fit);
            let cur_pan = self.editor.pan;
            let (to_z, to_pan, to_fit) = match req {
                ZoomReq::Factor(f) => {
                    let z = (cur_z * f).clamp(fit, 8.0);
                    if z <= fit * 1.001 {
                        (fit, egui::Vec2::ZERO, true)
                    } else {
                        (z, cur_pan * (z / cur_z), false)
                    }
                }
                ZoomReq::Toggle(pos) => {
                    if self.editor.zoom.is_some() {
                        (fit, egui::Vec2::ZERO, true)
                    } else {
                        // Keep the clicked image point under the cursor.
                        let z = (fit * 2.5).clamp(fit * 1.5, 4.0f32.max(fit * 1.5));
                        let rect_now = Rect::from_center_size(center + cur_pan, ts * cur_z);
                        let u = (pos - rect_now.min) / cur_z;
                        let pan = pos - center + ts * z * 0.5 - u * z;
                        (z, pan, false)
                    }
                }
            };
            let to_pan = to_pan.clamp(-max_pan(to_z), max_pan(to_z));
            self.editor.zoom_anim = Some(ZoomAnim {
                from_z: cur_z,
                to_z,
                to_fit,
                from_pan: cur_pan,
                to_pan,
                start: now,
            });
        }
        if let Some(a) = self.editor.zoom_anim {
            let t = ((now - a.start) / ZOOM_ANIM_SECS).clamp(0.0, 1.0) as f32;
            let e = 1.0 - (1.0 - t).powi(3);
            if t >= 1.0 {
                self.editor.zoom = if a.to_fit { None } else { Some(a.to_z) };
                self.editor.pan = a.to_pan;
                self.editor.zoom_anim = None;
            } else {
                // Geometric interpolation feels uniform when zooming.
                self.editor.zoom = Some(a.from_z * (a.to_z / a.from_z).powf(e));
                self.editor.pan = a.from_pan + (a.to_pan - a.from_pan) * e;
                ui.ctx().request_repaint();
            }
        } else if let Some(z) = self.editor.zoom {
            if z <= fit {
                self.editor.zoom = None;
                self.editor.pan = egui::Vec2::ZERO;
            }
        }
        let scale = self.editor.zoom.unwrap_or(fit);
        let size = ts * scale;
        // Clamp pan so the image never leaves the canvas entirely.
        let maxp = ((size - avail.size()) * 0.5).max(egui::Vec2::ZERO);
        self.editor.pan = self.editor.pan.clamp(-maxp, maxp);
        let rect = Rect::from_center_size(center + self.editor.pan, size);
        let resp = ui.allocate_rect(avail, Sense::click_and_drag());

        // Zoom with ⌘/Ctrl + scroll or pinch; scroll pans when zoomed.
        if resp.hovered() {
            let (zoom_delta, scroll) = ui.input(|inp| (inp.zoom_delta(), inp.smooth_scroll_delta));
            if zoom_delta != 1.0 {
                self.zoom_by(zoom_delta, fit);
            } else if self.editor.zoom.is_some() && scroll != egui::Vec2::ZERO {
                self.editor.pan += scroll;
            }
        }

        let p = ui.painter_at(avail);
        p.rect_filled(rect.expand(1.0), 0.0, Color32::from_black_alpha(40));
        p.image(
            tex.id(),
            rect,
            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            Color32::WHITE,
        );

        if cropping {
            self.crop_overlay(ui, &resp, rect, avail);
        } else if let Some(pick) = self.editor.pick {
            let label = match pick {
                Pick::WhiteBalance => "Click a neutral grey area · Esc to cancel",
                Pick::FilmBase => "Click the unexposed film border · Esc to cancel",
            };
            pill(&p, pos2(avail.center().x, avail.top() + 18.0), label);
            if resp.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
            }
            if resp.clicked() {
                if let Some(pos) = resp.interact_pointer_pos() {
                    if rect.contains(pos) {
                        let u = (pos.x - rect.left()) / rect.width();
                        let v = (pos.y - rect.top()) / rect.height();
                        self.do_pick(i, pick, u, v);
                    }
                }
                self.editor.pick = None;
            }
        } else {
            if resp.dragged() && self.editor.zoom.is_some() {
                self.editor.zoom_anim = None;
                self.editor.pan += resp.drag_delta();
                ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
            } else if resp.hovered() && self.editor.zoom.is_some() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
            }
            if resp.double_clicked() {
                if let Some(pos) = resp.interact_pointer_pos() {
                    self.editor.zoom_req = Some(ZoomReq::Toggle(pos));
                    ui.ctx().request_repaint();
                }
            }
        }

        if self.editor.before && !cropping {
            pill(&p, pos2(avail.left() + 50.0, avail.top() + 18.0), "BEFORE");
        }
        let zoom_txt = match self.editor.zoom {
            Some(z) => {
                let full = self
                    .editor
                    .src
                    .as_ref()
                    .map(|s| {
                        s.info.full_width.max(s.info.full_height) as f32
                            / s.width.max(s.height) as f32
                    })
                    .unwrap_or(1.0);
                format!("{:.0}%", z / full * 100.0)
            }
            None => "Fit".into(),
        };
        p.text(
            avail.right_bottom() + vec2(-10.0, -8.0),
            egui::Align2::RIGHT_BOTTOM,
            format!("{zoom_txt} · {:.0} ms", self.editor.render_ms),
            egui::FontId::proportional(11.0),
            Color32::from_gray(225),
        );
    }

    fn crop_overlay(&mut self, ui: &mut egui::Ui, resp: &egui::Response, img: Rect, avail: Rect) {
        let Some(mut c) = self.editor.crop else {
            return;
        };
        let to_px = |ins: [f32; 4]| {
            Rect::from_min_max(
                pos2(
                    img.left() + img.width() * ins[0] / 100.0,
                    img.top() + img.height() * ins[1] / 100.0,
                ),
                pos2(
                    img.right() - img.width() * ins[2] / 100.0,
                    img.bottom() - img.height() * ins[3] / 100.0,
                ),
            )
        };
        let r = to_px(c.insets);
        let dial_c = pos2(img.center().x, img.bottom() + 40.0);
        let dial_r = img.width().min(520.0) * 0.45;

        // Hit-testing.
        let hit = |pos: egui::Pos2| -> Option<Handle> {
            let near = |a: f32, b: f32| (a - b).abs() < 12.0;
            let (l, rr, t, b) = (
                near(pos.x, r.left()),
                near(pos.x, r.right()),
                near(pos.y, r.top()),
                near(pos.y, r.bottom()),
            );
            let inx = pos.x > r.left() - 12.0 && pos.x < r.right() + 12.0;
            let iny = pos.y > r.top() - 12.0 && pos.y < r.bottom() + 12.0;
            Some(match () {
                _ if l && t => Handle::TL,
                _ if rr && t => Handle::TR,
                _ if l && b => Handle::BL,
                _ if rr && b => Handle::BR,
                _ if l && iny => Handle::L,
                _ if rr && iny => Handle::R,
                _ if t && inx => Handle::T,
                _ if b && inx => Handle::B,
                _ if r.contains(pos) => Handle::Move,
                _ if pos.y > img.bottom() + 8.0 && (pos.x - dial_c.x).abs() < dial_r => {
                    Handle::Dial
                }
                _ => return None,
            })
        };
        if resp.drag_started() {
            c.drag = resp.interact_pointer_pos().and_then(hit);
        }
        if let (true, Some(h)) = (resp.dragged(), c.drag) {
            let d = resp.drag_delta();
            let (dx, dy) = (d.x / img.width() * 100.0, d.y / img.height() * 100.0);
            let ins = &mut c.insets;
            let (l, t, rr, b) = (ins[0], ins[1], ins[2], ins[3]);
            match h {
                Handle::Move => {
                    let mx = dx.clamp(-l, rr);
                    let my = dy.clamp(-t, b);
                    *ins = [l + mx, t + my, rr - mx, b - my];
                }
                Handle::Dial => c.angle = (c.angle + d.x * 0.08).clamp(-45.0, 45.0),
                _ => {
                    if matches!(h, Handle::L | Handle::TL | Handle::BL) {
                        ins[0] = (l + dx).clamp(0.0, 95.0 - rr);
                    }
                    if matches!(h, Handle::R | Handle::TR | Handle::BR) {
                        ins[2] = (rr - dx).clamp(0.0, 95.0 - l);
                    }
                    if matches!(h, Handle::T | Handle::TL | Handle::TR) {
                        ins[1] = (t + dy).clamp(0.0, 95.0 - b);
                    }
                    if matches!(h, Handle::B | Handle::BL | Handle::BR) {
                        ins[3] = (b - dy).clamp(0.0, 95.0 - t);
                    }
                }
            }
        }
        if resp.drag_stopped() {
            if c.drag == Some(Handle::Dial) {
                self.editor.dirty = true;
            }
            c.drag = None;
        }
        if c.drag == Some(Handle::Dial) || (resp.dragged() && c.drag.is_none()) {
            self.editor.dirty = true;
        }
        if let Some(pos) = resp.hover_pos() {
            let icon = match c.drag.or_else(|| hit(pos)) {
                Some(Handle::L | Handle::R) => egui::CursorIcon::ResizeHorizontal,
                Some(Handle::T | Handle::B) => egui::CursorIcon::ResizeVertical,
                Some(Handle::TL | Handle::BR) => egui::CursorIcon::ResizeNwSe,
                Some(Handle::TR | Handle::BL) => egui::CursorIcon::ResizeNeSw,
                Some(Handle::Move) => egui::CursorIcon::Move,
                Some(Handle::Dial) => egui::CursorIcon::ResizeHorizontal,
                None => egui::CursorIcon::Default,
            };
            ui.ctx().set_cursor_icon(icon);
        }
        let angle_changed = self
            .editor
            .crop
            .map(|o| o.angle != c.angle)
            .unwrap_or(false);
        self.editor.crop = Some(c);
        if angle_changed {
            self.editor.dirty = true;
        }

        // Drawing.
        let r = to_px(c.insets);
        let p = ui.painter_at(avail);
        let shade = Color32::from_black_alpha(120);
        p.rect_filled(
            Rect::from_min_max(img.min, pos2(img.right(), r.top())),
            0.0,
            shade,
        );
        p.rect_filled(
            Rect::from_min_max(pos2(img.left(), r.bottom()), img.max),
            0.0,
            shade,
        );
        p.rect_filled(
            Rect::from_min_max(pos2(img.left(), r.top()), pos2(r.left(), r.bottom())),
            0.0,
            shade,
        );
        p.rect_filled(
            Rect::from_min_max(pos2(r.right(), r.top()), pos2(img.right(), r.bottom())),
            0.0,
            shade,
        );
        p.rect_stroke(
            r,
            0.0,
            Stroke::new(1.5, Color32::WHITE),
            egui::StrokeKind::Middle,
        );
        let thirds = Stroke::new(0.6, Color32::from_white_alpha(110));
        for k in 1..3 {
            let fx = r.left() + r.width() * k as f32 / 3.0;
            let fy = r.top() + r.height() * k as f32 / 3.0;
            p.line_segment([pos2(fx, r.top()), pos2(fx, r.bottom())], thirds);
            p.line_segment([pos2(r.left(), fy), pos2(r.right(), fy)], thirds);
        }
        let hs = Stroke::new(4.0, Color32::WHITE);
        let len = 22.0f32.min(r.width() / 3.0).min(r.height() / 3.0);
        for (corner, sx, sy) in [
            (r.left_top(), 1.0, 1.0),
            (r.right_top(), -1.0, 1.0),
            (r.left_bottom(), 1.0, -1.0),
            (r.right_bottom(), -1.0, -1.0),
        ] {
            p.line_segment([corner, corner + vec2(len * sx, 0.0)], hs);
            p.line_segment([corner, corner + vec2(0.0, len * sy)], hs);
        }

        // Straighten dial: an arc with ticks; the angle indicator sits on it.
        let span = 0.55f32; // radians either side
        let arc_c = pos2(dial_c.x, dial_c.y - dial_r);
        let pt = |a: f32| pos2(arc_c.x + dial_r * a.sin(), arc_c.y + dial_r * a.cos());
        let pts: Vec<egui::Pos2> = (0..=48)
            .map(|k| pt(-span + 2.0 * span * k as f32 / 48.0))
            .collect();
        p.add(egui::Shape::line(pts, Stroke::new(1.5, Color32::WHITE)));
        for k in -3..=3 {
            let a = span * k as f32 / 3.0;
            let (o, n) = (pt(a), pt(a) - (pt(a) - arc_c).normalized() * 7.0);
            p.line_segment([o, n], Stroke::new(1.2, Color32::WHITE));
        }
        let ka = (c.angle / 45.0) * span;
        p.circle(pt(ka), 6.0, theme::ACCENT, Stroke::new(1.5, Color32::WHITE));
        p.text(
            dial_c + vec2(0.0, 14.0),
            egui::Align2::CENTER_TOP,
            format!("{:+.1}°", c.angle),
            egui::FontId::proportional(12.5),
            Color32::WHITE,
        );
    }

    fn do_pick(&mut self, i: usize, pick: Pick, u: f32, v: f32) {
        let before = self.frames[i].sc.settings.clone();
        let mut s = before.clone();
        match pick {
            Pick::WhiteBalance => {
                let Some(neg) = &self.editor.neg else { return };
                let f = &self.frames[i];
                let roll = f.sc.roll.as_ref().map(|r| r.origins);
                let Some(origins) =
                    pipeline::origins_for(&s, f.sc.analysis.as_ref(), roll.as_ref()).copied()
                else {
                    self.toast("Convert the negative first, then pick a neutral.", true);
                    return;
                };
                let avg = sample(&neg.data, neg.width, neg.height, u, v, 3);
                let lut = crate::engine::curve::CurveLut::from_curves(
                    &crate::engine::curve::update_tone_curve(&origins, &s),
                );
                let pos = [0, 1, 2].map(|c| lut.apply(c, avg[c]) as f64);
                match crate::engine::curve::eyedropper_wb(pos, &origins, &s) {
                    Ok((t, n)) => {
                        s.wb_temp = (t * 100.0).round() / 100.0;
                        s.wb_tint = (n * 100.0).round() / 100.0;
                        s.wb_mode = WbMode::Custom;
                    }
                    Err(msg) => {
                        self.toast(msg, true);
                        return;
                    }
                }
            }
            Pick::FilmBase => {
                let Some(src) = &self.editor.src else { return };
                let (d, w, h) = decode::geometry(
                    &src.data,
                    src.width,
                    src.height,
                    s.rotation,
                    s.flip_h,
                    s.straighten,
                    s.crop,
                );
                s.film_base_sample = sample(&d, w, h, u, v, 4);
                s.film_base = FilmBase::Sampled;
            }
        }
        self.apply_change(i, &before, s);
    }
}

/// Rounded dark label on the canvas.
fn pill(p: &egui::Painter, center: egui::Pos2, text: &str) {
    let g = p.layout_no_wrap(
        text.to_string(),
        egui::FontId::proportional(13.0),
        Color32::WHITE,
    );
    let r = Rect::from_center_size(center, g.size() + vec2(20.0, 10.0));
    p.rect_filled(r, 12.0, Color32::from_black_alpha(150));
    p.galley(r.center() - g.size() / 2.0, g, Color32::WHITE);
}

/// Average a (2r+1)² neighbourhood at normalised coordinates.
fn sample(d: &[[f32; 3]], w: usize, h: usize, u: f32, v: f32, r: isize) -> [f32; 3] {
    let cx = (u.clamp(0.0, 1.0) * (w as f32 - 1.0)).round() as isize;
    let cy = (v.clamp(0.0, 1.0) * (h as f32 - 1.0)).round() as isize;
    let mut acc = [0.0f32; 3];
    let mut n: f32 = 0.0;
    for y in (cy - r).max(0)..=(cy + r).min(h as isize - 1) {
        for x in (cx - r).max(0)..=(cx + r).min(w as isize - 1) {
            let p = d[y as usize * w + x as usize];
            for c in 0..3 {
                acc[c] += p[c];
            }
            n += 1.0;
        }
    }
    acc.map(|v| v / n.max(1.0))
}
