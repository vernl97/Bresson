//! Editor View: fine-tune one frame. Layout: a light toolbar (navigation,
//! zoom, history on the left; title centred; geometry and export on the
//! right), a grey canvas with zoom / pan and an interactive crop mode, and a
//! sidebar holding the plugin's Convert / Edit / Advanced controls.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use eframe::egui::{self, Color32, Rect, RichText, Sense, Stroke, pos2, vec2};

use super::widgets::{self, Icon, icon_button, icon_button_tinted, pill, theme};
use super::{NlpApp, View, gpu, to_color_image};
use crate::engine::decode::{self, SourceImage};
use crate::engine::pipeline::{self, Negative, Renderer};
use crate::engine::settings::*;
use crate::engine::sidecar::Flag;

mod crop;
mod heal;
mod sidebar;

pub(super) use crop::rotate_insets;
use crop::{CropEdit, CropGrid};
use heal::{HealEdit, SpotMap};
use sidebar::Section;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ColorTab {
    Mids,
    Highs,
    Shadows,
}

/// What the histogram panel shows.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum HistView {
    #[default]
    Histogram,
    Curves,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Pick {
    WhiteBalance,
    FilmBase,
}

type NegKey = (AnalysisKey, [i32; 4], i32, u8, bool, [i32; 3], bool, Vec<HealSpot>);

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
    neg_key: Option<NegKey>,
    /// 1024 px copy of `src` (and the `src` it was made from) that the
    /// preview uses while the straighten angle is being dragged.
    src_low: Option<(Arc<SourceImage>, Arc<SourceImage>)>,
    /// Small negative before straightening, keyed like `neg_key` with the
    /// angle left out; a drag only has to straighten it again.
    neg_pre: Option<(NegKey, Negative)>,
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
    hist_view: HistView,
    film_open: bool,
    /// Counts the photos opened in this session: the sidebar's section ids
    /// include it, so each photo starts with its default sections.
    epoch: u32,
    pick: Option<Pick>,
    committed: Option<Settings>,
    hist: [[u32; 256]; 3],
    /// The histogram is not following the preview (angle drag in progress).
    hist_frozen: bool,
    curves: Vec<[f32; 4]>,
    /// Display scale (screen points per preview pixel); `None` = fit.
    zoom: Option<f32>,
    pan: egui::Vec2,
    pub(super) crop: Option<CropEdit>,
    crop_grid: CropGrid,
    /// The spot healing tool is on; it stays on from frame to frame.
    pub(super) heal: Option<HealEdit>,
    /// What Visualize Spots shows of the current negative.
    spot_map: Option<SpotMap>,
    /// The Finishing section, across frames (section states reset per frame).
    finishing_open: bool,
    pub export_open: bool,
    zoom_req: Option<ZoomReq>,
    /// The frame edited last and its settings, and the edit before that on a
    /// different frame (what "Previous" applies). Kept across frames.
    last_edit: Option<(PathBuf, Settings)>,
    prev_edit: Option<Settings>,
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
    /// Info overlay at the canvas's top left (I).
    pub info: bool,
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
            src_low: None,
            neg_pre: None,
            tex: None,
            size: None,
            gpu_neg: None,
            gpu_view: None,
            gpu_before: None,
            dirty: true,
            before: false,
            color_tab: ColorTab::Mids,
            hist_view: HistView::default(),
            film_open: false,
            epoch: 0,
            pick: None,
            committed: None,
            hist: [[0; 256]; 3],
            hist_frozen: false,
            curves: Vec::new(),
            zoom: None,
            pan: egui::Vec2::ZERO,
            crop: None,
            crop_grid: CropGrid::default(),
            heal: None,
            spot_map: None,
            finishing_open: false,
            export_open: false,
            zoom_req: None,
            last_edit: None,
            prev_edit: None,
            zoom_anim: None,
            border_preview: false,
            px: Vec::new(),
            px_size: [0, 0],
            clip_view: false,
            info: false,
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
        let (keep, view, strip, epoch) = (
            self.color_tab,
            self.hist_view,
            self.filmstrip,
            self.epoch + 1,
        );
        let edits = (self.last_edit.take(), self.prev_edit.take());
        let heal = self.heal.map(HealEdit::for_next_frame);
        let crop_grid = self.crop_grid;
        *self = Editor {
            last_edit: edits.0,
            prev_edit: edits.1,
            path: Some(path.to_path_buf()),
            loading: true,
            committed: Some(settings),
            epoch,
            heal,
            crop_grid,
            ..Default::default()
        };
        self.color_tab = keep;
        self.hist_view = view;
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

/// Starting height of the editor's filmstrip; the user drags its top edge to resize it.
const FILMSTRIP_H: f32 = 104.0;
/// Smallest and largest filmstrip heights the drag handle allows.
const FILMSTRIP_H_RANGE: std::ops::RangeInclusive<f32> = 48.0..=320.0;
/// Height of the bar under the canvas.
const VIEW_BAR_H: f32 = 36.0;

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
        let path = &self.frames[i].path;
        if let Some((p, last)) = self.editor.last_edit.take()
            && p != *path
        {
            self.editor.prev_edit = Some(last);
        }
        self.editor.last_edit = Some((path.clone(), self.frames[i].sc.settings.clone()));
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
        self.editor.heal = None;
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

    /// The frame as converted, before any slider edit: every adjustment
    /// section back at its default, geometry and source kept. What
    /// Before and Split compare against.
    fn baseline_settings(&self, i: usize, s: &Settings) -> Settings {
        let mut reference = s.clone();
        self.reset_edits(i, &mut reference);
        let mut b = s.clone();
        for sec in [
            Section::Tone,
            Section::WhiteBalance,
            Section::PrintCharacter,
            Section::Colour,
        ] {
            sec.reset(&mut b, &reference);
        }
        b
    }

    /// Settings the preview renders with (crop mode shows the uncropped,
    /// live-straightened frame).
    fn preview_settings(&self, i: usize) -> Settings {
        let mut s = self.frames[i].sc.settings.clone();
        if let Some(c) = &self.editor.crop {
            s.crop = [0.0; 4];
            s.straighten = c.angle;
            s.rotation = c.rotation;
            s.flip_h = c.flip_h;
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
        let base_s =
            (self.editor.before || self.editor.split).then(|| self.baseline_settings(i, &s));
        let f = &self.frames[i];
        let base = f.base_for(&src);
        // Dragging the angle re-straightens on every frame, so do it on a
        // small copy and redo it at full size once the drag ends.
        let low = dragging
            && self.editor.crop.is_some()
            && s.flat_field.is_none()
            && src.long_edge() > 1100;
        let prep_src = if low {
            if !self
                .editor
                .src_low
                .as_ref()
                .is_some_and(|(from, _)| Arc::ptr_eq(from, &src))
            {
                let (data, width, height) =
                    decode::downscale(&src.data, src.width, src.height, 1024);
                let small = SourceImage {
                    width,
                    height,
                    data,
                    ..(*src).clone()
                };
                self.editor.src_low = Some((src.clone(), Arc::new(small)));
            }
            self.editor.src_low.as_ref().unwrap().1.clone()
        } else {
            src.clone()
        };
        let key = (
            s.analysis_key(),
            s.crop.map(|v| (v * 100.0).round() as i32),
            (s.straighten * 100.0).round() as i32,
            s.rotation,
            s.flip_h,
            base.map(|v| (v * 1e5) as i32),
            low,
            s.heal.clone(),
        );
        if self.editor.neg_key.as_ref() != Some(&key) || self.editor.neg.is_none() {
            let neg = if low {
                let pre_key = (
                    key.0.clone(),
                    key.1,
                    0,
                    key.3,
                    key.4,
                    key.5,
                    true,
                    key.7.clone(),
                );
                if self.editor.neg_pre.as_ref().map(|(k, _)| k) != Some(&pre_key) {
                    let mut flat = s.clone();
                    flat.straighten = 0.0;
                    let n = pipeline::prepare_negative(&prep_src, &flat, base);
                    self.editor.neg_pre = Some((pre_key, n));
                }
                let pre = &self.editor.neg_pre.as_ref().unwrap().1;
                let (data, width, height) =
                    decode::straighten(&pre.data, pre.width, pre.height, s.straighten);
                Negative {
                    width,
                    height,
                    data,
                }
            } else {
                self.editor.neg_pre = None;
                pipeline::prepare_negative(&prep_src, &s, base)
            };
            let (d, w, h) = decode::downscale(&neg.data, neg.width, neg.height, 1024);
            self.editor.neg_small = Some(Negative {
                width: w,
                height: h,
                data: d,
            });
            self.editor.gpu_neg = self.gpu.then(|| gpu::NegData::new(&neg, !low));
            self.editor.neg = Some(neg);
            self.editor.neg_key = Some(key);
            self.editor.spot_map = None;
        }
        let full = self.editor.neg.as_ref().unwrap();
        let small = self.editor.neg_small.as_ref().unwrap();
        let use_small = !self.gpu && dragging && full.width.max(full.height) > 1100;
        let cpu_neg = if self.gpu || use_small { small } else { full };
        // Before shows the frame without its slider edits, not the negative.
        let render_s = if self.editor.before {
            base_s.as_ref().unwrap_or(&s)
        } else {
            &s
        };
        let origins = pipeline::levels_for(render_s, f.sc.analysis.as_ref(), f.sc.roll.as_ref());
        let mp = (src.info.full_width * src.info.full_height) as f32 / 1e6;
        let r = Renderer::new(render_s, origins.as_ref(), OutputSpace::Srgb).with_megapixels(mp);
        let full_long = src.info.full_width.max(src.info.full_height).max(1) as f32;
        let crop_frac = 1.0 - (s.crop[0] + s.crop[2]).max(s.crop[1] + s.crop[3]) / 100.0;
        let scale = cpu_neg.width.max(cpu_neg.height) as f32 / (full_long * crop_frac.max(0.1));
        // The GPU draws the preview, so while the angle is dragged the CPU
        // copy for histogram and readout is left as it was (and shown dimmed).
        let frozen = low && self.gpu && !self.editor.px.is_empty();
        self.editor.hist_frozen = frozen;
        let px = if frozen {
            Vec::new()
        } else {
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
            px
        };
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
            let b = base_s.as_ref().unwrap_or(&s);
            let b_levels = pipeline::levels_for(b, f.sc.analysis.as_ref(), f.sc.roll.as_ref());
            let before = Renderer::new(b, b_levels.as_ref(), OutputSpace::Srgb).with_megapixels(mp);
            if self.gpu {
                self.editor.gpu_before = Some(gpu::ViewData::new(&before));
            } else {
                let img = to_color_image(&before.render(cpu_neg, scale.min(1.0)), w, h);
                set_texture(ctx, &mut self.editor.before_tex, "editor-before", img);
            }
        }
        self.editor.size = Some([full.width, full.height]);
        if !frozen {
            self.editor.px = px;
            self.editor.px_size = [w, h];
        }
        self.editor.dirty = use_small || low;
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
        // Laid out before the sidebar so it spans the full window width and the
        // sidebar sits on top of it instead of running down beside it.
        if self.editor.filmstrip && self.frames.len() > 1 {
            // Resizable from its top edge; egui keeps the chosen height for this panel id.
            egui::Panel::bottom("editor_filmstrip")
                .resizable(true)
                .default_size(FILMSTRIP_H)
                .size_range(FILMSTRIP_H_RANGE)
                .frame(
                    egui::Frame::new()
                        .fill(theme::BG)
                        .inner_margin(egui::Margin::symmetric(0, 2)),
                )
                .show(ui, |ui| self.filmstrip(ui, i));
            if self.editor_index() != Some(i) {
                return;
            }
        }

        // Also before the sidebar, so the bar runs as far as the filmstrip. Crop &
        // straighten swaps the bar's contents, never its place, so the picture stays put.
        egui::Panel::bottom("editor_view_bar")
            .exact_size(VIEW_BAR_H)
            .frame(
                egui::Frame::new()
                    .fill(theme::PANEL)
                    .inner_margin(egui::Margin::symmetric(10, 0)),
            )
            .show(ui, |ui| {
                if self.editor.crop.is_some() {
                    self.crop_bar(ui, i);
                } else if self.editor.heal.is_some() {
                    self.heal_bar(ui);
                } else {
                    self.view_bar(ui, i);
                }
            });
        // The view bar may have moved to another frame.
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
                        right: 6,
                        top: 0,
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
            .show(ui, |ui| {
                self.canvas(ui, i);
                self.info_overlay(ui, i);
            });
    }

    /// The text toggled with I, at the canvas's top left: name, roll and the
    /// frame's conversion state, as in Lightroom's info overlay.
    fn info_overlay(&self, ui: &egui::Ui, i: usize) {
        if !self.editor.info || self.editor.crop.is_some() {
            return;
        }
        let f = &self.frames[i];
        let s = &f.sc.settings;
        let mut lines: Vec<String> = vec![f.name.clone()];
        lines.extend(self.roll_name());
        if let Some(src) = &self.editor.src {
            let info = &src.info;
            lines.push(format!("{} × {} px", info.full_width, info.full_height));
        }
        if self.editor.before {
            lines.push("Before: without slider edits".into());
        } else if f.sc.is_converted() {
            if f.is_roll() {
                lines.push("Roll balance".into());
            }
            lines.push(s.tone_profile.label().to_string());
        } else {
            lines.push("Not converted".into());
        }
        let rect = ui.max_rect();
        let mut pos = rect.left_top() + vec2(16.0, 14.0);
        let p = ui.painter();
        for (k, line) in lines.iter().enumerate() {
            let (font, color) = if k == 0 {
                (theme::medium(13.5), theme::TEXT)
            } else {
                (egui::FontId::proportional(11.5), theme::MUTED)
            };
            let g = p.layout_no_wrap(line.clone(), font, color);
            let h = g.size().y;
            p.galley(pos, g, color);
            pos.y += h + 2.0;
        }
    }

    /// Display scale of 100 % relative to the preview's own size.
    fn full_scale(&self) -> f32 {
        self.editor
            .src
            .as_ref()
            .map(|s| {
                s.info.full_width.max(s.info.full_height) as f32 / s.width.max(s.height) as f32
            })
            .unwrap_or(1.0)
    }

    /// The strip under the canvas: compare and flags on the left, zoom on the
    /// right. The frame counter lives in the top bar; arrow keys and the
    /// filmstrip cover navigation.
    fn view_bar(&mut self, ui: &mut egui::Ui, i: usize) {
        #[derive(Clone, Copy, PartialEq)]
        enum Preset {
            Fit,
            One,
            Other,
        }
        let full = self.full_scale();
        let preset = match self.editor.zoom {
            None => Preset::Fit,
            Some(z) if (z / full - 1.0).abs() < 0.01 => Preset::One,
            Some(_) => Preset::Other,
        };
        ui.horizontal_centered(|ui| {
            let toggle = |ui: &mut egui::Ui, label: &str, on: bool, tip: &str| {
                let g = ui.painter().layout_no_wrap(
                    label.to_string(),
                    theme::medium(11.5),
                    theme::TEXT,
                );
                let (rect, resp) =
                    ui.allocate_exact_size(vec2(g.size().x + 16.0, 28.0), egui::Sense::click());
                if on {
                    ui.painter().rect_filled(rect, 5.0, theme::HOVER);
                } else if resp.hovered() {
                    ui.painter().rect_filled(rect, 5.0, theme::SURFACE);
                }
                let col = if on || resp.hovered() {
                    theme::TEXT
                } else {
                    theme::MUTED
                };
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    label,
                    theme::medium(11.5),
                    col,
                );
                resp.on_hover_text(tip)
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .clicked()
            };
            ui.spacing_mut().item_spacing.x = 2.0;
            let ib = |ui: &mut egui::Ui, icon: Icon, tip: &str, on: bool| {
                icon_button(ui, icon, tip, on, true).clicked()
            };
            // A thin vertical rule between groups of buttons.
            let divider = |ui: &mut egui::Ui| {
                ui.add_space(6.0);
                let (rect, _) = ui.allocate_exact_size(vec2(1.0, 16.0), egui::Sense::hover());
                ui.painter().rect_filled(rect, 0.0, theme::TRACK);
                ui.add_space(6.0);
            };
            if ib(
                ui,
                Icon::Before,
                "Before: the photo without slider edits (B)",
                self.editor.before,
            ) {
                self.editor.before = !self.editor.before;
                self.editor.split = false;
                self.editor.dirty = true;
            }
            if ib(
                ui,
                Icon::Split,
                "Before | after side by side (Shift+B)",
                self.editor.split,
            ) {
                self.editor.split = !self.editor.split;
                self.editor.before = false;
                self.editor.dirty = true;
            }
            divider(ui);
            // Flags: a click sets, a second click clears (P / X / U).
            let flag = self.frames[i].sc.flag;
            for (f, icon, tip) in [
                (Flag::Pick, Icon::Flag, "Pick (P)"),
                (Flag::Reject, Icon::Reject, "Reject (X)"),
            ] {
                let on = flag == f;
                let tint = match f {
                    Flag::Pick => theme::OK,
                    _ => theme::ERR,
                };
                if icon_button_tinted(ui, icon, tip, on, true, on.then_some(tint)).clicked() {
                    self.set_flag(&[i], if on { Flag::None } else { f });
                }
            }
            divider(ui);
            // Quarter turns, applied at once. Inside Crop & straighten they are pending.
            let cmd = egui::Modifiers::COMMAND;
            let key = |k| widgets::shortcut(ui.ctx(), cmd, k);
            let left = format!("Rotate left ({})", key(egui::Key::OpenBracket));
            let right = format!("Rotate right ({})", key(egui::Key::CloseBracket));
            if ib(ui, Icon::RotateLeft, &left, false) {
                self.rotate_frames(&[i], false);
            }
            if ib(ui, Icon::RotateRight, &right, false) {
                self.rotate_frames(&[i], true);
            }
            // Zoom presets in the same button style, right-aligned (laid out
            // right to left, so listed from the last).
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let presets = [
                    (Preset::One, "100%", "Zoom to 100 %"),
                    (Preset::Fit, "Fit", "Fit the whole picture"),
                ];
                for (pr, label, tip) in presets {
                    if toggle(ui, label, preset == pr, tip) {
                        self.editor.zoom_req = Some(match pr {
                            Preset::Fit | Preset::Other => ZoomReq::Absolute(0.0),
                            Preset::One => ZoomReq::Absolute(full),
                        });
                    }
                }
                // The level itself, left of the presets so they stay put.
                if preset == Preset::Other
                    && let Some(z) = self.editor.zoom
                {
                    let level = format!("{:.0}%", z / full * 100.0);
                    toggle(ui, &level, true, "Current zoom");
                }
            });
        });
    }

    /// Thumbnails of every frame; a click opens that frame.
    fn filmstrip(&mut self, ui: &mut egui::Ui, i: usize) {
        // The panel's inner height: the thumbnails fill whatever the drag leaves.
        let h = ui.available_height();
        let mut open = None;
        let scroll_id = ui.id().with("filmstrip_scrolled_to");
        let scrolled_to: Option<usize> = ui.data(|d| d.get_temp(scroll_id));
        egui::ScrollArea::horizontal()
            .auto_shrink([false, false])
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 1.0;
                    // Square cells like Lightroom's filmstrip: number and flag on
                    // top, the picture centred in its own shape.
                    for (j, f) in self.frames.iter().enumerate() {
                        let (rect, resp) = ui.allocate_exact_size(vec2(h, h), egui::Sense::click());
                        if j == i && scrolled_to != Some(i) {
                            resp.scroll_to_me(Some(egui::Align::Center));
                        }
                        if !ui.is_rect_visible(rect) {
                            continue;
                        }
                        let p = ui.painter();
                        let fill = if j == i {
                            egui::Color32::from_rgb(78, 78, 84)
                        } else if resp.hovered() {
                            theme::HOVER
                        } else {
                            theme::SURFACE
                        };
                        p.rect_filled(rect, 0.0, fill);
                        let head = (h * 0.2).clamp(12.0, 18.0);
                        // Symmetric padding keeps the picture vertically centred in the cell.
                        let area = rect.shrink2(vec2(6.0, head));
                        if let Some(t) = &f.thumb {
                            let a = t.size_vec2().x / t.size_vec2().y;
                            let fit = vec2(
                                area.width().min(area.height() * a),
                                area.height().min(area.width() / a),
                            );
                            let img = egui::Rect::from_center_size(area.center(), fit);
                            p.rect_filled(
                                img.translate(vec2(1.0, 1.0)),
                                0.0,
                                egui::Color32::from_black_alpha(70),
                            );
                            egui::Image::new((t.id(), t.size_vec2())).paint_at(ui, img);
                            ui.painter().rect_stroke(
                                img,
                                0.0,
                                egui::Stroke::new(1.0, egui::Color32::BLACK),
                                egui::StrokeKind::Outside,
                            );
                        }
                        let p = ui.painter();
                        let num = p.layout_no_wrap(
                            format!("{}", j + 1),
                            theme::medium(11.0),
                            if j == i { theme::TEXT } else { theme::MUTED },
                        );
                        let nw = num.size().x;
                        p.galley(
                            rect.min + vec2(6.0, (head - num.size().y) / 2.0 + 1.0),
                            num,
                            theme::MUTED,
                        );
                        let mark = egui::Rect::from_min_size(
                            rect.min + vec2(6.0 + nw + 4.0, (head - 11.0) / 2.0 + 1.0),
                            vec2(11.0, 11.0),
                        );
                        match f.sc.flag {
                            Flag::Pick => super::grid::paint_flag(p, mark, theme::OK, true),
                            Flag::Reject => {
                                widgets::paint_icon(p, mark, widgets::Icon::Reject, theme::ERR)
                            }
                            Flag::None => {}
                        }
                        if j == i {
                            ui.painter().rect_stroke(
                                rect,
                                0.0,
                                egui::Stroke::new(2.0, theme::ACCENT),
                                egui::StrokeKind::Inside,
                            );
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
        if ctx.egui_wants_keyboard_input() || self.shortcuts_open {
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
                !cmd && inp.key_pressed(egui::Key::I),
                cmd && !inp.modifiers.shift && inp.key_pressed(egui::Key::E),
                !cmd && inp.key_pressed(egui::Key::Q),
                !cmd && inp.key_pressed(egui::Key::OpenBracket),
                !cmd && inp.key_pressed(egui::Key::CloseBracket),
                inp.key_pressed(egui::Key::Delete) || inp.key_pressed(egui::Key::Backspace),
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
            info,
            export,
            q,
            smaller,
            larger,
            delete,
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
        if (rot_left || rot_right || flip) && self.editor.crop.is_some() {
            self.crop_transform(if flip { None } else { Some(rot_right) });
        } else if rot_left || rot_right || flip {
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
        if info {
            self.editor.info = !self.editor.info;
        }
        if export && self.export_targets.is_none() {
            self.open_export_dialog(vec![i]);
        }
        if let Some(flag) = Self::flag_key(ctx) {
            self.set_flag(&[i], flag);
            let what = match flag {
                crate::engine::sidecar::Flag::Pick => "Picked",
                crate::engine::sidecar::Flag::Reject => "Rejected",
                crate::engine::sidecar::Flag::None => "Unflagged",
            };
            self.toast(format!("{what} {}", self.frames[i].name), false);
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
        let path = self.frames[i].path.clone();
        if undo {
            self.history_step_for(&path, true);
        }
        if redo {
            self.history_step_for(&path, false);
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
        } else if enter {
            self.editor.heal = None;
        }
        if q {
            self.toggle_heal(i);
        }
        if self.editor.heal.is_some() {
            if smaller || larger {
                self.step_heal_size(i, larger);
            }
            if delete {
                self.delete_heal_spot(i);
            }
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
            } else if let Some(c) = self.editor.crop.as_mut().filter(|c| c.level) {
                (c.level, c.level_line) = (false, None);
            } else if self.editor.crop.is_some() {
                self.cancel_crop();
            } else if self.editor.heal.is_some() {
                self.editor.heal = None;
            } else {
                self.leave_editor();
                return true;
            }
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

    /// The roll's name, or its folder's when it has none.
    fn roll_name(&self) -> Option<String> {
        self.roll
            .info
            .as_ref()
            .map(|r| r.name.clone())
            .filter(|n| !n.trim().is_empty())
            .or_else(|| {
                self.folder
                    .as_ref()
                    .and_then(|f| f.file_name())
                    .map(|n| n.to_string_lossy().to_string())
            })
    }

    /// The frame's bar, built like the grid's: where you are on the left,
    /// what goes out of the roll in the far corner.
    fn toolbar(&mut self, ui: &mut egui::Ui, i: usize, s: &mut Settings) {
        let ctx = ui.ctx().clone();
        let cropping = self.editor.crop.is_some();
        ui.horizontal_centered(|ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            let back_tip = if self.single_mode {
                "Close image (Esc)"
            } else {
                "Grid view (Esc)"
            };
            if widgets::bar_button(ui, Icon::Grid, back_tip, false, true, None).clicked() {
                self.leave_editor();
                return;
            }
            if !self.single_mode
                && let Some(roll) = self.roll_name()
            {
                let back = ui.scope(|ui| {
                    ui.set_max_width(220.0);
                    ui.add(
                        egui::Label::new(RichText::new(roll).color(theme::MUTED))
                            .truncate()
                            .sense(Sense::click()),
                    )
                    .on_hover_text("Back to the roll")
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .clicked()
                });
                if back.inner {
                    self.leave_editor();
                    return;
                }
                ui.label(RichText::new("/").color(theme::MUTED));
            }
            // The frame's place in the roll, as "03 / 12"; its filename follows, secondary.
            let n = self.frames.len();
            let counter = if n > 1 {
                format!("{:02} / {}", i + 1, n)
            } else {
                format!("{:02}", i + 1)
            };
            ui.label(RichText::new(counter).font(theme::medium(13.5)));
            let name = self.frames[i].name.clone();
            ui.scope(|ui| {
                ui.set_max_width(240.0);
                ui.add(
                    egui::Label::new(RichText::new(name).small().color(theme::MUTED)).truncate(),
                );
            });
            // The frame's own stock, else the one it inherits from the roll.
            let stock = Some(self.frames[i].sc.metadata.film_stock.clone())
                .filter(|s| !s.trim().is_empty())
                .or_else(|| {
                    self.roll
                        .info
                        .as_ref()
                        .map(|r| r.metadata.film_stock.clone())
                        .filter(|s| !s.trim().is_empty())
                });
            ui.add_space(6.0);
            // The chip is always there: with no stock yet it invites one.
            let empty = stock.is_none();
            let stock = stock.unwrap_or_else(|| "Add film stock".to_string());
            {
                let font = egui::TextStyle::Small.resolve(ui.style());
                // A small chevron after the stock says the chip opens something.
                const CHEVRON_W: f32 = 12.0;
                let size = ui
                    .painter()
                    .layout_no_wrap(stock.clone(), font.clone(), theme::MUTED)
                    .size();
                let (rect, resp) =
                    ui.allocate_exact_size(size + vec2(14.0 + CHEVRON_W, 6.0), Sense::click());
                let hot = resp.hovered() && !cropping;
                let (fill, ink) = if hot {
                    (theme::HOVER, theme::TEXT)
                } else if empty {
                    (theme::SURFACE, theme::FAINT)
                } else {
                    (theme::SURFACE, theme::MUTED)
                };
                ui.painter().rect_filled(rect, 5.0, fill);
                if hot {
                    ui.painter().rect_stroke(
                        rect,
                        5.0,
                        egui::Stroke::new(1.0, theme::PRESSED),
                        egui::StrokeKind::Inside,
                    );
                }
                let g = ui.painter().layout_no_wrap(stock, font, ink);
                let text_at = egui::pos2(rect.left() + 7.0, rect.center().y - g.size().y / 2.0);
                ui.painter().galley(text_at, g, ink);
                // Chevron: a small "v" at the right end of the chip.
                let cc = egui::pos2(rect.right() - CHEVRON_W / 2.0 - 2.0, rect.center().y);
                let (w, h) = (2.8, 1.6);
                ui.painter().line_segment(
                    [cc + vec2(-w, -h), cc + vec2(0.0, h)],
                    egui::Stroke::new(1.2, ink),
                );
                ui.painter().line_segment(
                    [cc + vec2(0.0, h), cc + vec2(w, -h)],
                    egui::Stroke::new(1.2, ink),
                );
                let tip = if empty {
                    "Add a film stock · click for film & roll details"
                } else {
                    "Film stock · click for film & roll details"
                };
                if resp
                    .on_hover_text(tip)
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .clicked()
                    && !cropping
                {
                    self.editor.film_open = true;
                }
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Crop & straighten has its own Done and Cancel; nothing leaves meanwhile.
                if !cropping {
                    let tip = format!(
                        "Export a positive copy of this frame ({})",
                        widgets::shortcut(&ctx, egui::Modifiers::COMMAND, egui::Key::E)
                    );
                    if ui
                        .add(widgets::primary("Export").min_size(vec2(88.0, 0.0)))
                        .on_hover_text(tip)
                        .clicked()
                    {
                        self.open_export_dialog(vec![i]);
                    }
                    let more = icon_button(
                        ui,
                        Icon::More,
                        "Copy, paste, defaults and more",
                        false,
                        true,
                    );
                    egui::Popup::menu(&more).show(|ui| self.defaults_menu(ui, i, s));
                }
                if self.editor.loading || self.jobs.busy() > 0 {
                    ui.add_space(8.0);
                    ui.add(egui::Spinner::new().size(14.0).color(theme::MUTED));
                }
            });
        });
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
    fn reset_edits(&self, i: usize, s: &mut Settings) {
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

    /// The top bar's ⋯ menu: copy / paste, defaults, film details and unconvert.
    fn defaults_menu(&mut self, ui: &mut egui::Ui, i: usize, s: &mut Settings) {
        let ctx = ui.ctx().clone();
        let cmd = egui::Modifiers::COMMAND;
        let key = |m, k| RichText::new(widgets::shortcut(&ctx, m, k)).color(theme::MUTED);
        let paste = self.clipboard.is_some();
        if ui
            .add(egui::Button::new("Copy settings").shortcut_text(key(cmd, egui::Key::C)))
            .clicked()
        {
            self.copy_settings();
            ui.close();
        }
        if ui
            .add_enabled(
                paste,
                egui::Button::new("Paste settings").shortcut_text(key(cmd, egui::Key::V)),
            )
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
        if ui
            .button("Film & roll…")
            .on_hover_text("Film details for the EXIF, and roll analysis")
            .clicked()
        {
            self.editor.film_open = true;
            ui.close();
        }
        if ui
            .add(
                egui::Button::new("Keyboard shortcuts")
                    .shortcut_text(RichText::new("?").color(theme::MUTED)),
            )
            .clicked()
        {
            self.shortcuts_open = true;
            ui.close();
        }
        if self.frames[i].sc.is_converted()
            && ui
                .button("Re-convert…")
                .on_hover_text("Choose the conversion options and analyse this frame again")
                .clicked()
        {
            self.frames[i].sc.settings = s.clone();
            self.open_convert_dialog(vec![i], false);
            ui.close();
        }
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
        let margin = vec2(32.0, 32.0);
        let ts = vec2(tw as f32, th as f32);
        let fit = ((avail.width() - margin.x) / ts.x)
            .min((avail.height() - margin.y) / ts.y)
            .max(0.01);
        let center = avail.center();
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
        if let Some(hover) = ui
            .input(|inp| inp.pointer.hover_pos())
            .filter(|h| avail.contains(*h))
        {
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
                (Some(n), Some(b), _) => {
                    gpu_paint_clipped(&p, left, avail, 1, n, b, [0.0, 0.0, k, 1.0])
                }
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
        } else if self.editor.heal.is_some() {
            self.heal_overlay(ui, &resp, rect, avail, i);
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
        if !cropping && self.editor.pick.is_none() && self.editor.heal.is_none() {
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
    }

    /// Right-click menu on the preview.
    fn canvas_menu(&mut self, ui: &mut egui::Ui, i: usize) {
        let before = self.frames[i].sc.settings.clone();
        let mut s = before.clone();
        let converted = self.frames[i].sc.is_converted();
        if ui
            .button("Auto")
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
        if widgets::checkbox(ui, &mut self.editor.before.clone(), "Before / after").clicked() {
            self.editor.before = !self.editor.before;
            self.editor.split = false;
            self.editor.dirty = true;
            ui.close();
        }
        if widgets::checkbox(ui, &mut self.editor.split.clone(), "Split view").clicked() {
            self.editor.split = !self.editor.split;
            self.editor.before = false;
            self.editor.dirty = true;
            ui.close();
        }
        if converted
            && widgets::checkbox(ui, &mut self.editor.clip_view.clone(), "Show clipping").clicked()
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
