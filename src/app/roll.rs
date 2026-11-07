//! Roll info (`NLP/roll.json`): the dialog shown when a folder is opened for
//! the first time, and the Projects screen listing every opened roll.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use eframe::egui::{self, RichText, Sense, vec2};

use super::jobs::{self, JobResult};
use super::widgets::{self, Icon, theme};
use super::{NlpApp, picker};
use crate::engine::pipeline::{self, Renderer};
use crate::engine::settings::{FilmMetadata, OutputSpace, Source};
use crate::engine::sidecar::{self, Flag, RollInfo, Sidecar};

#[derive(Default)]
pub struct RollState {
    /// Roll info of the open folder.
    pub info: Option<RollInfo>,
    pub dialog: Option<RollDialog>,
    /// "New roll" banner over the grid on a folder's first open: the draft
    /// name and film stock.
    pub banner: Option<RollDialog>,
    /// The welcome screen shows the Projects list instead.
    pub projects_open: bool,
    /// What the Projects screen shows per roll, loaded as rows come into view.
    pub covers: HashMap<PathBuf, RollCover>,
}

impl RollState {
    pub fn set_summary(
        &mut self,
        dir: &Path,
        exists: bool,
        info: Option<RollInfo>,
        count: usize,
        edited: Option<u64>,
    ) {
        if let Some(c) = self.covers.get_mut(dir) {
            c.exists = exists;
            c.info = info;
            c.count = count;
            c.edited = edited;
            c.loaded = true;
        }
    }

    pub fn set_frames(&mut self, ctx: &egui::Context, dir: &Path, frames: Vec<egui::ColorImage>) {
        // Gone when a roll opened or the row left the list in the meantime.
        let Some(c) = self.covers.get_mut(dir) else {
            return;
        };
        let opts = egui::TextureOptions::LINEAR.with_mipmap_mode(Some(egui::TextureFilter::Linear));
        c.frames = frames
            .into_iter()
            .enumerate()
            .map(|(i, img)| ctx.load_texture(format!("cover-{}-{i}", dir.display()), img, opts))
            .collect();
    }
}

/// One row of the Projects screen.
pub struct RollCover {
    /// Whether the summary has arrived; until then the row assumes the folder is there.
    loaded: bool,
    exists: bool,
    info: Option<RollInfo>,
    count: usize,
    edited: Option<u64>,
    frames: Vec<egui::TextureHandle>,
}

impl Default for RollCover {
    fn default() -> Self {
        Self {
            loaded: false,
            exists: true,
            info: None,
            count: 0,
            edited: None,
            frames: Vec::new(),
        }
    }
}

/// Frames on a roll's card: one large, two small.
const COVER_FRAMES: usize = 3;
/// Widest the Rolls grid gets.
const LIST_MAX_W: f32 = 1600.0;
/// Narrowest a card gets before the grid drops a column.
const CARD_MIN_W: f32 = 300.0;
const CARD_GAP: f32 = 14.0;
const CARD_PAD: f32 = 10.0;
const MOSAIC_GAP: f32 = 3.0;
const TEXT_GAP: f32 = 10.0;
const NAME_H: f32 = 20.0;
const META_H: f32 = 18.0;

pub struct RollDialog {
    pub dir: PathBuf,
    pub info: RollInfo,
    /// First open of the folder (title "New roll", Skip instead of Cancel).
    pub new: bool,
    /// The roll's scan source, when the dialog is for the open roll.
    pub source: Option<Source>,
}

/// The film / capture fields in two groups, short fields side by side.
/// Returns true on change.
pub fn film_fields(ui: &mut egui::Ui, id: &str, m: &mut FilmMetadata, notes: bool) -> bool {
    let mut changed = false;
    ui.push_id(id, |ui| {
        ui.scope(|ui| {
            ui.spacing_mut().item_spacing.y = 6.0;
            ui.label(RichText::new("Film stock").small().color(theme::MUTED));
            let w = ui.available_width();
            changed |= widgets::stock_combo(ui, &mut m.film_stock, "Portra 400", w).changed();
            changed |= pair(
                ui,
                ("ISO", &mut m.film_iso, "400"),
                ("Shot at ISO", &mut m.shot_at_iso, "800 push"),
            );
            changed |= field(ui, "Developer", &mut m.developer, "C-41");

            group_heading(ui, "Camera");
            changed |= pair(
                ui,
                ("Make", &mut m.camera_make, "Nikon"),
                ("Model", &mut m.camera_model, "FM2"),
            );
            changed |= field(ui, "Lens", &mut m.lens, "50mm f/1.8");
            changed |= pair(
                ui,
                ("Shutter", &mut m.shutter_speed, "1/125"),
                ("Aperture", &mut m.aperture, "f/8"),
            );
            if notes {
                changed |= field(ui, "Notes", &mut m.development_notes, "");
            }
        });
    });
    changed
}

/// A muted sub-heading between the groups of film fields.
fn group_heading(ui: &mut egui::Ui, title: &str) {
    ui.add_space(4.0);
    ui.label(
        RichText::new(title)
            .font(theme::medium(11.5))
            .color(theme::MUTED),
    );
}

/// A label above a single-line field that fills its column.
fn field(ui: &mut egui::Ui, label: &str, v: &mut String, hint: &str) -> bool {
    ui.label(RichText::new(label).small().color(theme::MUTED));
    ui.add(
        egui::TextEdit::singleline(v)
            .hint_text(hint)
            // desired_width excludes the field's margin.
            .desired_width(ui.available_width() - 8.0),
    )
    .changed()
}

/// Two short fields side by side, each `(label, value, hint)`.
fn pair(ui: &mut egui::Ui, a: (&str, &mut String, &str), b: (&str, &mut String, &str)) -> bool {
    let (mut changed_a, mut changed_b) = (false, false);
    ui.columns(2, |cols| {
        changed_a = field(&mut cols[0], a.0, a.1, a.2);
        changed_b = field(&mut cols[1], b.0, b.1, b.2);
    });
    changed_a || changed_b
}

fn folder_name(dir: &Path) -> String {
    dir.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| dir.display().to_string())
}

impl NlpApp {
    /// Open the roll info dialog for `dir`, pre-filled from its `roll.json`.
    pub fn open_roll_dialog(&mut self, dir: &Path, new: bool) {
        let info = self.roll.info.clone().unwrap_or_else(|| RollInfo {
            name: folder_name(dir),
            ..Default::default()
        });
        let source = self
            .main_source()
            .filter(|_| self.folder.as_deref() == Some(dir));
        self.roll.dialog = Some(RollDialog {
            dir: dir.to_path_buf(),
            info,
            new,
            source,
        });
    }

    /// Show the "New roll" banner for a folder opened for the first time.
    pub fn offer_new_roll(&mut self, dir: &Path) {
        self.roll.banner = Some(RollDialog {
            dir: dir.to_path_buf(),
            info: RollInfo {
                name: folder_name(dir),
                ..Default::default()
            },
            new: true,
            source: None,
        });
    }

    /// The "New roll" banner: the film stock inline, the rest in the
    /// roll dialog. Dismissing it keeps the folder name as the roll name.
    pub fn roll_banner_ui(&mut self, ui: &mut egui::Ui) {
        let Some(mut d) = self.roll.banner.take() else {
            return;
        };
        let (mut save, mut more, mut dismiss) = (false, false, false);
        egui::Panel::top("roll_banner")
            .exact_size(38.0)
            .frame(
                egui::Frame::new()
                    .fill(theme::BG)
                    .inner_margin(egui::Margin::symmetric(12, 0)),
            )
            .show(ui, |ui| {
                ui.horizontal_centered(|ui| {
                    // The roll's name is already in the top bar (and in More
                    // details); the banner only asks what film it is.
                    ui.label(RichText::new("New roll").font(theme::medium(13.0)));
                    ui.label(RichText::new("· What film is it?").color(theme::MUTED));
                    ui.add_space(4.0);
                    let enter = |r: &egui::Response| {
                        r.lost_focus() && r.ctx.input(|i| i.key_pressed(egui::Key::Enter))
                    };
                    let r = widgets::stock_combo(
                        ui,
                        &mut d.info.metadata.film_stock,
                        "e.g. Portra 400",
                        220.0,
                    );
                    save |= enter(&r);
                    save |= ui.add(widgets::primary("Save")).clicked();
                    more = ui
                        .add(
                            egui::Button::new(
                                RichText::new("More details…").small().color(theme::MUTED),
                            )
                            .frame(false),
                        )
                        .clicked();
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        dismiss =
                            widgets::icon_button(ui, widgets::Icon::Close, "Dismiss", false, true)
                                .clicked();
                    });
                });
            });
        if save || dismiss {
            let RollDialog { dir, info, .. } = d;
            self.save_roll_info(&dir, info);
        } else if more {
            self.roll.dialog = Some(d);
        } else {
            self.roll.banner = Some(d);
        }
    }

    /// The open roll's custom export folder.
    pub fn roll_export_dir(&self) -> Option<PathBuf> {
        self.roll.info.as_ref().and_then(|r| r.export_dir.clone())
    }

    /// Remember a custom export folder (`None`: next to the scans), with
    /// the roll when a folder is open, else as the app default.
    pub fn set_export_dir(&mut self, dir: Option<PathBuf>) {
        match self.folder.clone().filter(|_| !self.single_mode) {
            Some(f) => {
                let mut info = self.roll.info.clone().unwrap_or_else(|| RollInfo {
                    name: folder_name(&f),
                    ..Default::default()
                });
                info.export_dir = dir;
                self.save_roll_info(&f, info);
                self.defaults.export.output_dir = None;
            }
            None => self.defaults.export.output_dir = dir,
        }
        let _ = sidecar::save_defaults(&self.defaults);
    }

    /// Write the roll info and give it to frames without film details of their own.
    pub fn save_roll_info(&mut self, dir: &Path, mut info: RollInfo) {
        if info.created == 0 {
            info.created = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |d| d.as_secs());
        }
        if let Err(e) = sidecar::save_roll_info(dir, &info) {
            self.toast(format!("Could not save roll info: {e}"), true);
            return;
        }
        let old = self
            .roll
            .info
            .as_ref()
            .map(|r| r.metadata.clone())
            .unwrap_or_default();
        // A stock from the list also selects that film's crosstalk matrix.
        let crosstalk = (info.metadata.film_stock != old.film_stock)
            .then(|| crate::engine::presets::crosstalk_id_for_stock(&info.metadata.film_stock))
            .flatten();
        for f in &mut self.frames {
            if f.sc.metadata == FilmMetadata::default() || f.sc.metadata == old {
                f.sc.metadata = info.metadata.clone();
                f.dirty_since = Some(self.time);
                if let Some(id) = crosstalk {
                    f.sc.settings.crosstalk = id.to_string();
                    f.thumb_dirty = true;
                }
            }
        }
        if crosstalk.is_some() {
            // The frame open in the editor re-prepares with the new matrix.
            self.editor.invalidate();
        }
        self.roll.covers.remove(dir);
        self.roll.info = Some(info);
    }

    pub fn roll_dialog_ui(&mut self, ctx: &egui::Context) {
        let Some(mut d) = self.roll.dialog.take() else {
            return;
        };
        let mut save = false;
        let mut close = false;
        let frame = egui::Frame::new()
            .fill(theme::PANEL)
            .stroke(egui::Stroke::new(1.0, theme::LINE))
            .corner_radius(12.0)
            .inner_margin(egui::Margin::same(22))
            .shadow(ctx.global_style().visuals.window_shadow);
        let r = egui::Modal::new(egui::Id::new("roll_modal"))
            .backdrop_color(egui::Color32::from_black_alpha(150))
            .frame(frame)
            .show(ctx, |ui| {
                ui.set_width(420.0);
                ui.label(
                    RichText::new(if d.new { "New roll" } else { "Roll info" })
                        .font(theme::medium(16.0))
                        .color(theme::TEXT),
                );
                ui.label(
                    RichText::new("Applies to every frame in this folder")
                        .small()
                        .color(theme::MUTED),
                );
                ui.add_space(14.0);
                egui::Grid::new("roll_head")
                    .num_columns(2)
                    .spacing(vec2(8.0, 4.0))
                    .min_col_width(78.0)
                    .show(ui, |ui| {
                        ui.label(RichText::new("Name").small().color(theme::MUTED));
                        ui.add(
                            egui::TextEdit::singleline(&mut d.info.name)
                                .desired_width(ui.available_width() - 8.0),
                        );
                        ui.end_row();
                        ui.label(RichText::new("Date").small().color(theme::MUTED));
                        ui.add(
                            egui::TextEdit::singleline(&mut d.info.date)
                                .hint_text("e.g. 2026-10-01")
                                .desired_width(ui.available_width() - 8.0),
                        );
                        ui.end_row();
                    });
                ui.add_space(4.0);
                film_fields(ui, "roll_film", &mut d.info.metadata, false);
                if let Some(source) = d.source.as_mut() {
                    ui.add_space(4.0);
                    widgets::fl_combo(
                        ui,
                        "Scan source",
                        source,
                        Source::ALL,
                        Source::label,
                        "Detected from the files. Changing it converts the whole roll again.",
                    );
                }
                ui.add_space(4.0);
                ui.label(RichText::new("Notes").small().color(theme::MUTED));
                ui.add(
                    egui::TextEdit::multiline(&mut d.info.notes)
                        .desired_rows(3)
                        .desired_width(f32::INFINITY),
                );
                ui.add_space(16.0);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(widgets::primary("Save")).clicked() {
                        save = true;
                    }
                    if ui
                        .add(widgets::ghost(if d.new { "Skip" } else { "Cancel" }))
                        .clicked()
                    {
                        close = true;
                    }
                });
            });
        if r.should_close() {
            close = true;
        }
        if save {
            let RollDialog {
                dir, info, source, ..
            } = d;
            self.save_roll_info(&dir, info);
            if let Some(source) = source.filter(|&s| Some(s) != self.main_source()) {
                self.set_roll_source(source);
            }
        } else if close {
            // Skipping still writes the roll so the dialog doesn't come back.
            if d.new {
                let info = RollInfo {
                    name: folder_name(&d.dir),
                    ..Default::default()
                };
                self.save_roll_info(&d.dir, info);
            }
        } else {
            self.roll.dialog = Some(d);
        }
    }

    pub fn show_projects(&mut self) {
        self.roll.covers.clear();
        self.roll.projects_open = true;
    }

    /// Choose or create a folder and open it as a roll: its empty grid is
    /// where the Scan button lives.
    pub fn choose_new_roll(&mut self) {
        let start = self
            .defaults
            .last_folder
            .as_deref()
            .and_then(Path::parent)
            .map(Path::to_path_buf);
        if let Some(dir) = picker::pick_folder(start.as_deref(), Some(picker::NEW_ROLL_PROMPT)) {
            self.open_folder(&dir);
        }
    }

    /// Load a row's details and strip off the UI thread: the folder may be on
    /// a slow or unplugged drive.
    fn request_cover(&mut self, dir: &Path) {
        self.roll
            .covers
            .insert(dir.to_path_buf(), RollCover::default());
        let dir = dir.to_path_buf();
        let tx = self.jobs.sender();
        let ctx = self.jobs.ctx().clone();
        self.jobs.spawn(move || {
            let exists = dir.is_dir();
            let scans = if exists {
                jobs::folder_scans(&dir)
            } else {
                Vec::new()
            };
            let sidecars: Vec<_> = scans.iter().map(|p| sidecar::load(p)).collect();
            let edited = scans
                .iter()
                .filter_map(|p| {
                    std::fs::metadata(sidecar::sidecar_path(p))
                        .ok()?
                        .modified()
                        .ok()
                })
                .max()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs());
            let _ = tx.send(JobResult::RollSummary {
                dir: dir.clone(),
                exists,
                info: sidecar::load_roll_info(&dir),
                count: scans.len(),
                edited,
            });
            ctx.request_repaint();
            let candidates: Vec<(Flag, bool)> = scans
                .iter()
                .zip(&sidecars)
                .map(|(p, sc)| match sc {
                    Some(sc) => (
                        sc.flag,
                        sc.is_converted() && sidecar::thumb_path(p).is_some_and(|t| t.exists()),
                    ),
                    None => (Flag::None, false),
                })
                .collect();
            let mut picked = pick_cover(&candidates, COVER_FRAMES);
            pick_leads(&mut picked, &candidates);
            let frames = picked
                .into_iter()
                .enumerate()
                .filter_map(|(k, i)| render_cover_frame(&scans[i], sidecars[i].as_ref()?, k == 0))
                .collect();
            JobResult::RollCover { dir, frames }
        });
    }

    /// Every roll opened before, most recent first.
    pub fn projects_ui(&mut self, ui: &mut egui::Ui) {
        let mut action: Option<(PathBuf, RowAction)> = None;
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(theme::BG)
                    .inner_margin(egui::Margin::symmetric(24, 20)),
            )
            .show(ui, |ui| {
                let w = ui.available_width().min(LIST_MAX_W);
                ui.vertical_centered(|ui| {
                    ui.set_max_width(w);
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("Rolls")
                                .font(theme::medium(20.0))
                                .color(theme::TEXT),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui
                                .add(widgets::primary("Open…"))
                                .on_hover_text("Open a folder or scans (or drop them here)")
                                .clicked()
                            {
                                self.choose_anything();
                            }
                            if ui
                                .add(widgets::ghost("New roll…"))
                                .on_hover_text("Choose or create a folder, then scan film into it")
                                .clicked()
                            {
                                self.choose_new_roll();
                            }
                        });
                    });
                    ui.add_space(14.0);
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        let (cols, card) = grid_layout(ui.available_width());
                        let dirs = self.defaults.projects.clone();
                        for row in dirs.chunks(cols) {
                            let (_, rect) = ui.allocate_space(vec2(ui.available_width(), card.y));
                            for (i, dir) in row.iter().enumerate() {
                                let x = i as f32 * (card.x + CARD_GAP);
                                let r = egui::Rect::from_min_size(rect.min + vec2(x, 0.0), card);
                                let mut child = ui.new_child(egui::UiBuilder::new().max_rect(r));
                                let (a, visible) =
                                    roll_card(&mut child, r, dir, self.roll.covers.get(dir));
                                if visible && !self.roll.covers.contains_key(dir) {
                                    self.request_cover(dir);
                                }
                                if let Some(a) = a {
                                    action = Some((dir.clone(), a));
                                }
                            }
                            ui.add_space(CARD_GAP);
                        }
                        // Open's tooltip and the drop overlay teach this once there are rolls.
                        if dirs.is_empty() {
                            ui.add_space(6.0);
                            ui.label(
                                RichText::new("Drop a roll folder or scans anywhere to open them")
                                    .small()
                                    .color(theme::FAINT),
                            );
                        }
                    });
                });
            });
        widgets::drop_overlay(ui.ctx());
        let Some((dir, action)) = action else {
            return;
        };
        match action {
            RowAction::Open => {
                self.roll.projects_open = false;
                self.open_folder(&dir);
            }
            // Opened, so that the stock also reaches the frames (`save_roll_info`).
            RowAction::Details => {
                self.roll.projects_open = false;
                self.open_folder(&dir);
                self.roll.banner = None;
                self.open_roll_dialog(&dir, false);
            }
            RowAction::Remove => {
                let name = self
                    .roll
                    .covers
                    .get(&dir)
                    .and_then(|c| c.info.as_ref())
                    .map_or_else(|| folder_name(&dir), |i| roll_name(&dir, Some(i)));
                sidecar::forget_project(&mut self.defaults, &dir);
                self.roll.covers.remove(&dir);
                let _ = sidecar::save_defaults(&self.defaults);
                self.toast(format!("Removed {name} from the list"), false);
            }
            RowAction::Locate => {
                let start = dir.ancestors().skip(1).find(|p| p.is_dir());
                if let Some(new) = picker::pick_folder(start, Some(picker::LOCATE_PROMPT)) {
                    sidecar::relocate_project(&mut self.defaults, &dir, &new);
                    self.roll.covers.remove(&dir);
                    self.roll.covers.remove(&new);
                    let _ = sidecar::save_defaults(&self.defaults);
                }
            }
        }
    }
}

/// Which frames stand for a roll: `(flag, usable)` per frame, in roll order.
/// Picks come first, and the rest of the strip is filled evenly from the
/// unflagged frames. Rejected and unusable frames never show.
fn pick_cover(frames: &[(Flag, bool)], n: usize) -> Vec<usize> {
    let of = |flag: Flag| -> Vec<usize> {
        frames
            .iter()
            .enumerate()
            .filter(|(_, (f, usable))| *usable && *f == flag)
            .map(|(i, _)| i)
            .collect()
    };
    let spaced = |v: &[usize], n: usize| -> Vec<usize> {
        if v.len() <= n {
            return v.to_vec();
        }
        (0..n).map(|k| v[k * v.len() / n]).collect()
    };
    let mut out = spaced(&of(Flag::Pick), n);
    out.extend(spaced(&of(Flag::None), n - out.len()));
    out.sort_unstable();
    out
}

/// Put the roll's first pick in the card's large spot.
fn pick_leads(picked: &mut [usize], frames: &[(Flag, bool)]) {
    if let Some(p) = picked.iter().position(|&i| frames[i].0 == Flag::Pick) {
        picked[..=p].rotate_right(1);
    }
}

/// A frame as the grid renders it, from its cached thumbnail source, or for
/// the `large` spot from its cached proxy when there is one. A card is never
/// worth decoding a RAW for, so a frame with neither is left out.
fn render_cover_frame(image: &Path, sc: &Sidecar, large: bool) -> Option<egui::ColorImage> {
    let src = large
        .then(|| jobs::read_cached_proxy(image))
        .flatten()
        .or_else(|| jobs::read_thumb(image))?;
    let analysis = sc.analysis.as_ref()?;
    let neg = pipeline::prepare_negative(&src, &sc.settings, analysis.base);
    let levels = pipeline::levels_for(&sc.settings, Some(analysis), sc.roll.as_ref());
    let r = Renderer::new(&sc.settings, levels.as_ref(), OutputSpace::Srgb);
    Some(super::to_color_image(
        &r.render(&neg, 0.0),
        neg.width,
        neg.height,
    ))
}

/// How many columns fit in `width`, and the size of a card that fills its column.
fn grid_layout(width: f32) -> (usize, egui::Vec2) {
    let cols = (((width + CARD_GAP) / (CARD_MIN_W + CARD_GAP)).floor() as usize).max(1);
    let w = (width - CARD_GAP * (cols - 1) as f32) / cols as f32;
    let inner = w - CARD_PAD * 2.0;
    // Three frames make squares: the large one the mosaic's height, two small beside it.
    let picture = (inner - MOSAIC_GAP / 2.0) / 1.5;
    let h = CARD_PAD * 2.0 + picture + TEXT_GAP + NAME_H + META_H;
    (cols, vec2(w, h))
}

/// Where `n` frames sit in a card's picture.
fn mosaic(rect: egui::Rect, n: usize) -> Vec<egui::Rect> {
    let rect_at = |x: f32, y: f32, w: f32, h: f32| {
        egui::Rect::from_min_size(rect.min + vec2(x, y), vec2(w, h))
    };
    let (w, h) = (rect.width(), rect.height());
    match n {
        0 => Vec::new(),
        1 => vec![rect],
        2 => {
            let half = (w - MOSAIC_GAP) / 2.0;
            vec![
                rect_at(0.0, 0.0, half, h),
                rect_at(half + MOSAIC_GAP, 0.0, half, h),
            ]
        }
        _ => {
            let small = (h - MOSAIC_GAP) / 2.0;
            let x = w - small;
            vec![
                rect_at(0.0, 0.0, x - MOSAIC_GAP, h),
                rect_at(x, 0.0, small, small),
                rect_at(x, small + MOSAIC_GAP, small, small),
            ]
        }
    }
}

/// The centred part of a `[w, h]` image with the shape of `target`.
fn cover_uv([w, h]: [usize; 2], target: egui::Vec2) -> egui::Rect {
    let image = w.max(1) as f32 / h.max(1) as f32;
    let shape = target.x / target.y.max(1.0);
    let size = if image > shape {
        vec2(shape / image, 1.0)
    } else {
        vec2(1.0, image / shape)
    };
    egui::Rect::from_center_size(egui::pos2(0.5, 0.5), size)
}

fn roll_name(dir: &Path, info: Option<&RollInfo>) -> String {
    info.map(|i| i.name.clone())
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| folder_name(dir))
}

/// "Today", "3 days ago", … for a time `secs` ago.
fn time_ago(secs: u64) -> String {
    const DAY: u64 = 86_400;
    let plural = |n: u64, unit: &str| format!("{n} {unit}{} ago", if n == 1 { "" } else { "s" });
    match secs / DAY {
        0 => "Today".into(),
        1 => "Yesterday".into(),
        d @ 2..=13 => plural(d, "day"),
        d @ 14..=59 => plural(d / 7, "week"),
        d @ 60..=729 => plural(d / 30, "month"),
        d => plural(d / 365, "year"),
    }
}

/// The metadata line of a row.
fn row_meta(c: &RollCover, now: u64) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(m) = c.info.as_ref().map(|i| &i.metadata) {
        if !m.film_stock.is_empty() {
            parts.push(m.film_stock.clone());
        }
        let cam = format!("{} {}", m.camera_make, m.camera_model);
        if !cam.trim().is_empty() {
            parts.push(cam.trim().to_string());
        }
    }
    if let Some(date) = c.info.as_ref().map(|i| &i.date).filter(|d| !d.is_empty()) {
        parts.push(date.clone());
    }
    parts.push(format!(
        "{} frame{}",
        c.count,
        if c.count == 1 { "" } else { "s" }
    ));
    if let Some(t) = c.edited {
        parts.push(time_ago(now.saturating_sub(t)));
    }
    parts.join(" · ")
}

enum RowAction {
    Open,
    Details,
    Remove,
    Locate,
}

fn truncated(
    ui: &egui::Ui,
    text: String,
    font: egui::FontId,
    color: egui::Color32,
    max_w: f32,
) -> std::sync::Arc<egui::Galley> {
    let mut job = egui::text::LayoutJob::simple_singleline(text, font, color);
    job.wrap = egui::text::TextWrapping::truncate_at_width(max_w.max(1.0));
    ui.fonts_mut(|f| f.layout_job(job))
}

/// A card's picture: the roll's frames, or what stands in for them.
fn paint_picture(ui: &egui::Ui, rect: egui::Rect, cover: Option<&RollCover>) {
    let frames: &[egui::TextureHandle] = cover.map_or(&[], |c| &c.frames);
    if frames.is_empty() {
        let count = cover.filter(|c| c.loaded).map_or(COVER_FRAMES, |c| c.count);
        for r in mosaic(rect, count.clamp(1, COVER_FRAMES)) {
            ui.painter().rect_filled(r, 2.0, theme::BG);
        }
        return;
    }
    for (t, r) in frames.iter().zip(mosaic(rect, frames.len())) {
        ui.painter().image(
            t.id(),
            r,
            cover_uv(t.size(), r.size()),
            egui::Color32::WHITE,
        );
    }
}

/// One roll: the whole card opens it. Returns the action and whether the card
/// is on screen. It lays itself out in `rect` and allocates nothing, so the
/// hover-only button can't move its neighbours.
fn roll_card(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    dir: &Path,
    cover: Option<&RollCover>,
) -> (Option<RowAction>, bool) {
    if !ui.is_rect_visible(rect) {
        return (None, false);
    }
    let resp = ui.interact(rect, ui.id().with(("roll_card", dir)), Sense::click());
    let mut action = None;
    let exists = cover.is_none_or(|c| c.exists);
    let info = cover.and_then(|c| c.info.as_ref());
    let path = dir.display().to_string();
    let hovered = ui.rect_contains_pointer(rect);
    ui.painter().rect(
        rect,
        10.0,
        if hovered && exists {
            theme::SURFACE
        } else {
            theme::PANEL
        },
        egui::Stroke::new(1.0, theme::LINE),
        egui::StrokeKind::Inside,
    );
    let inner = rect.shrink(CARD_PAD);
    let picture = egui::Rect::from_min_max(
        inner.min,
        egui::pos2(inner.right(), inner.bottom() - TEXT_GAP - NAME_H - META_H),
    );
    let small = egui::FontId::proportional(12.0);
    paint_picture(ui, picture, cover);

    // In the picture's place: why there are no frames.
    let empty = cover.is_some_and(|c| c.loaded && c.exists && c.count == 0);
    if !exists || empty {
        let (text, color) = if exists {
            ("No frames yet", theme::MUTED)
        } else {
            ("Folder not found", theme::WARN)
        };
        let centre = picture.center() - vec2(0.0, if exists { 0.0 } else { 16.0 });
        ui.painter().text(
            centre,
            egui::Align2::CENTER_CENTER,
            text,
            small.clone(),
            color,
        );
        if !exists {
            let r =
                egui::Rect::from_center_size(picture.center() + vec2(0.0, 14.0), vec2(90.0, 30.0));
            let mut child = ui.new_child(egui::UiBuilder::new().max_rect(r).layout(
                egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
            ));
            if child
                .add(widgets::ghost("Locate…"))
                .on_hover_text("Find the folder this roll moved to")
                .clicked()
            {
                action = Some(RowAction::Locate);
            }
        }
    }

    // Under the picture: the name and details, with the remove icon on hover.
    let name_y = picture.bottom() + TEXT_GAP;
    let meta_y = name_y + NAME_H;
    let close = egui::Rect::from_center_size(
        egui::pos2(inner.right() - 13.0, (name_y + inner.bottom()) / 2.0),
        vec2(26.0, 26.0),
    );
    let text_w = close.left() - 8.0 - inner.left();
    let title = truncated(
        ui,
        roll_name(dir, info),
        theme::medium(14.0),
        if exists { theme::TEXT } else { theme::FAINT },
        text_w,
    );
    ui.painter()
        .galley(egui::pos2(inner.left(), name_y), title, theme::TEXT);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    if !exists {
        let g = truncated(ui, widgets::short_path(&path), small, theme::FAINT, text_w);
        ui.painter()
            .galley(egui::pos2(inner.left(), meta_y), g, theme::FAINT);
    } else if let Some(c) = cover.filter(|c| c.loaded) {
        let no_stock = info.is_none_or(|i| i.metadata.film_stock.is_empty());
        let link = no_stock.then(|| {
            ui.painter()
                .layout_no_wrap("Add film details".into(), small.clone(), theme::ACCENT)
        });
        let link_w = link.as_ref().map_or(0.0, |l| l.size().x + 12.0);
        let g = truncated(ui, row_meta(c, now), small, theme::MUTED, text_w - link_w);
        let end = inner.left() + g.size().x;
        ui.painter()
            .galley(egui::pos2(inner.left(), meta_y), g, theme::MUTED);
        // Only while the card is hovered, like the grid's flag buttons: it
        // shouldn't compete with the pictures on every card.
        if let Some(l) = link.filter(|_| hovered) {
            let r = egui::Rect::from_min_size(egui::pos2(end + 12.0, meta_y), l.size());
            let id = ui.id().with(("film_details", dir));
            let lr = ui.interact(r, id, Sense::click());
            ui.painter().galley(r.min, l, theme::ACCENT);
            if lr.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                ui.painter().hline(
                    r.x_range(),
                    r.bottom(),
                    egui::Stroke::new(1.0, theme::ACCENT),
                );
            }
            if lr.clicked() {
                action = Some(RowAction::Details);
            }
        }
    }

    if hovered {
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(close));
        if widgets::icon_button(
            &mut child,
            Icon::Close,
            "Remove from this list (no files are deleted)",
            false,
            true,
        )
        .clicked()
        {
            action = Some(RowAction::Remove);
        }
    }
    if resp.hovered() && exists {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    if resp.clicked() && exists {
        action = Some(RowAction::Open);
    }
    resp.clone().on_hover_text(widgets::short_path(&path));
    resp.context_menu(|ui| {
        if exists {
            if ui.button("Open").clicked() {
                action = Some(RowAction::Open);
            }
            if ui.button("Roll info…").clicked() {
                action = Some(RowAction::Details);
            }
        } else if ui.button("Locate…").clicked() {
            action = Some(RowAction::Locate);
        }
        ui.separator();
        if ui.button("Remove from list").clicked() {
            action = Some(RowAction::Remove);
        }
    });
    (action, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cover_prefers_picks_and_skips_rejects_and_unusable() {
        let n = (Flag::None, true);
        let p = (Flag::Pick, true);
        let r = (Flag::Reject, true);
        let unusable = (Flag::None, false);
        assert_eq!(pick_cover(&[r, n, unusable, p, n, n], 4), vec![1, 3, 4, 5]);
        // Plenty of picks: only picks, spread over them, in roll order.
        assert_eq!(pick_cover(&[p, n, p, n, p, p, p, p, p, p], 2), vec![0, 6]);
        assert!(pick_cover(&[r, unusable], 4).is_empty());
        assert_eq!(pick_cover(&[], 4), Vec::<usize>::new());
    }

    /// Clicks reach the part of the row they land on, not the row beneath it.
    #[test]
    fn row_clicks_reach_the_right_control() {
        use egui_kittest::Harness;

        #[derive(Default)]
        struct State {
            themed: bool,
            rect: Option<egui::Rect>,
            action: Option<&'static str>,
        }
        let cover = RollCover {
            loaded: true,
            count: 36,
            ..Default::default()
        };
        let mut h = Harness::builder()
            .with_size(vec2(800.0, 480.0))
            .build_ui_state(
                move |ui, st: &mut State| {
                    // The app's fonts take effect on the next pass.
                    if !st.themed {
                        theme::apply(ui.ctx());
                        st.themed = true;
                        return;
                    }
                    let min = ui.next_widget_position();
                    let r = egui::Rect::from_min_size(min, grid_layout(360.0).1);
                    let mut card = ui.new_child(egui::UiBuilder::new().max_rect(r));
                    let (a, _) = roll_card(&mut card, r, Path::new("/rolls/one"), Some(&cover));
                    st.rect = Some(r);
                    if let Some(a) = a {
                        st.action = Some(match a {
                            RowAction::Open => "open",
                            RowAction::Details => "details",
                            RowAction::Remove => "remove",
                            RowAction::Locate => "locate",
                        });
                    }
                },
                State::default(),
            );
        h.run_steps(3);
        let rect = h.state().rect.unwrap();
        let mut click = |pos: egui::Pos2| {
            h.state_mut().action = None;
            h.input_mut().events.push(egui::Event::PointerMoved(pos));
            h.run_steps(3);
            for pressed in [true, false] {
                h.input_mut().events.push(egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: egui::Modifiers::NONE,
                });
                h.run_steps(2);
            }
            h.state().action
        };
        // The text under the picture ends in the remove icon; the picture opens.
        let meta_y = rect.bottom() - CARD_PAD - META_H / 2.0;
        let remove = egui::pos2(rect.right() - CARD_PAD - 13.0, meta_y - NAME_H / 2.0);
        assert_eq!(click(remove), Some("remove"));
        assert_eq!(click(rect.center()), Some("open"));
        // No film stock, so the details end in the link while hovered.
        let details = egui::pos2(rect.left() + CARD_PAD + 110.0, meta_y);
        assert_eq!(click(details), Some("details"));
    }

    #[test]
    fn grid_fills_the_width_with_columns_that_fit() {
        for (width, cols) in [
            (200.0, 1),
            (640.0, 2),
            (1152.0, 3),
            (1382.0, 4),
            (1600.0, 5),
        ] {
            let (n, card) = grid_layout(width);
            assert_eq!(n, cols, "{width}");
            let used = card.x * n as f32 + CARD_GAP * (n - 1) as f32;
            assert!((used - width).abs() < 1e-3, "{width}");
        }
    }

    #[test]
    fn three_frames_make_squares() {
        let (_, card) = grid_layout(1000.0);
        let w = card.x - CARD_PAD * 2.0;
        let h = card.y - CARD_PAD * 2.0 - TEXT_GAP - NAME_H - META_H;
        let rect = egui::Rect::from_min_size(egui::pos2(5.0, 7.0), vec2(w, h));
        let tiles = mosaic(rect, 3);
        for t in &tiles {
            assert!((t.width() - t.height()).abs() < 1e-3, "{t:?}");
        }
        assert!((tiles[0].height() - h).abs() < 1e-3);
        assert!((tiles[1].right() - rect.right()).abs() < 1e-3);
        assert!((tiles[2].bottom() - rect.bottom()).abs() < 1e-3);
        assert_eq!(mosaic(rect, 1), vec![rect]);
        assert_eq!(mosaic(rect, 2).len(), 2);
        assert!(mosaic(rect, 0).is_empty());
    }

    #[test]
    fn cover_crops_to_the_centred_shape() {
        let square = vec2(100.0, 100.0);
        let landscape = cover_uv([300, 200], square);
        assert!((landscape.width() - 2.0 / 3.0).abs() < 1e-6);
        assert_eq!(landscape.height(), 1.0);
        assert_eq!(landscape.center(), egui::pos2(0.5, 0.5));
        let portrait = cover_uv([200, 300], square);
        assert_eq!(portrait.width(), 1.0);
        assert!((portrait.height() - 2.0 / 3.0).abs() < 1e-6);
        assert_eq!(cover_uv([100, 100], square).size(), vec2(1.0, 1.0));
    }

    #[test]
    fn first_pick_takes_the_large_spot() {
        let n = (Flag::None, true);
        let p = (Flag::Pick, true);
        let mut picked = vec![0, 2, 3];
        pick_leads(&mut picked, &[n, n, p, p]);
        assert_eq!(picked, vec![2, 0, 3]);
        let mut none = vec![0, 1];
        pick_leads(&mut none, &[n, n]);
        assert_eq!(none, vec![0, 1]);
    }

    #[test]
    fn time_ago_words() {
        const D: u64 = 86_400;
        assert_eq!(time_ago(60), "Today");
        assert_eq!(time_ago(D), "Yesterday");
        assert_eq!(time_ago(3 * D), "3 days ago");
        assert_eq!(time_ago(14 * D), "2 weeks ago");
        assert_eq!(time_ago(90 * D), "3 months ago");
        assert_eq!(time_ago(800 * D), "2 years ago");
    }
}
