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

/// Frames in a roll's strip on the Projects screen.
const COVER_FRAMES: usize = 4;
const ROW_H: f32 = 64.0;
const TILE_H: f32 = 44.0;

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
            let frames = pick_cover(&candidates, COVER_FRAMES)
                .into_iter()
                .filter_map(|i| render_cover_frame(&scans[i], sidecars[i].as_ref()?))
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
                let w = ui.available_width().min(760.0);
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
                        for dir in self.defaults.projects.clone() {
                            let (row, visible) = project_row(ui, &dir, self.roll.covers.get(&dir));
                            if visible && !self.roll.covers.contains_key(&dir) {
                                self.request_cover(&dir);
                            }
                            if let Some(a) = row {
                                action = Some((dir.clone(), a));
                            }
                            ui.add_space(6.0);
                        }
                        ui.add_space(10.0);
                        ui.label(
                            RichText::new("Drop a roll folder or scans anywhere to open them")
                                .small()
                                .color(theme::FAINT),
                        );
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

/// A frame as the grid renders it, from its cached thumbnail source. A strip
/// is never worth decoding a RAW for, so a frame without one is left out.
fn render_cover_frame(image: &Path, sc: &Sidecar) -> Option<egui::ColorImage> {
    let src = jobs::read_thumb(image)?;
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

/// The centred square of a `[w, h]` image, so landscape and portrait frames
/// sit in a strip at one size.
fn square_uv([w, h]: [usize; 2]) -> egui::Rect {
    let (w, h) = (w.max(1) as f32, h.max(1) as f32);
    let side = w.min(h);
    egui::Rect::from_center_size(egui::pos2(0.5, 0.5), vec2(side / w, side / h))
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

/// One roll: the whole row opens it. Returns the action and whether the row
/// is on screen.
fn project_row(
    ui: &mut egui::Ui,
    dir: &Path,
    cover: Option<&RollCover>,
) -> (Option<RowAction>, bool) {
    let w = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(vec2(w, ROW_H), Sense::click());
    if !ui.is_rect_visible(rect) {
        return (None, false);
    }
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
    let inner = rect.shrink2(vec2(14.0, 10.0));

    // Right edge: the remove icon on hover, then the strip or Locate.
    let mut right = inner.right();
    let close = egui::Rect::from_min_size(
        egui::pos2(right - 26.0, inner.center().y - 13.0),
        vec2(26.0, 26.0),
    );
    right = close.left() - 8.0;
    let mut left_limit = right;
    if exists {
        let count = cover.map_or(0, |c| c.count);
        let frames: &[egui::TextureHandle] = cover.map_or(&[], |c| &c.frames);
        let placeholders = if frames.is_empty() {
            count.min(COVER_FRAMES)
        } else {
            0
        };
        let more = count.saturating_sub(frames.len() + placeholders);
        let y = inner.center().y - TILE_H / 2.0;
        // A fixed column, so strips line up from row to row.
        right -= 32.0;
        if more > 0 {
            let g = ui.painter().layout_no_wrap(
                format!("+{more}"),
                egui::FontId::proportional(11.5),
                theme::FAINT,
            );
            ui.painter().galley(
                egui::pos2(right, inner.center().y - g.size().y / 2.0),
                g,
                theme::FAINT,
            );
        }
        right -= 8.0;
        let tile = |ui: &egui::Ui, right: &mut f32, tex: Option<&egui::TextureHandle>| {
            *right -= TILE_H;
            let r = egui::Rect::from_min_size(egui::pos2(*right, y), vec2(TILE_H, TILE_H));
            match tex {
                Some(t) => {
                    ui.painter()
                        .image(t.id(), r, square_uv(t.size()), egui::Color32::WHITE);
                }
                None => {
                    ui.painter().rect_filled(r, 2.0, theme::SURFACE);
                }
            }
            *right -= 3.0;
        };
        for t in frames.iter().rev() {
            tile(ui, &mut right, Some(t));
        }
        for _ in 0..placeholders {
            tile(ui, &mut right, None);
        }
        left_limit = right - 8.0;
    } else {
        let r = egui::Rect::from_min_max(
            egui::pos2(right - 90.0, inner.top()),
            egui::pos2(right, inner.bottom()),
        );
        ui.scope_builder(
            egui::UiBuilder::new()
                .max_rect(r)
                .layout(egui::Layout::right_to_left(egui::Align::Center)),
            |ui| {
                if ui
                    .add(widgets::ghost("Locate…"))
                    .on_hover_text("Find the folder this roll moved to")
                    .clicked()
                {
                    action = Some(RowAction::Locate);
                }
            },
        );
        left_limit -= 98.0;
    }

    // Left: name, then the metadata line.
    let text_w = left_limit - inner.left();
    let title = truncated(
        ui,
        roll_name(dir, info),
        theme::medium(14.0),
        if exists { theme::TEXT } else { theme::FAINT },
        text_w,
    );
    let meta_y = inner.center().y + 2.0;
    ui.painter().galley(
        egui::pos2(inner.left(), meta_y - title.size().y - 1.0),
        title,
        theme::TEXT,
    );
    let small = egui::FontId::proportional(12.0);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    if !exists {
        let g = truncated(
            ui,
            format!("Folder not found · {}", widgets::short_path(&path)),
            small,
            theme::WARN,
            text_w,
        );
        ui.painter()
            .galley(egui::pos2(inner.left(), meta_y), g, theme::WARN);
    } else if let Some(c) = cover.filter(|c| c.loaded) {
        let no_stock = info.is_none_or(|i| i.metadata.film_stock.is_empty());
        let link = no_stock.then(|| {
            ui.painter()
                .layout_no_wrap("Add film details".into(), small.clone(), theme::ACCENT)
        });
        let link_w = link.as_ref().map_or(0.0, |l| l.size().x + 16.0);
        let g = truncated(ui, row_meta(c, now), small, theme::MUTED, text_w - link_w);
        let end = inner.left() + g.size().x;
        ui.painter()
            .galley(egui::pos2(inner.left(), meta_y), g, theme::MUTED);
        if let Some(l) = link {
            let r = egui::Rect::from_min_size(egui::pos2(end + 16.0, meta_y), l.size());
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
        ui.scope_builder(egui::UiBuilder::new().max_rect(close), |ui| {
            if widgets::icon_button(
                ui,
                Icon::Close,
                "Remove from this list (no files are deleted)",
                false,
                true,
            )
            .clicked()
            {
                action = Some(RowAction::Remove);
            }
        });
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
            .with_size(vec2(800.0, 200.0))
            .build_ui_state(
                move |ui, st: &mut State| {
                    // The app's fonts take effect on the next pass.
                    if !st.themed {
                        theme::apply(ui.ctx());
                        st.themed = true;
                        return;
                    }
                    let min = ui.next_widget_position();
                    let (a, _) = project_row(ui, Path::new("/rolls/one"), Some(&cover));
                    st.rect = Some(egui::Rect::from_min_size(
                        min,
                        vec2(ui.available_width(), ROW_H),
                    ));
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
        // The right edge holds the remove icon, the middle of the row is empty.
        let remove = egui::pos2(rect.right() - 14.0 - 13.0, rect.center().y);
        assert_eq!(click(remove), Some("remove"));
        assert_eq!(click(rect.center()), Some("open"));
        // No film stock, so the metadata line ends in the link.
        let details = egui::pos2(rect.left() + 14.0 + 100.0, rect.center().y + 9.0);
        assert_eq!(click(details), Some("details"));
    }

    #[test]
    fn strip_tiles_crop_to_the_centred_square() {
        let landscape = square_uv([300, 200]);
        assert!((landscape.width() - 2.0 / 3.0).abs() < 1e-6);
        assert_eq!(landscape.height(), 1.0);
        assert_eq!(landscape.center(), egui::pos2(0.5, 0.5));
        let portrait = square_uv([200, 300]);
        assert_eq!(portrait.width(), 1.0);
        assert!((portrait.height() - 2.0 / 3.0).abs() < 1e-6);
        assert_eq!(square_uv([100, 100]).size(), vec2(1.0, 1.0));
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
