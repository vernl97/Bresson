//! Editor View: fine-tune one frame. Layout: a light toolbar (navigation,
//! zoom, history on the left; title centred; geometry and export on the
//! right), a grey canvas with zoom / pan and an interactive crop mode, and a
//! sidebar holding the plugin's Convert / Edit / Advanced controls.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use eframe::egui::{self, Color32, Rect, RichText, Sense, Stroke, pos2, vec2};

use super::folder::source_instructions;
use super::widgets::{
    self, Icon, fl_combo, fl_slider, fl_slider_u8, grad, icon_button, pill, segmented, theme,
};
use super::{NlpApp, View, gpu, to_color_image};
use crate::engine::decode::{self, SourceImage};
use crate::engine::pipeline::{self, Negative, Renderer};
use crate::engine::settings::*;

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
pub(super) struct CropEdit {
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
    /// Go to this display scale (e.g. 100 %), around the canvas centre.
    Absolute(f32),
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
    /// CPU-rendered preview (without wgpu).
    tex: Option<egui::TextureHandle>,
    /// Size of the shown negative in pixels; `None` until the first render.
    size: Option<[usize; 2]>,
    /// GPU preview: the negative, and how to draw it and its "before" side.
    gpu_neg: Option<Arc<gpu::NegData>>,
    gpu_view: Option<Arc<gpu::ViewData>>,
    gpu_before: Option<Arc<gpu::ViewData>>,
    dirty: bool,
    pub before: bool,
    color_tab: ColorTab,
    pick: Option<Pick>,
    committed: Option<Settings>,
    hist: [[u32; 256]; 3],
    curves: Vec<[f32; 4]>,
    /// Display scale (screen points per preview pixel); `None` = fit.
    zoom: Option<f32>,
    pan: egui::Vec2,
    pub(super) crop: Option<CropEdit>,
    pub export_open: bool,
    zoom_req: Option<ZoomReq>,
    zoom_anim: Option<ZoomAnim>,
    /// The Border buffer slider is held: outline the analysis area.
    border_preview: bool,
    /// The rendered preview (display sRGB, 0..1) and its size, for the
    /// pixel readout and clipping view (a 1024 px render when the preview
    /// is drawn on the GPU).
    px: Vec<[f32; 3]>,
    px_size: [usize; 2],
    /// Paint clipped highlights red and blocked shadows blue (J).
    pub clip_view: bool,
    clip_tex: Option<egui::TextureHandle>,
    /// Before/after split view (⇧B), the divider at `split_pos` (0..1).
    pub split: bool,
    split_pos: f32,
    before_tex: Option<egui::TextureHandle>,
    /// The thumbnail strip under the canvas (F).
    pub filmstrip: bool,
}

impl Editor {
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
            size: None,
            gpu_neg: None,
            gpu_view: None,
            gpu_before: None,
            dirty: true,
            before: false,
            color_tab: ColorTab::Mids,
            pick: None,
            committed: None,
            hist: [[0; 256]; 3],
            curves: Vec::new(),
            zoom: None,
            pan: egui::Vec2::ZERO,
            crop: None,
            export_open: false,
            zoom_req: None,
            zoom_anim: None,
            border_preview: false,
            px: Vec::new(),
            px_size: [0, 0],
            clip_view: false,
            clip_tex: None,
            split: false,
            split_pos: 0.5,
            before_tex: None,
            filmstrip: true,
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
        let (keep, strip) = (self.color_tab, self.filmstrip);
        *self = Editor {
            path: Some(path.to_path_buf()),
            loading: true,
            committed: Some(settings),
            ..Default::default()
        };
        self.color_tab = keep;
        self.filmstrip = strip;
    }

    /// The settings an undo step brought back: the base for the next one.
    pub fn set_committed(&mut self, s: Settings) {
        self.committed = Some(s);
        self.invalidate();
    }

    /// Force the negative to be re-prepared and the preview re-rendered.
    pub fn invalidate(&mut self) {
        self.neg_key = None;
        self.dirty = true;
    }
}

/// Drag distance in points for one lab correction click.
const LAB_DRAG_PX: f32 = 12.0;

/// Height of the editor's filmstrip.
const FILMSTRIP_H: f32 = 84.0;

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
    fn apply_change(&mut self, i: usize, before: &Settings, s: Settings) {
        if s == *before {
            return;
        }
        let wb_changed = s.wb_mode != before.wb_mode;
        self.frames[i].sc.settings = s;
        if wb_changed {
            self.frames[i].apply_wb_mode();
        }
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
                let step = vec![(self.frames[i].path.clone(), c.clone())];
                self.editor.committed = Some(cur.clone());
                self.history.push(step);
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

    /// Previous / next frame, wrapping around, through every frame in the
    /// folder (converted or not).
    fn goto(&mut self, i: usize, delta: isize) {
        let n = self.frames.len() as isize;
        if n < 2 {
            return;
        }
        let j = (i as isize + delta.signum()).rem_euclid(n);
        self.save_frame(i);
        self.selected.clear();
        self.selected.insert(j as usize);
        self.open_editor(j as usize);
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

    /// (Re)render the preview if needed. With wgpu the full-size preview is
    /// drawn by the GPU on every frame, and the CPU only renders a 1024 px
    /// copy for the histogram, readout and clipping view; without it the
    /// CPU renders the preview itself (from the 1024 px copy while a slider
    /// is dragged).
    fn update_preview(&mut self, ctx: &egui::Context, i: usize, dragging: bool) {
        let Some(src) = self.editor.src.clone() else {
            return;
        };
        if !self.editor.dirty && self.editor.size.is_some() {
            return;
        }
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
            self.editor.gpu_neg = self.gpu.then(|| gpu::NegData::new(&neg));
            self.editor.neg = Some(neg);
            self.editor.neg_key = Some(key);
        }
        let full = self.editor.neg.as_ref().unwrap();
        let small = self.editor.neg_small.as_ref().unwrap();
        let use_small = !self.gpu && dragging && full.width.max(full.height) > 1100;
        let cpu_neg = if self.gpu || use_small { small } else { full };
        let origins = if self.editor.before {
            None
        } else {
            pipeline::levels_for(&s, f.sc.analysis.as_ref(), f.sc.roll.as_ref())
        };
        let mp = (src.info.full_width * src.info.full_height) as f32 / 1e6;
        let r = Renderer::new(&s, origins.as_ref(), OutputSpace::Srgb).with_megapixels(mp);
        let full_long = src.info.full_width.max(src.info.full_height).max(1) as f32;
        let crop_frac = 1.0 - (s.crop[0] + s.crop[2]).max(s.crop[1] + s.crop[3]) / 100.0;
        let scale = cpu_neg.width.max(cpu_neg.height) as f32 / (full_long * crop_frac.max(0.1));
        let px = r.render(cpu_neg, scale.min(1.0));
        let mut hist = [[0u32; 256]; 3];
        let step = if self.gpu { 1 } else { 3 };
        for p in px.iter().step_by(step) {
            for c in 0..3 {
                hist[c][(p[c].clamp(0.0, 1.0) * 255.0) as usize] += 1;
            }
        }
        self.editor.hist = hist;
        self.editor.curves = r.curve_lut().map(|l| l.sample(96)).unwrap_or_default();
        let (w, h) = (cpu_neg.width, cpu_neg.height);
        if self.gpu {
            self.editor.gpu_view = Some(gpu::ViewData::new(&r));
            self.editor.tex = None;
        } else {
            set_texture(
                ctx,
                &mut self.editor.tex,
                "editor-preview",
                to_color_image(&px, w, h),
            );
        }
        let cropping = self.editor.crop.is_some();
        if self.editor.clip_view && !self.editor.before && !cropping {
            let img = clip_image(&px, w, h);
            set_texture(ctx, &mut self.editor.clip_tex, "editor-clipping", img);
        } else {
            self.editor.clip_tex = None;
        }
        self.editor.gpu_before = None;
        self.editor.before_tex = None;
        if self.editor.split && !self.editor.before && !cropping {
            let before = Renderer::new(&s, None, OutputSpace::Srgb).with_megapixels(mp);
            if self.gpu {
                self.editor.gpu_before = Some(gpu::ViewData::new(&before));
            } else {
                let img = to_color_image(&before.render(cpu_neg, scale.min(1.0)), w, h);
                set_texture(ctx, &mut self.editor.before_tex, "editor-before", img);
            }
        }
        self.editor.size = Some([full.width, full.height]);
        self.editor.px = px;
        self.editor.px_size = [w, h];
        self.editor.dirty = use_small;
    }

    /// Zoom by `factor` keeping the image point under `rel` (cursor relative
    /// to the canvas centre) fixed.
    fn zoom_by(&mut self, factor: f32, fit: f32, rel: egui::Vec2) {
        self.editor.zoom_anim = None;
        let cur = self.editor.zoom.unwrap_or(fit);
        let next = (cur * factor).clamp(fit, 8.0f32.max(fit));
        if next <= fit * 1.001 && factor < 1.0 {
            self.editor.zoom = None;
            self.editor.pan = egui::Vec2::ZERO;
        } else {
            let k = next / cur;
            self.editor.pan = (self.editor.pan - rel) * k + rel;
            self.editor.zoom = Some(next);
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

    /// Propose the detected frame as the pending crop.
    fn detect_frame(&mut self, i: usize) {
        let Some(src) = self.editor.src.clone() else {
            return;
        };
        let f = &self.frames[i];
        match crate::engine::frame::detect(&src, &f.sc.settings, f.base_for(&src)) {
            Some(d) => {
                if let Some(c) = &mut self.editor.crop {
                    (c.insets, c.angle) = (d.crop, d.straighten);
                    self.editor.dirty = true;
                }
            }
            None => self.toast("No frame edge found", false),
        }
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
            .exact_size(44.0)
            .frame(
                egui::Frame::new()
                    .fill(theme::PANEL)
                    .inner_margin(egui::Margin::symmetric(10, 0)),
            )
            .show(ui, |ui| self.toolbar(ui, i, &mut s));

        // The toolbar may have left the editor (back) or switched frames.
        if self.view != View::Editor || self.editor_index() != Some(i) {
            return;
        }
        egui::Panel::right("editor_sidebar")
            .resizable(true)
            .default_size(340.0)
            .min_size(300.0)
            .max_size(480.0)
            .frame(
                egui::Frame::new()
                    .fill(theme::PANEL)
                    .inner_margin(egui::Margin {
                        left: 16,
                        right: 12,
                        top: 12,
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

        if self.editor.filmstrip && self.frames.len() > 1 && self.editor.crop.is_none() {
            egui::Panel::bottom("editor_filmstrip")
                .exact_size(FILMSTRIP_H)
                .frame(
                    egui::Frame::new()
                        .fill(theme::PANEL)
                        .inner_margin(egui::Margin::symmetric(8, 6)),
                )
                .show(ui, |ui| self.filmstrip(ui, i));
            if self.editor_index() != Some(i) {
                return;
            }
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(theme::CANVAS))
            .show(ui, |ui| self.canvas(ui, i));
    }

    /// Thumbnails of every frame; a click opens that frame.
    fn filmstrip(&mut self, ui: &mut egui::Ui, i: usize) {
        let h = FILMSTRIP_H - 12.0;
        let mut open = None;
        let scroll_id = ui.id().with("filmstrip_scrolled_to");
        let scrolled_to: Option<usize> = ui.data(|d| d.get_temp(scroll_id));
        egui::ScrollArea::horizontal()
            .auto_shrink([false, false])
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;
                    for (j, f) in self.frames.iter().enumerate() {
                        let w = f
                            .thumb
                            .as_ref()
                            .map_or(h * 1.5, |t| t.size_vec2().x / t.size_vec2().y * h);
                        let (rect, resp) = ui.allocate_exact_size(vec2(w, h), egui::Sense::click());
                        if j == i && scrolled_to != Some(i) {
                            resp.scroll_to_me(Some(egui::Align::Center));
                        }
                        if !ui.is_rect_visible(rect) {
                            continue;
                        }
                        match &f.thumb {
                            Some(t) => {
                                egui::Image::new((t.id(), t.size_vec2()))
                                    .corner_radius(2.0)
                                    .paint_at(ui, rect);
                            }
                            None => {
                                ui.painter().rect_filled(rect, 2.0, egui::Color32::from_white_alpha(8));
                            }
                        }
                        if j == i {
                            ui.painter().rect_stroke(
                                rect.expand(2.0),
                                3.0,
                                egui::Stroke::new(2.0, theme::ACCENT),
                                egui::StrokeKind::Outside,
                            );
                        } else if resp.hovered() {
                            ui.painter().rect_filled(rect, 2.0, egui::Color32::from_white_alpha(20));
                        }
                        if resp.on_hover_text(&f.name).clicked() && j != i {
                            open = Some(j);
                        }
                    }
                });
            });
        ui.data_mut(|d| d.insert_temp(scroll_id, i));
        if let Some(j) = open {
            self.save_frame(i);
            self.selected.clear();
            self.selected.insert(j);
            self.open_editor(j);
        }
    }

    /// Returns true when the view changed and drawing should stop.
    fn editor_shortcuts(&mut self, ctx: &egui::Context, i: usize) -> bool {
        if ctx.egui_wants_keyboard_input() {
            return false;
        }
        let mac = ctx.os() == egui::os::OperatingSystem::Mac;
        let k = ctx.input(|inp| {
            let cmd = inp.modifiers.command;
            [
                inp.key_pressed(egui::Key::B) && !inp.modifiers.shift,
                cmd && !inp.modifiers.shift && inp.key_pressed(egui::Key::Z),
                (cmd && inp.modifiers.shift && inp.key_pressed(egui::Key::Z))
                    || (cmd && !mac && inp.key_pressed(egui::Key::Y)),
                inp.key_pressed(egui::Key::ArrowLeft),
                inp.key_pressed(egui::Key::ArrowRight),
                inp.key_pressed(egui::Key::Escape),
                inp.key_pressed(egui::Key::W),
                inp.key_pressed(egui::Key::Space),
                inp.key_pressed(egui::Key::Plus) || inp.key_pressed(egui::Key::Equals),
                inp.key_pressed(egui::Key::Minus),
                inp.key_pressed(egui::Key::Enter),
                inp.key_pressed(egui::Key::C) && !cmd,
                inp.key_pressed(egui::Key::B) && inp.modifiers.shift,
                inp.key_pressed(egui::Key::J) && !cmd,
                cmd && inp.key_pressed(egui::Key::C),
                cmd && !inp.modifiers.shift && inp.key_pressed(egui::Key::V),
                cmd && inp.modifiers.shift && inp.key_pressed(egui::Key::V),
                cmd && inp.key_pressed(egui::Key::OpenBracket),
                cmd && inp.key_pressed(egui::Key::CloseBracket),
                !cmd && inp.key_pressed(egui::Key::H),
                !cmd && inp.key_pressed(egui::Key::F),
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
            shift_b,
            j,
            copy,
            paste,
            sync,
            rot_left,
            rot_right,
            flip,
            strip,
        ] = k;
        if copy {
            self.copy_settings();
        }
        if paste {
            self.paste_settings_to(vec![i]);
        }
        if sync {
            self.paste_settings_to((0..self.frames.len()).collect());
        }
        if (rot_left || rot_right || flip) && self.editor.crop.is_none() {
            let s = &mut self.frames[i].sc.settings;
            if flip {
                s.flip_h = !s.flip_h;
                s.crop = [s.crop[2], s.crop[1], s.crop[0], s.crop[3]];
            } else if rot_right {
                s.rotation = (s.rotation + 1) % 4;
                s.crop = rotate_insets(s.crop, true);
            } else {
                s.rotation = (s.rotation + 3) % 4;
                s.crop = rotate_insets(s.crop, false);
            }
            self.mark_changed(i);
            self.editor.dirty = true;
        }
        if strip {
            self.editor.filmstrip = !self.editor.filmstrip;
        }
        if b {
            self.editor.before = !self.editor.before;
            self.editor.split = false;
            self.editor.dirty = true;
        }
        if shift_b {
            self.editor.split = !self.editor.split;
            self.editor.before = false;
            self.editor.dirty = true;
        }
        if j {
            self.editor.clip_view = !self.editor.clip_view;
            self.editor.dirty = true;
        }
        if let Some(key) = super::lab_key(ctx).filter(|_| self.editor.crop.is_none()) {
            if self.apply_lab_key(i, key) {
                self.editor.dirty = true;
            } else {
                self.toast("Convert the frame before correcting it", false);
            }
        }
        if undo {
            self.history_step(true);
        }
        if redo {
            self.history_step(false);
        }
        if plus {
            self.editor.zoom_req = Some(ZoomReq::Factor(1.5));
        }
        if minus {
            self.editor.zoom_req = Some(ZoomReq::Factor(1.0 / 1.5));
        }
        if c {
            if self.editor.crop.is_some() {
                self.apply_crop(i);
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
        let ctx = ui.ctx().clone();
        let cmd = egui::Modifiers::COMMAND;
        let undo_key = widgets::shortcut(&ctx, cmd, egui::Key::Z);
        let redo_key = widgets::shortcut(&ctx, cmd | egui::Modifiers::SHIFT, egui::Key::Z);
        let rect = ui.max_rect();
        // Centred title.
        let f = &self.frames[i];
        let sub = if self.editor.crop.is_some() {
            "Crop & straighten".to_string()
        } else if self.editor.before {
            "Before: negative".to_string()
        } else if f.sc.is_converted() {
            format!(
                "{}{}{}",
                s.tone_profile.label(),
                if f.is_roll() { " · roll" } else { "" },
                lab::readout(s)
                    .map(|r| format!(" · {r}"))
                    .unwrap_or_default()
            )
        } else {
            "Not converted".to_string()
        };
        let p = ui.painter();
        p.text(
            pos2(rect.center().x, rect.center().y - 7.0),
            egui::Align2::CENTER_CENTER,
            &f.name,
            theme::medium(13.5),
            theme::TEXT,
        );
        p.text(
            pos2(rect.center().x, rect.center().y + 9.0),
            egui::Align2::CENTER_CENTER,
            sub,
            egui::FontId::proportional(11.5),
            theme::MUTED,
        );
        // Explain the "D+1 C0 …" readout in the subtitle.
        if lab::readout(s).is_some() && self.editor.crop.is_none() && !self.editor.before {
            let r = Rect::from_center_size(
                pos2(rect.center().x, rect.center().y + 9.0),
                vec2(240.0, 14.0),
            );
            ui.interact(r, ui.id().with("lab_readout"), Sense::hover())
                .on_hover_text(super::LAB_KEYS_HELP);
        }

        let n = self.frames.len();
        ui.horizontal_centered(|ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            let back_tip = if self.single_mode {
                "Close image (Space)"
            } else {
                "Back to folder (Space)"
            };
            if icon_button(ui, Icon::Back, back_tip, false, true).clicked() {
                self.leave_editor();
                return;
            }
            if n > 1 {
                ui.add_space(8.0);
                if icon_button(ui, Icon::Prev, "Previous image (←)", false, true).clicked() {
                    self.goto(i, -1);
                    return;
                }
                if icon_button(ui, Icon::Next, "Next image (→)", false, true).clicked() {
                    self.goto(i, 1);
                    return;
                }
                ui.add_space(4.0);
                ui.label(
                    RichText::new(format!("{} / {}", i + 1, n))
                        .small()
                        .color(theme::MUTED),
                );
            }
            ui.add_space(10.0);
            if icon_button(
                ui,
                Icon::Undo,
                &format!("Undo ({undo_key})"),
                false,
                self.history.can_undo(),
            )
            .clicked()
            {
                self.history_step(true);
            }
            if icon_button(
                ui,
                Icon::Redo,
                &format!("Redo ({redo_key})"),
                false,
                self.history.can_redo(),
            )
            .clicked()
            {
                self.history_step(false);
            }
            ui.add_space(6.0);
            if ui
                .add_enabled(self.editor.crop.is_none(), widgets::ghost("Reset"))
                .on_hover_text("Reset all edits to the defaults (undo with the arrow)")
                .clicked()
            {
                self.reset_edits(i, s);
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 2.0;
                if widgets::icon_button_tinted(
                    ui,
                    Icon::Export,
                    "Export positive copy",
                    self.export_targets.is_some() || self.editor.export_open,
                    true,
                    Some(theme::ACCENT),
                )
                .clicked()
                {
                    self.open_export_dialog(vec![i]);
                }
                ui.add_space(10.0);
                let cropping = self.editor.crop.is_some();
                if icon_button(ui, Icon::Crop, "Crop & straighten (C)", cropping, true).clicked() {
                    if cropping {
                        self.apply_crop(i);
                    } else {
                        self.enter_crop(i);
                    }
                }
                let ctx = ui.ctx().clone();
                let key = |m, k| widgets::shortcut(&ctx, m, k);
                let cmd = egui::Modifiers::COMMAND;
                let flip_tip = format!("Flip horizontally ({})", key(egui::Modifiers::NONE, egui::Key::H));
                let right_tip = format!("Rotate right ({})", key(cmd, egui::Key::CloseBracket));
                let left_tip = format!("Rotate left ({})", key(cmd, egui::Key::OpenBracket));
                if icon_button(ui, Icon::Flip, &flip_tip, s.flip_h, !cropping).clicked() {
                    s.flip_h = !s.flip_h;
                    s.crop = [s.crop[2], s.crop[1], s.crop[0], s.crop[3]];
                }
                if icon_button(ui, Icon::RotateRight, &right_tip, false, !cropping).clicked() {
                    s.rotation = (s.rotation + 1) % 4;
                    s.crop = rotate_insets(s.crop, true);
                }
                if icon_button(ui, Icon::RotateLeft, &left_tip, false, !cropping).clicked() {
                    s.rotation = (s.rotation + 3) % 4;
                    s.crop = rotate_insets(s.crop, false);
                }
                ui.add_space(10.0);
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
                    self.editor.split = false;
                    self.editor.dirty = true;
                }
                if self.editor.loading || self.jobs.busy() > 0 {
                    ui.add_space(8.0);
                    ui.add(egui::Spinner::new().size(14.0).color(theme::MUTED));
                }
            });
        });
    }

    // ---- sidebar ----------------------------------------------------------------

    fn sidebar(&mut self, ui: &mut egui::Ui, i: usize, s: &mut Settings) {
        if self.editor.crop.is_some() {
            self.crop_panel(ui, i);
        }
        widgets::histogram(ui, &self.editor.hist, vec2(ui.available_width(), 72.0));
        ui.add_space(4.0);
        let converted = self.frames[i].sc.is_converted();
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.set_width(ui.available_width() - 6.0);
                self.conversion_header(ui, i, s);
                if converted {
                    self.conversion_summary(ui, s);
                    self.adjust_sections(ui, s);
                } else {
                    self.conversion_settings(ui, s);
                }
                self.film_section(ui, i);
                self.engine_section(ui, i, s);
                ui.add_space(16.0);
            });
    }

    /// Film and capture details: written to the exported EXIF and used by
    /// the export file name tokens.
    fn film_section(&mut self, ui: &mut egui::Ui, i: usize) {
        if !widgets::fold_with(ui, "Film", false) {
            return;
        }
        let changed =
            super::roll::film_fields(ui, "film_meta", &mut self.frames[i].sc.metadata, true);
        if changed {
            self.frames[i].dirty_since = Some(self.time);
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
            ui.add_space(4.0);
            let label = if roll.is_some() {
                "Apply to roll"
            } else {
                "Apply to folder"
            };
            if ui
                .add(widgets::ghost(label))
                .on_hover_text(format!(
                    "Copy these details to {} other frame(s)",
                    others.len()
                ))
                .clicked()
            {
                let m = self.frames[i].sc.metadata.clone();
                for &k in &others {
                    self.frames[k].sc.metadata = m.clone();
                    self.frames[k].dirty_since = Some(self.time);
                }
                self.toast(
                    format!("Film details copied to {} frame(s)", others.len()),
                    false,
                );
            }
        }
        ui.add_space(8.0);
    }

    fn crop_panel(&mut self, ui: &mut egui::Ui, i: usize) {
        ui.label(
            RichText::new("Crop & straighten")
                .font(theme::medium(12.5))
                .color(theme::TEXT),
        );
        ui.add_space(4.0);
        if let Some(c) = &mut self.editor.crop {
            let mut a = c.angle;
            if fl_slider(
                ui,
                "Straighten",
                &mut a,
                -45.0..=45.0,
                0.0,
                1,
                "Fine rotation in degrees. Drag the frame edges or corners on the image; drag inside to move it.",
                None,
            ) {
                c.angle = a;
                self.editor.dirty = true;
            }
        }
        ui.horizontal(|ui| {
            if ui.add(widgets::ghost("Reset")).clicked() {
                if let Some(c) = &mut self.editor.crop {
                    c.insets = [0.0; 4];
                    c.angle = 0.0;
                    self.editor.dirty = true;
                }
            }
            if ui
                .add(widgets::ghost("Detect"))
                .on_hover_text("Find the frame inside the film rebate and holder")
                .clicked()
            {
                self.detect_frame(i);
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .add(widgets::primary("Done"))
                    .on_hover_text("Enter")
                    .clicked()
                {
                    self.apply_crop(i);
                }
                if ui
                    .add(widgets::ghost("Cancel"))
                    .on_hover_text("Esc")
                    .clicked()
                {
                    self.cancel_crop();
                }
            });
        });
        ui.add_space(14.0);
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
            .on_hover_text("Analyse this frame on its own (convert rolls from the folder view)")
            .clicked()
        {
            self.frames[i].sc.settings = s.clone();
            self.convert(vec![i], false);
        }
    }

    /// A full-width dropdown with an eyedropper button on its right.
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
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            let w = ui.available_width() - 36.0;
            // Top-down, so the combo's trailing space isn't added sideways
            // (that overflowed the row and widened the panel every frame).
            ui.allocate_ui_with_layout(
                vec2(w, 30.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    fl_combo(ui, "", v, all, name, "");
                },
            );
            let picking = self.editor.pick == Some(pick);
            if icon_button(ui, Icon::Eyedropper, tip, picking, enabled).clicked() {
                self.editor.pick = if picking { None } else { Some(pick) };
            }
        });
    }

    /// Convert / Re-convert with the ⋯ menu beside it.
    fn conversion_header(&mut self, ui: &mut egui::Ui, i: usize, s: &mut Settings) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            let w = ui.available_width() - 36.0;
            ui.allocate_ui(vec2(w, 28.0), |ui| self.convert_button(ui, i, s));
            let more = icon_button(ui, Icon::More, "Defaults and more", false, true);
            egui::Popup::menu(&more).show(|ui| self.defaults_menu(ui, i, s));
        });
    }

    /// Converted frames: a one-line recap of the conversion inputs, which
    /// expands into the full settings.
    fn conversion_summary(&mut self, ui: &mut egui::Ui, s: &mut Settings) {
        let open = widgets::fold_with(ui, "Conversion", false);
        if open {
            self.conversion_settings(ui, s);
        } else {
            ui.label(
                RichText::new(format!(
                    "{} · {} · base {}",
                    s.source.label(),
                    s.color_model.label(),
                    s.film_base.label().to_lowercase(),
                ))
                .small()
                .color(theme::FAINT),
            );
        }
    }

    /// The inputs the analysis depends on (everything in `AnalysisKey`).
    fn conversion_settings(&mut self, ui: &mut egui::Ui, s: &mut Settings) {
        {
            if fl_combo(
                ui,
                "Image type",
                &mut s.image_type,
                ImageType::ALL,
                ImageType::label,
                "",
            ) {
                if s.image_type == ImageType::Negative {
                    let d = Settings::default();
                    (s.tone_profile, s.wb_mode) = (d.tone_profile, d.wb_mode);
                } else {
                    s.apply_positive_defaults();
                }
            }
        }
        if widgets::fold(ui, "Source") {
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
            ui.add(
                egui::Label::new(
                    RichText::new(source_instructions(s.source))
                        .small()
                        .color(theme::FAINT),
                )
                .wrap(),
            );
            ui.add_space(8.0);
            if fl_combo(
                ui,
                "Colour model",
                &mut s.color_model,
                ColorModel::ALL,
                ColorModel::label,
                "Basic: neutral · Frontier: teal-blues, golden yellows, warm skin · Noritsu: cleaner and cooler · B+W: monochrome · None: no calibration",
            ) && s.color_model == ColorModel::BlackAndWhite
            {
                s.apply_mono_defaults();
            }
            ui.label(RichText::new("Film base").small().color(theme::MUTED))
                .on_hover_text("Neutralises the orange mask before analysis");
            self.combo_with_picker(
                ui,
                &mut s.film_base,
                FilmBase::ALL,
                FilmBase::label,
                Pick::FilmBase,
                "Sample the unexposed film border",
                true,
            );
            ui.add_space(8.0);
            let start = self
                .editor
                .path
                .as_ref()
                .and_then(|p| p.parent().map(|p| p.to_path_buf()));
            super::folder::flat_field_ui(ui, &mut s.flat_field, start.as_deref());
        }
        self.editor.border_preview = false;
        if widgets::fold_with(ui, "Analysis area", false) {
            fl_slider_u8(
                ui,
                "Pre-saturation",
                &mut s.pre_saturation,
                1..=5,
                3,
                "1–2: fewer distinct hues, warmer · 4–5: more separation, cooler",
            );
            let (_, active) = widgets::fl_slider_active(
                ui,
                "Border buffer",
                &mut s.border_buffer,
                0.0..=25.0,
                5.0,
                None,
                0,
                "Percent of the frame edge ignored by the analysis",
                None,
            );
            self.editor.border_preview = active;
        }
    }

    fn adjust_sections(&mut self, ui: &mut egui::Ui, s: &mut Settings) {
        if widgets::fold(ui, "Tone") {
            fl_combo(
                ui,
                "Tone profile",
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
            if widgets::fold_with(ui, "Hyper Tone", false) {
                let amount = |ui: &mut egui::Ui, l: &str, v: &mut f32, tip: &str| {
                    fl_slider(ui, l, v, 0.0..=100.0, 0.0, 0, tip, None)
                };
                amount(
                    ui,
                    "Burn highlights",
                    &mut s.hyper_tone_highlights,
                    "Holds back large bright areas (sky, window light) while keeping their detail",
                );
                amount(
                    ui,
                    "Dodge shadows",
                    &mut s.hyper_tone_shadows,
                    "Opens large dark areas (backlit subjects, shade) while keeping their detail",
                );
            }
            if widgets::fold_with(ui, "Glow, fade & clipping", false) {
                tone(ui, "Lab glow", &mut s.glow, "Silky compressed highlights");
                tone(ui, "Lab fade", &mut s.fade, "Open, compressed deep shadows");
                fl_slider(
                    ui,
                    "White clip",
                    &mut s.white_clip,
                    -50.0..=50.0,
                    0.0,
                    0,
                    "Negative values recover clipped highlights",
                    None,
                );
                fl_slider(
                    ui,
                    "Black clip",
                    &mut s.black_clip,
                    -50.0..=50.0,
                    0.0,
                    0,
                    "Negative values recover crushed shadows",
                    None,
                );
            }
        }
        if widgets::fold(ui, "White balance") {
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
            let confidence = self
                .editor_index()
                .and_then(|i| self.frames[i].auto_wb(s))
                .map(|w| w.confidence);
            if auto && confidence.is_some_and(|c| c < pipeline::LOW_CONFIDENCE) {
                widgets::status_line(
                    ui,
                    Icon::Warn,
                    theme::WARN,
                    &format!(
                        "Few neutral tones to balance on. Pick a grey ({wb_key}) or use the roll."
                    ),
                );
            }
            let mut t = s.wb_temp;
            if fl_slider(
                ui,
                "Temp",
                &mut t,
                -50.0..=50.0,
                0.0,
                1,
                "Yellow printing filtration in CC: blue ↔ yellow",
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
                -50.0..=50.0,
                0.0,
                1,
                "Magenta printing filtration in CC: green ↔ magenta",
                Some(grad::TINT),
            ) {
                s.wb_tint = k;
                s.wb_mode = WbMode::Custom;
            }
        }
        if widgets::fold(ui, "Lab correction") {
            let clicks = lab::clicks(s);
            let rows = [
                (
                    lab::Channel::D,
                    "D",
                    "Print lighter (6)",
                    "Print darker (4)",
                ),
                (lab::Channel::C, "C", "Remove cyan (1)", "Add cyan (7)"),
                (
                    lab::Channel::M,
                    "M",
                    "Remove magenta (2)",
                    "Add magenta (8)",
                ),
                (lab::Channel::Y, "Y", "Remove yellow (3)", "Add yellow (9)"),
            ];
            ui.horizontal(|ui| {
                for (k, (ch, name, minus_tip, plus_tip)) in rows.into_iter().enumerate() {
                    ui.vertical(|ui| {
                        let v = match clicks[k] {
                            0 => "0".to_string(),
                            v if v > 0 => format!("+{v}"),
                            v => format!("−{}", -v),
                        };
                        // Drag the value sideways to step through clicks;
                        // double-click to zero it.
                        let r = ui
                            .add(
                                egui::Label::new(RichText::new(format!("{name} {v}")).monospace().color(
                                    if clicks[k] == 0 {
                                        theme::MUTED
                                    } else {
                                        theme::TEXT
                                    },
                                ))
                                .sense(Sense::click_and_drag()),
                            )
                            .on_hover_cursor(egui::CursorIcon::ResizeHorizontal)
                            .on_hover_text("Drag to change · double-click to reset");
                        let acc_id = r.id.with("drag_acc");
                        let mut acc: f32 = ui.data(|d| d.get_temp(acc_id)).unwrap_or(0.0);
                        acc += r.drag_delta().x;
                        let steps = (acc / LAB_DRAG_PX).trunc();
                        if steps != 0.0 {
                            lab::nudge(s, ch, steps as i32);
                            acc -= steps * LAB_DRAG_PX;
                        }
                        if r.drag_stopped() {
                            acc = 0.0;
                        }
                        ui.data_mut(|d| d.insert_temp(acc_id, acc));
                        if r.double_clicked() {
                            lab::nudge(s, ch, -clicks[k]);
                        }
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 2.0;
                            if ui
                                .add(widgets::ghost("−"))
                                .on_hover_text(minus_tip)
                                .clicked()
                            {
                                lab::nudge(s, ch, -1);
                            }
                            if ui
                                .add(widgets::ghost("+"))
                                .on_hover_text(plus_tip)
                                .clicked()
                            {
                                lab::nudge(s, ch, 1);
                            }
                        });
                    });
                    ui.add_space(4.0);
                }
            });
            ui.add(
                egui::Label::new(
                    RichText::new(super::LAB_KEYS_HELP)
                        .small()
                        .color(theme::MUTED),
                )
                .wrap(),
            );
            if ui
                .add_enabled(lab::readout(s).is_some(), widgets::ghost("Reset (0)"))
                .clicked()
            {
                lab::reset(s);
            }
        }
        if widgets::fold(ui, "Colour toning") {
            segmented(
                ui,
                &mut self.editor.color_tab,
                &[
                    (ColorTab::Mids, "Mids"),
                    (ColorTab::Highs, "Highs"),
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
        if widgets::fold(ui, "Look") {
            fl_combo(ui, "LUT", &mut s.lut, Lut::ALL, Lut::label, "");
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
                "Look saturation (5 = default)",
            );
            fl_combo(
                ui,
                "Sharpening",
                &mut s.sharpening,
                Sharpening::ALL,
                Sharpening::label,
                "",
            );
        }
    }

    /// Recompute Temp / Tint for `s`'s WB mode on this frame: stored values
    /// only carry over for Custom WB, as when the plugin applies saved settings.
    fn refresh_wb(&self, i: usize, s: &mut Settings) {
        if s.wb_mode == WbMode::Custom {
            return;
        }
        (s.wb_temp, s.wb_tint) = self.frames[i].wb_values(s).unwrap_or((0.0, 0.0));
    }

    /// `resetSettings`: back to the positive, B+W or colour defaults,
    /// keeping the frame's source fields and engine preset.
    fn reset_edits(&mut self, i: usize, s: &mut Settings) {
        let keep = s.clone();
        *s = Settings::default();
        s.keep_frame_fields(&keep);
        if s.image_type == ImageType::Positive {
            s.apply_positive_defaults();
        } else if s.is_monochrome() {
            s.apply_mono_defaults();
        }
        self.refresh_wb(i, s);
    }

    /// The ⋯ menu next to Re-convert: defaults, unconvert and the LUT folder.
    fn defaults_menu(&mut self, ui: &mut egui::Ui, i: usize, s: &mut Settings) {
        let ctx = ui.ctx().clone();
        let cmd = egui::Modifiers::COMMAND;
        let key = |m, k| RichText::new(widgets::shortcut(&ctx, m, k)).color(theme::FAINT);
        if ui
            .add(egui::Button::new("Copy settings").shortcut_text(key(cmd, egui::Key::C)))
            .clicked()
        {
            self.copy_settings();
        }
        let paste = self.clipboard.is_some();
        if ui
            .add_enabled(paste, egui::Button::new("Paste settings").shortcut_text(key(cmd, egui::Key::V)))
            .clicked()
        {
            self.paste_settings_to(vec![i]);
        }
        if ui
            .add_enabled(
                paste,
                egui::Button::new("Paste to every frame")
                    .shortcut_text(key(cmd | egui::Modifiers::SHIFT, egui::Key::V)),
            )
            .on_hover_text("Sync the copied edit to the whole roll")
            .clicked()
        {
            self.paste_settings_to((0..self.frames.len()).collect());
        }
        ui.separator();
        if ui
            .button("Save as default")
            .on_hover_text("New frames start with these settings")
            .clicked()
        {
            // Like the plugin's saved defaults, leave out the frame's
            // source, colour model, pre-saturation and border.
            let mut d = s.clone();
            d.keep_frame_fields(&Settings {
                image_type: d.image_type,
                film_base: d.film_base,
                film_base_sample: d.film_base_sample,
                ..Settings::default()
            });
            self.defaults.settings = d;
            let _ = crate::engine::sidecar::save_defaults(&self.defaults);
            self.toast("Saved as default settings", false);
            ui.close();
        }
        if ui
            .button("Apply default")
            .on_hover_text("Replace this frame's edits with the saved default")
            .clicked()
        {
            let keep = s.clone();
            *s = self.defaults.settings.clone();
            s.keep_frame_fields(&keep);
            self.refresh_wb(i, s);
            ui.close();
        }
        ui.separator();
        if self.frames[i].sc.is_converted()
            && ui
                .button("Unconvert")
                .on_hover_text("Discard the analysis for this frame")
                .clicked()
        {
            self.unconvert(&[i]);
            ui.close();
        }
    }

    /// The print curves and the analysed density levels, collapsed by default.
    fn engine_section(&mut self, ui: &mut egui::Ui, i: usize, s: &mut Settings) {
        if widgets::fold_with(ui, "Analysis", false) {
            if !self.editor.curves.is_empty() {
                ui.add_space(8.0);
                widgets::curves_plot(ui, &self.editor.curves, vec2(ui.available_width(), 150.0));
            }
            let f = &self.frames[i];
            if let Some(o) = pipeline::levels_for(s, f.sc.analysis.as_ref(), f.sc.roll.as_ref()) {
                ui.add_space(6.0);
                egui::Grid::new("origins")
                    .num_columns(4)
                    .spacing([18.0, 2.0])
                    .show(ui, |ui| {
                        for h in ["", "Black D", "White D", "Median D"] {
                            ui.label(RichText::new(h).small().color(theme::MUTED));
                        }
                        ui.end_row();
                        for (c, n) in ["R", "G", "B"].iter().enumerate() {
                            ui.label(*n);
                            ui.label(format!("{:.2}", o[c].lo));
                            ui.label(format!("{:.2}", o[c].hi));
                            ui.label(format!("{:.2}", o[c].mid));
                            ui.end_row();
                        }
                    });
                if f.sc.roll.is_some() {
                    ui.checkbox(&mut s.use_roll, "Use roll analysis");
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
        }
    }

    // ---- canvas -----------------------------------------------------------------

    fn canvas(&mut self, ui: &mut egui::Ui, i: usize) {
        let avail = ui.max_rect();
        let Some([tw, th]) = self.editor.size else {
            ui.centered_and_justified(|ui| {
                ui.add(egui::Spinner::new().size(24.0).color(theme::MUTED));
            });
            return;
        };
        let cropping = self.editor.crop.is_some();
        let margin = if cropping {
            vec2(48.0, 110.0)
        } else {
            vec2(32.0, 32.0)
        };
        let ts = vec2(tw as f32, th as f32);
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
                ZoomReq::Absolute(a) => {
                    let z = a.clamp(fit, 8.0f32.max(fit));
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
        // Zoom with ⌘/Ctrl + scroll or pinch (about the cursor); scroll pans
        // when zoomed. Applied before the geometry below so it takes effect
        // in the same frame.
        if let Some(hover) = ui.input(|inp| inp.pointer.hover_pos()).filter(|h| avail.contains(*h)) {
            let (zoom_delta, scroll) = ui.input(|inp| (inp.zoom_delta(), inp.smooth_scroll_delta));
            if zoom_delta != 1.0 {
                self.zoom_by(zoom_delta, fit, hover - center);
            } else if self.editor.zoom.is_some() && scroll != egui::Vec2::ZERO {
                self.editor.zoom_anim = None;
                self.editor.pan += scroll;
            }
        }
        let scale = self.editor.zoom.unwrap_or(fit);
        let size = ts * scale;
        // Clamp pan so the image never leaves the canvas entirely.
        let maxp = ((size - avail.size()) * 0.5).max(egui::Vec2::ZERO);
        self.editor.pan = self.editor.pan.clamp(-maxp, maxp);
        let rect = Rect::from_center_size(center + self.editor.pan, size);
        let resp = ui.allocate_rect(avail, Sense::click_and_drag());

        let p = ui.painter_at(avail);
        p.add(
            egui::Shadow {
                offset: [0, 6],
                blur: 28,
                spread: 0,
                color: Color32::from_black_alpha(90),
            }
            .as_shape(rect, 0.0),
        );
        let full_uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
        match (
            &self.editor.gpu_neg,
            &self.editor.gpu_view,
            &self.editor.tex,
        ) {
            (Some(n), Some(v), _) if self.gpu => {
                gpu_paint_clipped(&p, rect, avail, 0, n, v, [0.0, 0.0, 1.0, 1.0])
            }
            (_, _, Some(t)) => {
                p.image(t.id(), rect, full_uv, Color32::WHITE);
            }
            _ => {}
        }
        if let Some(t) = self.editor.clip_tex.as_ref().filter(|_| !cropping) {
            p.image(
                t.id(),
                rect,
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        }
        let split =
            !cropping && (self.editor.before_tex.is_some() || self.editor.gpu_before.is_some());
        if split {
            let k = self.editor.split_pos;
            let x = rect.left() + rect.width() * k;
            let left = Rect::from_min_max(rect.min, pos2(x, rect.bottom()));
            match (
                &self.editor.gpu_neg,
                &self.editor.gpu_before,
                &self.editor.before_tex,
            ) {
                (Some(n), Some(b), _) => gpu_paint_clipped(&p, left, avail, 1, n, b, [0.0, 0.0, k, 1.0]),
                (_, _, Some(t)) => {
                    p.image(
                        t.id(),
                        left,
                        Rect::from_min_max(pos2(0.0, 0.0), pos2(k, 1.0)),
                        Color32::WHITE,
                    );
                }
                _ => {}
            }
            let shown = rect.intersect(avail);
            p.line_segment(
                [pos2(x, shown.top()), pos2(x, shown.bottom())],
                Stroke::new(1.5, Color32::WHITE),
            );
            let knob = pos2(x, shown.center().y);
            p.circle_filled(knob, 9.0, Color32::WHITE);
            p.text(
                knob,
                egui::Align2::CENTER_CENTER,
                "↔",
                egui::FontId::proportional(11.0),
                theme::CANVAS,
            );
            let handle =
                Rect::from_center_size(pos2(x, shown.center().y), vec2(24.0, shown.height()));
            let h = ui.interact(handle, ui.id().with("split_handle"), Sense::drag());
            if h.hovered() || h.dragged() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
            }
            if h.dragged() {
                if let Some(pos) = h.interact_pointer_pos() {
                    self.editor.split_pos =
                        ((pos.x - rect.left()) / rect.width()).clamp(0.02, 0.98);
                }
            }
        }

        if self.editor.border_preview && !cropping {
            // Mirror `pipeline::analysis_buffer`: equal % insets per side.
            let pct = self.frames[i].sc.settings.border_buffer.clamp(0.0, 40.0);
            let b = pct / 100.0;
            let d = vec2(rect.width() * b, rect.height() * b);
            let inner = Rect::from_min_max(rect.min + d, rect.max - d);
            let dim = Color32::from_black_alpha(160);
            for strip in [
                Rect::from_min_max(rect.min, pos2(rect.right(), inner.top())),
                Rect::from_min_max(pos2(rect.left(), inner.bottom()), rect.max),
                Rect::from_min_max(
                    pos2(rect.left(), inner.top()),
                    pos2(inner.left(), inner.bottom()),
                ),
                Rect::from_min_max(
                    pos2(inner.right(), inner.top()),
                    pos2(rect.right(), inner.bottom()),
                ),
            ] {
                p.rect_filled(strip, 0.0, dim);
            }
            p.rect_stroke(
                inner,
                0.0,
                Stroke::new(1.0, theme::ACCENT),
                egui::StrokeKind::Inside,
            );
            pill(
                &p,
                pos2(avail.center().x, avail.top() + 22.0),
                &format!("Analysis area · {pct:.0}% border"),
            );
            ui.ctx().request_repaint();
        }

        if cropping {
            self.crop_overlay(ui, &resp, rect, avail);
        } else if let Some(pick) = self.editor.pick {
            let label = match pick {
                Pick::WhiteBalance => "Click a neutral grey area · Esc to cancel",
                Pick::FilmBase => "Click the unexposed film border · Esc to cancel",
            };
            pill(&p, pos2(avail.center().x, avail.top() + 22.0), label);
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
        if !cropping && self.editor.pick.is_none() {
            resp.context_menu(|ui| self.canvas_menu(ui, i));
        }

        if self.editor.before && !cropping {
            pill(&p, pos2(avail.left() + 44.0, avail.top() + 22.0), "Before");
        }
        if split {
            pill(&p, pos2(avail.left() + 44.0, avail.top() + 22.0), "Before");
            pill(&p, pos2(avail.right() - 40.0, avail.top() + 22.0), "After");
        }
        if self.editor.clip_tex.is_some() && !cropping {
            pill(
                &p,
                pos2(avail.center().x, avail.top() + 22.0),
                "Clipping · red: blown highlights · blue: blocked shadows · J",
            );
        }
        if cropping {
            return;
        }
        // Pixel readout: output RGB and the negative's density above base.
        if let Some(pos) = resp.hover_pos().filter(|p| rect.contains(*p)) {
            let u = (pos.x - rect.left()) / rect.width();
            let v = (pos.y - rect.top()) / rect.height();
            if let Some(text) = self.readout(u, v) {
                let w = p
                    .layout_no_wrap(text.clone(), theme::medium(12.0), theme::TEXT)
                    .size()
                    .x;
                pill(
                    &p,
                    pos2(
                        avail.left() + 16.0 + (w + 22.0) / 2.0,
                        avail.bottom() - 22.0,
                    ),
                    &text,
                );
            }
        }
        // Zoom chip: click to toggle between fit and 100 %.
        let full = self
            .editor
            .src
            .as_ref()
            .map(|s| {
                s.info.full_width.max(s.info.full_height) as f32 / s.width.max(s.height) as f32
            })
            .unwrap_or(1.0);
        let zoom_txt = match self.editor.zoom {
            Some(z) => format!("{:.0}%", z / full * 100.0),
            None => "Fit".into(),
        };
        let chip = pill(&p, avail.right_bottom() + vec2(-36.0, -22.0), &zoom_txt);
        let chip_resp = ui
            .interact(chip, ui.id().with("zoom_chip"), Sense::click())
            .on_hover_text("Toggle fit / 100 %  ·  double-click the image to zoom there");
        if chip_resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        if chip_resp.clicked() {
            self.editor.zoom_req = Some(if self.editor.zoom.is_some() {
                ZoomReq::Absolute(0.0)
            } else {
                ZoomReq::Absolute(full)
            });
        }
    }

    /// Right-click menu on the preview.
    fn canvas_menu(&mut self, ui: &mut egui::Ui, i: usize) {
        let before = self.frames[i].sc.settings.clone();
        let mut s = before.clone();
        let converted = self.frames[i].sc.is_converted();
        if ui
            .button("Auto crop")
            .on_hover_text("Detect the frame and open it in Crop & straighten")
            .clicked()
        {
            self.enter_crop(i);
            self.detect_frame(i);
            ui.close();
        }
        if ui.button("Crop & straighten…").clicked() {
            self.enter_crop(i);
            ui.close();
        }
        ui.separator();
        if ui.button("Rotate left").clicked() {
            s.rotation = (s.rotation + 3) % 4;
            s.crop = rotate_insets(s.crop, false);
            ui.close();
        }
        if ui.button("Rotate right").clicked() {
            s.rotation = (s.rotation + 1) % 4;
            s.crop = rotate_insets(s.crop, true);
            ui.close();
        }
        if ui.button("Flip horizontally").clicked() {
            s.flip_h = !s.flip_h;
            s.crop = [s.crop[2], s.crop[1], s.crop[0], s.crop[3]];
            ui.close();
        }
        ui.separator();
        if ui.button("Pick white balance").clicked() {
            self.editor.pick = Some(Pick::WhiteBalance);
            ui.close();
        }
        if ui.button("Pick film base").clicked() {
            self.editor.pick = Some(Pick::FilmBase);
            ui.close();
        }
        ui.separator();
        if ui
            .checkbox(&mut self.editor.before.clone(), "Before / after")
            .clicked()
        {
            self.editor.before = !self.editor.before;
            self.editor.split = false;
            self.editor.dirty = true;
            ui.close();
        }
        if ui
            .checkbox(&mut self.editor.split.clone(), "Split view")
            .clicked()
        {
            self.editor.split = !self.editor.split;
            self.editor.before = false;
            self.editor.dirty = true;
            ui.close();
        }
        if converted
            && ui
                .checkbox(&mut self.editor.clip_view.clone(), "Show clipping")
                .clicked()
        {
            self.editor.clip_view = !self.editor.clip_view;
            self.editor.dirty = true;
            ui.close();
        }
        if ui
            .button(if self.editor.zoom.is_some() {
                "Zoom to fit"
            } else {
                "Zoom in"
            })
            .clicked()
        {
            self.editor.zoom_req = Some(if self.editor.zoom.is_some() {
                ZoomReq::Absolute(0.0)
            } else {
                ZoomReq::Factor(2.0)
            });
            ui.close();
        }
        ui.separator();
        if ui.button("Reset edits").clicked() {
            self.reset_edits(i, &mut s);
            ui.close();
        }
        self.apply_change(i, &before, s);
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
        let hs = Stroke::new(3.0, theme::TEXT);
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

    /// "RGB 231 220 198 · D 0.42 0.51 0.63" at (u, v) of the preview.
    fn readout(&self, u: f32, v: f32) -> Option<String> {
        let [w, h] = self.editor.px_size;
        if self.editor.px.len() != w * h || w == 0 {
            return None;
        }
        let rgb =
            sample(&self.editor.px, w, h, u, v, 1).map(|c| (c.clamp(0.0, 1.0) * 255.0).round());
        let mut text = format!("RGB {} {} {}", rgb[0], rgb[1], rgb[2]);
        if let Some(neg) = &self.editor.neg {
            let d = sample(&neg.data, neg.width, neg.height, u, v, 1);
            text += &format!("  ·  D {:.2} {:.2} {:.2}", d[0], d[1], d[2]);
        }
        Some(text)
    }

    fn do_pick(&mut self, i: usize, pick: Pick, u: f32, v: f32) {
        let before = self.frames[i].sc.settings.clone();
        let mut s = before.clone();
        match pick {
            Pick::WhiteBalance => {
                let Some(neg) = &self.editor.neg else { return };
                let f = &self.frames[i];
                let Some(levels) =
                    pipeline::levels_for(&s, f.sc.analysis.as_ref(), f.sc.roll.as_ref())
                else {
                    self.toast("Convert the negative first, then pick a neutral.", true);
                    return;
                };
                let avg = sample(&neg.data, neg.width, neg.height, u, v, 3).map(|d| d as f64);
                match crate::engine::curve::eyedropper_wb(avg, &levels, &s) {
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

/// Average a (2r+1)² neighbourhood at normalised coordinates.
fn set_texture(
    ctx: &egui::Context,
    slot: &mut Option<egui::TextureHandle>,
    name: &str,
    img: egui::ColorImage,
) {
    match slot {
        Some(t) => t.set(img, egui::TextureOptions::LINEAR),
        None => *slot = Some(ctx.load_texture(name, img, egui::TextureOptions::LINEAR)),
    }
}

/// Overlay marking clipped pixels: red where a channel is blown, blue where
/// every channel is blocked.
fn clip_image(px: &[[f32; 3]], w: usize, h: usize) -> egui::ColorImage {
    let pixels = px
        .iter()
        .map(|p| {
            if p.iter().any(|v| *v >= 0.999) {
                Color32::from_rgb(255, 40, 40)
            } else if p.iter().all(|v| *v <= 0.002) {
                Color32::from_rgb(40, 110, 255)
            } else {
                Color32::TRANSPARENT
            }
        })
        .collect();
    egui::ColorImage::new([w, h], pixels)
}

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

/// Draw the part of `rect` that lies inside `clip`. The GPU callback's
/// viewport is clipped to the canvas, so a zoomed rectangle has to be cut down
/// together with its UVs, otherwise the whole image is squeezed into the view.
fn gpu_paint_clipped(
    painter: &egui::Painter,
    rect: Rect,
    clip: Rect,
    slot: u8,
    neg: &std::sync::Arc<gpu::NegData>,
    view: &std::sync::Arc<gpu::ViewData>,
    part: [f32; 4],
) {
    let vis = rect.intersect(clip);
    if vis.width() <= 0.0 || vis.height() <= 0.0 {
        return;
    }
    let fx = |x: f32| (x - rect.left()) / rect.width();
    let fy = |y: f32| (y - rect.top()) / rect.height();
    let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t;
    let uv = [
        lerp(part[0], part[2], fx(vis.left())),
        lerp(part[1], part[3], fy(vis.top())),
        lerp(part[0], part[2], fx(vis.right())),
        lerp(part[1], part[3], fy(vis.bottom())),
    ];
    gpu::paint(painter, vis, slot, neg, view, uv);
}
