//! The dodge & burn brush, laid out like the heal tool: a panel under the tool strip
//! (Dodge / Burn / Erase, Exposure, Size, Feather, Flow, Reset, Close) and a bar under
//! the canvas (Show Overlay, Done). Dragging paints a stroke of local print time,
//! shown live; it reaches the frame's settings on release, one undo step per stroke.

use eframe::egui::{self, Color32, Rect, pos2};

use super::heal::{CanvasMap, ring};
use crate::app::NlpApp;
use crate::app::widgets::{self, pill, theme};
use crate::engine::settings::{BurnStroke, Settings};

/// Brush radius as a fraction of the source's long edge: the range.
const MIN_RADIUS: f32 = 0.004;
const MAX_RADIUS: f32 = 0.3;
/// A `[` or `]` press changes the size by this factor.
const SIZE_STEP: f32 = 1.2;
/// Screen points the pointer moves before the stroke gets a new point.
const POINT_STEP: f32 = 2.0;

/// Size as the panel shows it: 1..=100, even steps being even ratios of the radius.
fn size_of(radius: f32) -> f32 {
    1.0 + 99.0 * (radius / MIN_RADIUS).ln() / (MAX_RADIUS / MIN_RADIUS).ln()
}

fn radius_of(size: f32) -> f32 {
    MIN_RADIUS * ((size - 1.0) / 99.0 * (MAX_RADIUS / MIN_RADIUS).ln()).exp()
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum BurnMode {
    Dodge,
    Burn,
    Erase,
}

#[derive(Clone, PartialEq)]
pub(crate) struct BurnEdit {
    pub(crate) mode: BurnMode,
    /// Stops at full coverage, 0.05..=2.
    pub(crate) ev: f32,
    pub(crate) radius: f32,
    pub(crate) feather: f32,
    pub(crate) flow: f32,
    /// The stroke being painted, and the screen point it last grew at.
    stroke: Option<(BurnStroke, egui::Pos2)>,
    pub(crate) overlay: bool,
}

impl Default for BurnEdit {
    fn default() -> Self {
        let s = BurnStroke::default();
        Self {
            mode: BurnMode::Dodge,
            ev: s.ev,
            radius: s.radius,
            feather: s.feather,
            flow: s.flow,
            stroke: None,
            overlay: false,
        }
    }
}

impl BurnEdit {
    /// The tool on the next frame: same brush, nothing being painted.
    pub(super) fn for_next_frame(self) -> Self {
        Self {
            stroke: None,
            ..self
        }
    }

    /// The stroke being painted, while it is.
    pub(super) fn pending(&self) -> Option<&BurnStroke> {
        self.stroke.as_ref().map(|(s, _)| s)
    }

    /// A new stroke at `at` in `mode` (the panel's, or its opposite with Alt).
    fn start(&self, at: [f32; 2], mode: BurnMode) -> BurnStroke {
        BurnStroke {
            points: vec![at],
            radius: self.radius,
            feather: self.feather,
            flow: self.flow,
            ev: match mode {
                BurnMode::Dodge => self.ev,
                BurnMode::Burn => -self.ev,
                BurnMode::Erase => 0.0,
            },
            erase: mode == BurnMode::Erase,
        }
    }
}

/// The overlay's picture of a dodge & burn map: warm where it dodges, blue where it
/// burns, stronger with more stops.
pub(super) fn overlay_image(map: &[f32], w: usize, h: usize) -> egui::ColorImage {
    let px = map
        .iter()
        .map(|&v| {
            let a = ((v.abs() / 1.0).min(1.0) * 150.0) as u8;
            if v >= 0.0 {
                Color32::from_rgba_unmultiplied(255, 170, 60, a)
            } else {
                Color32::from_rgba_unmultiplied(70, 140, 255, a)
            }
        })
        .collect();
    egui::ColorImage::new([w, h], px)
}

impl NlpApp {
    pub(super) fn toggle_burn(&mut self, i: usize) {
        if self.editor.burn.take().is_some() {
            self.editor.dirty = true;
            return;
        }
        if self.editor.crop.is_some() {
            self.apply_crop(i);
        }
        self.editor.pick = None;
        self.editor.heal = None;
        if self.editor.before || self.editor.split {
            self.editor.before = false;
            self.editor.split = false;
            self.editor.dirty = true;
        }
        self.editor.burn = Some(BurnEdit::default());
    }

    /// `[` and `]`: the brush size.
    pub(super) fn step_burn_size(&mut self, larger: bool) {
        if let Some(b) = self.editor.burn.as_mut() {
            let k = if larger { SIZE_STEP } else { 1.0 / SIZE_STEP };
            b.radius = (b.radius * k).clamp(MIN_RADIUS, MAX_RADIUS);
        }
    }

    /// Under the tool strip while the brush is on.
    pub(super) fn burn_panel(&mut self, ui: &mut egui::Ui, s: &mut Settings) {
        let Some(mut b) = self.editor.burn.clone() else {
            return;
        };
        let (mut reset, mut close) = (false, false);
        let d = BurnEdit::default();
        ui.add_space(4.0);
        egui::Frame::new()
            .fill(theme::SURFACE)
            .corner_radius(6.0)
            .inner_margin(egui::Margin::symmetric(10, 8))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("Dodge & Burn")
                            .color(theme::MUTED)
                            .size(12.0),
                    );
                    ui.add_space(12.0);
                    widgets::segmented(
                        ui,
                        &mut b.mode,
                        &[
                            (BurnMode::Dodge, "Dodge"),
                            (BurnMode::Burn, "Burn"),
                            (BurnMode::Erase, "Erase"),
                        ],
                    );
                });
                ui.add_space(4.0);
                ui.add_enabled_ui(b.mode != BurnMode::Erase, |ui| {
                    widgets::fl_slider(
                        ui,
                        "Exposure",
                        &mut b.ev,
                        0.05..=2.0,
                        d.ev,
                        2,
                        "Stops of print time the brush holds back (Dodge) or adds (Burn) where it covers fully",
                        None,
                    );
                });
                let mut size = size_of(b.radius);
                if widgets::fl_slider(
                    ui,
                    "Size",
                    &mut size,
                    1.0..=100.0,
                    size_of(d.radius),
                    0,
                    "Brush size ([ and ])",
                    None,
                ) {
                    b.radius = radius_of(size);
                }
                widgets::fl_slider(
                    ui,
                    "Feather",
                    &mut b.feather,
                    0.0..=100.0,
                    d.feather,
                    0,
                    "How softly the brush's edge fades",
                    None,
                );
                widgets::fl_slider(
                    ui,
                    "Flow",
                    &mut b.flow,
                    1.0..=100.0,
                    d.flow,
                    0,
                    "How quickly a stroke builds up to its full exposure",
                    None,
                );
                ui.add_space(4.0);
                ui.allocate_ui_with_layout(
                    egui::vec2(ui.available_width(), 28.0),
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        ui.spacing_mut().item_spacing.x = 6.0;
                        close = ui.add(widgets::ghost("Close")).clicked();
                        reset = ui
                            .add_enabled(!s.burn.is_empty(), widgets::ghost("Reset"))
                            .on_hover_text("Remove every stroke from this frame")
                            .clicked();
                    },
                );
            });
        if reset {
            s.burn.clear();
            b.stroke = None;
        }
        if close {
            self.editor.dirty = true;
        }
        self.editor.burn = (!close).then_some(b);
    }

    /// The bar under the canvas while the brush is on: Show Overlay, Done.
    pub(super) fn burn_bar(&mut self, ui: &mut egui::Ui) {
        let Some(mut b) = self.editor.burn.clone() else {
            return;
        };
        let mut done = false;
        ui.horizontal_centered(|ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            widgets::checkbox(ui, &mut b.overlay, "Show Overlay")
                .on_hover_text("Tint the strokes: warm where they dodge, blue where they burn (O)");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                done = ui
                    .add(widgets::primary("Done"))
                    .on_hover_text("Enter")
                    .clicked();
            });
        });
        if done {
            self.editor.dirty = true;
        }
        self.editor.burn = (!done).then_some(b);
    }

    /// The strokes' tint over the picture, made again whenever the map changes.
    fn paint_burn_overlay(&mut self, ctx: &egui::Context, p: &egui::Painter, rect: Rect) {
        let Some(neg) = &self.editor.neg else {
            return;
        };
        let Some(map) = neg.local.as_ref() else {
            return;
        };
        if self.editor.burn_tex.is_none() {
            let img = overlay_image(map, neg.width, neg.height);
            self.editor.burn_tex =
                Some(ctx.load_texture("burn-overlay", img, egui::TextureOptions::LINEAR));
        }
        if let Some(t) = &self.editor.burn_tex {
            let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
            p.image(t.id(), rect, uv, Color32::WHITE);
        }
    }

    pub(super) fn burn_overlay(
        &mut self,
        ui: &mut egui::Ui,
        resp: &egui::Response,
        rect: Rect,
        avail: Rect,
        i: usize,
    ) {
        let Some(src) = self.editor.src.clone() else {
            return;
        };
        let Some(mut b) = self.editor.burn.clone() else {
            return;
        };
        let canvas = CanvasMap::new(&src, &self.frames[i].sc.settings, rect);
        let (alt, space) = ui.input(|inp| (inp.modifiers.alt, inp.key_down(egui::Key::Space)));
        let mode = match (b.mode, alt) {
            (BurnMode::Dodge, true) => BurnMode::Burn,
            (BurnMode::Burn, true) => BurnMode::Dodge,
            (m, _) => m,
        };
        let mut commit = None;
        if space && self.editor.zoom.is_some() {
            // Space-drag pans a zoomed picture.
            if resp.dragged() {
                self.editor.zoom_anim = None;
                self.editor.pan += resp.drag_delta();
            }
            ui.ctx().set_cursor_icon(if resp.dragged() {
                egui::CursorIcon::Grabbing
            } else {
                egui::CursorIcon::Grab
            });
        } else {
            if resp.drag_started()
                && let Some(origin) = ui.input(|inp| inp.pointer.press_origin())
                && rect.contains(origin)
            {
                b.stroke = Some((b.start(canvas.to_uv(origin), mode), origin));
                self.editor.dirty = true;
            }
            if resp.dragged()
                && let (Some((stroke, last)), Some(pos)) =
                    (b.stroke.as_mut(), resp.interact_pointer_pos())
                && (pos - *last).length() >= POINT_STEP
            {
                stroke.points.push(canvas.to_uv(pos));
                *last = pos;
                self.editor.dirty = true;
            }
            if resp.drag_stopped() {
                commit = b.stroke.take().map(|(s, _)| s);
            }
            // A click is a single dab.
            if resp.clicked()
                && let Some(pos) = resp.interact_pointer_pos().filter(|p| rect.contains(*p))
            {
                commit = Some(b.start(canvas.to_uv(pos), mode));
            }
        }

        let p = ui.painter_at(rect.intersect(avail));
        if b.overlay {
            let ctx = ui.ctx().clone();
            self.paint_burn_overlay(&ctx, &p, rect);
        }
        if let Some(pos) = resp.hover_pos().filter(|pos| rect.contains(*pos))
            && !(space && self.editor.zoom.is_some())
        {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
            let r = canvas.screen_radius(b.radius);
            let colour = match mode {
                BurnMode::Erase => Color32::from_white_alpha(140),
                _ => Color32::from_white_alpha(210),
            };
            ring(&p, pos, r, colour, 1.0);
            let inner = r * (1.0 - b.feather / 100.0);
            if inner > 2.0 && b.feather > 0.0 {
                ring(&p, pos, inner, Color32::from_white_alpha(110), 1.0);
            }
            let label = match mode {
                BurnMode::Dodge => "+",
                BurnMode::Burn => "−",
                BurnMode::Erase => "×",
            };
            p.text(
                pos,
                egui::Align2::CENTER_CENTER,
                label,
                egui::FontId::proportional(13.0),
                Color32::from_white_alpha(200),
            );
        }
        if self.frames[i].sc.settings.burn.is_empty() && b.stroke.is_none() {
            pill(
                ui.painter(),
                pos2(avail.center().x, avail.top() + 22.0),
                "Drag to dodge or burn · Alt swaps them · [ ] change the size",
            );
        }
        self.editor.burn = Some(b);
        if let Some(stroke) = commit {
            let before = self.frames[i].sc.settings.clone();
            let mut s = before.clone();
            s.burn.push(stroke);
            self.apply_change(i, &before, s);
        }
    }
}
