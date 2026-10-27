//! Roll info (`NLP/roll.json`): the dialog shown when a folder is opened for
//! the first time, and the Projects screen listing every opened roll.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use eframe::egui::{self, RichText, Sense, vec2};

use super::NlpApp;
use super::widgets::{self, theme};
use crate::engine::settings::FilmMetadata;
use crate::engine::sidecar::{self, RollInfo};

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
    /// Roll info per project folder, read when the Projects screen opens.
    cache: HashMap<PathBuf, Option<RollInfo>>,
}

pub struct RollDialog {
    pub dir: PathBuf,
    pub info: RollInfo,
    /// First open of the folder (title "New roll", Skip instead of Cancel).
    pub new: bool,
}

/// The film / capture fields as a two-column grid. Returns true on change.
pub fn film_fields(ui: &mut egui::Ui, id: &str, m: &mut FilmMetadata, notes: bool) -> bool {
    let mut changed = false;
    let mut fields: Vec<(&str, &mut String, &str)> = vec![
        ("Film stock", &mut m.film_stock, "Portra 400"),
        ("Film ISO", &mut m.film_iso, "400"),
        ("Shot at ISO", &mut m.shot_at_iso, "800 (push)"),
        ("Developer", &mut m.developer, "C-41 lab"),
        ("Camera make", &mut m.camera_make, "Nikon"),
        ("Camera model", &mut m.camera_model, "FM2"),
        ("Lens", &mut m.lens, "50mm f/1.8"),
        ("Shutter", &mut m.shutter_speed, "1/125"),
        ("Aperture", &mut m.aperture, "f/8"),
    ];
    if notes {
        fields.push(("Notes", &mut m.development_notes, ""));
    }
    egui::Grid::new(id)
        .num_columns(2)
        .spacing(vec2(8.0, 4.0))
        .show(ui, |ui| {
            for (label, v, hint) in fields {
                ui.label(RichText::new(label).small().color(theme::MUTED));
                changed |= ui
                    .add(
                        egui::TextEdit::singleline(v)
                            .hint_text(hint)
                            .desired_width(ui.available_width()),
                    )
                    .changed();
                ui.end_row();
            }
        });
    changed
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
        self.roll.dialog = Some(RollDialog {
            dir: dir.to_path_buf(),
            info,
            new,
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
        });
    }

    /// The "New roll" banner: name and film stock inline, the rest in the
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
                    ui.label(RichText::new("New roll").font(theme::medium(13.0)));
                    ui.add_space(8.0);
                    let enter = |r: &egui::Response| {
                        r.lost_focus() && r.ctx.input(|i| i.key_pressed(egui::Key::Enter))
                    };
                    let r = ui.add(
                        egui::TextEdit::singleline(&mut d.info.name)
                            .hint_text("Name")
                            .desired_width(160.0),
                    );
                    save |= enter(&r);
                    let r = ui.add(
                        egui::TextEdit::singleline(&mut d.info.metadata.film_stock)
                            .hint_text("Film stock, e.g. Portra 400")
                            .desired_width(180.0),
                    );
                    save |= enter(&r);
                    save |= ui.add(widgets::primary("Save")).clicked();
                    more = ui
                        .add(egui::Button::new(RichText::new("More details…").small().color(theme::MUTED)).frame(false))
                        .clicked();
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        dismiss = widgets::icon_button(ui, widgets::Icon::Close, "Dismiss", false, true)
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
        for f in &mut self.frames {
            if f.sc.metadata == FilmMetadata::default() || f.sc.metadata == old {
                f.sc.metadata = info.metadata.clone();
                f.dirty_since = Some(self.time);
            }
        }
        self.roll
            .cache
            .insert(dir.to_path_buf(), Some(info.clone()));
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
                                .desired_width(ui.available_width()),
                        );
                        ui.end_row();
                        ui.label(RichText::new("Date").small().color(theme::MUTED));
                        ui.add(
                            egui::TextEdit::singleline(&mut d.info.date)
                                .hint_text("2026-10-01")
                                .desired_width(ui.available_width()),
                        );
                        ui.end_row();
                    });
                ui.add_space(4.0);
                film_fields(ui, "roll_film", &mut d.info.metadata, false);
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
            let RollDialog { dir, info, .. } = d;
            self.save_roll_info(&dir, info);
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
        self.roll.cache = self
            .defaults
            .projects
            .iter()
            .map(|p| (p.clone(), sidecar::load_roll_info(p)))
            .collect();
        self.roll.projects_open = true;
    }

    /// Every roll opened before, most recent first.
    pub fn projects_ui(&mut self, ui: &mut egui::Ui) {
        let mut open: Option<PathBuf> = None;
        let mut remove: Option<PathBuf> = None;
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(theme::BG)
                    .inner_margin(egui::Margin::symmetric(24, 20)),
            )
            .show(ui, |ui| {
                let w = ui.available_width().min(720.0);
                ui.vertical_centered(|ui| {
                    ui.set_max_width(w);
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("Projects")
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
                        });
                    });
                    ui.add_space(14.0);
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for dir in self.defaults.projects.clone() {
                            let exists = dir.is_dir();
                            let info = self.roll.cache.get(&dir).cloned().flatten();
                            match project_row(ui, &dir, info.as_ref(), exists) {
                                Some(RowAction::Open) => open = Some(dir.clone()),
                                Some(RowAction::Remove) => remove = Some(dir.clone()),
                                None => {}
                            }
                            ui.add_space(6.0);
                        }
                    });
                });
            });
        if let Some(dir) = remove {
            self.defaults.projects.retain(|p| p != &dir);
            // Otherwise startup would re-add it from `last_folder`.
            if self.defaults.last_folder.as_ref() == Some(&dir) {
                self.defaults.last_folder = None;
            }
            let _ = sidecar::save_defaults(&self.defaults);
        }
        if let Some(dir) = open {
            self.roll.projects_open = false;
            self.open_folder(&dir);
        }
    }
}

enum RowAction {
    Open,
    Remove,
}

/// One project card with Open / Remove buttons.
fn project_row(
    ui: &mut egui::Ui,
    dir: &Path,
    info: Option<&RollInfo>,
    exists: bool,
) -> Option<RowAction> {
    let mut action = None;
    let w = ui.available_width();
    egui::Frame::new()
        .fill(theme::PANEL)
        .stroke(egui::Stroke::new(1.0, theme::LINE))
        .corner_radius(10.0)
        .inner_margin(egui::Margin::symmetric(14, 10))
        .show(ui, |ui| {
            ui.set_width(w - 28.0);
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    let name = info
                        .map(|i| i.name.clone())
                        .filter(|n| !n.is_empty())
                        .unwrap_or_else(|| folder_name(dir));
                    let col = if exists { theme::TEXT } else { theme::FAINT };
                    let title = ui.add(
                        egui::Label::new(RichText::new(name).font(theme::medium(14.0)).color(col))
                            .sense(Sense::click()),
                    );
                    if exists && title.clicked() {
                        action = Some(RowAction::Open);
                    }
                    let mut parts: Vec<String> = Vec::new();
                    if let Some(i) = info {
                        let m = &i.metadata;
                        if !m.film_stock.is_empty() {
                            parts.push(m.film_stock.clone());
                        }
                        if !m.film_iso.is_empty() {
                            parts.push(format!("ISO {}", m.film_iso));
                        }
                        let cam = format!("{} {}", m.camera_make, m.camera_model);
                        if !cam.trim().is_empty() {
                            parts.push(cam.trim().to_string());
                        }
                        if !i.date.is_empty() {
                            parts.push(i.date.clone());
                        }
                    }
                    if exists {
                        let n = sidecar::sidecar_count(dir);
                        parts.push(format!("{n} frame{}", if n == 1 { "" } else { "s" }));
                    } else {
                        parts.push("Folder not found".into());
                    }
                    ui.label(RichText::new(parts.join(" · ")).small().color(theme::MUTED));
                    ui.label(
                        RichText::new(dir.display().to_string())
                            .small()
                            .color(theme::FAINT),
                    );
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if exists && ui.add(widgets::primary("Open")).clicked() {
                        action = Some(RowAction::Open);
                    }
                    if ui
                        .add(widgets::ghost("Remove"))
                        .on_hover_text("Remove from this list (no files are deleted)")
                        .clicked()
                    {
                        action = Some(RowAction::Remove);
                    }
                });
            });
        });
    action
}
