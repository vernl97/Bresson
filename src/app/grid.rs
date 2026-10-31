//! The roll's thumbnail grid: justified rows shaped like the frames, frame
//! numbers, pick / reject marks, a "needs a look" check, filters, box
//! selection, hover actions, negatives on demand and a quick look.

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
    NeedsLook,
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

#[derive(Clone, Copy)]
enum CellAction {
    Open,
    Convert,
    ConvertRoll,
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

const GAP: f32 = 12.0;
const NAME_H: f32 = 24.0;
/// Clipped share of the print that earns a "needs a look".
const CLIP_WHITE: f32 = 0.03;
const CLIP_BLACK: f32 = 0.06;
/// Space kept free under the grid for the floating selection bar.
const SELECTION_BAR_ROOM: f32 = 64.0;
/// Density (≈ 1 stop) away from the roll's median that earns one.
const DENSITY_OUTLIER: f64 = 0.3;

impl NlpApp {
    /// Whether the grid should show negatives this frame.
    pub fn negatives_shown(&self, ctx: &egui::Context) -> bool {
        self.grid.negatives
            || (!ctx.egui_wants_keyboard_input()
                && !self.grid.quick_look
                && ctx.input(|i| i.key_down(egui::Key::N) && !i.modifiers.any()))
    }

    /// Median mean-green density of the converted frames (four or more).
    fn roll_density(&self) -> Option<f64> {
        let mut d: Vec<f64> = self
            .frames
            .iter()
            .filter_map(|f| f.sc.analysis.as_ref().map(|a| a.auto_wb.mean[1]))
            .filter(|v| v.is_finite() && *v != 0.0)
            .collect();
        if d.len() < 4 {
            return None;
        }
        d.sort_by(f64::total_cmp);
        Some(d[d.len() / 2])
    }

    /// Why frame `i` deserves a closer look (empty when it looks fine).
    fn look_issues(&self, i: usize, roll_density: Option<f64>) -> Vec<String> {
        let f = &self.frames[i];
        let mut out = Vec::new();
        let Some(a) = f.sc.analysis.as_ref() else {
            return out;
        };
        let s = &f.sc.settings;
        let auto_wb = matches!(
            s.wb_mode,
            WbMode::AutoNeutral | WbMode::AutoWarm | WbMode::AutoCool | WbMode::AutoMix
        );
        if auto_wb && a.auto_wb.confidence < pipeline::LOW_CONFIDENCE {
            out.push("Few neutral tones to balance on: check the colour".into());
        }
        if let Some([white, black]) = f.clip {
            if white > CLIP_WHITE {
                out.push(format!("{:.0}% of the print clips to white", white * 100.0));
            }
            if black > CLIP_BLACK {
                out.push(format!(
                    "{:.0}% of the print blocks up to black",
                    black * 100.0
                ));
            }
        }
        if let Some(m) = roll_density {
            let d = a.auto_wb.mean[1] - m;
            if d > DENSITY_OUTLIER {
                out.push("Much denser negative than the rest of the roll".into());
            } else if d < -DENSITY_OUTLIER {
                out.push("Much thinner negative than the rest of the roll".into());
            }
        }
        out
    }

    /// Frames the grid shows under the current filter, in roll order.
    pub fn shown_frames(&self) -> Vec<usize> {
        let density = self.roll_density();
        (0..self.frames.len())
            .filter(|&i| match self.grid.filter {
                GridFilter::All => true,
                GridFilter::Picks => self.frames[i].sc.flag == Flag::Pick,
                GridFilter::NeedsLook => !self.look_issues(i, density).is_empty(),
            })
            .collect()
    }

    /// (picks, needing a look) for the filter tabs.
    pub fn filter_counts(&self) -> (usize, usize) {
        let density = self.roll_density();
        let picks = self
            .frames
            .iter()
            .filter(|f| f.sc.flag == Flag::Pick)
            .count();
        let look = (0..self.frames.len())
            .filter(|&i| !self.look_issues(i, density).is_empty())
            .count();
        (picks, look)
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
        if self.grid.cells.is_empty() || self.export_targets.is_some() {
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
        let density = self.roll_density();
        let shown = self.shown_frames();
        // Frames the filter hides drop out of the selection, so actions
        // never reach frames you can't see.
        self.selected.retain(|i| shown.contains(i));
        if shown.is_empty() {
            ui.add_space((ui.available_height() * 0.35).max(24.0));
            ui.vertical_centered(|ui| {
                let (title, hint) = match self.grid.filter {
                    GridFilter::Picks => ("No picks yet", "Press P on a frame to pick it."),
                    _ => ("Nothing needs a look", "Every converted frame looks fine."),
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
        let target_h = (self.thumb_size * 0.62).round();
        let mut clicked: Option<(usize, egui::Modifiers)> = None;
        let mut double: Option<usize> = None;
        let mut ctx_action: Option<(usize, CellAction)> = None;
        let mut hover_action: Option<(usize, CellAction)> = None;
        let mut bg_click = false;
        let multi = self.selected.len() >= 2;
        let scroll_to = self.grid.scroll_to.take();
        let mut cells: Vec<Cell> = Vec::new();
        let mut visible: Vec<usize> = Vec::new();
        let mut marquee_hits: Vec<(usize, Rect)> = Vec::new();
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let w = ui.available_width();
                // Justified rows: frames keep their shape, each row fills the width.
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
                let mut rows: Vec<(Vec<usize>, f32)> = Vec::new();
                let mut row: Vec<usize> = Vec::new();
                let mut sum = 0.0;
                for &i in &shown {
                    row.push(i);
                    sum += aspect(i);
                    let natural = sum * target_h + GAP * (row.len() - 1) as f32;
                    if natural >= w {
                        let h = (w - GAP * (row.len() - 1) as f32) / sum;
                        rows.push((std::mem::take(&mut row), h));
                        sum = 0.0;
                    }
                }
                if !row.is_empty() {
                    rows.push((row, target_h));
                }
                // Room under the last row so the selection bar never covers it.
                let height: f32 =
                    rows.iter().map(|r| r.1 + NAME_H + GAP).sum::<f32>() + SELECTION_BAR_ROOM;
                let (content, bg) = ui.allocate_exact_size(
                    vec2(w, height.max(ui.available_height())),
                    Sense::click_and_drag(),
                );

                let mut y = content.top();
                for (r, (items, h)) in rows.iter().enumerate() {
                    let mut x = content.left();
                    for &i in items {
                        let iw = aspect(i) * h;
                        let img = Rect::from_min_size(pos2(x, y), vec2(iw, *h));
                        let cell =
                            Rect::from_min_max(img.min, pos2(img.right(), img.bottom() + NAME_H));
                        x += iw + GAP;
                        cells.push(Cell {
                            idx: i,
                            row: r,
                            cx: img.center().x,
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
                        self.paint_cell(ui, i, img, cell, &resp, negatives, density, now);
                        // Hover actions over the picture.
                        if (resp.hovered() || ui.rect_contains_pointer(img))
                            && let Some(a) = hover_buttons(ui, i, img, self.frames[i].sc.flag)
                        {
                            hover_action = Some((i, a));
                        }
                        if resp.double_clicked() {
                            double = Some(i);
                        } else if resp.clicked() {
                            clicked = Some((i, ui.input(|inp| inp.modifiers)));
                        }
                        let has_clip = self.clipboard.is_some();
                        let sel = self.selected.contains(&i);
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
                            item(ui, "Convert", "", CellAction::Convert, true);
                            if multi && sel {
                                item(ui, "Convert as roll", "", CellAction::ConvertRoll, true);
                            }
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
                    y += h + NAME_H + GAP;
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
            match act {
                CellAction::Rotate(cw) => self.rotate_frames(&idx, cw),
                CellAction::Flag(f) => self.set_flag(&idx, f),
                _ => {}
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
                CellAction::Convert => self.open_convert_dialog(self.sel_vec(), false),
                CellAction::ConvertRoll => self.open_convert_dialog(self.sel_vec(), true),
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

    /// One cell: picture, badges, selection ring and the label row.
    #[allow(clippy::too_many_arguments)]
    fn paint_cell(
        &self,
        ui: &egui::Ui,
        i: usize,
        img: Rect,
        cell: Rect,
        resp: &egui::Response,
        negatives: bool,
        density: Option<f64>,
        now: f64,
    ) {
        let f = &self.frames[i];
        let sel = self.selected.contains(&i);
        let p = ui.painter();
        let rejected = f.sc.flag == Flag::Reject;
        let tex = if negatives {
            f.thumb_neg.as_ref().or(f.thumb.as_ref())
        } else {
            f.thumb.as_ref()
        };
        match (tex, &f.proxy) {
            (Some(t), _) => {
                let mut image = egui::Image::new((t.id(), t.size_vec2())).corner_radius(3.0);
                if rejected {
                    image = image.tint(Color32::from_gray(80));
                }
                image.paint_at(ui, img);
            }
            (None, ProxyState::Failed(e)) => {
                resp.clone().on_hover_text(e);
                p.rect_filled(img, 3.0, theme::PANEL);
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
                p.rect_filled(img, 3.0, Color32::from_white_alpha(a));
                ui.ctx()
                    .request_repaint_after(std::time::Duration::from_millis(50));
            }
        }
        if sel {
            p.rect_stroke(
                img.expand(3.0),
                5.0,
                egui::Stroke::new(2.0, theme::ACCENT),
                egui::StrokeKind::Outside,
            );
        } else if resp.hovered() {
            p.rect_stroke(
                img.expand(2.0),
                4.0,
                egui::Stroke::new(1.0, Color32::from_white_alpha(40)),
                egui::StrokeKind::Outside,
            );
        }

        // Badges: flag and "needs a look" top-left, lab correction bottom-left.
        let mut bx = img.left() + 6.0;
        let badge = |x: f32| Rect::from_min_size(pos2(x, img.top() + 6.0), vec2(20.0, 20.0));
        match f.sc.flag {
            Flag::Pick => {
                let r = badge(bx);
                p.circle_filled(r.center(), 10.0, Color32::from_black_alpha(170));
                paint_flag(p, r.shrink(4.0), theme::OK, true);
                bx += 24.0;
            }
            Flag::Reject => {
                let r = badge(bx);
                p.circle_filled(r.center(), 10.0, Color32::from_black_alpha(170));
                widgets::paint_icon(p, r.shrink(3.0), Icon::Close, theme::ERR);
                bx += 24.0;
            }
            Flag::None => {}
        }
        let issues = self.look_issues(i, density);
        if !issues.is_empty() {
            let r = badge(bx);
            p.circle_filled(r.center(), 10.0, Color32::from_black_alpha(170));
            widgets::paint_icon(p, r.shrink(3.0), Icon::Warn, theme::WARN);
            ui.interact(r, ui.id().with(("issues", i)), Sense::hover())
                .on_hover_text(format!("Needs a look\n• {}", issues.join("\n• ")));
        }
        if f.sc.is_converted()
            && let Some(r) = lab::readout(&f.sc.settings)
        {
            let g = p.layout_no_wrap(r, egui::FontId::monospace(9.5), theme::TEXT);
            let pill = Rect::from_min_size(
                img.left_bottom() + vec2(5.0, -g.size().y - 9.0),
                g.size() + vec2(10.0, 4.0),
            );
            p.rect_filled(pill, 4.0, Color32::from_black_alpha(170));
            ui.interact(pill, ui.id().with(("lab_readout", i)), Sense::hover())
                .on_hover_text(super::LAB_KEYS_HELP);
            p.galley(pill.min + vec2(5.0, 2.0), g, theme::TEXT);
        }

        // Label row: frame number, name, status markers on the right.
        let row_y = cell.bottom() - NAME_H / 2.0 + 1.0;
        let mut x = cell.right() - 2.0;
        let converting = self.auto.in_flight.contains(&f.path) || self.auto.queue.contains(&f.path);
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
                egui::Stroke::new(1.0, theme::FAINT),
            );
            x -= 12.0;
        } else if f.is_roll() {
            let g = p.layout_no_wrap(
                "Roll".into(),
                egui::FontId::proportional(10.0),
                theme::MUTED,
            );
            let chip = Rect::from_min_max(
                pos2(x - g.size().x - 10.0, row_y - 8.0),
                pos2(x, row_y + 8.0),
            );
            p.rect_stroke(
                chip,
                8.0,
                egui::Stroke::new(1.0, Color32::from_white_alpha(40)),
                egui::StrokeKind::Inside,
            );
            p.galley(chip.center() - g.size() / 2.0, g, theme::MUTED);
            x = chip.left() - 6.0;
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
            }
            if f.exporting {
                tip.push("exporting");
            }
            ui.interact(markers, ui.id().with(("markers", i)), Sense::hover())
                .on_hover_text(tip.join(" · "));
        }
        let num = p.layout_no_wrap(
            format!("{:02}", i + 1),
            theme::medium(11.5),
            if sel { theme::TEXT } else { theme::MUTED },
        );
        let nx = cell.left() + 2.0;
        let num_w = num.size().x;
        p.galley(pos2(nx, row_y - num.size().y / 2.0), num, theme::MUTED);
        let name_x = nx + num_w + 6.0;
        let mut job = egui::text::LayoutJob::simple_singleline(
            f.name.clone(),
            egui::FontId::proportional(11.0),
            theme::FAINT,
        );
        job.wrap = egui::text::TextWrapping::truncate_at_width((x - name_x - 4.0).max(0.0));
        let g = p.layout_job(job);
        p.galley(pos2(name_x, row_y - g.size().y / 2.0), g, theme::FAINT);
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
        let issues = self.look_issues(i, self.roll_density());
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
                let hint = if issues.is_empty() {
                    "← → move · P pick · X reject · U unflag · Enter edit · Space close".to_string()
                } else {
                    format!("Needs a look: {}", issues.join(" · "))
                };
                p.text(
                    pos2(r.center().x, r.bottom() - 20.0),
                    egui::Align2::CENTER_CENTER,
                    hint,
                    egui::FontId::proportional(11.5),
                    if issues.is_empty() {
                        theme::MUTED
                    } else {
                        theme::WARN
                    },
                );
                resp
            })
            .inner;
        if resp.clicked() {
            self.grid.quick_look = false;
        }
    }

    /// The floating action bar over the bottom of the grid: the frames on
    /// show (convert, export) or, while frames are selected, the selection
    /// (cull, rotate, copy / paste, convert, export).
    pub fn selection_bar_ui(&mut self, ui: &mut egui::Ui) {
        if self.grid.quick_look || self.grid.marquee.is_some() {
            return;
        }
        let n = self.selected.len();
        if n == 0 {
            self.roll_bar_ui(ui);
            return;
        }
        let ctx = ui.ctx().clone();
        let area = ui.max_rect();
        let sel = self.sel_vec();
        let cmd = egui::Modifiers::COMMAND;
        let key = |m, k| widgets::shortcut(&ctx, m, k);
        let all = |f: Flag| sel.iter().all(|&i| self.frames[i].sc.flag == f);
        let (picked, rejected) = (all(Flag::Pick), all(Flag::Reject));
        let converted = sel.iter().all(|&i| self.frames[i].sc.is_converted());
        let has_clip = self.clipboard.is_some();
        let mut act: Option<BarAction> = None;
        bar_area(&ctx, area, |ui| {
            ui.horizontal(|ui| {
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
                    |p, r, c| {
                        paint_flag(p, r.shrink(7.0), if picked { theme::OK } else { c }, picked)
                    },
                )
                .clicked()
                {
                    act = Some(BarAction::Flag(if picked {
                        Flag::None
                    } else {
                        Flag::Pick
                    }));
                }
                if bar_icon(
                    ui,
                    if rejected { "Unflag (U)" } else { "Reject (X)" },
                    rejected,
                    |p, r, c| {
                        widgets::paint_icon(
                            p,
                            r.shrink(6.0),
                            Icon::Close,
                            if rejected { theme::ERR } else { c },
                        )
                    },
                )
                .clicked()
                {
                    act = Some(BarAction::Flag(if rejected {
                        Flag::None
                    } else {
                        Flag::Reject
                    }));
                }
                divider(ui);
                let left = format!("Rotate left ({})", key(cmd, egui::Key::OpenBracket));
                let right = format!("Rotate right ({})", key(cmd, egui::Key::CloseBracket));
                if bar_icon(ui, &left, false, |p, r, c| {
                    widgets::paint_icon(p, r.shrink(5.0), Icon::RotateLeft, c)
                })
                .clicked()
                {
                    act = Some(BarAction::Rotate(false));
                }
                if bar_icon(ui, &right, false, |p, r, c| {
                    widgets::paint_icon(p, r.shrink(5.0), Icon::RotateRight, c)
                })
                .clicked()
                {
                    act = Some(BarAction::Rotate(true));
                }
                divider(ui);
                if ui
                    .add_enabled(n == 1, bar_text("Copy settings"))
                    .on_hover_text(format!(
                        "Copy this frame's settings ({})",
                        key(cmd, egui::Key::C)
                    ))
                    .on_disabled_hover_text("Select one frame to copy from")
                    .clicked()
                {
                    act = Some(BarAction::Copy);
                }
                if ui
                    .add_enabled(has_clip, bar_text("Paste settings"))
                    .on_hover_text(format!(
                        "Paste the copied settings ({})",
                        key(cmd, egui::Key::V)
                    ))
                    .on_disabled_hover_text("Copy a frame's settings first")
                    .clicked()
                {
                    act = Some(BarAction::Paste);
                }
                divider(ui);
                if ui
                    .add(bar_text(if converted {
                        "Re-convert…"
                    } else {
                        "Convert…"
                    }))
                    .on_hover_text("Analyse each selected frame on its own")
                    .clicked()
                {
                    act = Some(BarAction::Convert(false));
                }
                if n >= 2
                    && ui
                        .add(bar_text("As roll…"))
                        .on_hover_text(
                            "Convert the selection together, for one consistent roll balance",
                        )
                        .clicked()
                {
                    act = Some(BarAction::Convert(true));
                }
                ui.add_space(4.0);
                if ui
                    .add(widgets::primary(&if n == 1 {
                        "Export".to_string()
                    } else {
                        format!("Export {n}")
                    }))
                    .on_hover_text(format!("Export the selection ({})", key(cmd, egui::Key::E)))
                    .clicked()
                {
                    act = Some(BarAction::Export);
                }
                ui.add_space(2.0);
                if ui
                    .add(egui::Button::new(RichText::new("Clear").color(theme::MUTED)).frame(false))
                    .on_hover_text("Clear the selection (Esc)")
                    .clicked()
                {
                    act = Some(BarAction::Clear);
                }
            });
        });
        match act {
            Some(BarAction::Flag(f)) => self.set_flag(&sel, f),
            Some(BarAction::Rotate(cw)) => self.rotate_frames(&sel, cw),
            Some(BarAction::Copy) => self.copy_settings(),
            Some(BarAction::Paste) => self.paste_settings(),
            Some(BarAction::Convert(roll)) => self.open_convert_dialog(sel, roll),
            Some(BarAction::Export) => self.open_export_dialog(sel),
            Some(BarAction::Clear) => self.selected.clear(),
            None => {}
        }
    }

    /// The action bar with nothing selected: convert or export the frames on show.
    fn roll_bar_ui(&mut self, ui: &mut egui::Ui) {
        let shown = self.shown_frames();
        if shown.is_empty() {
            return;
        }
        let ctx = ui.ctx().clone();
        let area = ui.max_rect();
        let t = shown.len();
        let (scope, export) = match self.grid.filter {
            GridFilter::All => (
                format!("{t} frame{}", if t == 1 { "" } else { "s" }),
                "Export all".to_string(),
            ),
            GridFilter::Picks => (
                format!("{t} pick{}", if t == 1 { "" } else { "s" }),
                "Export picks".to_string(),
            ),
            GridFilter::NeedsLook => (format!("{t} to look at"), format!("Export {t}")),
        };
        let converted = shown.iter().all(|&i| self.frames[i].sc.is_converted());
        let export_key = widgets::shortcut(&ctx, egui::Modifiers::COMMAND, egui::Key::E);
        let mut act: Option<BarAction> = None;
        bar_area(&ctx, area, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                ui.label(
                    RichText::new(scope)
                        .font(theme::medium(12.5))
                        .color(theme::TEXT),
                );
                divider(ui);
                if ui
                    .add(bar_text(if converted {
                        "Re-convert…"
                    } else {
                        "Convert…"
                    }))
                    .on_hover_text("Analyse each frame on its own, with new conversion options")
                    .clicked()
                {
                    act = Some(BarAction::Convert(false));
                }
                if t >= 2
                    && ui
                        .add(bar_text("As roll…"))
                        .on_hover_text(
                            "Analyse the frames together for one consistent roll balance",
                        )
                        .clicked()
                {
                    act = Some(BarAction::Convert(true));
                }
                ui.add_space(4.0);
                if ui
                    .add(widgets::primary(&export))
                    .on_hover_text(format!("Export positive copies ({export_key})"))
                    .clicked()
                {
                    act = Some(BarAction::Export);
                }
            });
        });
        match act {
            Some(BarAction::Convert(roll)) => self.open_convert_dialog(shown, roll),
            Some(BarAction::Export) => self.open_export_dialog(shown),
            _ => {}
        }
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

/// Share of pixels clipped to white and to black in a rendered thumbnail.
pub fn clip_share(px: &[[f32; 3]]) -> [f32; 2] {
    if px.is_empty() {
        return [0.0, 0.0];
    }
    let (mut w, mut b) = (0usize, 0usize);
    for p in px {
        let lo = p[0].min(p[1]).min(p[2]);
        let hi = p[0].max(p[1]).max(p[2]);
        if lo >= 0.995 {
            w += 1;
        }
        if hi <= 0.005 {
            b += 1;
        }
    }
    let n = px.len() as f32;
    [w as f32 / n, b as f32 / n]
}

/// Pick / rotate buttons over a hovered picture; the one clicked.
fn hover_buttons(ui: &egui::Ui, i: usize, img: Rect, flag: Flag) -> Option<CellAction> {
    if img.width() < 90.0 {
        return None;
    }
    let mut out = None;
    let size = 22.0;
    let mut x = img.right() - 6.0 - size;
    let picked = flag == Flag::Pick;
    let buttons = [
        (CellAction::Rotate(true), "Rotate right"),
        (CellAction::Rotate(false), "Rotate left"),
        (
            CellAction::Flag(if picked { Flag::None } else { Flag::Pick }),
            if picked { "Unflag (U)" } else { "Pick (P)" },
        ),
    ];
    for (k, (act, tip)) in buttons.into_iter().enumerate() {
        let r = Rect::from_min_size(pos2(x, img.top() + 6.0), vec2(size, size));
        x -= size + 4.0;
        let resp = ui
            .interact(r, ui.id().with(("hover_btn", i, k)), Sense::click())
            .on_hover_text(tip);
        let p = ui.painter();
        p.circle_filled(
            r.center(),
            size / 2.0,
            Color32::from_black_alpha(if resp.hovered() { 220 } else { 160 }),
        );
        let col = if resp.hovered() {
            theme::TEXT
        } else {
            theme::MUTED
        };
        match act {
            CellAction::Rotate(true) => {
                widgets::paint_icon(p, r.shrink(4.0), Icon::RotateRight, col)
            }
            CellAction::Rotate(false) => {
                widgets::paint_icon(p, r.shrink(4.0), Icon::RotateLeft, col)
            }
            _ => paint_flag(
                p,
                r.shrink(5.0),
                if picked { theme::OK } else { col },
                picked,
            ),
        }
        if resp.clicked() {
            out = Some(act);
        }
    }
    out
}

/// A small pennant flag.
fn paint_flag(p: &egui::Painter, r: Rect, color: Color32, filled: bool) {
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

#[derive(Clone, Copy)]
enum BarAction {
    Flag(Flag),
    Rotate(bool),
    Copy,
    Paste,
    Convert(bool),
    Export,
    Clear,
}

/// Quiet text button for the selection bar.
fn bar_text(text: &str) -> egui::Button<'_> {
    egui::Button::new(RichText::new(text).color(theme::TEXT))
        .fill(Color32::TRANSPARENT)
        .corner_radius(6.0)
        .min_size(vec2(0.0, 28.0))
}

/// Square icon button for the selection bar; `on` shows it pressed.
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

/// Thin vertical rule between groups of the selection bar.
fn divider(ui: &mut egui::Ui) {
    let (rect, _) = ui.allocate_exact_size(vec2(9.0, 18.0), Sense::hover());
    ui.painter().vline(
        rect.center().x,
        rect.y_range(),
        egui::Stroke::new(1.0, Color32::from_white_alpha(18)),
    );
}

/// The floating frame of the action bar, centred at the bottom of `area`.
fn bar_area(ctx: &egui::Context, area: Rect, add: impl FnOnce(&mut egui::Ui)) {
    egui::Area::new(egui::Id::new("action_bar"))
        .pivot(egui::Align2::CENTER_BOTTOM)
        .fixed_pos(pos2(area.center().x, area.bottom() - 4.0))
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(theme::SURFACE)
                .stroke(egui::Stroke::new(1.0, Color32::from_white_alpha(14)))
                .corner_radius(12.0)
                .inner_margin(egui::Margin::symmetric(10, 6))
                .shadow(ui.visuals().popup_shadow)
                .show(ui, add);
        });
}
