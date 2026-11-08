//! Crop & straighten, laid out like Lightroom's: a panel under the tool strip (Aspect
//! and its lock, turns, level, Auto, Angle, Reset, Close), a bar under the canvas (Tool
//! Overlay, Done), and the pending crop's overlay on the canvas.

use eframe::egui::{self, Color32, Rect, Stroke, pos2, vec2};

use super::Tool;
use crate::app::NlpApp;
use crate::app::widgets::{self, Icon, icon_button, pill, theme};
use crate::engine::settings::*;

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
}

/// What the Aspect row leaves beside its dropdown: the lock button and the gap to it.
/// Less than they take and the sidebar grows on every frame.
const LOCK_W: f32 = 40.0;

/// When the crop's rule-of-thirds grid shows (the bar's Tool Overlay).
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub(in crate::app) enum CropGrid {
    #[default]
    Always,
    /// While the crop or the angle is being changed.
    Auto,
    Never,
}

impl CropGrid {
    const ALL: [Self; 3] = [Self::Always, Self::Auto, Self::Never];

    fn label(self) -> &'static str {
        match self {
            Self::Always => "Always",
            Self::Auto => "Auto",
            Self::Never => "Never",
        }
    }
}

/// The level tool: off, armed for the next drag, or drawing its line.
#[derive(Clone, Copy, Default, PartialEq)]
pub(in crate::app) enum Level {
    #[default]
    Off,
    Armed,
    Drawing([egui::Pos2; 2]),
}

impl Level {
    pub(in crate::app) fn on(self) -> bool {
        self != Self::Off
    }
}

/// Uncommitted crop / straighten state while crop mode is open.
#[derive(Clone, Copy)]
pub(in crate::app) struct CropEdit {
    pub(super) insets: [f32; 4],
    pub(super) angle: f32,
    aspect: CropAspect,
    /// The shape the lock goes back to.
    locked_aspect: CropAspect,
    /// A fixed ratio turned the other way (portrait on a landscape frame).
    swapped: bool,
    /// Quarter turns and mirror, pending like the crop: Cancel undoes them.
    pub(super) rotation: u8,
    pub(super) flip_h: bool,
    drag: Option<Handle>,
    /// The next drag draws a line that should be horizontal or vertical.
    pub(super) level: Level,
    /// The Angle slider is being dragged, which counts as changing the crop for the grid.
    straightening: bool,
}

impl CropEdit {
    /// Width : height the crop keeps for a frame of aspect `frame`, if any.
    fn ratio(&self, frame: f32) -> Option<f32> {
        self.aspect
            .ratio(frame)
            .map(|r| if self.swapped { 1.0 / r } else { r })
    }
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
        Handle::Move => {}
    }
}

/// Rotate crop insets [l, t, r, b] with the image.
pub(crate) fn rotate_insets(c: [f32; 4], clockwise: bool) -> [f32; 4] {
    if clockwise {
        [c[3], c[0], c[1], c[2]]
    } else {
        [c[1], c[2], c[3], c[0]]
    }
}

impl NlpApp {
    pub(crate) fn enter_crop(&mut self, i: usize) {
        let s = &self.frames[i].sc.settings;
        let mut c = CropEdit {
            insets: s.crop,
            angle: s.straighten,
            aspect: s.crop_aspect,
            locked_aspect: if s.crop_aspect == CropAspect::Free {
                CropAspect::Original
            } else {
                s.crop_aspect
            },
            swapped: false,
            rotation: s.rotation,
            flip_h: s.flip_h,
            drag: None,
            level: Level::Off,
            straightening: false,
        };
        // A fixed ratio saved the other way round reopens that way round.
        if let Some(frame) = self.frame_aspect_for(c.rotation)
            && let Some(r) = c.aspect.ratio(frame)
        {
            let [l, t, rr, b] = c.insets;
            let shape = (100.0 - l - rr) * frame / (100.0 - t - b).max(0.01);
            c.swapped = (shape >= 1.0) != (r >= 1.0) && (r - 1.0).abs() > 1e-3;
        }
        self.editor.set_tool(Tool::Crop(c));
        self.fit_pending_crop();
        self.editor.zoom = None;
        self.editor.pan = egui::Vec2::ZERO;
        self.editor.dirty = true;
    }

    pub(super) fn apply_crop(&mut self, i: usize) {
        if let Some(c) = self.editor.crop().copied() {
            self.editor.close_tool();
            let before = self.frames[i].sc.settings.clone();
            let mut s = before.clone();
            s.crop = c.insets;
            s.straighten = c.angle;
            s.crop_aspect = c.aspect;
            s.rotation = c.rotation;
            s.flip_h = c.flip_h;
            self.apply_change(i, &before, s);
        }
        self.editor.dirty = true;
    }

    /// Aspect (w / h) of the frame after its quarter turns, which the crop
    /// insets are relative to.
    fn frame_aspect(&self) -> Option<f32> {
        let rotation = match self.editor.crop() {
            Some(c) => c.rotation,
            None => self
                .active
                .and_then(|i| self.frames.get(i))
                .map_or(0, |f| f.sc.settings.rotation),
        };
        self.frame_aspect_for(rotation)
    }

    fn frame_aspect_for(&self, rotation: u8) -> Option<f32> {
        let src = self.editor.src()?;
        let (w, h) = (src.width as f32, src.height as f32);
        Some(if rotation % 2 == 1 { h / w } else { w / h })
    }

    /// Shrink the pending crop to the selected aspect ratio.
    fn fit_pending_crop(&mut self) {
        let Some(frame) = self.frame_aspect() else {
            return;
        };
        if let Some(c) = self.editor.crop_mut()
            && let Some(ratio) = c.ratio(frame)
        {
            c.insets = crate::engine::frame::fit_aspect(c.insets, frame, ratio);
        }
    }

    /// Quarter turn or mirror inside Crop & straighten. It is pending like the
    /// crop, so the insets turn with the picture and Cancel puts both back.
    pub(super) fn crop_transform(&mut self, turn: Option<bool>) {
        if let Some(c) = self.editor.crop_mut() {
            match turn {
                Some(cw) => {
                    c.rotation = (c.rotation + if cw { 1 } else { 3 }) % 4;
                    c.insets = rotate_insets(c.insets, cw);
                }
                None => {
                    c.flip_h = !c.flip_h;
                    c.insets = [c.insets[2], c.insets[1], c.insets[0], c.insets[3]];
                }
            }
        }
        self.fit_pending_crop();
        self.editor.dirty = true;
    }

    /// Propose the detected frame as the pending crop. The frame is found
    /// free-form and the aspect lock released, so the crop follows the frame.
    pub(super) fn detect_frame(&mut self, i: usize) {
        let Some(src) = self.editor.src().cloned() else {
            return;
        };
        let f = &self.frames[i];
        let mut settings = f.sc.settings.clone();
        settings.crop_aspect = CropAspect::Free;
        if let Some(c) = self.editor.crop() {
            settings.rotation = c.rotation;
            settings.flip_h = c.flip_h;
        }
        let found = crate::engine::frame::detect(&src, &settings, f.base_for(&src));
        let shared = self
            .stored_roll_shapes()
            .of(crate::engine::frame::is_landscape(&src, &settings));
        match crate::engine::frame::settle(found, shared) {
            Some(d) => {
                if let Some(c) = self.editor.crop_mut() {
                    (c.insets, c.angle) = (d.crop, d.straighten);
                    c.aspect = CropAspect::Free;
                    c.swapped = false;
                    self.editor.dirty = true;
                }
            }
            None => self.toast("No frame edge found", false),
        }
    }

    pub(super) fn cancel_crop(&mut self) {
        if self.editor.crop().is_some() {
            self.editor.close_tool();
        }
        self.editor.dirty = true;
    }

    /// Under the tool strip while cropping: Aspect and its lock, turns, level, Auto,
    /// Angle, Reset and Close.
    pub(super) fn crop_panel(&mut self, ui: &mut egui::Ui, i: usize) {
        let Some(mut c) = self.editor.crop().copied() else {
            return;
        };
        let mut changed = false;
        let mut turn: Option<Option<bool>> = None;
        let (mut auto, mut reset, mut close) = (false, false, false);
        ui.add_space(4.0);
        egui::Frame::new()
            .fill(theme::SURFACE)
            .corner_radius(6.0)
            .inner_margin(egui::Margin::symmetric(10, 8))
            .show(ui, |ui| {
                ui.label(
                    egui::RichText::new("Crop & Straighten")
                        .color(theme::MUTED)
                        .size(12.0),
                );
                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    if widgets::combo_row(
                        ui,
                        "Aspect",
                        &mut c.aspect,
                        CropAspect::ALL,
                        CropAspect::label,
                        LOCK_W,
                    ) {
                        c.swapped = false;
                        changed = true;
                    }
                    let locked = c.aspect != CropAspect::Free;
                    let (icon, tip) = if locked {
                        (Icon::Lock, "Aspect locked: click to drag each edge freely")
                    } else {
                        (Icon::Unlock, "Aspect unlocked: click to keep a fixed shape")
                    };
                    if icon_button(ui, icon, tip, locked, true).clicked() {
                        if locked {
                            c.locked_aspect = c.aspect;
                            c.aspect = CropAspect::Free;
                        } else {
                            c.aspect = c.locked_aspect;
                        }
                        c.swapped = false;
                        changed = true;
                    }
                });
                ui.add_space(4.0);
                ui.allocate_ui_with_layout(
                    vec2(ui.available_width(), 28.0),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        ui.spacing_mut().item_spacing.x = 2.0;
                        if icon_button(ui, Icon::RotateLeft, "Rotate left (Cmd+[)", false, true)
                            .clicked()
                        {
                            turn = Some(Some(false));
                        }
                        if icon_button(ui, Icon::RotateRight, "Rotate right (Cmd+])", false, true)
                            .clicked()
                        {
                            turn = Some(Some(true));
                        }
                        if icon_button(ui, Icon::Flip, "Flip horizontally", c.flip_h, true)
                            .clicked()
                        {
                            turn = Some(None);
                        }
                        if icon_button(
                            ui,
                            Icon::Level,
                            "Draw a line along something that should be level",
                            c.level.on(),
                            true,
                        )
                        .clicked()
                        {
                            c.level = if c.level.on() {
                                Level::Off
                            } else {
                                Level::Armed
                            };
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            auto = ui
                                .add(widgets::ghost("Auto"))
                                .on_hover_text("Find the frame inside the film rebate and holder")
                                .clicked();
                        });
                    },
                );
                let moved = widgets::fl_slider(
                    ui,
                    "Angle",
                    &mut c.angle,
                    -45.0..=45.0,
                    0.0,
                    1,
                    "Straighten; or draw a line with the level tool",
                    None,
                );
                c.straightening =
                    (c.straightening || moved) && ui.input(|inp| inp.pointer.any_down());
                changed |= moved;
                ui.add_space(4.0);
                ui.allocate_ui_with_layout(
                    vec2(ui.available_width(), 28.0),
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        ui.spacing_mut().item_spacing.x = 6.0;
                        close = ui
                            .add(widgets::ghost("Close"))
                            .on_hover_text("Apply the crop; Esc cancels it")
                            .clicked();
                        reset = ui
                            .add(widgets::ghost("Reset"))
                            .on_hover_text("Back to the whole frame, unstraightened")
                            .clicked();
                    },
                );
            });
        if reset {
            c.insets = [0.0; 4];
            c.angle = 0.0;
            changed = true;
        }
        self.editor.put_back(Tool::Crop(c), true);
        if changed {
            self.fit_pending_crop();
            self.editor.dirty = true;
        }
        if let Some(t) = turn {
            self.crop_transform(t);
        }
        if auto {
            self.detect_frame(i);
        }
        if close {
            self.apply_crop(i);
        }
    }

    /// The bar under the canvas while cropping: Tool Overlay and Done.
    pub(super) fn crop_bar(&mut self, ui: &mut egui::Ui, i: usize) {
        let mut done = false;
        ui.horizontal_centered(|ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            ui.label(
                egui::RichText::new("Tool Overlay")
                    .color(theme::MUTED)
                    .size(12.0),
            );
            widgets::combo("crop_grid")
                .width(92.0)
                .selected_text(self.editor.crop_grid.label())
                .show_ui(ui, |ui| {
                    for o in CropGrid::ALL {
                        if ui
                            .selectable_label(self.editor.crop_grid == o, o.label())
                            .clicked()
                        {
                            self.editor.crop_grid = o;
                        }
                    }
                })
                .response
                .on_hover_text("When the rule-of-thirds grid shows");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                done = ui
                    .add(widgets::primary("Done"))
                    .on_hover_text("Enter")
                    .clicked();
            });
        });
        if done {
            self.apply_crop(i);
        }
    }

    pub(super) fn crop_overlay(
        &mut self,
        ui: &mut egui::Ui,
        resp: &egui::Response,
        img: Rect,
        avail: Rect,
    ) {
        let Some(mut c) = self.editor.crop().copied() else {
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
                _ => return None,
            })
        };
        if c.level.on() {
            // Level tool: the drag is a line, not a crop edit.
            if let Some(pos) = resp.interact_pointer_pos() {
                if resp.drag_started() {
                    c.level = Level::Drawing([pos, pos]);
                } else if resp.dragged()
                    && let Level::Drawing(line) = &mut c.level
                {
                    line[1] = pos;
                }
            }
            if resp.drag_stopped() {
                if let Level::Drawing([a, b]) = c.level {
                    let d = b - a;
                    if d.length() > 24.0 {
                        // Tilt from the nearest axis, so a vertical edge levels as well.
                        let mut tilt = d.y.atan2(d.x).to_degrees();
                        tilt -= (tilt / 90.0).round() * 90.0;
                        // Content turns clockwise with a positive angle, so undo the tilt.
                        c.angle = ((c.angle - tilt).clamp(-45.0, 45.0) * 10.0).round() / 10.0;
                        self.editor.dirty = true;
                    }
                }
                c.level = Level::Off;
            }
            if resp.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
            }
        } else if resp.drag_started() {
            c.drag = resp.interact_pointer_pos().and_then(hit);
        }
        if let (true, Some(h)) = (resp.dragged(), c.drag.filter(|_| !c.level.on())) {
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
            c.drag = None;
        }
        if let Some(pos) = resp.hover_pos().filter(|_| !c.level.on()) {
            let icon = match c.drag.or_else(|| hit(pos)) {
                Some(Handle::L | Handle::R) => egui::CursorIcon::ResizeHorizontal,
                Some(Handle::T | Handle::B) => egui::CursorIcon::ResizeVertical,
                Some(Handle::TL | Handle::BR) => egui::CursorIcon::ResizeNwSe,
                Some(Handle::TR | Handle::BL) => egui::CursorIcon::ResizeNeSw,
                Some(Handle::Move) => egui::CursorIcon::Move,
                None => egui::CursorIcon::Default,
            };
            ui.ctx().set_cursor_icon(icon);
        }
        let angle_changed = self
            .editor
            .crop()
            .map(|o| o.angle != c.angle)
            .unwrap_or(false);
        self.editor.put_back(Tool::Crop(c), true);
        if angle_changed {
            self.editor.dirty = true;
        }

        // Drawing.
        let r = to_px(c.insets);
        let p = ui.painter_at(avail);
        // Dark enough that a bright holder or scanner bed outside the crop ends
        // up near the canvas tone, not as a light surround to the print.
        let shade = Color32::from_black_alpha(205);
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
        // Dark under light, so the frame and handles read on a pale picture too.
        let ink = Color32::from_black_alpha(170);
        p.rect_stroke(r, 0.0, Stroke::new(3.5, ink), egui::StrokeKind::Middle);
        p.rect_stroke(
            r,
            0.0,
            Stroke::new(1.5, Color32::WHITE),
            egui::StrokeKind::Middle,
        );
        let changing = c.drag.is_some() || c.straightening;
        let grid = match self.editor.crop_grid {
            CropGrid::Always => true,
            CropGrid::Auto => changing,
            CropGrid::Never => false,
        };
        if grid {
            let back = Stroke::new(2.0, Color32::from_black_alpha(90));
            let thirds = Stroke::new(0.8, Color32::from_white_alpha(150));
            for k in 1..3 {
                let fx = r.left() + r.width() * k as f32 / 3.0;
                let fy = r.top() + r.height() * k as f32 / 3.0;
                for st in [back, thirds] {
                    p.line_segment([pos2(fx, r.top()), pos2(fx, r.bottom())], st);
                    p.line_segment([pos2(r.left(), fy), pos2(r.right(), fy)], st);
                }
            }
        }
        let len = 22.0f32.min(r.width() / 3.0).min(r.height() / 3.0);
        for (corner, sx, sy) in [
            (r.left_top(), 1.0, 1.0),
            (r.right_top(), -1.0, 1.0),
            (r.left_bottom(), 1.0, -1.0),
            (r.right_bottom(), -1.0, -1.0),
        ] {
            for hs in [Stroke::new(5.5, ink), Stroke::new(3.0, theme::TEXT)] {
                p.line_segment([corner, corner + vec2(len * sx, 0.0)], hs);
                p.line_segment([corner, corner + vec2(0.0, len * sy)], hs);
            }
        }
        // The level line being drawn.
        if let Level::Drawing([a, b]) = c.level {
            p.line_segment([a, b], Stroke::new(3.5, ink));
            p.line_segment([a, b], Stroke::new(1.5, theme::ACCENT));
            for e in [a, b] {
                p.circle(e, 4.0, theme::ACCENT, Stroke::new(1.5, Color32::WHITE));
            }
        } else if c.level.on() {
            pill(
                &p,
                pos2(avail.center().x, avail.top() + 22.0),
                "Drag along something that should be level or upright · Esc to cancel",
            );
        }
    }
}

#[cfg(test)]
mod crop_lock_tests {
    use super::*;

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
