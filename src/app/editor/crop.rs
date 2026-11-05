//! Crop & straighten: the pending crop, the bar under the canvas and the overlay on it.

use eframe::egui::{self, Color32, Rect, Stroke, pos2, vec2};

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

/// Uncommitted crop / straighten state while crop mode is open.
#[derive(Clone, Copy)]
pub(in crate::app) struct CropEdit {
    pub(super) insets: [f32; 4],
    pub(super) angle: f32,
    aspect: CropAspect,
    /// A fixed ratio turned the other way (portrait on a landscape frame).
    swapped: bool,
    /// Quarter turns and mirror, pending like the crop: Cancel undoes them.
    pub(super) rotation: u8,
    pub(super) flip_h: bool,
    drag: Option<Handle>,
    /// The level tool is armed: the next drag draws a line that should be horizontal or vertical.
    pub(super) level: bool,
    pub(super) level_line: Option<[egui::Pos2; 2]>,
    /// The ruler is being dragged, so the thirds show as they do while cropping.
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

/// Degrees either side of 0 the straighten ruler spans; the typed value reaches further.
const RULER_SPAN: f32 = 15.0;
/// Rotate crop insets [l, t, r, b] with the image.
pub(crate) fn rotate_insets(c: [f32; 4], clockwise: bool) -> [f32; 4] {
    if clockwise {
        [c[3], c[0], c[1], c[2]]
    } else {
        [c[1], c[2], c[3], c[0]]
    }
}

/// The straighten ruler: ticks every degree, 0 in the middle, the angle as a marker.
fn paint_ruler(ui: &egui::Ui, rect: Rect, angle: f32, hot: bool) {
    let p = ui.painter();
    let mid = rect.center();
    let half = rect.width() / 2.0;
    let x_of = |a: f32| mid.x + a / RULER_SPAN * half;
    let base = rect.bottom() - 5.0;
    for k in -(RULER_SPAN as i32)..=RULER_SPAN as i32 {
        let (len, col) = match k {
            0 => (12.0, theme::TEXT),
            k if k % 5 == 0 => (9.0, theme::MUTED),
            _ => (5.0, theme::FAINT),
        };
        let x = x_of(k as f32);
        p.line_segment([pos2(x, base), pos2(x, base - len)], Stroke::new(1.0, col));
    }
    let x = x_of(angle.clamp(-RULER_SPAN, RULER_SPAN));
    let col = if hot { theme::TEXT } else { theme::ACCENT };
    p.line_segment(
        [pos2(x, rect.top() + 3.0), pos2(x, base)],
        Stroke::new(2.0, col),
    );
    p.circle_filled(pos2(x, rect.top() + 4.0), 3.0, col);
}

impl NlpApp {
    pub(crate) fn enter_crop(&mut self, i: usize) {
        let s = &self.frames[i].sc.settings;
        let mut c = CropEdit {
            insets: s.crop,
            angle: s.straighten,
            aspect: s.crop_aspect,
            swapped: false,
            rotation: s.rotation,
            flip_h: s.flip_h,
            drag: None,
            level: false,
            level_line: None,
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
        self.editor.crop = Some(c);
        self.fit_pending_crop();
        self.editor.pick = None;
        self.editor.zoom = None;
        self.editor.pan = egui::Vec2::ZERO;
        self.editor.dirty = true;
    }

    pub(super) fn apply_crop(&mut self, i: usize) {
        if let Some(c) = self.editor.crop.take() {
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
        let rotation = match &self.editor.crop {
            Some(c) => c.rotation,
            None => self
                .active
                .and_then(|i| self.frames.get(i))
                .map_or(0, |f| f.sc.settings.rotation),
        };
        self.frame_aspect_for(rotation)
    }

    fn frame_aspect_for(&self, rotation: u8) -> Option<f32> {
        let src = self.editor.src.as_ref()?;
        let (w, h) = (src.width as f32, src.height as f32);
        Some(if rotation % 2 == 1 { h / w } else { w / h })
    }

    /// Shrink the pending crop to the selected aspect ratio.
    fn fit_pending_crop(&mut self) {
        let Some(frame) = self.frame_aspect() else {
            return;
        };
        if let Some(c) = &mut self.editor.crop {
            if let Some(ratio) = c.ratio(frame) {
                c.insets = crate::engine::frame::fit_aspect(c.insets, frame, ratio);
            }
        }
    }

    /// Quarter turn or mirror inside Crop & straighten. It is pending like the
    /// crop, so the insets turn with the picture and Cancel puts both back.
    pub(super) fn crop_transform(&mut self, turn: Option<bool>) {
        if let Some(c) = &mut self.editor.crop {
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
        let Some(src) = self.editor.src.clone() else {
            return;
        };
        let f = &self.frames[i];
        let mut settings = f.sc.settings.clone();
        settings.crop_aspect = CropAspect::Free;
        if let Some(c) = &self.editor.crop {
            settings.rotation = c.rotation;
            settings.flip_h = c.flip_h;
        }
        let found = crate::engine::frame::detect(&src, &settings, f.base_for(&src));
        let shared = self
            .stored_roll_shapes()
            .of(crate::engine::frame::is_landscape(&src, &settings));
        match crate::engine::frame::settle(found, shared) {
            Some(d) => {
                if let Some(c) = &mut self.editor.crop {
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
        self.editor.crop = None;
        self.editor.dirty = true;
    }

    /// The strip under the canvas while cropping, in the view bar's place: shape,
    /// turns, the straighten ruler, then what proposes a crop and what ends the mode.
    pub(super) fn crop_bar(&mut self, ui: &mut egui::Ui, i: usize) {
        let Some(mut c) = self.editor.crop else {
            return;
        };
        let mut changed = false;
        let mut turn: Option<Option<bool>> = None;
        let (mut detect, mut reset, mut done, mut cancel) = (false, false, false, false);
        let mut angle_edit = false;
        ui.horizontal_centered(|ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            let divider = |ui: &mut egui::Ui| {
                ui.add_space(6.0);
                let (rect, _) = ui.allocate_exact_size(vec2(1.0, 16.0), egui::Sense::hover());
                ui.painter().rect_filled(rect, 0.0, theme::TRACK);
                ui.add_space(6.0);
            };
            // Aspect: Free is a choice like the others, so the box always says what the crop does.
            widgets::combo("crop_aspect")
                .width(104.0)
                .height(400.0)
                .selected_text(c.aspect.label())
                .show_ui(ui, |ui| {
                    for &o in CropAspect::ALL {
                        if ui.selectable_label(c.aspect == o, o.label()).clicked() {
                            c.aspect = o;
                            c.swapped = false;
                            changed = true;
                        }
                    }
                })
                .response
                .on_hover_text("Crop shape; Free drags each edge on its own");
            divider(ui);
            if icon_button(ui, Icon::RotateLeft, "Rotate left (Cmd+[)", false, true).clicked() {
                turn = Some(Some(false));
            }
            if icon_button(ui, Icon::RotateRight, "Rotate right (Cmd+])", false, true).clicked() {
                turn = Some(Some(true));
            }
            if icon_button(ui, Icon::Flip, "Flip horizontally", c.flip_h, true).clicked() {
                turn = Some(None);
            }
            divider(ui);

            // Everything right of the ruler has a fixed size; the ruler takes the rest.
            const RIGHT_W: f32 = 322.0;
            const VALUE_W: f32 = 58.0;
            let ruler_w = (ui.available_width() - RIGHT_W - VALUE_W - 40.0).clamp(90.0, 300.0);
            let (rect, resp) =
                ui.allocate_exact_size(vec2(ruler_w, 28.0), egui::Sense::click_and_drag());
            if let Some(pos) = resp.interact_pointer_pos().filter(|_| resp.dragged() || resp.clicked())
            {
                let a = (pos.x - rect.center().x) / (rect.width() / 2.0) * RULER_SPAN;
                let a = (a.clamp(-RULER_SPAN, RULER_SPAN) * 10.0).round() / 10.0;
                // A pull towards level: within a tenth of a degree it lands on 0.
                c.angle = if a.abs() < 0.15 { 0.0 } else { a };
                angle_edit = true;
            }
            if resp.double_clicked() {
                c.angle = 0.0;
                angle_edit = true;
            }
            c.straightening = resp.dragged();
            paint_ruler(ui, rect, c.angle, resp.hovered() || resp.dragged());
            resp.on_hover_text("Straighten: drag along the ruler, double-click for 0°");
            let mut angle = c.angle;
            let typed = ui
                .add_sized(
                    vec2(VALUE_W, 24.0),
                    egui::DragValue::new(&mut angle)
                        .range(-45.0..=45.0)
                        .speed(0.05)
                        .custom_formatter(|x, _| format!("{x:+.1}°"))
                        .custom_parser(|t| {
                            t.trim()
                                .trim_end_matches('°')
                                .replace('−', "-")
                                .parse()
                                .ok()
                        }),
                )
                .on_hover_text("Straighten angle: drag, or click to type")
                .changed();
            if typed {
                c.angle = angle;
                angle_edit = true;
            }
            if icon_button(
                ui,
                Icon::Level,
                "Draw a line along something that should be level",
                c.level,
                true,
            )
            .clicked()
            {
                c.level = !c.level;
                c.level_line = None;
            }
            divider(ui);
            if ui
                .add(widgets::ghost("Detect frame"))
                .on_hover_text("Find the frame inside the film rebate and holder")
                .clicked()
            {
                detect = true;
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 6.0;
                if ui
                    .add(widgets::primary("Done"))
                    .on_hover_text("Enter")
                    .clicked()
                {
                    done = true;
                }
                if ui
                    .add(widgets::ghost("Cancel"))
                    .on_hover_text("Esc")
                    .clicked()
                {
                    cancel = true;
                }
                if ui
                    .add(widgets::ghost("Reset"))
                    .on_hover_text("Back to the whole frame, unstraightened")
                    .clicked()
                {
                    reset = true;
                }
            });
        });
        if reset {
            c.insets = [0.0; 4];
            c.angle = 0.0;
            changed = true;
            angle_edit = true;
        }
        let drag = self.editor.crop.map(|o| o.drag).unwrap_or(None);
        c.drag = drag;
        self.editor.crop = Some(c);
        if changed {
            self.fit_pending_crop();
            self.editor.dirty = true;
        }
        if angle_edit {
            self.editor.dirty = true;
        }
        if let Some(t) = turn {
            self.crop_transform(t);
        }
        if detect {
            self.detect_frame(i);
        }
        if done {
            self.apply_crop(i);
        } else if cancel {
            self.cancel_crop();
        }
    }

    pub(super) fn crop_overlay(
        &mut self,
        ui: &mut egui::Ui,
        resp: &egui::Response,
        img: Rect,
        avail: Rect,
    ) {
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
        if c.level {
            // Level tool: the drag is a line, not a crop edit.
            if let Some(pos) = resp.interact_pointer_pos() {
                if resp.drag_started() {
                    c.level_line = Some([pos, pos]);
                } else if resp.dragged()
                    && let Some(line) = &mut c.level_line
                {
                    line[1] = pos;
                }
            }
            if resp.drag_stopped() {
                if let Some([a, b]) = c.level_line.take() {
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
                c.level = false;
            }
            if resp.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
            }
        } else if resp.drag_started() {
            c.drag = resp.interact_pointer_pos().and_then(hit);
        }
        if let (true, Some(h)) = (resp.dragged(), c.drag.filter(|_| !c.level)) {
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
        if let Some(pos) = resp.hover_pos().filter(|_| !c.level) {
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
        p.rect_stroke(
            r,
            0.0,
            Stroke::new(3.5, ink),
            egui::StrokeKind::Middle,
        );
        p.rect_stroke(
            r,
            0.0,
            Stroke::new(1.5, Color32::WHITE),
            egui::StrokeKind::Middle,
        );
        // Thirds only while the crop or the angle is being changed.
        if c.drag.is_some() || c.straightening {
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
        if let Some([a, b]) = c.level_line {
            p.line_segment([a, b], Stroke::new(3.5, ink));
            p.line_segment([a, b], Stroke::new(1.5, theme::ACCENT));
            for e in [a, b] {
                p.circle(e, 4.0, theme::ACCENT, Stroke::new(1.5, Color32::WHITE));
            }
        } else if c.level {
            pill(&p, pos2(avail.center().x, avail.top() + 22.0), "Drag along something that should be level or upright · Esc to cancel");
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
