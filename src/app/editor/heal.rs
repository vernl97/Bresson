//! The spot healing tool, laid out like Lightroom's: a Spot Edit panel under the tool
//! strip (Clone / Heal, Size, Opacity, Reset, Close) and a bar under the canvas (Tool
//! Overlay, Visualize Spots, Done). A click heals a speck from the best clean place
//! nearby, and a drag paints a stroke over a hair or a scratch, healed as one spot;
//! the shapes on the canvas move the spot and its source, and a round spot's rim its
//! size. Space-drag pans a zoomed picture. A change only reaches the frame's settings
//! on release, since each one prepares the negative again.

use eframe::egui::{self, Color32, Rect, Stroke, pos2};

use super::Tool;
use crate::app::NlpApp;
use crate::app::widgets::{self, pill, theme};
use crate::engine::decode::{GeometryMap, SourceImage};
use crate::engine::heal;
use crate::engine::settings::{HealMode, HealSpot, Settings};

/// Spot radius as a fraction of the source's long edge: the default and the range.
const MIN_RADIUS: f32 = 0.0008;
const MAX_RADIUS: f32 = 0.08;
/// A `[` or `]` press changes the size by this factor.
const SIZE_STEP: f32 = 1.2;
/// How near the rim (screen points) a drag resizes rather than moves.
const RIM_GRAB: f32 = 5.0;
/// A press that wanders less than this (screen points) is a click: one round spot.
const CLICK_SLOP: f32 = 4.0;

/// When the spots' circles show.
#[derive(Clone, Copy, PartialEq)]
enum ToolOverlay {
    Always,
    /// While the pointer is over the picture.
    Auto,
    Selected,
    Never,
}

impl ToolOverlay {
    const ALL: [Self; 4] = [Self::Always, Self::Auto, Self::Selected, Self::Never];

    fn label(self) -> &'static str {
        match self {
            Self::Always => "Always",
            Self::Auto => "Auto",
            Self::Selected => "Selected",
            Self::Never => "Never",
        }
    }
}

/// Size as the panel shows it: 1..=100, even steps being even ratios of the radius.
fn size_of(radius: f32) -> f32 {
    1.0 + 99.0 * (radius / MIN_RADIUS).ln() / (MAX_RADIUS / MIN_RADIUS).ln()
}

fn radius_of(size: f32) -> f32 {
    MIN_RADIUS * ((size - 1.0) / 99.0 * (MAX_RADIUS / MIN_RADIUS).ln()).exp()
}

/// The negative's [`heal::spot_map`] and its picture at the current sensitivity.
pub(super) struct SpotMap {
    map: Vec<f32>,
    level: f32,
    tex: egui::TextureHandle,
}

fn spot_map_image(map: &[f32], w: usize, h: usize, level: f32) -> egui::ColorImage {
    let gain = 100f32.powf(level);
    let px = map
        .iter()
        .map(|v| Color32::from_gray(((v * gain).min(1.0) * 255.0) as u8))
        .collect();
    egui::ColorImage::new([w, h], px)
}

#[derive(Clone, Copy, PartialEq)]
enum Grab {
    Spot,
    Source,
    Rim,
}

#[derive(Clone)]
struct HealDrag {
    index: usize,
    grab: Grab,
    /// The spot as it stands; committed on release.
    spot: HealSpot,
    /// From the pointer to the centre it holds, in screen points.
    offset: egui::Vec2,
}

/// A stroke being painted.
#[derive(Clone)]
struct HealPaint {
    /// The brush centres, normalised source coordinates.
    points: Vec<[f32; 2]>,
    /// Where it was pressed, and the last point added, on screen.
    start: egui::Pos2,
    last: egui::Pos2,
    /// It went further than a click's wobble.
    moved: bool,
}

#[derive(Clone)]
pub(crate) struct HealEdit {
    /// The next spot's settings, and the selected spot's.
    radius: f32,
    opacity: f32,
    mode: HealMode,
    selected: Option<usize>,
    drag: Option<HealDrag>,
    /// A stroke being painted, committed on release.
    paint: Option<HealPaint>,
    /// A panel control is held: applied to the selected spot on release.
    pending: bool,
    overlay: ToolOverlay,
    visualize: bool,
    /// Visualize Spots' sensitivity, 0..=1.
    level: f32,
}

impl Default for HealEdit {
    fn default() -> Self {
        let spot = HealSpot::default();
        Self {
            radius: spot.radius,
            opacity: spot.opacity,
            mode: spot.mode,
            selected: None,
            drag: None,
            paint: None,
            pending: false,
            overlay: ToolOverlay::Always,
            visualize: false,
            level: 0.5,
        }
    }
}

impl HealEdit {
    /// The tool on the next frame: same settings, nothing selected.
    pub(super) fn for_next_frame(self) -> Self {
        Self {
            selected: None,
            drag: None,
            paint: None,
            pending: false,
            ..self
        }
    }

    fn select(&mut self, n: usize, spot: &HealSpot) {
        self.selected = Some(n);
        self.radius = spot.radius;
        self.opacity = spot.opacity;
        self.mode = spot.mode;
    }

    #[cfg(test)]
    pub(crate) fn set_visualize(&mut self, on: bool) {
        self.visualize = on;
    }
}

/// Between source pixels of the normalised spot coordinates and the canvas.
pub(super) struct CanvasMap {
    map: GeometryMap,
    w: f32,
    h: f32,
    long: f32,
    rect: Rect,
}

impl CanvasMap {
    /// For the source `src` drawn with `s`'s geometry into `rect`.
    pub(super) fn new(src: &SourceImage, s: &Settings, rect: Rect) -> Self {
        Self {
            map: GeometryMap::new(
                src.width,
                src.height,
                s.rotation,
                s.flip_h,
                s.straighten,
                s.crop,
            ),
            w: src.width as f32,
            h: src.height as f32,
            long: src.width.max(src.height) as f32,
            rect,
        }
    }

    pub(super) fn to_screen(&self, uv: [f32; 2]) -> egui::Pos2 {
        let (x, y) = self.map.to_view(uv[0] * self.w - 0.5, uv[1] * self.h - 0.5);
        let (ow, oh) = (self.map.out.0 as f32, self.map.out.1 as f32);
        pos2(
            self.rect.left() + (x + 0.5) / ow * self.rect.width(),
            self.rect.top() + (y + 0.5) / oh * self.rect.height(),
        )
    }

    pub(super) fn to_uv(&self, pos: egui::Pos2) -> [f32; 2] {
        let (ow, oh) = (self.map.out.0 as f32, self.map.out.1 as f32);
        let x = (pos.x - self.rect.left()) / self.rect.width() * ow - 0.5;
        let y = (pos.y - self.rect.top()) / self.rect.height() * oh - 0.5;
        let (sx, sy) = self.map.to_source(x, y);
        [
            ((sx + 0.5) / self.w).clamp(0.0, 1.0),
            ((sy + 0.5) / self.h).clamp(0.0, 1.0),
        ]
    }

    /// A radius on the canvas, in screen points.
    pub(super) fn screen_radius(&self, radius: f32) -> f32 {
        radius * self.long * self.rect.width() / self.map.out.0 as f32
    }

    /// Whether a source point shows inside the crop.
    fn visible(&self, uv: [f32; 2]) -> bool {
        self.rect.contains(self.to_screen(uv))
    }
}

/// A circle that reads on any picture: light over a dark edge.
pub(super) fn ring(p: &egui::Painter, c: egui::Pos2, r: f32, light: Color32, width: f32) {
    p.circle_stroke(
        c,
        r,
        Stroke::new(width + 1.5, Color32::from_black_alpha(130)),
    );
    p.circle_stroke(c, r, Stroke::new(width, light));
}

/// The line from a spot to its source, between their circles, ending in an arrowhead.
fn arrow(p: &egui::Painter, from: egui::Pos2, to: egui::Pos2, from_r: f32, to_r: f32) {
    let d = to - from;
    let len = d.length();
    // Too little room between the circles for an arrow to read.
    if len <= from_r + to_r + 5.0 {
        return;
    }
    let u = d / len;
    let n = u.rot90();
    let (a, b) = (from + u * from_r, to - u * to_r);
    let head = (len - from_r - to_r).min(7.0);
    let wings = [b - u * head + n * head / 2.0, b - u * head - n * head / 2.0];
    let shadow = Stroke::new(2.5, Color32::from_black_alpha(130));
    let light = Stroke::new(1.0, Color32::WHITE);
    p.line_segment([a, b], shadow);
    p.line_segment([a, b], light);
    p.add(egui::Shape::convex_polygon(
        vec![b, wings[0], wings[1]],
        Color32::WHITE,
        Stroke::new(0.5, Color32::from_black_alpha(130)),
    ));
}

impl NlpApp {
    pub(super) fn toggle_heal(&mut self, i: usize) {
        if self.editor.heal().is_some() {
            self.editor.close_tool();
            return;
        }
        if self.editor.crop().is_some() {
            self.apply_crop(i);
        }
        // The tool works on the edited picture, side by side only gets in the way.
        if self.editor.before || self.editor.split {
            self.editor.before = false;
            self.editor.split = false;
            self.editor.dirty = true;
        }
        self.editor.set_tool(Tool::Heal(HealEdit::default()));
    }

    fn edit_spots(&mut self, i: usize, edit: impl FnOnce(&mut Vec<HealSpot>)) {
        let before = self.frames[i].sc.settings.clone();
        let mut s = before.clone();
        edit(&mut s.heal);
        self.apply_change(i, &before, s);
    }

    /// `[` and `]`: the size of the selected spot, and of the next one.
    pub(super) fn step_heal_size(&mut self, i: usize, larger: bool) {
        let Some(h) = self.editor.heal_mut() else {
            return;
        };
        let k = if larger { SIZE_STEP } else { 1.0 / SIZE_STEP };
        h.radius = (h.radius * k).clamp(MIN_RADIUS, MAX_RADIUS);
        let (radius, selected) = (h.radius, h.selected);
        if let Some(n) = selected {
            self.edit_spots(i, |spots| {
                if let Some(spot) = spots.get_mut(n) {
                    spot.radius = radius;
                }
            });
        }
    }

    /// Delete / Backspace, and the bar's button: remove the selected spot.
    pub(super) fn delete_heal_spot(&mut self, i: usize) {
        let Some(h) = self.editor.heal_mut() else {
            return;
        };
        let Some(n) = h.selected.take() else {
            return;
        };
        self.edit_spots(i, |spots| {
            if n < spots.len() {
                spots.remove(n);
            }
        });
    }

    /// Under the tool strip while healing: Clone / Heal, Size, Opacity, Reset, Close.
    pub(super) fn heal_panel(&mut self, ui: &mut egui::Ui, s: &mut Settings) {
        let Some(mut h) = self.editor.heal().cloned() else {
            return;
        };
        let (mut reset, mut close) = (false, false);
        ui.add_space(4.0);
        egui::Frame::new()
            .fill(theme::SURFACE)
            .corner_radius(6.0)
            .inner_margin(egui::Margin::symmetric(10, 8))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("Spot Edit")
                            .color(theme::MUTED)
                            .size(12.0),
                    );
                    ui.add_space(12.0);
                    h.pending |= widgets::segmented(
                        ui,
                        &mut h.mode,
                        &[(HealMode::Clone, "Clone"), (HealMode::Heal, "Heal")],
                    );
                });
                ui.add_space(4.0);
                let spot = HealSpot::default();
                let mut size = size_of(h.radius);
                if widgets::fl_slider(
                    ui,
                    "Size",
                    &mut size,
                    1.0..=100.0,
                    size_of(spot.radius),
                    0,
                    "Size of the selected spot, or of the next one ([ and ])",
                    None,
                ) {
                    h.radius = radius_of(size);
                    h.pending = true;
                }
                h.pending |= widgets::fl_slider(
                    ui,
                    "Opacity",
                    &mut h.opacity,
                    0.0..=100.0,
                    spot.opacity,
                    0,
                    "How much of the selected spot, or of the next one, shows",
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
                        .add_enabled(!s.heal.is_empty(), widgets::ghost("Reset"))
                        .on_hover_text(
                            "Remove every spot from this frame; Delete removes the selected one",
                        )
                        .clicked();
                    },
                );
            });
        if h.pending && !ui.input(|inp| inp.pointer.any_down()) {
            h.pending = false;
            if let Some(spot) = h.selected.and_then(|n| s.heal.get_mut(n)) {
                (spot.radius, spot.opacity, spot.mode) = (h.radius, h.opacity, h.mode);
            }
        }
        if reset {
            s.heal.clear();
            h.selected = None;
            h.drag = None;
        }
        self.editor.put_back(Tool::Heal(h), !close);
    }

    /// The bar under the canvas while healing: Tool Overlay, Visualize Spots, Done.
    pub(super) fn heal_bar(&mut self, ui: &mut egui::Ui) {
        let Some(mut h) = self.editor.heal().cloned() else {
            return;
        };
        let mut done = false;
        ui.horizontal_centered(|ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            ui.label(
                egui::RichText::new("Tool Overlay")
                    .color(theme::MUTED)
                    .size(12.0),
            );
            widgets::combo("heal_overlay")
                .width(92.0)
                .selected_text(h.overlay.label())
                .show_ui(ui, |ui| {
                    for o in ToolOverlay::ALL {
                        if ui.selectable_label(h.overlay == o, o.label()).clicked() {
                            h.overlay = o;
                        }
                    }
                });
            ui.add_space(14.0);
            widgets::checkbox(ui, &mut h.visualize, "Visualize Spots").on_hover_text(
                "Show the picture as black and white detail, so that dust stands out",
            );
            ui.add_enabled_ui(h.visualize, |ui| {
                widgets::mini_slider(ui, &mut h.level, 0.0..=1.0, 150.0)
                    .on_hover_text("Sensitivity");
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                done = ui
                    .add(widgets::primary("Done"))
                    .on_hover_text("Enter")
                    .clicked();
            });
        });
        self.editor.put_back(Tool::Heal(h), !done);
    }

    /// Draw Visualize Spots over the picture, working the map out when it's gone.
    fn paint_spot_map(&mut self, ctx: &egui::Context, p: &egui::Painter, rect: Rect, level: f32) {
        let Some(neg) = &self.editor.neg else {
            return;
        };
        let (w, h) = (neg.width, neg.height);
        if self.editor.spot_map.is_none() {
            let map = heal::spot_map(&neg.data, w, h);
            let tex = ctx.load_texture(
                "heal-spot-map",
                spot_map_image(&map, w, h, level),
                egui::TextureOptions::LINEAR,
            );
            self.editor.spot_map = Some(SpotMap { map, level, tex });
        }
        let Some(m) = self.editor.spot_map.as_mut() else {
            return;
        };
        if m.level != level {
            m.level = level;
            m.tex.set(
                spot_map_image(&m.map, w, h, level),
                egui::TextureOptions::LINEAR,
            );
        }
        let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
        p.image(m.tex.id(), rect, uv, Color32::WHITE);
    }

    /// A new spot through `points` (one for a round spot, more for a painted
    /// stroke), its source found on the picture as healed so far.
    fn place_spot(&mut self, i: usize, points: &[[f32; 2]], h: &HealEdit, canvas: &CanvasMap) {
        let Some(src) = self.editor.src().cloned() else {
            return;
        };
        let Some(&first) = points.first() else {
            return;
        };
        let (at, path) = if points.len() > 1 {
            HealSpot::stroke(points)
        } else {
            (first, Vec::new())
        };
        let settings = &self.frames[i].sc.settings;
        let healed = (!settings.heal.is_empty())
            .then(|| heal::apply(&src.data, src.width, src.height, &settings.heal));
        let data = healed.as_deref().unwrap_or(&src.data);
        let from = heal::find_source(data, src.width, src.height, at, &path, h.radius, &|uv| {
            canvas.visible(uv)
        });
        let n = settings.heal.len();
        let spot = HealSpot {
            at,
            from,
            radius: h.radius,
            mode: h.mode,
            opacity: h.opacity,
            path,
        };
        self.edit_spots(i, |spots| spots.push(spot));
        if let Some(h) = self.editor.heal_mut() {
            h.selected = Some(n);
        }
    }

    pub(super) fn heal_overlay(
        &mut self,
        ui: &mut egui::Ui,
        resp: &egui::Response,
        rect: Rect,
        avail: Rect,
        i: usize,
    ) {
        let Some(src) = self.editor.src().cloned() else {
            return;
        };
        let Some(mut h) = self.editor.heal().cloned() else {
            return;
        };
        let s = &self.frames[i].sc.settings;
        let canvas = CanvasMap::new(&src, s, rect);
        let mut spots = s.heal.clone();
        // An undo can take the selected spot away.
        h.selected = h.selected.filter(|n| *n < spots.len());
        if h.drag.as_ref().is_some_and(|d| d.index >= spots.len()) {
            h.drag = None;
        }

        // Which shapes are on show, and so can be picked up.
        let overlay_on = resp.hovered() || resp.dragged();
        let overlay = h.overlay;
        let shown = |n: usize, selected: Option<usize>| match overlay {
            ToolOverlay::Always => true,
            ToolOverlay::Auto => overlay_on,
            ToolOverlay::Selected => selected == Some(n),
            ToolOverlay::Never => false,
        };
        let hit = |spots: &[HealSpot], selected: Option<usize>, pos: egui::Pos2| {
            if let Some(n) = selected.filter(|n| shown(*n, selected)) {
                let spot = &spots[n];
                let r = canvas.screen_radius(spot.radius);
                if shape_dist(&source_on_screen(&canvas, spot), pos) <= r.max(6.0) {
                    return Some((n, Grab::Source, canvas.to_screen(spot.from)));
                }
            }
            // The latest spot lies on top.
            spots.iter().enumerate().rev().find_map(|(n, spot)| {
                if !shown(n, selected) {
                    return None;
                }
                let r = canvas.screen_radius(spot.radius);
                let c = canvas.to_screen(spot.at);
                if spot.path.len() > 1 {
                    let d = shape_dist(&spot_on_screen(&canvas, spot), pos);
                    return (d < r.max(4.0)).then_some((n, Grab::Spot, c));
                }
                let d = (pos - c).length();
                if (d - r).abs() <= RIM_GRAB {
                    Some((n, Grab::Rim, c))
                } else if d < r {
                    Some((n, Grab::Spot, c))
                } else {
                    None
                }
            })
        };

        // Space-drag pans a zoomed picture, as in the dodge & burn brush.
        let space = ui.input(|inp| inp.key_down(egui::Key::Space));
        let panning =
            space && self.editor.zoom.is_some() && h.drag.is_none() && h.paint.is_none();
        let brush = canvas.screen_radius(h.radius);
        if resp.drag_started()
            && !panning
            && let Some(origin) = ui.input(|inp| inp.pointer.press_origin())
        {
            if let Some((n, grab, c)) = hit(&spots, h.selected, origin) {
                h.select(n, &spots[n]);
                h.drag = Some(HealDrag {
                    index: n,
                    grab,
                    spot: spots[n].clone(),
                    offset: c - origin,
                });
            } else if rect.contains(origin) {
                // A drag away from the spots paints a stroke.
                h.selected = None;
                h.paint = Some(HealPaint {
                    points: vec![canvas.to_uv(origin)],
                    start: origin,
                    last: origin,
                    moved: false,
                });
            }
        }
        if resp.dragged() {
            match (&mut h.drag, &mut h.paint, resp.interact_pointer_pos()) {
                (Some(d), _, Some(pos)) => match d.grab {
                    Grab::Spot => d.spot.at = canvas.to_uv(pos + d.offset),
                    Grab::Source => d.spot.from = canvas.to_uv(pos + d.offset),
                    Grab::Rim => {
                        let c = canvas.to_screen(d.spot.at);
                        let unit = canvas.screen_radius(1.0).max(1e-6);
                        d.spot.radius = ((pos - c).length() / unit).clamp(MIN_RADIUS, MAX_RADIUS);
                        h.radius = d.spot.radius;
                    }
                },
                (None, Some(paint), Some(pos)) => {
                    let pos = pos.clamp(rect.min, rect.max);
                    paint.moved |= (pos - paint.start).length() > CLICK_SLOP;
                    if (pos - paint.last).length() >= (brush / 3.0).max(2.0) {
                        paint.points.push(canvas.to_uv(pos));
                        paint.last = pos;
                    }
                }
                _ if panning => {
                    self.editor.zoom_anim = None;
                    self.editor.pan += resp.drag_delta();
                }
                _ => {}
            }
        }
        let (released, painted) = if resp.drag_stopped() {
            (h.drag.take(), h.paint.take())
        } else {
            (None, None)
        };

        // The spots as they will be, painted before any commit reorders them.
        if let Some(d) = h.drag.as_ref().or(released.as_ref()) {
            spots[d.index] = d.spot.clone();
        }
        if h.pending
            && let Some(n) = h.selected
        {
            spots[n].radius = h.radius;
        }
        let hover = resp.hover_pos().filter(|pos| rect.contains(*pos));
        let busy = h.drag.is_some() || h.paint.is_some();
        let hovered = hover
            .filter(|_| !busy && !panning)
            .and_then(|pos| hit(&spots, h.selected, pos));

        let p = ui.painter_at(rect.intersect(avail));
        if h.visualize {
            let ctx = ui.ctx().clone();
            self.paint_spot_map(&ctx, &p, rect, h.level);
        }
        for (n, spot) in spots.iter().enumerate() {
            if !shown(n, h.selected) {
                continue;
            }
            let pts = spot_on_screen(&canvas, spot);
            let r = canvas.screen_radius(spot.radius);
            if h.selected == Some(n) {
                let (c, from) = (canvas.to_screen(spot.at), canvas.to_screen(spot.from));
                if spot.path.len() > 1 {
                    arrow(&p, c, from, 0.0, 0.0);
                } else {
                    arrow(&p, c, from, r, r);
                }
                outline(&p, &source_on_screen(&canvas, spot), r, Color32::WHITE, 1.0);
                outline(&p, &pts, r, Color32::WHITE, 2.0);
            } else {
                let on = hovered.is_some_and(|(k, ..)| k == n);
                let alpha = if on { 230 } else { 120 };
                outline(&p, &pts, r, Color32::from_white_alpha(alpha), 1.0);
            }
        }
        if let Some(paint) = &h.paint {
            let pts: Vec<egui::Pos2> = paint.points.iter().map(|uv| canvas.to_screen(*uv)).collect();
            outline(&p, &pts, brush, Color32::WHITE, 1.5);
        }
        match hovered.map(|(_, grab, _)| grab) {
            Some(Grab::Rim) => ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeNeSw),
            Some(_) => ui.ctx().set_cursor_icon(egui::CursorIcon::Grab),
            None => {
                if let Some(pos) = hover.filter(|_| h.drag.is_none() && !panning) {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
                    ring(&p, pos, brush, Color32::from_white_alpha(200), 1.0);
                }
            }
        }
        if h.drag.is_some() || (panning && resp.dragged()) {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
        } else if panning && resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
        }
        if spots.is_empty() && h.paint.is_none() {
            let hint = if self.editor.zoom.is_some() {
                "Click a speck or paint over a hair to heal it · [ ] change the size · Space-drag to pan"
            } else {
                "Click a speck or paint over a hair to heal it · [ ] change the size"
            };
            pill(ui.painter(), pos2(avail.center().x, avail.top() + 22.0), hint);
        }

        let clicked = resp
            .clicked()
            .then(|| resp.interact_pointer_pos())
            .flatten()
            .filter(|pos| rect.contains(*pos));
        let click_hit = clicked.and_then(|pos| hit(&spots, h.selected, pos));
        if let Some((n, ..)) = click_hit {
            h.select(n, &spots[n]);
        }
        self.editor.put_back(Tool::Heal(h.clone()), true);
        if let Some(d) = released {
            self.edit_spots(i, |list| {
                if let Some(slot) = list.get_mut(d.index) {
                    *slot = d.spot;
                }
            });
        }
        if let (Some(pos), None) = (clicked, click_hit) {
            self.place_spot(i, &[canvas.to_uv(pos)], &h, &canvas);
        }
        if let Some(paint) = painted {
            if paint.moved && paint.points.len() > 1 {
                self.place_spot(i, &paint.points, &h, &canvas);
            } else {
                self.place_spot(i, &[canvas.to_uv(paint.start)], &h, &canvas);
            }
        }
    }
}

/// A spot's brush centres on screen.
fn spot_on_screen(canvas: &CanvasMap, spot: &HealSpot) -> Vec<egui::Pos2> {
    spot.points().iter().map(|uv| canvas.to_screen(*uv)).collect()
}

/// Where a spot is copied from: its shape moved to `from`.
fn source_on_screen(canvas: &CanvasMap, spot: &HealSpot) -> Vec<egui::Pos2> {
    let d = [spot.from[0] - spot.at[0], spot.from[1] - spot.at[1]];
    spot.points()
        .iter()
        .map(|uv| canvas.to_screen([uv[0] + d[0], uv[1] + d[1]]))
        .collect()
}

/// Distance from `pos` to the line through `pts` (to the point, for one).
fn shape_dist(pts: &[egui::Pos2], pos: egui::Pos2) -> f32 {
    let line = |a: egui::Pos2, b: egui::Pos2| {
        let ab = b - a;
        let t = if ab.length_sq() > 0.0 {
            ((pos - a).dot(ab) / ab.length_sq()).clamp(0.0, 1.0)
        } else {
            0.0
        };
        (pos - (a + ab * t)).length()
    };
    match pts {
        [] => f32::INFINITY,
        [a] => (pos - *a).length(),
        _ => pts
            .windows(2)
            .map(|w| line(w[0], w[1]))
            .fold(f32::INFINITY, f32::min),
    }
}

/// The outline of a spot's shape: a circle, or a stroke's brush circles where they
/// reach out of the stroke.
fn outline(p: &egui::Painter, pts: &[egui::Pos2], r: f32, light: Color32, width: f32) {
    if pts.len() < 2 {
        if let Some(c) = pts.first() {
            ring(p, *c, r, light, width);
        }
        return;
    }
    if r < 2.0 {
        let line: Vec<egui::Pos2> = pts.to_vec();
        p.add(egui::Shape::line(
            line.clone(),
            Stroke::new(width + 1.5, Color32::from_black_alpha(130)),
        ));
        p.add(egui::Shape::line(line, Stroke::new(width, light)));
        return;
    }
    let raw: Vec<[f32; 2]> = pts.iter().map(|q| [q.x, q.y]).collect();
    let centre = heal::Polyline::new(raw.clone(), r + 2.0);
    const STEPS: usize = 32;
    let mut segs = Vec::new();
    let tol = 0.05 * r + 0.5;
    for c in heal::resample(&raw, r / 3.0) {
        let on: Vec<Option<egui::Pos2>> = (0..=STEPS)
            .map(|k| {
                let a = std::f32::consts::TAU * k as f32 / STEPS as f32;
                let q = [c[0] + a.cos() * r, c[1] + a.sin() * r];
                (centre.dist(q) >= r - tol).then_some(pos2(q[0], q[1]))
            })
            .collect();
        for w in on.windows(2) {
            if let (Some(a), Some(b)) = (w[0], w[1]) {
                segs.push([a, b]);
            }
        }
    }
    let shadow = Stroke::new(width + 1.5, Color32::from_black_alpha(130));
    for s in &segs {
        p.line_segment(*s, shadow);
    }
    for s in &segs {
        p.line_segment(*s, Stroke::new(width, light));
    }
}
