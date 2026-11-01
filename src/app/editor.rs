//! Editor View: fine-tune one frame. Layout: a light toolbar (navigation,
//! zoom, history on the left; title centred; geometry and export on the
//! right), a grey canvas with zoom / pan and an interactive crop mode, and a
//! sidebar holding the plugin's Convert / Edit / Advanced controls.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use eframe::egui::{self, Color32, Rect, RichText, Sense, Stroke, pos2, vec2};

use super::widgets::{
    self, Icon, fl_combo, fl_slider, fl_slider_u8, grad, icon_button, pill, segmented, theme,
};
use super::{NlpApp, View, gpu, to_color_image};
use crate::engine::decode::{self, SourceImage};
use crate::engine::pipeline::{self, Negative, Renderer};
use crate::engine::settings::*;
use crate::engine::sidecar::Flag;

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

type NegKey = (AnalysisKey, [i32; 4], i32, u8, bool, [i32; 3], bool);

/// Uncommitted crop / straighten state while crop mode is open.
#[derive(Clone, Copy)]
pub(super) struct CropEdit {
    insets: [f32; 4],
    angle: f32,
    aspect: CropAspect,
    /// The ratio the lock returns to after Free.
    locked_aspect: CropAspect,
    drag: Option<Handle>,
}

/// Make the insets keep `k` (width : height in percent units) while `h` is
/// dragged; the opposite edge or corner stays put.
fn lock_aspect(ins: &mut [f32; 4], h: Handle, k: f32) {
    let [l, t, r, b] = *ins;
    let (w, hh) = (100.0 - l - r, 100.0 - t - b);
    match h {
        Handle::TL | Handle::TR | Handle::BL | Handle::BR => {
            let left = matches!(h, Handle::TL | Handle::BL);
            let top = matches!(h, Handle::TL | Handle::TR);
            let max_w = 100.0 - if left { r } else { l };
            let max_h = 100.0 - if top { b } else { t };
            let nw = w.max(hh * k).min(max_w.min(max_h * k)).max(1.0);
            let nh = nw / k;
            if left {
                ins[0] = 100.0 - r - nw;
            } else {
                ins[2] = 100.0 - l - nw;
            }
            if top {
                ins[1] = 100.0 - b - nh;
            } else {
                ins[3] = 100.0 - t - nh;
            }
        }
        Handle::L | Handle::R => {
            let cy = t + hh / 2.0;
            let max_w = 100.0 - if h == Handle::L { r } else { l };
            let nw = w.min(max_w).min(2.0 * cy.min(100.0 - cy) * k).max(1.0);
            let nh = nw / k;
            if h == Handle::L {
                ins[0] = 100.0 - r - nw;
            } else {
                ins[2] = 100.0 - l - nw;
            }
            ins[1] = cy - nh / 2.0;
            ins[3] = 100.0 - cy - nh / 2.0;
        }
        Handle::T | Handle::B => {
            let cx = l + w / 2.0;
            let max_h = 100.0 - if h == Handle::T { b } else { t };
            let nh = hh.min(max_h).min(2.0 * cx.min(100.0 - cx) / k).max(1.0);
            let nw = nh * k;
            if h == Handle::T {
                ins[1] = 100.0 - b - nh;
            } else {
                ins[3] = 100.0 - t - nh;
            }
            ins[0] = cx - nw / 2.0;
            ins[2] = 100.0 - cx - nw / 2.0;
        }
        Handle::Move | Handle::Dial => {}
    }
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
            export_open: false,
            zoom_req: None,
            last_edit: None,
            prev_edit: None,
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
        let (keep, view, strip, epoch) = (
            self.color_tab,
            self.hist_view,
            self.filmstrip,
            self.epoch + 1,
        );
        let edits = (self.last_edit.take(), self.prev_edit.take());
        *self = Editor {
            last_edit: edits.0,
            prev_edit: edits.1,
            path: Some(path.to_path_buf()),
            loading: true,
            committed: Some(settings),
            epoch,
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

/// Sidebar sections that can be reset on their own.
#[derive(Clone, Copy)]
enum Section {
    /// White balance and tone together, like Lightroom's Basic panel.
    Basic,
    Tone,
    WhiteBalance,
    /// No sidebar section: the lab keys act on it, and Before / Split reset it.
    Lab,
    /// Hyper Tone, glow, fade and clipping.
    PrintCharacter,
    /// Colour toning, saturation and LUT.
    Colour,
}

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
            Section::Lab => "Lab correction",
            Section::PrintCharacter => "Print character",
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
            Section::Lab => lab::readout(s).is_some(),
            Section::PrintCharacter => differs!(s, r; hyper_tone_highlights, hyper_tone_shadows,
                glow, fade, white_clip, black_clip),
            Section::Colour => differs!(s, r; mid_cyan, mid_tint, mid_temp, high_cyan, high_tint,
                high_temp, high_range, shadow_cyan, shadow_tint, shadow_temp, shadow_range,
                lut, lut_strength, post_saturation),
        }
    }

    fn reset(self, s: &mut Settings, r: &Settings) {
        match self {
            Section::Basic => {
                Section::Tone.reset(s, r);
                Section::WhiteBalance.reset(s, r);
            }
            Section::Tone => take!(s, r; tone_profile, print_exposure, brightness, contrast,
                highlights, shadows, whites, blacks),
            Section::WhiteBalance => take!(s, r; wb_mode, wb_temp, wb_tint),
            Section::Lab => lab::reset(s),
            Section::PrintCharacter => take!(s, r; hyper_tone_highlights, hyper_tone_shadows,
                glow, fade, white_clip, black_clip),
            Section::Colour => take!(s, r; mid_cyan, mid_tint, mid_temp, high_cyan, high_tint,
                high_temp, high_range, shadow_cyan, shadow_tint, shadow_temp, shadow_range,
                lut, lut_strength, post_saturation),
        }
    }
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

/// Starting height of the editor's filmstrip; the user drags its top edge to resize it.
const FILMSTRIP_H: f32 = 84.0;
/// Smallest and largest filmstrip heights the drag handle allows.
const FILMSTRIP_H_RANGE: std::ops::RangeInclusive<f32> = 48.0..=320.0;
/// Height of the bar that collapses and expands the filmstrip.
const VIEW_BAR_H: f32 = 30.0;
const FILMSTRIP_TOGGLE_H: f32 = 16.0;

/// Rotate crop insets [l, t, r, b] with the image.
pub(crate) fn rotate_insets(c: [f32; 4], clockwise: bool) -> [f32; 4] {
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
            Section::Lab,
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
        );
        if self.editor.neg_key.as_ref() != Some(&key) || self.editor.neg.is_none() {
            let neg = if low {
                let pre_key = (key.0.clone(), key.1, 0, key.3, key.4, key.5, true);
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

    pub(crate) fn enter_crop(&mut self, i: usize) {
        let s = &self.frames[i].sc.settings;
        self.editor.crop = Some(CropEdit {
            insets: s.crop,
            angle: s.straighten,
            aspect: s.crop_aspect,
            locked_aspect: if s.crop_aspect == CropAspect::Free {
                CropAspect::Original
            } else {
                s.crop_aspect
            },
            drag: None,
        });
        self.fit_pending_crop();
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
            s.crop_aspect = c.aspect;
            self.apply_change(i, &before, s);
        }
        self.editor.dirty = true;
    }

    /// Aspect (w / h) of the frame after its quarter turns, which the crop
    /// insets are relative to.
    fn frame_aspect(&self) -> Option<f32> {
        let src = self.editor.src.as_ref()?;
        let (w, h) = (src.width as f32, src.height as f32);
        let turned = self
            .active
            .and_then(|i| self.frames.get(i))
            .is_some_and(|f| f.sc.settings.rotation % 2 == 1);
        Some(if turned { h / w } else { w / h })
    }

    /// Shrink the pending crop to the selected aspect ratio.
    fn fit_pending_crop(&mut self) {
        let Some(frame) = self.frame_aspect() else {
            return;
        };
        if let Some(c) = &mut self.editor.crop {
            if let Some(ratio) = c.aspect.ratio(frame) {
                c.insets = crate::engine::frame::fit_aspect(c.insets, frame, ratio);
            }
        }
    }

    /// Propose the detected frame as the pending crop. The frame is found
    /// free-form and the aspect lock released, so the crop follows the frame.
    fn detect_frame(&mut self, i: usize) {
        let Some(src) = self.editor.src.clone() else {
            return;
        };
        let f = &self.frames[i];
        let mut settings = f.sc.settings.clone();
        settings.crop_aspect = CropAspect::Free;
        match crate::engine::frame::detect(&src, &settings, f.base_for(&src)) {
            Some(d) => {
                if let Some(c) = &mut self.editor.crop {
                    (c.insets, c.angle) = (d.crop, d.straighten);
                    c.aspect = CropAspect::Free;
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

        if self.frames.len() > 1 && self.editor.crop.is_none() {
            egui::Panel::bottom("editor_filmstrip_toggle")
                .exact_size(FILMSTRIP_TOGGLE_H)
                .frame(egui::Frame::new().fill(theme::PANEL))
                .show(ui, |ui| {
                    let (rect, resp) =
                        ui.allocate_exact_size(ui.available_size(), egui::Sense::click());
                    let c = if resp.hovered() {
                        theme::ACCENT
                    } else {
                        egui::Color32::from_gray(140)
                    };
                    // Chevron points down to collapse, up to expand.
                    let (m, w, d) = (rect.center(), 5.0, 2.5);
                    let d = if self.editor.filmstrip { d } else { -d };
                    ui.painter().line(
                        vec![
                            egui::pos2(m.x - w, m.y - d),
                            egui::pos2(m.x, m.y + d),
                            egui::pos2(m.x + w, m.y - d),
                        ],
                        egui::Stroke::new(1.5, c),
                    );
                    let tip = if self.editor.filmstrip {
                        "Hide filmstrip (F)"
                    } else {
                        "Show filmstrip (F)"
                    };
                    if resp.on_hover_text(tip).clicked() {
                        self.editor.filmstrip = !self.editor.filmstrip;
                    }
                });
        }
        if self.editor.filmstrip && self.frames.len() > 1 && self.editor.crop.is_none() {
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

        if self.editor.crop.is_none() {
            egui::Panel::bottom("editor_view_bar")
                .exact_size(VIEW_BAR_H)
                .frame(
                    egui::Frame::new()
                        .fill(theme::PANEL)
                        .inner_margin(egui::Margin::symmetric(10, 0)),
                )
                .show(ui, |ui| self.view_bar(ui, i));
        }
        // The view bar may have moved to another frame.
        if self.view != View::Editor || self.editor_index() != Some(i) {
            return;
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(theme::CANVAS))
            .show(ui, |ui| self.canvas(ui, i));
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

    /// The strip under the canvas: view toggles and flags on the left, zoom
    /// presets on the right.
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
                    ui.allocate_exact_size(vec2(g.size().x + 16.0, 22.0), egui::Sense::click());
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
            if ib(
                ui,
                Icon::Clipping,
                "Show clipped tones (J)",
                self.editor.clip_view,
            ) {
                self.editor.clip_view = !self.editor.clip_view;
                self.editor.dirty = true;
            }
            divider(ui);
            // Flags: a click sets, a second click clears (P / X / U).
            let flag = self.frames[i].sc.flag;
            for (f, icon, tip) in [
                (Flag::Pick, Icon::Flag, "Pick (P)"),
                (Flag::Reject, Icon::Close, "Reject (X)"),
            ] {
                let on = flag == f;
                if ib(ui, icon, tip, on) {
                    self.set_flag(&[i], if on { Flag::None } else { f });
                }
            }
            // Zoom presets in the same button style, right-aligned (laid out
            // right to left, so listed from the last).
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Frame counter, right of the zoom presets.
                let n = self.frames.len();
                if n > 1 {
                    ui.label(
                        RichText::new(format!("{} / {}", i + 1, n))
                            .small()
                            .color(theme::MUTED),
                    );
                    divider(ui);
                }
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
        let density = self.roll_density();
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
                            Flag::Pick => super::grid::paint_flag(p, mark, theme::TEXT, true),
                            Flag::Reject => {
                                widgets::paint_icon(p, mark, widgets::Icon::Close, theme::TEXT)
                            }
                            Flag::None => {}
                        }
                        let issues = self.look_issues(j, density);
                        if !issues.is_empty() {
                            let mark = Rect::from_min_size(
                                pos2(rect.right() - 6.0 - 12.0, mark.top()),
                                vec2(12.0, 12.0),
                            );
                            widgets::paint_icon(
                                ui.painter(),
                                mark,
                                widgets::Icon::Warn,
                                theme::TEXT,
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
        if let Some(key) = super::lab_key(ctx).filter(|_| self.editor.crop.is_none()) {
            if self.apply_lab_key(i, key) {
                self.editor.dirty = true;
            } else {
                self.toast("Convert the frame before correcting it", false);
            }
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

    fn toolbar(&mut self, ui: &mut egui::Ui, i: usize, s: &mut Settings) {
        let rect = ui.max_rect();
        // Centred title.
        let f = &self.frames[i];
        let sub = if self.editor.crop.is_some() {
            "Crop & straighten".to_string()
        } else if self.editor.before {
            "Before: without slider edits".to_string()
        } else if f.sc.is_converted() {
            let mut parts: Vec<String> = Vec::new();
            parts.extend(self.roll_name());
            if f.is_roll() {
                parts.push("roll balance".into());
            }
            parts.extend(lab::readout(s));
            if parts.is_empty() {
                s.tone_profile.label().to_string()
            } else {
                parts.join(" · ")
            }
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
                if self.editor.loading || self.jobs.busy() > 0 {
                    ui.add_space(8.0);
                    ui.add(egui::Spinner::new().size(14.0).color(theme::MUTED));
                }
            });
        });
    }

    // ---- sidebar ----------------------------------------------------------------

    /// Collapsible histogram. Like Lightroom's, its five regions are drag
    /// handles for Blacks, Darks, Exposure, Lights and Whites, and the corner
    /// triangles toggle the clipping view.
    fn histogram_panel(&mut self, ui: &mut egui::Ui, i: usize, s: &mut Settings) {
        let (open, _) = widgets::fold_section(ui, "Histogram", true, false);
        if !open {
            return;
        }
        ui.add_space(2.0);
        if !self.editor.curves.is_empty() {
            widgets::segmented(
                ui,
                &mut self.editor.hist_view,
                &[
                    (HistView::Histogram, "Histogram"),
                    (HistView::Curves, "Curves"),
                ],
            );
            ui.add_space(4.0);
            if self.editor.hist_view == HistView::Curves {
                widgets::curves_plot(ui, &self.editor.curves, vec2(ui.available_width(), 96.0));
                ui.add_space(2.0);
                return;
            }
        }
        use widgets::HistRegion as R;
        let editable = self.frames[i].sc.is_converted() && !self.editor.before;
        let fmt = |v: f32, ev: bool| {
            if ev {
                format!("{v:+.1} EV")
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
        // Reserve the content width, then draw 8 px wider on each side in a
        // child ui, so the panel does not grow to fit the overhang.
        let (slot, _) = ui.allocate_exact_size(vec2(ui.available_width(), 96.0), Sense::hover());
        let mut hui = ui.new_child(egui::UiBuilder::new().max_rect(slot.expand2(vec2(8.0, 0.0))));
        let ev = widgets::histogram(
            &mut hui,
            &self.editor.hist,
            vec2(slot.width() + 16.0, 96.0),
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
                R::Blacks => (&mut s.blacks, 50.0, 0.4),
                R::Darks => (&mut s.shadows, 50.0, 0.4),
                R::Exposure => (&mut s.print_exposure, 5.0, 0.02),
                R::Lights => (&mut s.highlights, 50.0, 0.4),
                R::Whites => (&mut s.whites, 50.0, 0.4),
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

    /// Row of tool icons under the histogram, like Lightroom's. Each tool is
    /// one `icon_button`; add new ones here.
    fn tool_strip(&mut self, ui: &mut egui::Ui, i: usize) {
        widgets::tool_band(ui, |ui| {
            if widgets::tool_button(ui, Icon::Crop, "Crop & straighten (C)", false).clicked() {
                self.enter_crop(i);
            }
        });
    }

    fn sidebar(&mut self, ui: &mut egui::Ui, i: usize, s: &mut Settings) {
        if self.editor.crop.is_some() {
            self.crop_panel(ui, i);
        }
        // Crop & straighten gets the sidebar to itself, like Lightroom.
        if self.editor.crop.is_some() {
            widgets::histogram(
                ui,
                &self.editor.hist,
                vec2(ui.available_width(), 72.0),
                true,
                false,
                None,
            );
            return;
        }
        self.histogram_panel(ui, i, s);
        self.tool_strip(ui, i);
        self.film_dialog(ui, i, s);
        let converted = self.frames[i].sc.is_converted();
        let needs_convert = self.needs_convert(i, s);
        // All actions live in one bar at the bottom: Convert while the frame
        // needs it, otherwise Previous and Reset; the menu holds the rest.
        egui::Panel::bottom("sidebar_actions")
            .exact_size(44.0)
            .frame(egui::Frame::new().inner_margin(egui::Margin {
                top: 8,
                ..Default::default()
            }))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;
                    let w = ui.available_width() - 32.0 - 6.0;
                    if needs_convert {
                        ui.allocate_ui(vec2(w, 32.0), |ui| self.convert_button(ui, i, s));
                    } else {
                        let bw = (w - 6.0) / 2.0;
                        let prev = self.editor.prev_edit.clone();
                        if ui
                            .add_enabled(
                                prev.is_some(),
                                widgets::ghost("Previous").min_size(vec2(bw, 32.0)),
                            )
                            .on_hover_text(
                                "Apply the settings of the frame you edited before this one",
                            )
                            .clicked()
                            && let Some(p) = prev
                        {
                            self.paste_edit_to(vec![i], &p);
                        }
                        if ui
                            .add_enabled(
                                self.editor.crop.is_none(),
                                widgets::ghost("Reset").min_size(vec2(bw, 32.0)),
                            )
                            .on_hover_text(
                                "Reset all edits to the defaults (Undo brings them back)",
                            )
                            .clicked()
                        {
                            self.reset_edits(i, s);
                        }
                    }
                    let more = icon_button(
                        ui,
                        Icon::More,
                        "Copy, paste, defaults and more",
                        false,
                        true,
                    );
                    egui::Popup::menu(&more).show(|ui| self.defaults_menu(ui, i, s));
                });
            });
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                // Section open states are remembered by id; scoping the ids to
                // this photo makes each one open with its default state.
                ui.push_id(self.editor.epoch, |ui| {
                    ui.set_width(ui.available_width() - 6.0);
                    if converted {
                        self.adjust_sections(ui, s);
                    } else {
                        ui.add_space(12.0);
                        ui.label(
                            RichText::new(
                                "This negative is not converted yet. Convert it to edit its \
                                 tone and colour; the film details above are kept either way.",
                            )
                            .small()
                            .color(theme::MUTED),
                        );
                    }
                    ui.add_space(16.0);
                });
            });
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
            super::roll::film_fields(ui, "film_meta", &mut self.frames[i].sc.metadata, true);
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

    fn crop_panel(&mut self, ui: &mut egui::Ui, i: usize) {
        ui.add_space(12.0);
        ui.label(
            RichText::new("Crop & straighten")
                .font(theme::medium(14.5))
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
        if let Some(c) = &mut self.editor.crop {
            let free = c.aspect == CropAspect::Free;
            let mut changed = false;
            ui.label(RichText::new("Aspect ratio").small().color(theme::MUTED));
            ui.horizontal(|ui| {
                let tip = if free {
                    "Unlocked: free-form crop. Click to lock the aspect ratio."
                } else {
                    "Locked to the aspect ratio. Click to crop free-form."
                };
                let icon = if free { Icon::Unlock } else { Icon::Lock };
                if icon_button(ui, icon, tip, !free, true).clicked() {
                    c.aspect = if free {
                        c.locked_aspect
                    } else {
                        CropAspect::Free
                    };
                    changed = true;
                }
                egui::ComboBox::from_id_salt("crop_aspect")
                    .width((ui.available_width() - 4.0).max(60.0))
                    .height(400.0)
                    .selected_text(c.locked_aspect.label())
                    .show_ui(ui, |ui| {
                        for &o in CropAspect::ALL.iter().filter(|o| **o != CropAspect::Free) {
                            if ui
                                .selectable_label(c.locked_aspect == o && !free, o.label())
                                .clicked()
                            {
                                c.locked_aspect = o;
                                c.aspect = o;
                                changed = true;
                            }
                        }
                    });
            });
            ui.add_space(4.0);
            if changed {
                self.fit_pending_crop();
                self.editor.dirty = true;
            }
        }
        ui.horizontal(|ui| {
            if ui.add(widgets::ghost("Reset")).clicked() {
                if let Some(c) = &mut self.editor.crop {
                    c.insets = [0.0; 4];
                    c.angle = 0.0;
                }
                self.fit_pending_crop();
                self.editor.dirty = true;
            }
            if ui
                .add(widgets::ghost("Auto"))
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
            // The eyedropper sits in the label column, right-aligned like the
            // labels, and the menu on the track column, like the Profile row.
            ui.spacing_mut().item_spacing.x = 0.0;
            let picking = self.editor.pick == Some(pick);
            ui.allocate_ui_with_layout(
                vec2(widgets::TRACK_INDENT - 7.0, 30.0),
                egui::Layout::right_to_left(egui::Align::Center),
                |ui| {
                    if icon_button(ui, Icon::Eyedropper, tip, picking, enabled).clicked() {
                        self.editor.pick = if picking { None } else { Some(pick) };
                    }
                },
            );
            ui.add_space(7.0);
            // "WB :" and the value as plain text, like the Profile row.
            let caption = match pick {
                Pick::WhiteBalance => "WB",
                Pick::FilmBase => "Base",
            };
            ui.allocate_ui_with_layout(
                vec2(48.0, 30.0),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    ui.label(RichText::new(format!("{caption} :")).color(theme::MUTED));
                },
            );
            ui.add_space(6.0);
            // The value and its menu use a smaller button font than the rest of the row.
            ui.scope(|ui| {
                ui.style_mut()
                    .text_styles
                    .insert(egui::TextStyle::Button, egui::FontId::proportional(11.5));
                widgets::plain_combo(ui, &format!("picker_{caption}"), v, all, name, 150.0);
            });
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
            widgets::profile_row(
                ui,
                "Profile",
                &mut s.tone_profile,
                ToneProfile::ALL,
                ToneProfile::label,
                "",
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
            tone(ui, "Lights", &mut s.highlights, "Upper midtones");
            tone(ui, "Darks", &mut s.shadows, "Lower midtones");
            tone(ui, "Whites", &mut s.whites, "White point roll-off");
            tone(
                ui,
                "Blacks",
                &mut s.blacks,
                "Black point: up = matte, down = deeper",
            );
        }
        if section(ui, Section::PrintCharacter, s, r, false) {
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
            widgets::group_divider(ui);
            let tone = |ui: &mut egui::Ui, l: &str, v: &mut f32, tip: &str| {
                fl_slider(ui, l, v, -50.0..=50.0, 0.0, 0, tip, None)
            };
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
        if section(ui, Section::Colour, s, r, false) {
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
            widgets::group_divider(ui);
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

    /// The ⋯ menu beside the conversion buttons: defaults, unconvert and the LUT folder.
    fn defaults_menu(&mut self, ui: &mut egui::Ui, i: usize, s: &mut Settings) {
        let ctx = ui.ctx().clone();
        let cmd = egui::Modifiers::COMMAND;
        let key = |m, k| RichText::new(widgets::shortcut(&ctx, m, k)).color(theme::FAINT);
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
                    .shortcut_text(RichText::new("?").color(theme::FAINT)),
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
            let k = c
                .aspect
                .ratio(img.width() / img.height())
                .map(|r| r / (img.width() / img.height()));
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
                    if let Some(k) = k {
                        lock_aspect(ins, h, k);
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

#[cfg(test)]
mod crop_lock_tests {
    use super::*;

    #[test]
    fn sections_mark_and_reset_only_their_own_fields() {
        let r = Settings::default();
        let mut s = r.clone();
        assert!(!Section::Colour.modified(&s, &r));
        s.lut_strength = 60.0;
        s.brightness = 12.0;
        lab::nudge(&mut s, lab::Channel::C, 2);
        assert!(Section::Colour.modified(&s, &r));
        assert!(Section::Tone.modified(&s, &r) && Section::Lab.modified(&s, &r));
        Section::Colour.reset(&mut s, &r);
        assert!(!Section::Colour.modified(&s, &r));
        assert_eq!(s.brightness, 12.0, "other sections keep their edits");
        Section::Lab.reset(&mut s, &r);
        assert_eq!(s.filtration, [0.0; 3]);
        assert!(Section::Tone.modified(&s, &r));
    }

    #[test]
    fn dragging_keeps_the_ratio_and_the_anchor() {
        let k = 1.5;
        for h in [
            Handle::TL,
            Handle::BR,
            Handle::L,
            Handle::R,
            Handle::T,
            Handle::B,
        ] {
            let mut ins = [10.0, 12.0, 30.0, 8.0];
            match h {
                Handle::TL | Handle::L => ins[0] = 3.0,
                Handle::BR | Handle::R => ins[2] = 20.0,
                Handle::T => ins[1] = 4.0,
                _ => ins[3] = 2.0,
            }
            lock_aspect(&mut ins, h, k);
            let (w, hh) = (100.0 - ins[0] - ins[2], 100.0 - ins[1] - ins[3]);
            assert!((w / hh - k).abs() < 1e-3, "{h:?} {ins:?}");
            assert!(ins.iter().all(|v| *v >= -1e-3), "{h:?} {ins:?}");
        }
    }
}
