//! The roll's thumbnail grid: uniform cells like Lightroom's, each with its
//! picture centred in its own shape, frame numbers, pick / reject marks,
//! filters, box selection, hover actions, negatives on demand and a quick look. Also the strip of actions for the selection.

use std::collections::BTreeSet;
use std::path::PathBuf;

use eframe::egui::{self, Color32, Pos2, Rect, RichText, Sense, pos2, vec2};

use super::widgets::{self, Icon, theme};
use super::{NlpApp, ProxyState, to_color_image};
use crate::engine::pipeline::{self, Renderer};
use crate::engine::settings::*;
use crate::engine::sidecar::Flag;

/// Which frames the grid shows.
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub enum GridFilter {
    #[default]
    All,
    Picks,
    /// Frames still to decide on.
    Unflagged,
    Rejected,
}

/// One laid-out cell, for keyboard navigation: frame, row and centre x.
#[derive(Clone, Copy)]
struct Cell {
    idx: usize,
    row: usize,
    cx: f32,
}

#[derive(Default)]
pub struct GridState {
    pub filter: GridFilter,
    pub quick_look: bool,
    /// Show the scanned negatives instead of the conversions (also while N is held).
    pub negatives: bool,
    /// The quick look render: frame path, the settings it shows, texture.
    ql: Option<(PathBuf, Settings, egui::TextureHandle)>,
    /// Box selection in progress: start (content coordinates) and the
    /// selection it adds to.
    marquee: Option<(egui::Vec2, BTreeSet<usize>)>,
    /// The last layout, in display order.
    cells: Vec<Cell>,
    /// Frame to scroll into view on the next paint.
    scroll_to: Option<usize>,
}

impl GridState {
    /// Another roll opened: what pointed at the old frames goes, and the
    /// filter returns to All (the new roll may have no picks or rejects).
    /// Showing negatives is a view choice and stays.
    pub fn reset_for_roll(&mut self) {
        *self = Self {
            negatives: self.negatives,
            ..Default::default()
        };
    }
}

#[derive(Clone, Copy)]
enum CellAction {
    Open,
    Convert,
    Export,
    Copy,
    Paste,
    Unconvert,
    DetectFrame,
    Rotate(bool),
    Flag(Flag),
}

/// egui memory key: the frame indices the grid painted last, for
/// visible-first loading.
pub const GRID_VISIBLE: &str = "grid_visible";

/// Gutter between tiles, and between the tiles and the grid's edge.
const GAP: f32 = 8.0;
const MARGIN: f32 = 12.0;
const TILE_RADIUS: f32 = 4.0;
/// Header strip of a tile: frame number, flags, status, the name while hovered.
const HEADER_H: f32 = 26.0;
/// Space around the picture, on the sides and below it.
const PICTURE_PAD: f32 = 8.0;
/// Room for the frame number, so the flags line up from tile to tile.
const NUMBER_W: f32 = 24.0;
/// The pick and reject icons in the header,
const FLAG_SIZE: f32 = 18.0;
/// and their click targets, wide enough to keep the two opposite actions apart.
const FLAG_TARGET: f32 = 24.0;
/// Header text: the frame number and the name.
const HEADER_TEXT: f32 = 11.0;
impl NlpApp {
    /// Whether the grid should show negatives this frame.
    pub fn negatives_shown(&self, ctx: &egui::Context) -> bool {
        self.grid.negatives
            || (!ctx.egui_wants_keyboard_input()
                && !self.grid.quick_look
                && ctx.input(|i| i.key_down(egui::Key::N) && !i.modifiers.any()))
    }

    /// Frames the grid shows under the current filter, in roll order.
    pub fn shown_frames(&self) -> Vec<usize> {
        (0..self.frames.len())
            .filter(|&i| match self.grid.filter {
                GridFilter::All => true,
                GridFilter::Picks => self.frames[i].sc.flag == Flag::Pick,
                GridFilter::Unflagged => self.frames[i].sc.flag == Flag::None,
                GridFilter::Rejected => self.frames[i].sc.flag == Flag::Reject,
            })
            .collect()
    }

    /// Picks, unflagged and rejected frames, for the filter tabs.
    pub fn filter_counts(&self) -> (usize, usize, usize) {
        let count = |flag| self.frames.iter().filter(|f| f.sc.flag == flag).count();
        (count(Flag::Pick), count(Flag::None), count(Flag::Reject))
    }

    /// The frames Export and the contact sheet act on: the selection, or else
    /// the frames on show without the rejected ones.
    pub fn export_scope(&self) -> Vec<usize> {
        if !self.selected.is_empty() {
            return self.sel_vec();
        }
        self.shown_frames()
            .into_iter()
            .filter(|&i| self.frames[i].sc.flag != Flag::Reject)
            .collect()
    }

    /// Rejected frames on show that [`Self::export_scope`] leaves out.
    pub fn rejects_skipped(&self) -> usize {
        if !self.selected.is_empty() {
            return 0;
        }
        self.shown_frames()
            .into_iter()
            .filter(|&i| self.frames[i].sc.flag == Flag::Reject)
            .count()
    }

    /// The Export button's label. "All" is only said when it is true.
    pub fn export_label(&self) -> String {
        let n = self.export_scope().len();
        if self.selected.is_empty() {
            match self.grid.filter {
                GridFilter::All if self.rejects_skipped() == 0 => return "Export all".into(),
                GridFilter::Picks => return "Export picks".into(),
                _ => {}
            }
        }
        if n == 0 {
            return "Export".into();
        }
        format!("Export {n}")
    }

    /// Mark frames as picked, rejected or neither (as one undo-free step:
    /// flags are not edits).
    pub fn set_flag(&mut self, idx: &[usize], flag: Flag) {
        for &i in idx {
            if self.frames[i].sc.flag != flag {
                self.frames[i].sc.flag = flag;
                self.frames[i].dirty_since = Some(self.time);
            }
        }
    }

    /// Flag the selection from the keyboard. Where the filter then hides it,
    /// move on to the next frame on show, so culling carries on without a click.
    pub fn flag_selection(&mut self, flag: Flag) {
        let idx = self.sel_vec();
        let before = self.shown_frames();
        self.set_flag(&idx, flag);
        let after = self.shown_frames();
        if idx.is_empty() || idx.iter().any(|i| after.contains(i)) {
            return;
        }
        let last = idx
            .iter()
            .filter_map(|i| before.iter().position(|b| b == i))
            .max();
        let next = last.and_then(|k| {
            let later = before[k + 1..].iter().find(|b| after.contains(b));
            later.or(before[..k].iter().rev().find(|b| after.contains(b)))
        });
        self.selected.clear();
        self.active = next.copied();
        if let Some(&n) = next {
            self.selected.insert(n);
            self.anchor = Some(n);
            self.grid.scroll_to = Some(n);
        }
    }

    /// P / X / U pressed this frame.
    pub fn flag_key(ctx: &egui::Context) -> Option<Flag> {
        ctx.input(|i| {
            if i.modifiers.any() {
                return None;
            }
            if i.key_pressed(egui::Key::P) {
                Some(Flag::Pick)
            } else if i.key_pressed(egui::Key::X) {
                Some(Flag::Reject)
            } else if i.key_pressed(egui::Key::U) {
                Some(Flag::None)
            } else {
                None
            }
        })
    }

    /// Arrow keys move the selection through the laid-out grid (up / down
    /// to the nearest frame of the next row); Shift extends it.
    pub fn grid_arrows(&mut self, ctx: &egui::Context) {
        let (key, shift) = ctx.input(|i| {
            let k = [
                egui::Key::ArrowLeft,
                egui::Key::ArrowRight,
                egui::Key::ArrowUp,
                egui::Key::ArrowDown,
            ]
            .into_iter()
            .find(|k| i.key_pressed(*k));
            (k, i.modifiers.shift)
        });
        let Some(key) = key else { return };
        if self.grid.cells.is_empty() {
            return;
        }
        let cells = &self.grid.cells;
        let cur = self
            .active
            .filter(|i| self.selected.contains(i))
            .or(self.selected.last().copied())
            .and_then(|i| cells.iter().position(|c| c.idx == i));
        let next = match cur {
            None => 0,
            Some(k) => {
                let c = cells[k];
                match key {
                    egui::Key::ArrowLeft => k.saturating_sub(1),
                    egui::Key::ArrowRight => (k + 1).min(cells.len() - 1),
                    _ => {
                        let row = if key == egui::Key::ArrowUp {
                            c.row.checked_sub(1)
                        } else {
                            Some(c.row + 1)
                        };
                        row.and_then(|r| {
                            cells
                                .iter()
                                .enumerate()
                                .filter(|(_, o)| o.row == r)
                                .min_by(|a, b| {
                                    (a.1.cx - c.cx).abs().total_cmp(&(b.1.cx - c.cx).abs())
                                })
                                .map(|(j, _)| j)
                        })
                        .unwrap_or(k)
                    }
                }
            }
        };
        let next = cells[next].idx;
        if shift {
            let a = self.anchor.unwrap_or(next);
            let (pa, pn) = (
                cells.iter().position(|c| c.idx == a).unwrap_or(0),
                cells.iter().position(|c| c.idx == next).unwrap_or(0),
            );
            self.selected = cells[pa.min(pn)..=pa.max(pn)]
                .iter()
                .map(|c| c.idx)
                .collect();
        } else {
            self.selected.clear();
            self.selected.insert(next);
            self.anchor = Some(next);
        }
        self.active = Some(next);
        self.grid.scroll_to = Some(next);
    }

    pub fn grid(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        let cmd = egui::Modifiers::COMMAND;
        let none = egui::Modifiers::NONE;
        let key = |m, k| widgets::shortcut(&ctx, m, k);
        let keys = [
            key(none, egui::Key::Enter),
            key(cmd, egui::Key::E),
            key(cmd, egui::Key::C),
            key(cmd, egui::Key::V),
            key(cmd, egui::Key::OpenBracket),
            key(cmd, egui::Key::CloseBracket),
        ];
        let negatives = self.negatives_shown(&ctx);
        let shown = self.shown_frames();
        // Frames the filter hides drop out of the selection, so actions
        // never reach frames you can't see.
        self.selected.retain(|i| shown.contains(i));
        if shown.is_empty() {
            ui.add_space((ui.available_height() * 0.35).max(24.0));
            ui.vertical_centered(|ui| {
                let (title, hint) = match self.grid.filter {
                    GridFilter::Unflagged => (
                        "Every frame is flagged",
                        "Picks and rejects have their own tabs.",
                    ),
                    GridFilter::Rejected => {
                        ("No rejected frames", "Press X on a frame to reject it.")
                    }
                    _ => ("No picks yet", "Press P on a frame to pick it."),
                };
                ui.label(RichText::new(title).font(theme::medium(16.0)));
                ui.label(RichText::new(hint).color(theme::MUTED));
                ui.add_space(8.0);
                if ui.add(widgets::ghost("Show all frames")).clicked() {
                    self.grid.filter = GridFilter::All;
                }
            });
            return;
        }
        let now = ui.input(|i| i.time);
        // Pinch or Ctrl/Cmd + scroll over the grid resizes the thumbnails,
        // like the slider in the status strip.
        if ui.rect_contains_pointer(ui.max_rect()) {
            let z = ui.input(|i| i.zoom_delta());
            if z != 1.0 {
                self.thumb_size = (self.thumb_size * z).clamp(110.0, 340.0);
            }
        }
        let mut clicked: Option<(usize, egui::Modifiers)> = None;
        let mut double: Option<usize> = None;
        let mut ctx_action: Option<(usize, CellAction)> = None;
        let mut hover_action: Option<(usize, CellAction)> = None;
        let mut bg_click = false;
        let roll_frames = self.frames.iter().filter(|f| f.is_roll()).count();
        let scroll_to = self.grid.scroll_to.take();
        let mut cells: Vec<Cell> = Vec::new();
        let mut visible: Vec<usize> = Vec::new();
        let mut marquee_hits: Vec<(usize, Rect)> = Vec::new();
        // A quiet scrollbar, and a row cut off by the bar below fades out
        // rather than ending in what reads as empty tiles.
        let scroll = &mut ui.spacing_mut().scroll;
        scroll.foreground_color = false;
        scroll.fade.strength = 1.0;
        scroll.fade.size = 36.0;
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let w = ui.available_width();
                let inner = w - 2.0 * MARGIN;
                // Uniform tiles, as many columns as fit; the picture sits
                // centred in its tile with its own shape.
                let cols = (((inner + GAP) / (self.thumb_size + GAP)).floor() as usize).max(1);
                let cell_w = (inner - GAP * (cols - 1) as f32) / cols as f32;
                // The picture's square, plus the header and padding around it.
                let cell_h = cell_w - 2.0 * PICTURE_PAD + HEADER_H + PICTURE_PAD;
                let aspect = |i: usize| {
                    let f = &self.frames[i];
                    let t = if negatives {
                        f.thumb_neg.as_ref().or(f.thumb.as_ref())
                    } else {
                        f.thumb.as_ref()
                    };
                    match t {
                        Some(t) => {
                            let s = t.size_vec2();
                            (s.x / s.y).clamp(0.4, 3.0)
                        }
                        None if f.sc.settings.rotation % 2 == 1 => 2.0 / 3.0,
                        None => 1.5,
                    }
                };
                let nrows = shown.len().div_ceil(cols);
                let height = nrows as f32 * (cell_h + GAP) - GAP + 2.0 * MARGIN;
                let (content, bg) = ui.allocate_exact_size(
                    vec2(w, height.max(ui.available_height())),
                    Sense::click_and_drag(),
                );
                ui.painter().rect_filled(content, 0.0, theme::BG);

                for (r, items) in shown.chunks(cols).enumerate() {
                    let y = content.top() + MARGIN + r as f32 * (cell_h + GAP);
                    for (c, &i) in items.iter().enumerate() {
                        let x = content.left() + MARGIN + c as f32 * (cell_w + GAP);
                        let cell = Rect::from_min_size(pos2(x, y), vec2(cell_w, cell_h));
                        let area = Rect::from_min_max(
                            cell.min + vec2(PICTURE_PAD, HEADER_H),
                            cell.max - vec2(PICTURE_PAD, PICTURE_PAD),
                        );
                        let a = aspect(i);
                        let fit = vec2(
                            area.width().min(area.height() * a),
                            area.height().min(area.width() / a),
                        );
                        let img = Rect::from_center_size(area.center(), fit);
                        cells.push(Cell {
                            idx: i,
                            row: r,
                            cx: cell.center().x,
                        });
                        marquee_hits.push((i, cell));
                        if scroll_to == Some(i) {
                            ui.scroll_to_rect(cell.expand(GAP), None);
                        }
                        if !ui.is_rect_visible(cell) {
                            continue;
                        }
                        visible.push(i);
                        let resp = ui.interact(cell, ui.id().with(("cell", i)), Sense::click());
                        self.paint_cell(ui, i, img, cell, &resp, negatives, roll_frames, now);
                        // Pick and reject in the header while hovered.
                        if (resp.hovered() || ui.rect_contains_pointer(cell))
                            && let Some(a) = hover_buttons(ui, i, cell, self.frames[i].sc.flag)
                        {
                            hover_action = Some((i, a));
                        }
                        if resp.double_clicked() {
                            double = Some(i);
                        } else if resp.clicked() {
                            clicked = Some((i, ui.input(|inp| inp.modifiers)));
                        }
                        let has_clip = self.clipboard.is_some();
                        let flag = self.frames[i].sc.flag;
                        resp.context_menu(|ui| {
                            let mut item = |ui: &mut egui::Ui, text: &str, key: &str, a, on| {
                                let b = egui::Button::new(text)
                                    .shortcut_text(RichText::new(key).color(theme::FAINT));
                                if ui.add_enabled(on, b).clicked() {
                                    ctx_action = Some((i, a));
                                    ui.close();
                                }
                            };
                            item(ui, "Open in editor", &keys[0], CellAction::Open, true);
                            ui.separator();
                            item(
                                ui,
                                "Pick",
                                "P",
                                CellAction::Flag(Flag::Pick),
                                flag != Flag::Pick,
                            );
                            item(
                                ui,
                                "Reject",
                                "X",
                                CellAction::Flag(Flag::Reject),
                                flag != Flag::Reject,
                            );
                            item(
                                ui,
                                "Unflag",
                                "U",
                                CellAction::Flag(Flag::None),
                                flag != Flag::None,
                            );
                            ui.separator();
                            item(ui, "Convert…", "", CellAction::Convert, true);
                            item(ui, "Export…", &keys[1], CellAction::Export, true);
                            ui.separator();
                            item(ui, "Detect frame", "", CellAction::DetectFrame, true);
                            item(ui, "Rotate left", &keys[4], CellAction::Rotate(false), true);
                            item(ui, "Rotate right", &keys[5], CellAction::Rotate(true), true);
                            ui.separator();
                            item(ui, "Copy settings", &keys[2], CellAction::Copy, true);
                            item(ui, "Paste settings", &keys[3], CellAction::Paste, has_clip);
                            ui.separator();
                            item(ui, "Unconvert", "", CellAction::Unconvert, true);
                        });
                    }
                }

                // Box selection: drag from anywhere that isn't a button.
                if bg.drag_started()
                    && let Some(p) = ui.input(|i| i.pointer.press_origin())
                {
                    let keep = ui.input(|i| i.modifiers.shift || i.modifiers.command);
                    let base = if keep {
                        self.selected.clone()
                    } else {
                        BTreeSet::new()
                    };
                    self.grid.marquee = Some((p - content.min, base));
                }
                if let Some((start, base)) = &self.grid.marquee {
                    if let Some(p) = ui.ctx().pointer_latest_pos() {
                        let r = Rect::from_two_pos(content.min + *start, p);
                        let mut sel = base.clone();
                        sel.extend(
                            marquee_hits
                                .iter()
                                .filter(|(_, c)| c.intersects(r))
                                .map(|(i, _)| *i),
                        );
                        self.selected = sel;
                        ui.painter().rect(
                            r,
                            2.0,
                            theme::ACCENT.gamma_multiply(0.12),
                            egui::Stroke::new(1.0, theme::ACCENT),
                            egui::StrokeKind::Inside,
                        );
                    }
                    if bg.drag_stopped() || !ui.input(|i| i.pointer.primary_down()) {
                        self.grid.marquee = None;
                    }
                }
                if bg.clicked() {
                    bg_click = true;
                }
            });
        ctx.data_mut(|d| d.insert_temp(egui::Id::new(GRID_VISIBLE), visible));
        self.grid.cells = cells;
        if bg_click && clicked.is_none() {
            self.selected.clear();
        }
        if let Some((i, m)) = clicked {
            let order: Vec<usize> = self.grid.cells.iter().map(|c| c.idx).collect();
            if m.shift {
                let a = self.anchor.unwrap_or(i);
                let pos = |x: usize| order.iter().position(|&o| o == x).unwrap_or(0);
                let (lo, hi) = (pos(a).min(pos(i)), pos(a).max(pos(i)));
                if !m.command {
                    self.selected.clear();
                }
                self.selected.extend(order[lo..=hi].iter().copied());
            } else if m.command {
                if !self.selected.remove(&i) {
                    self.selected.insert(i);
                }
                self.anchor = Some(i);
            } else {
                self.selected.clear();
                self.selected.insert(i);
                self.anchor = Some(i);
            }
            self.active = Some(i);
        }
        if let Some(i) = double {
            self.open_editor(i);
        }
        // Hover buttons act on that frame alone, or on the selection it is part of.
        if let Some((i, act)) = hover_action {
            let idx = if self.selected.contains(&i) {
                self.sel_vec()
            } else {
                vec![i]
            };
            if let CellAction::Flag(f) = act {
                self.set_flag(&idx, f);
            }
        }
        if let Some((i, act)) = ctx_action {
            if !self.selected.contains(&i) {
                self.selected.clear();
                self.selected.insert(i);
            }
            self.active = Some(i);
            match act {
                CellAction::Open => self.open_editor(i),
                CellAction::Convert => self.open_convert_for(self.sel_vec()),
                CellAction::Export => self.open_export_dialog(self.sel_vec()),
                CellAction::Copy => self.copy_settings(),
                CellAction::Paste => self.paste_settings(),
                CellAction::Unconvert => self.unconvert(&self.sel_vec()),
                CellAction::DetectFrame => self.detect_frames(&self.sel_vec()),
                CellAction::Rotate(cw) => self.rotate_frames(&self.sel_vec(), cw),
                CellAction::Flag(f) => self.set_flag(&self.sel_vec(), f),
            }
        }
    }

    /// One tile: its ground, the picture and the header row.
    #[allow(clippy::too_many_arguments)]
    fn paint_cell(
        &self,
        ui: &egui::Ui,
        i: usize,
        img: Rect,
        cell: Rect,
        resp: &egui::Response,
        negatives: bool,
        roll_frames: usize,
        now: f64,
    ) {
        let f = &self.frames[i];
        let sel = self.selected.contains(&i);
        let p = ui.painter();
        // Rejected frames are dimmed among the others, not where they are
        // being reviewed.
        let dimmed = f.sc.flag == Flag::Reject && self.grid.filter != GridFilter::Rejected;
        let active = sel && self.active == Some(i);
        // Each frame is a tile holding its number, flags and picture. The ground
        // is the editor canvas's neutral grey, so a frame is judged on the same
        // tone in both views; hover and selection only lighten it.
        let hovered = ui.rect_contains_pointer(cell);
        let ground = if sel {
            theme::PRESSED
        } else if resp.hovered() || hovered {
            theme::HOVER
        } else {
            theme::CANVAS
        };
        p.rect_filled(cell, TILE_RADIUS, ground);
        let tex = if negatives {
            f.thumb_neg.as_ref().or(f.thumb.as_ref())
        } else {
            f.thumb.as_ref()
        };
        if tex.is_some() {
            p.rect_filled(
                img.translate(vec2(1.0, 1.0)),
                0.0,
                Color32::from_black_alpha(70),
            );
        }
        match (tex, &f.proxy) {
            (Some(t), _) => {
                let mut image = egui::Image::new((t.id(), t.size_vec2()));
                if dimmed {
                    image = image.tint(Color32::from_gray(80));
                }
                image.paint_at(ui, img);
            }
            (None, ProxyState::Failed(e)) => {
                resp.clone().on_hover_text(e);
                p.rect_filled(img, 0.0, theme::PANEL);
                widgets::paint_icon(
                    p,
                    Rect::from_center_size(img.center(), vec2(24.0, 24.0)),
                    Icon::Warn,
                    theme::ERR,
                );
            }
            _ => {
                // Loading shimmer.
                let a = (((now * 2.4 + i as f64 * 0.35).sin() * 0.5 + 0.5) * 10.0) as u8 + 6;
                p.rect_filled(img, 0.0, Color32::from_white_alpha(a));
                ui.ctx()
                    .request_repaint_after(std::time::Duration::from_millis(50));
            }
        }
        if sel {
            // Selected tiles get a soft outline, the lead frame a bright one.
            let color = if active {
                theme::TEXT
            } else {
                theme::TEXT.gamma_multiply(0.45)
            };
            p.rect_stroke(
                cell,
                TILE_RADIUS,
                egui::Stroke::new(1.5, color),
                egui::StrokeKind::Inside,
            );
        }
        // The flag sits where its hover button does, so it doesn't jump on hover.
        if !hovered {
            match f.sc.flag {
                Flag::Pick => paint_flag_icon(p, flag_slot(cell, 0), Flag::Pick, theme::OK, true),
                Flag::Reject => {
                    paint_flag_icon(p, flag_slot(cell, 1), Flag::Reject, theme::ERR, true)
                }
                Flag::None => {}
            }
        }
        // Header: frame number, status markers on the right, the name while hovered.
        let row_y = cell.top() + HEADER_H / 2.0;
        let mut x = cell.right() - 8.0;
        let converting = f.auto == super::auto::AutoConvert::Queued
            || f.convert_ticket.is_some_and(|t| t.auto);
        let markers_right = x;
        if f.exporting {
            let a = ((now * 4.0).sin() * 0.35 + 0.65) as f32;
            p.circle_filled(pos2(x - 3.0, row_y), 3.0, theme::WARN.gamma_multiply(a));
            x -= 12.0;
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(50));
        }
        if converting {
            let a = ((now * 4.0).sin() * 0.35 + 0.65) as f32;
            p.circle_filled(pos2(x - 3.0, row_y), 3.0, theme::MUTED.gamma_multiply(a));
            x -= 12.0;
            ui.ctx().request_repaint();
        } else if !f.sc.is_converted() {
            p.circle_stroke(
                pos2(x - 3.0, row_y),
                2.5,
                egui::Stroke::new(1.0, theme::MUTED),
            );
            x -= 12.0;
        } else if !f.is_roll() && roll_frames > 0 {
            // Most of a roll is balanced together: only the exception is marked.
            p.circle_filled(pos2(x - 3.0, row_y), 3.0, theme::MUTED);
            x -= 12.0;
        }
        let markers = Rect::from_min_max(pos2(x, row_y - 8.0), pos2(markers_right, row_y + 8.0));
        if markers.width() > 6.0 {
            let mut tip = vec![if converting {
                "Converting…"
            } else if f.sc.is_converted() {
                "Converted"
            } else {
                "Not converted"
            }];
            if f.is_roll() {
                tip.push("balanced with the roll");
            } else if f.sc.is_converted() && roll_frames > 0 {
                tip.push("left out of the roll's balance");
            }
            if f.exporting {
                tip.push("exporting");
            }
            ui.interact(markers, ui.id().with(("markers", i)), Sense::hover())
                .on_hover_text(tip.join(" · "));
        }
        let dim = if dimmed { 0.6 } else { 1.0 };
        let number = if sel || hovered {
            theme::TEXT
        } else {
            theme::MUTED
        };
        let num = p.layout_no_wrap(
            format!("{:02}", i + 1),
            theme::medium(HEADER_TEXT),
            number.gamma_multiply(dim),
        );
        let nx = cell.left() + 8.0;
        p.galley(pos2(nx, row_y - num.size().y / 2.0), num, number);
        let name_x = flag_slot(cell, 2).left() + 4.0;
        let name_w = x - 4.0 - name_x;
        if hovered && name_w > 24.0 {
            let mut job = egui::text::LayoutJob::simple_singleline(
                f.name.clone(),
                egui::FontId::proportional(HEADER_TEXT),
                theme::TEXT,
            );
            job.wrap = egui::text::TextWrapping::truncate_at_width(name_w);
            let g = p.layout_job(job);
            p.galley(
                pos2(x - 4.0 - g.size().x, row_y - g.size().y / 2.0),
                g,
                theme::TEXT,
            );
        }
    }

    /// Space over the grid: the selected frame large, arrows to move, P / X / U
    /// to cull, Space or Esc to close.
    pub fn quick_look_ui(&mut self, ui: &mut egui::Ui) {
        if !self.grid.quick_look {
            return;
        }
        let Some(i) = self.active.filter(|i| *i < self.frames.len()) else {
            self.grid.quick_look = false;
            return;
        };
        let ctx = ui.ctx().clone();
        // A render from the 1024 px proxy, redone when the frame or its edit changes.
        let f = &self.frames[i];
        let stale = self
            .grid
            .ql
            .as_ref()
            .is_none_or(|(p, s, _)| *p != f.path || *s != f.sc.settings);
        if stale && let Some(src) = f.proxy() {
            let s = &f.sc.settings;
            let neg = pipeline::prepare_negative(&src, s, f.base_for(&src));
            let levels = pipeline::levels_for(s, f.sc.analysis.as_ref(), f.sc.roll.as_ref());
            let r = Renderer::new(s, levels.as_ref(), OutputSpace::Srgb);
            let img = to_color_image(&r.render(&neg, 0.0), neg.width, neg.height);
            let tex = ctx.load_texture("quick-look", img, egui::TextureOptions::LINEAR);
            self.grid.ql = Some((f.path.clone(), s.clone(), tex));
        }
        let f = &self.frames[i];
        let tex = match &self.grid.ql {
            Some((p, _, t)) if *p == f.path => Some(t.clone()),
            _ => f.thumb.clone(),
        };
        let area = ui.max_rect();
        let flag = f.sc.flag;
        let title = format!("{:02}  {}", i + 1, f.name);
        let resp = egui::Area::new(egui::Id::new("quick_look"))
            .order(egui::Order::Foreground)
            .fixed_pos(area.min)
            .show(&ctx, |ui| {
                let (r, resp) = ui.allocate_exact_size(area.size(), Sense::click());
                let p = ui.painter();
                p.rect_filled(r, 0.0, Color32::from_black_alpha(235));
                let pic = r.shrink2(vec2(32.0, 56.0));
                if let Some(t) = tex {
                    let s = t.size_vec2();
                    let k = (pic.width() / s.x).min(pic.height() / s.y);
                    let rect = Rect::from_center_size(pic.center() - vec2(0.0, 8.0), s * k);
                    egui::Image::new((t.id(), s))
                        .corner_radius(2.0)
                        .paint_at(ui, rect);
                }
                let mut line = title.clone();
                match flag {
                    Flag::Pick => line.push_str("  ·  Picked"),
                    Flag::Reject => line.push_str("  ·  Rejected"),
                    Flag::None => {}
                }
                p.text(
                    pos2(r.center().x, r.bottom() - 40.0),
                    egui::Align2::CENTER_CENTER,
                    line,
                    theme::medium(13.0),
                    theme::TEXT,
                );
                let hint = "← → move · P pick · X reject · U unflag · Enter edit · Space close";
                p.text(
                    pos2(r.center().x, r.bottom() - 20.0),
                    egui::Align2::CENTER_CENTER,
                    hint,
                    egui::FontId::proportional(11.5),
                    theme::MUTED,
                );
                resp
            })
            .inner;
        if resp.clicked() {
            self.grid.quick_look = false;
        }
    }

    /// The selection's part of the status strip: its count and what you do
    /// frame by frame while culling. The rest is in the ⋯ menu and on right-click.
    pub fn selection_strip_ui(&mut self, ui: &mut egui::Ui) {
        let n = self.selected.len();
        let ctx = ui.ctx().clone();
        let sel = self.sel_vec();
        let cmd = egui::Modifiers::COMMAND;
        let key = |m, k| widgets::shortcut(&ctx, m, k);
        let all = |f: Flag| sel.iter().all(|&i| self.frames[i].sc.flag == f);
        let (picked, rejected) = (all(Flag::Pick), all(Flag::Reject));
        ui.spacing_mut().item_spacing.x = 4.0;
        ui.label(
            RichText::new(format!("{n} selected"))
                .font(theme::medium(12.5))
                .color(theme::TEXT),
        );
        divider(ui);
        if bar_icon(
            ui,
            if picked { "Unflag (U)" } else { "Pick (P)" },
            picked,
            |p, r, c| paint_flag(p, r.shrink(7.0), if picked { theme::OK } else { c }, picked),
        )
        .clicked()
        {
            self.set_flag(&sel, if picked { Flag::None } else { Flag::Pick });
        }
        if bar_icon(
            ui,
            if rejected { "Unflag (U)" } else { "Reject (X)" },
            rejected,
            |p, r, c| {
                let color = if rejected { theme::ERR } else { c };
                widgets::paint_icon(p, r.shrink(5.0), Icon::Reject, color)
            },
        )
        .clicked()
        {
            self.set_flag(&sel, if rejected { Flag::None } else { Flag::Reject });
        }
        divider(ui);
        let left = format!("Rotate left ({})", key(cmd, egui::Key::OpenBracket));
        let right = format!("Rotate right ({})", key(cmd, egui::Key::CloseBracket));
        if bar_icon(ui, &left, false, |p, r, c| {
            widgets::paint_icon(p, r.shrink(5.0), Icon::RotateLeft, c)
        })
        .clicked()
        {
            self.rotate_frames(&sel, false);
        }
        if bar_icon(ui, &right, false, |p, r, c| {
            widgets::paint_icon(p, r.shrink(5.0), Icon::RotateRight, c)
        })
        .clicked()
        {
            self.rotate_frames(&sel, true);
        }
        divider(ui);
        if ui
            .add(egui::Button::new(RichText::new("Deselect").color(theme::MUTED)).frame(false))
            .on_hover_text("Clear the selection (Esc)")
            .clicked()
        {
            self.selected.clear();
        }
        divider(ui);
        ui.label(
            RichText::new(format!(
                "Space to preview · {} {} to rotate",
                key(cmd, egui::Key::OpenBracket),
                key(cmd, egui::Key::CloseBracket),
            ))
            .small()
            .color(theme::MUTED),
        );
    }

    /// Render the negative thumbnails the grid needs (a few per frame).
    pub fn render_negative_thumbs(&mut self, ctx: &egui::Context) {
        if !self.negatives_shown(ctx) {
            return;
        }
        let mut budget = 6;
        for f in self.frames.iter_mut() {
            if f.thumb_neg.is_some() {
                continue;
            }
            let Some(src) = &f.thumb_src else { continue };
            if budget == 0 {
                ctx.request_repaint();
                break;
            }
            budget -= 1;
            let s = &f.sc.settings;
            let neg = pipeline::prepare_negative(src, s, f.base_for(src));
            let px = Renderer::new(s, None, OutputSpace::Srgb).render(&neg, 0.0);
            f.thumb_neg = Some(ctx.load_texture(
                format!("thumb-neg-{}", f.path.display()),
                to_color_image(&px, neg.width, neg.height),
                egui::TextureOptions::LINEAR,
            ));
        }
    }
}

/// Pick and reject buttons in a hovered cell's header; the one clicked.
fn hover_buttons(ui: &egui::Ui, i: usize, cell: Rect, flag: Flag) -> Option<CellAction> {
    let mut out = None;
    let (picked, rejected) = (flag == Flag::Pick, flag == Flag::Reject);
    let buttons = [
        (
            Flag::Pick,
            if picked { "Unflag (U)" } else { "Pick (P)" },
            picked,
        ),
        (
            Flag::Reject,
            if rejected { "Unflag (U)" } else { "Reject (X)" },
            rejected,
        ),
    ];
    for (k, (target, tip, on)) in buttons.into_iter().enumerate() {
        let r = flag_slot(cell, k);
        let resp = ui
            .interact(r, ui.id().with(("hover_btn", i, k)), Sense::click())
            .on_hover_text(tip);
        let p = ui.painter();
        if resp.hovered() {
            p.rect_filled(r, 3.0, Color32::from_white_alpha(24));
        }
        let idle = if resp.hovered() {
            theme::TEXT
        } else {
            theme::MUTED
        };
        let color = match (on, target) {
            (true, Flag::Pick) => theme::OK,
            (true, _) => theme::ERR,
            _ => idle,
        };
        paint_flag_icon(p, r, target, color, on);
        if resp.clicked() {
            out = Some(CellAction::Flag(if on { Flag::None } else { target }));
        }
    }
    out
}

/// Where the header shows a flag: slot 0 is pick, slot 1 reject.
fn flag_slot(cell: Rect, k: usize) -> Rect {
    Rect::from_min_size(
        pos2(
            cell.left() + 8.0 + NUMBER_W + FLAG_TARGET * k as f32,
            cell.top() + (HEADER_H - FLAG_TARGET) / 2.0,
        ),
        vec2(FLAG_TARGET, FLAG_TARGET),
    )
}

/// The pick pennant or the reject cross, in a flag slot.
fn paint_flag_icon(p: &egui::Painter, slot: Rect, flag: Flag, color: Color32, filled: bool) {
    let r = Rect::from_center_size(slot.center(), vec2(FLAG_SIZE, FLAG_SIZE));
    if flag == Flag::Pick {
        paint_flag(p, r.shrink(FLAG_SIZE * 0.25), color, filled);
    } else {
        widgets::paint_icon(p, r.shrink(FLAG_SIZE * 0.14), Icon::Reject, color);
    }
}

/// A small pennant flag.
pub(super) fn paint_flag(p: &egui::Painter, r: Rect, color: Color32, filled: bool) {
    let pole_x = r.left() + r.width() * 0.25;
    let st = egui::Stroke::new(1.5, color);
    p.line_segment([pos2(pole_x, r.top()), pos2(pole_x, r.bottom())], st);
    let pts: Vec<Pos2> = vec![
        pos2(pole_x, r.top()),
        pos2(r.right(), r.top() + r.height() * 0.28),
        pos2(pole_x, r.top() + r.height() * 0.56),
    ];
    if filled {
        p.add(egui::Shape::convex_polygon(pts, color, egui::Stroke::NONE));
    } else {
        p.add(egui::Shape::closed_line(pts, st));
    }
}

/// Square icon button for the selection strip; `on` shows it pressed.
fn bar_icon(
    ui: &mut egui::Ui,
    tip: &str,
    on: bool,
    paint: impl FnOnce(&egui::Painter, Rect, Color32),
) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(30.0, 28.0), Sense::click());
    let hot = resp.hovered();
    let p = ui.painter();
    if on {
        p.rect_filled(rect, 6.0, theme::PRESSED);
    } else if hot {
        p.rect_filled(rect, 6.0, theme::HOVER);
    }
    paint(p, rect, if hot || on { theme::TEXT } else { theme::MUTED });
    resp.on_hover_text(tip)
}

/// Thin vertical rule between groups of the selection strip.
fn divider(ui: &mut egui::Ui) {
    let (rect, _) = ui.allocate_exact_size(vec2(9.0, 18.0), Sense::hover());
    ui.painter().vline(
        rect.center().x,
        rect.y_range(),
        egui::Stroke::new(1.0, Color32::from_white_alpha(18)),
    );
}
