//! Folder View: browse a folder of scans, convert (individually or as a roll)
//! and batch-edit / export the selection.

use eframe::egui::{self, Color32, RichText, Sense, pos2, vec2};

use super::NlpApp;
use super::grid::GridFilter;
pub use super::grid::{GRID_VISIBLE, GridState};
use super::widgets::{self, Icon, icon_button, theme};
use crate::engine::settings::*;

/// One-line hint shown under the scan source.
pub fn source_instructions(s: Source) -> &'static str {
    match s {
        Source::DigitalCamera => {
            "The film base is neutralised automatically, or sample the unexposed border."
        }
        Source::VueScanRaw => "Sample the film border, or leave the film base on Auto.",
        Source::TifScan => "TIF scans are not white balanced: the film base stays off.",
        Source::LinearScan => "Linear (gamma 1.0) scans are read as stored; no preparation needed.",
    }
}

/// Short name for the scan source chip.
fn source_short(s: Source) -> &'static str {
    match s {
        Source::DigitalCamera => "Camera scan",
        Source::VueScanRaw => "VueScan / SilverFast",
        Source::TifScan => "TIF scan",
        Source::LinearScan => "Linear scan",
    }
}

impl NlpApp {
    /// The source most of the roll's frames use.
    pub fn main_source(&self) -> Option<Source> {
        Source::ALL
            .iter()
            .copied()
            .max_by_key(|&s| {
                self.frames
                    .iter()
                    .filter(|f| f.sc.settings.source == s)
                    .count()
            })
            .filter(|_| !self.frames.is_empty())
    }

    /// The roll's scan source, detected from the files: shows what was
    /// found and sets the source of every frame.
    pub fn source_chip(&mut self, ui: &mut egui::Ui) {
        let mut counts: Vec<(Source, usize)> = Source::ALL
            .iter()
            .map(|&s| {
                (
                    s,
                    self.frames
                        .iter()
                        .filter(|f| f.sc.settings.source == s)
                        .count(),
                )
            })
            .filter(|c| c.1 > 0)
            .collect();
        counts.sort_by(|a, b| b.1.cmp(&a.1));
        let Some(&(main, _)) = counts.first() else {
            return;
        };
        // Detected and rarely changed, so it lives in Roll info. The bar only
        // shows it when the roll is mixed, or a single file has no roll to open.
        if counts.len() == 1 && !self.single_mode {
            return;
        }
        let text = match counts.len() {
            1 => source_short(main).to_string(),
            n => format!("{} +{}", source_short(main), n - 1),
        };
        let r = ui
            .add(
                egui::Button::new(RichText::new(text).small().color(theme::MUTED))
                    .corner_radius(10.0)
                    .min_size(vec2(0.0, 20.0)),
            )
            .on_hover_text(
                "Scan source, detected from the files. Click to change it for the roll.",
            );
        egui::Popup::menu(&r).show(|ui| {
            ui.set_width(270.0);
            ui.label(RichText::new("Scan source").small().color(theme::MUTED));
            for &(s, n) in &counts {
                ui.label(format!("{n} × {}", s.label()));
            }
            ui.separator();
            ui.label(
                RichText::new("Set for the whole roll")
                    .small()
                    .color(theme::MUTED),
            );
            for &s in Source::ALL {
                let only = counts.len() == 1 && main == s;
                if ui.selectable_label(only, s.label()).clicked() && !only {
                    self.set_roll_source(s);
                }
            }
            ui.add_space(4.0);
            ui.label(
                RichText::new(source_instructions(main))
                    .small()
                    .color(theme::FAINT),
            );
        });
    }
}

impl NlpApp {
    /// Slim notices under the top bar, each dismissible.
    fn banners_ui(&mut self, ui: &mut egui::Ui) {
        let Some(spread) = self.auto.exposure_note else {
            return;
        };
        egui::Panel::top("folder_banner")
            .exact_size(32.0)
            .frame(
                egui::Frame::new()
                    .fill(theme::BG)
                    .inner_margin(egui::Margin::symmetric(12, 0)),
            )
            .show(ui, |ui| {
                ui.horizontal_centered(|ui| {
                    widgets::paint_icon(
                        ui.painter(),
                        egui::Rect::from_center_size(ui.cursor().min + vec2(7.0, 0.0), vec2(14.0, 14.0)),
                        Icon::Warn,
                        theme::WARN,
                    );
                    ui.add_space(18.0);
                    ui.label(
                        RichText::new(format!(
                            "Exposures vary by {spread:.1} EV, so the roll was balanced by the images instead"
                        ))
                        .small()
                        .color(theme::MUTED),
                    );
                    if ui.small_button("Use exposure data").clicked() {
                        self.auto.exposure_note = None;
                        self.convert((0..self.frames.len()).collect(), true);
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if icon_button(ui, Icon::Close, "Dismiss", false, true).clicked() {
                            self.auto.exposure_note = None;
                        }
                    });
                });
            });
    }
}

/// The dye crosstalk matrix (generic C-41 or a bundled film) and its
/// strength. Colour negatives only.
pub fn crosstalk_ui(ui: &mut egui::Ui, s: &mut Settings) {
    if s.image_type != ImageType::Negative || s.is_monochrome() {
        return;
    }
    let films = crate::engine::presets::crosstalk_profiles();
    let mut pick = films
        .iter()
        .position(|p| p.id == s.crosstalk)
        .map_or(0, |i| i + 1);
    let all: Vec<usize> = (0..=films.len()).collect();
    if widgets::fl_combo(
        ui,
        "Crosstalk",
        &mut pick,
        &all,
        |i| {
            if i == 0 {
                "Generic C-41"
            } else {
                films[i - 1].name
            }
        },
        "Unmixes the dyes' leakage into each other's channels. Film matrices are read off \
         spec sheets for a narrowband scanner (approx); Generic is the engine's own.",
    ) {
        s.crosstalk = if pick == 0 {
            String::new()
        } else {
            films[pick - 1].id.to_string()
        };
    }
    widgets::fl_slider(
        ui,
        "Crosstalk strength",
        &mut s.crosstalk_strength,
        0.0..=150.0,
        100.0,
        0,
        "Percent of the crosstalk correction: 0 is off, more separates colours further",
        None,
    );
}

/// The flat-field input: a frame of the bare light source or clear film
/// leader, shot like the negatives. Shows the file name with Choose / Clear.
pub fn flat_field_ui(
    ui: &mut egui::Ui,
    flat: &mut Option<std::path::PathBuf>,
    start: Option<&std::path::Path>,
) {
    ui.label(RichText::new("Flat field").small().color(theme::MUTED))
        .on_hover_text(
            "A frame of the bare light source (or clear film leader) shot with the same camera, \
         lens and light. Divides out uneven light and lens falloff.",
        );
    ui.horizontal(|ui| {
        let name = flat
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "None".into());
        ui.label(RichText::new(name).color(theme::TEXT));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if flat.is_some() && ui.add(widgets::ghost("Clear")).clicked() {
                *flat = None;
            }
            if ui.add(widgets::ghost("Choose…")).clicked() {
                if let Some(p) = super::picker::pick_files(start).into_iter().next() {
                    *flat = Some(p);
                }
            }
        });
    });
}

impl NlpApp {
    pub fn sel_vec(&self) -> Vec<usize> {
        self.selected.iter().copied().collect()
    }

    /// Save and close the open folder, then show the Projects (rolls) list.
    fn close_to_projects(&mut self) {
        // The scanner writes into the open roll: release it with the roll, or a
        // running scan keeps adding frames behind the Rolls screen.
        self.close_scan_panel();
        self.save_all();
        self.frames.clear();
        self.selected.clear();
        self.active = None;
        self.anchor = None;
        self.single_mode = false;
        self.editor = super::editor::Editor::default();
        self.show_projects();
    }

    /// Start screen shown when there are no projects yet.
    fn welcome_ui(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(theme::BG))
            .show(ui, |ui| {
                let avail = ui.max_rect();
                let zone_w = (avail.width() - 64.0).clamp(280.0, 520.0);
                let zone_h = (avail.height() * 0.26).clamp(150.0, 210.0);
                let content_h = 64.0 + 24.0 + zone_h + 16.0 + 28.0 + 12.0 + 18.0;
                ui.add_space(((avail.height() - content_h) * 0.45).max(24.0));
                ui.vertical_centered(|ui| {
                    ui.label(
                        RichText::new("NLP Studio")
                            .font(theme::medium(26.0))
                            .color(theme::TEXT),
                    );
                    ui.label(RichText::new("Film negative conversion").color(theme::MUTED));
                    ui.add_space(24.0);

                    let (zone, resp) = ui.allocate_exact_size(vec2(zone_w, zone_h), Sense::click());
                    let p = ui.painter();
                    let (fill, stroke, fg) = if resp.hovered() {
                        (
                            theme::PANEL,
                            egui::Stroke::new(1.0, Color32::from_white_alpha(28)),
                            theme::TEXT,
                        )
                    } else {
                        (
                            Color32::TRANSPARENT,
                            egui::Stroke::new(1.0, Color32::from_white_alpha(20)),
                            theme::MUTED,
                        )
                    };
                    p.rect(zone, 14.0, fill, stroke, egui::StrokeKind::Inside);
                    let c = zone.center();
                    widgets::paint_icon(
                        p,
                        egui::Rect::from_center_size(c - vec2(0.0, 26.0), vec2(34.0, 34.0)),
                        Icon::Folder,
                        fg,
                    );
                    p.text(
                        c + vec2(0.0, 14.0),
                        egui::Align2::CENTER_CENTER,
                        "Drop a folder or scans here",
                        theme::medium(15.0),
                        theme::TEXT,
                    );
                    p.text(
                        c + vec2(0.0, 36.0),
                        egui::Align2::CENTER_CENTER,
                        "or click to browse",
                        egui::FontId::proportional(12.5),
                        theme::MUTED,
                    );
                    if resp.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                    if resp.clicked() {
                        self.choose_anything();
                    }

                    ui.add_space(16.0);
                    if ui
                        .add(widgets::ghost("New roll…"))
                        .on_hover_text("Choose or create a folder, then scan film into it")
                        .clicked()
                    {
                        self.choose_new_roll();
                    }
                    ui.add_space(12.0);
                    ui.label(
                        RichText::new(
                            "Camera RAW · TIFF · JPEG · PNG, or scan film into a new roll",
                        )
                        .small()
                        .color(theme::FAINT),
                    );
                });
            });
        widgets::drop_overlay(ui.ctx());
    }

    pub fn folder_ui(&mut self, ui: &mut egui::Ui) {
        self.open_shortcut(ui.ctx());
        // An open roll shows its (maybe still empty) grid, so its Scan button is reachable.
        let empty_roll = self.folder.is_some() && !self.single_mode && !self.roll.projects_open;
        if self.frames.is_empty() && !empty_roll {
            if self.roll.projects_open && !self.defaults.projects.is_empty() {
                self.projects_ui(ui);
            } else {
                self.welcome_ui(ui);
            }
            return;
        }
        let ctx = ui.ctx().clone();
        self.folder_shortcuts(&ctx);
        let cmd = egui::Modifiers::COMMAND;
        let key_text = |m, k| RichText::new(widgets::shortcut(&ctx, m, k)).color(theme::FAINT);
        let (picks, unflagged, rejected) = self.filter_counts();
        // Export, Convert and the contact sheet act on the selection, or else
        // on the frames on show. Selection changes their scope, never their place.
        let shown = self.shown_frames();
        let scope = self.export_scope();
        let export_label = self.export_label();
        let skipped = self.rejects_skipped();
        let convert_targets = if self.selected.is_empty() {
            shown.clone()
        } else {
            self.sel_vec()
        };
        let converted = convert_targets
            .iter()
            .all(|&i| self.frames[i].sc.is_converted());
        egui::Panel::top("folder_top")
            .exact_size(44.0)
            .frame(
                egui::Frame::new()
                    .fill(theme::PANEL)
                    .inner_margin(egui::Margin::symmetric(10, 0)),
            )
            .show(ui, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;
                    // Left: where you are. Breadcrumb, then the roll's film and scan source.
                    let back = ui
                        .add(egui::Button::new(RichText::new("‹  Rolls").color(theme::MUTED)).frame(false))
                        .on_hover_text("Back to all rolls")
                        .on_hover_cursor(egui::CursorIcon::PointingHand);
                    if back.clicked() {
                        self.close_to_projects();
                    }
                    if let Some(f) = self.folder.clone() {
                        ui.label(RichText::new("/").color(theme::FAINT));
                        let name = self
                            .roll
                            .info
                            .as_ref()
                            .map(|r| r.name.clone())
                            .filter(|n| !n.trim().is_empty())
                            .or_else(|| f.file_name().map(|n| n.to_string_lossy().to_string()))
                            .unwrap_or_else(|| f.display().to_string());
                        ui.scope(|ui| {
                            ui.set_max_width(280.0);
                            let label = egui::Label::new(RichText::new(name).font(theme::medium(13.5)))
                                .truncate();
                            if self.single_mode {
                                ui.add(label).on_hover_text(f.display().to_string());
                            } else if ui
                                .add(label.sense(Sense::click()))
                                .on_hover_cursor(egui::CursorIcon::PointingHand)
                                .on_hover_text(format!(
                                    "{}\nClick for roll info: film, camera, notes",
                                    f.display()
                                ))
                                .clicked()
                            {
                                self.open_roll_dialog(&f, false);
                            }
                        });
                        // The film stock reads with the name and opens the same dialog.
                        let stock = self
                            .roll
                            .info
                            .as_ref()
                            .map(|r| r.metadata.film_stock.clone())
                            .filter(|s| !s.trim().is_empty());
                        // The chip is always there: with no stock yet it invites one.
                        let empty = stock.is_none();
                        let stock = stock.unwrap_or_else(|| "Add film stock".to_string());
                        {
                            // A chip like the editor's, with a chevron: it opens the dialog.
                            const CHEVRON_W: f32 = 12.0;
                            let font = egui::TextStyle::Small.resolve(ui.style());
                            let size = ui
                                .painter()
                                .layout_no_wrap(stock.clone(), font.clone(), theme::MUTED)
                                .size();
                            let (rect, resp) = ui.allocate_exact_size(
                                size + vec2(14.0 + CHEVRON_W, 6.0),
                                Sense::click(),
                            );
                            let hot = resp.hovered();
                            let (fill, ink) = if hot {
                                (theme::HOVER, theme::TEXT)
                            } else if empty {
                                (theme::SURFACE, theme::FAINT)
                            } else {
                                (theme::SURFACE, theme::MUTED)
                            };
                            ui.painter().rect_filled(rect, 5.0, fill);
                            if hot {
                                ui.painter().rect_stroke(
                                    rect,
                                    5.0,
                                    egui::Stroke::new(1.0, theme::PRESSED),
                                    egui::StrokeKind::Inside,
                                );
                            }
                            let g = ui.painter().layout_no_wrap(stock, font, ink);
                            let text_at =
                                pos2(rect.left() + 7.0, rect.center().y - g.size().y / 2.0);
                            ui.painter().galley(text_at, g, ink);
                            let cc = pos2(rect.right() - CHEVRON_W / 2.0 - 2.0, rect.center().y);
                            let (w, h) = (2.8, 1.6);
                            for seg in [
                                [cc + vec2(-w, -h), cc + vec2(0.0, h)],
                                [cc + vec2(0.0, h), cc + vec2(w, -h)],
                            ] {
                                ui.painter()
                                    .line_segment(seg, egui::Stroke::new(1.2, ink));
                            }
                            if resp
                                .on_hover_cursor(egui::CursorIcon::PointingHand)
                                .on_hover_text(if empty {
                                    "Add a film stock · click for roll info"
                                } else {
                                    "Film stock · click for roll info"
                                })
                                .clicked()
                            {
                                self.open_roll_dialog(&f, false);
                            }
                        }
                    }
                    self.source_chip(ui);

                    // Middle: which frames are on show.
                    if !self.frames.is_empty() {
                        ui.add_space(14.0);
                        let n = self.frames.len();
                        let tabs = [
                            (GridFilter::All, format!("All {n}"), "Every frame of the roll", true),
                            (GridFilter::Picks, format!("Picks {picks}"), "Frames picked with P", true),
                            // Only once culling has begun: until then it is All.
                            (
                                GridFilter::Unflagged,
                                format!("Unflagged {unflagged}"),
                                "Frames still to decide on",
                                unflagged < n,
                            ),
                            (
                                GridFilter::Rejected,
                                format!("Rejected {rejected}"),
                                "Frames rejected with X, left out of exports",
                                rejected > 0,
                            ),
                        ];
                        for (f, label, tip, show) in tabs {
                            if !show && self.grid.filter != f {
                                continue;
                            }
                            if filter_tab(ui, self.grid.filter == f, &label)
                                .on_hover_text(tip)
                                .clicked()
                            {
                                self.grid.filter = f;
                            }
                        }
                    }

                    // Right: what goes into and out of the roll. Export holds the
                    // far corner in every state; only its label changes.
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.spacing_mut().item_spacing.x = 6.0;
                        if !self.frames.is_empty() {
                            let mut tip = format!(
                                "Export positive copies ({})",
                                widgets::shortcut(&ctx, cmd, egui::Key::E)
                            );
                            if skipped > 0 {
                                tip.push_str(&format!(
                                    "\nLeaves out {skipped} rejected frame{}",
                                    if skipped == 1 { "" } else { "s" }
                                ));
                            }
                            if ui
                                .add_enabled(
                                    !scope.is_empty(),
                                    widgets::primary(&export_label).min_size(vec2(88.0, 0.0)),
                                )
                                .on_hover_text(tip)
                                .clicked()
                            {
                                self.open_export_dialog(scope.clone());
                            }
                        }
                        if !self.single_mode
                            && ui
                                .add(if self.scan.open { widgets::primary("Scan") } else { widgets::ghost("Scan") })
                                .on_hover_text("Scan film from a Plustek OpticFilm scanner or a SANE flatbed into this roll")
                                .clicked()
                        {
                            self.toggle_scan_panel();
                        }
                        let r = icon_button(ui, Icon::More, "More", false, true);
                        egui::Popup::menu(&r).show(|ui| {
                            ui.set_min_width(220.0);
                            if ui.button("Open folder…").clicked() {
                                self.choose_folder();
                            }
                            if ui.button("Open images…").clicked() {
                                self.choose_files();
                            }
                            ui.separator();
                            if ui
                                .add(egui::Button::new("Select all").shortcut_text(key_text(cmd, egui::Key::A)))
                                .clicked()
                            {
                                self.selected = shown.iter().copied().collect();
                            }
                            if ui
                                .add_enabled(
                                    self.selected.len() == 1,
                                    egui::Button::new("Copy settings")
                                        .shortcut_text(key_text(cmd, egui::Key::C)),
                                )
                                .on_disabled_hover_text("Select one frame to copy from")
                                .clicked()
                            {
                                self.copy_settings();
                            }
                            if ui
                                .add_enabled(
                                    self.clipboard.is_some() && !self.selected.is_empty(),
                                    egui::Button::new("Paste settings")
                                        .shortcut_text(key_text(cmd, egui::Key::V)),
                                )
                                .on_disabled_hover_text("Copy a frame's settings, then select where to paste them")
                                .clicked()
                            {
                                self.paste_settings();
                            }
                            ui.separator();
                            if ui
                                .add_enabled(
                                    !convert_targets.is_empty(),
                                    egui::Button::new(if converted { "Re-convert…" } else { "Convert…" }),
                                )
                                .on_hover_text("Analyse the selection, or else the frames on show, with new conversion options")
                                .clicked()
                            {
                                self.open_convert_for(convert_targets.clone());
                            }
                            if ui
                                .add_enabled(!scope.is_empty(), egui::Button::new("Contact sheet"))
                                .on_hover_text("Save a JPEG contact sheet of the same frames Export would take, in the NLP folder")
                                .clicked()
                            {
                                self.make_contact_sheet(scope.clone());
                            }
                            ui.separator();
                            let mut neg = self.grid.negatives;
                            if ui
                                .add(egui::Button::selectable(neg, "Show negatives").shortcut_text(
                                    RichText::new("hold N").color(theme::FAINT),
                                ))
                                .clicked()
                            {
                                neg = !neg;
                                self.grid.negatives = neg;
                            }
                            if ui
                                .add(egui::Button::new("Keyboard shortcuts").shortcut_text(RichText::new("?").color(theme::FAINT)))
                                .clicked()
                            {
                                self.shortcuts_open = true;
                            }
                        });
                        // Fixed slot so the buttons don't shift when jobs start/finish.
                        let (slot, _) = ui.allocate_exact_size(vec2(16.0, 16.0), Sense::hover());
                        if self.jobs.busy() > 0 {
                            ui.put(slot, egui::Spinner::new().size(14.0).color(theme::MUTED));
                        }
                    });
                });
            });
        self.roll_banner_ui(ui);
        self.banners_ui(ui);

        egui::Panel::bottom("folder_status")
            .exact_size(34.0)
            .frame(
                egui::Frame::new()
                    .fill(theme::PANEL)
                    .inner_margin(egui::Margin::symmetric(14, 0)),
            )
            .show(ui, |ui| {
                ui.horizontal_centered(|ui| {
                    // Left: what's selected and what to do with it, or a hint.
                    if !self.selected.is_empty() {
                        self.selection_strip_ui(ui);
                    } else if !self.frames.is_empty() {
                        let waiting = self.frames.iter().filter(|f| !f.sc.is_converted()).count();
                        let hint = if waiting > 0 && !self.auto.busy() {
                            format!("{waiting} not converted · Space to preview · ? for shortcuts")
                        } else {
                            "Space to preview · ? for shortcuts".to_string()
                        };
                        ui.label(RichText::new(hint).small().color(theme::MUTED));
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.spacing_mut().item_spacing.x = 6.0;
                        let tiny =
                            |t: &str| RichText::new(t.to_owned()).small().color(theme::MUTED);
                        widgets::mini_slider(ui, &mut self.thumb_size, 110.0..=340.0, 90.0)
                            .on_hover_text("Thumbnail size");
                        ui.label(tiny("Size"));
                        let (done, total) = (self.auto.convert_done, self.auto.convert_total);
                        if self.auto.busy() || done < total {
                            ui.add_space(12.0);
                            let auto = !self.auto.queue.is_empty()
                                || self.auto.roll_pass == super::auto::RollPass::Running;
                            if auto
                                && ui
                                    .add(egui::Button::new(tiny("Stop")).frame(false))
                                    .on_hover_text("Stop converting new frames by themselves")
                                    .clicked()
                            {
                                self.stop_auto();
                            }
                            let label = if self.auto.roll_pass == super::auto::RollPass::Running {
                                "Balancing the roll…".to_string()
                            } else {
                                format!("Converting {}/{}", done.min(total), total.max(1))
                            };
                            ui.add(
                                egui::ProgressBar::new(done as f32 / total.max(1) as f32)
                                    .desired_width(120.0)
                                    .desired_height(4.0)
                                    .fill(theme::ACCENT),
                            );
                            ui.label(RichText::new(label).small().color(theme::MUTED));
                        }
                        if self.export_total > 0 && self.export_done < self.export_total {
                            ui.add_space(12.0);
                            ui.add(
                                egui::ProgressBar::new(
                                    self.export_done as f32 / self.export_total as f32,
                                )
                                .desired_width(120.0)
                                .desired_height(4.0)
                                .fill(theme::ACCENT),
                            );
                            ui.label(
                                RichText::new(format!(
                                    "Exporting {}/{}",
                                    self.export_done, self.export_total
                                ))
                                .small()
                                .color(theme::MUTED),
                            );
                        }
                    });
                });
            });

        self.scan_panel_ui(ui);

        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(theme::BG)
                    .inner_margin(egui::Margin::same(16)),
            )
            .show(ui, |ui| {
                if self.frames.is_empty() {
                    ui.add_space((ui.available_height() * 0.35).max(24.0));
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new("No scans in this roll yet").font(theme::medium(16.0)),
                        );
                        ui.label(
                            RichText::new("Drop scans here, or scan film into it with Scan.")
                                .color(theme::MUTED),
                        );
                    });
                } else {
                    self.grid(ui);
                    self.quick_look_ui(ui);
                }
            });
    }

    /// Cmd+O, on the start screens as well as in a roll.
    fn open_shortcut(&mut self, ctx: &egui::Context) {
        let busy = ctx.egui_wants_keyboard_input()
            || self.shortcuts_open
            || self.scan.open
            || self.export_targets.is_some()
            || self.convert_dialog.is_some()
            || self.roll.dialog.is_some();
        if !busy && ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::O)) {
            self.choose_anything();
        }
    }

    fn folder_shortcuts(&mut self, ctx: &egui::Context) {
        if ctx.egui_wants_keyboard_input() || self.shortcuts_open {
            return;
        }
        let mac = ctx.os() == egui::os::OperatingSystem::Mac;
        let (undo, redo) = ctx.input(|i| {
            let cmd = i.modifiers.command;
            (
                cmd && !i.modifiers.shift && i.key_pressed(egui::Key::Z),
                (cmd && i.modifiers.shift && i.key_pressed(egui::Key::Z))
                    || (cmd && !mac && i.key_pressed(egui::Key::Y)),
            )
        });
        if undo || redo {
            self.history_step(undo);
        }
        let (all, none, enter, copy, paste, export, space) = ctx.input(|i| {
            (
                i.modifiers.command && i.key_pressed(egui::Key::A),
                i.key_pressed(egui::Key::Escape),
                i.key_pressed(egui::Key::Enter),
                i.modifiers.command && i.key_pressed(egui::Key::C),
                i.modifiers.command && i.key_pressed(egui::Key::V),
                i.modifiers.command && i.key_pressed(egui::Key::E),
                !i.modifiers.any() && i.key_pressed(egui::Key::Space),
            )
        });
        let dialog = self.export_targets.is_some();
        if space && !dialog {
            if self.grid.quick_look {
                self.grid.quick_look = false;
            } else if !self.frames.is_empty() {
                if self.selected.is_empty()
                    && let Some(&first) = self.shown_frames().first()
                {
                    self.selected.insert(first);
                    self.anchor = Some(first);
                    self.active = Some(first);
                }
                if self.active.is_none_or(|a| !self.selected.contains(&a)) {
                    self.active = self.selected.first().copied();
                }
                self.grid.quick_look = self.active.is_some();
            }
        }
        if let Some(flag) = Self::flag_key(ctx).filter(|_| !dialog) {
            self.flag_selection(flag);
        }
        let rotate = ctx.input(|i| {
            let cmd = i.modifiers.command;
            if cmd && i.key_pressed(egui::Key::OpenBracket) {
                Some(false)
            } else if cmd && i.key_pressed(egui::Key::CloseBracket) {
                Some(true)
            } else {
                None
            }
        });
        if let Some(cw) = rotate.filter(|_| self.export_targets.is_none()) {
            self.rotate_frames(&self.sel_vec(), cw);
        }
        if all {
            self.selected = self.shown_frames().into_iter().collect();
        }
        self.grid_arrows(ctx);
        if none && !dialog {
            if self.grid.quick_look {
                self.grid.quick_look = false;
            } else {
                self.selected.clear();
            }
        }
        if enter && !dialog {
            let lead = self
                .active
                .filter(|a| self.selected.contains(a))
                .or(self.selected.first().copied());
            if let Some(i) = lead {
                self.grid.quick_look = false;
                self.open_editor(i);
            }
        }
        if copy {
            self.copy_settings();
        }
        if paste {
            self.paste_settings();
        }
        if export {
            self.open_export_dialog(self.export_scope());
        }
    }

    /// What the roll's frames share, from the crops they already have, for
    /// settling a detection made by hand on one of them.
    pub fn stored_roll_shapes(&self) -> crate::engine::frame::RollShapes {
        crate::engine::frame::roll_shapes(self.frames.iter().filter_map(|f| {
            let s = &f.sc.settings;
            let cropped = s.crop != [0.0; 4] || s.straighten != 0.0;
            let src = f.proxy().filter(|_| cropped)?;
            Some((
                crate::engine::frame::is_landscape(&src, s),
                crate::engine::frame::Detected {
                    crop: s.crop,
                    straighten: s.straighten,
                },
            ))
        }))
    }

    /// Crop and straighten each frame to its detected frame, free-form.
    pub fn detect_frames(&mut self, idx: &[usize]) {
        self.record(idx);
        let shapes = self.stored_roll_shapes();
        let mut missed = 0;
        for &i in idx {
            let f = &self.frames[i];
            let Some(src) = f.proxy() else {
                missed += 1;
                continue;
            };
            let mut settings = f.sc.settings.clone();
            settings.crop_aspect = CropAspect::Free;
            let found = crate::engine::frame::detect(&src, &settings, f.base_for(&src));
            let shared = shapes.of(crate::engine::frame::is_landscape(&src, &settings));
            match crate::engine::frame::settle(found, shared) {
                Some(d) => {
                    let s = &mut self.frames[i].sc.settings;
                    (s.crop, s.straighten) = (d.crop, d.straighten);
                    s.crop_aspect = CropAspect::Free;
                    self.mark_changed(i);
                }
                None => missed += 1,
            }
        }
        self.editor.invalidate();
        if missed > 0 {
            self.toast(format!("No frame edge found in {missed} frame(s)"), false);
        }
    }

    pub fn copy_settings(&mut self) {
        let lead = self
            .active
            .filter(|i| self.selected.contains(i))
            .or(self.selected.first().copied());
        if let Some(i) = lead {
            self.clipboard = Some(self.frames[i].sc.settings.clone());
            let n = self.frames[i].name.clone();
            self.toast(format!("Copied settings from {n}"), false);
        }
    }

    pub fn paste_settings(&mut self) {
        self.paste_settings_to(self.sel_vec());
    }

    /// Paste the copied edit onto frames `idx`, as one undo step.
    pub fn paste_settings_to(&mut self, idx: Vec<usize>) {
        let Some(c) = self.clipboard.clone() else {
            return;
        };
        self.paste_edit_to(idx, &c);
    }

    /// Apply the edit `c` to frames `idx`, as one undo step.
    pub fn paste_edit_to(&mut self, idx: Vec<usize>, c: &Settings) {
        self.record(&idx);
        for &i in &idx {
            self.frames[i].sc.settings.copy_edit_from(&c);
            if c.wb_mode != WbMode::Custom {
                self.frames[i].apply_wb_mode();
            }
            self.mark_changed(i);
            if self.editor.path.as_ref() == Some(&self.frames[i].path) {
                self.editor
                    .set_committed(self.frames[i].sc.settings.clone());
            }
        }
        self.editor.invalidate();
        let n = idx.len();
        self.toast(format!("Pasted settings to {n} frame(s)"), false);
    }

    /// Rotate frames a quarter turn, as one undo step (clockwise when `cw`).
    pub fn rotate_frames(&mut self, idx: &[usize], cw: bool) {
        if idx.is_empty() {
            return;
        }
        self.record(idx);
        for &i in idx {
            let s = &mut self.frames[i].sc.settings;
            s.rotation = (s.rotation + if cw { 1 } else { 3 }) % 4;
            s.crop = super::editor::rotate_insets(s.crop, cw);
            self.mark_changed(i);
            if self.editor.path.as_ref() == Some(&self.frames[i].path) {
                self.editor
                    .set_committed(self.frames[i].sc.settings.clone());
            }
        }
        self.editor.invalidate();
    }
}

/// State of the Convert dialog shown before a folder-view conversion.
pub struct ConvertDialog {
    pub targets: Vec<usize>,
    pub as_roll: bool,
    /// Conversion options, seeded from the first selected frame.
    pub settings: Settings,
}

/// The conversion options of the folder Convert dialog.
pub fn conversion_fields(ui: &mut egui::Ui, s: &mut Settings, start: Option<&std::path::Path>) {
    widgets::fl_combo(
        ui,
        "Image type",
        &mut s.image_type,
        ImageType::ALL,
        ImageType::label,
        "",
    );
    if widgets::fl_combo(
        ui,
        "Scan source",
        &mut s.source,
        Source::ALL,
        Source::label,
        "",
    ) {
        s.film_base = match s.source {
            Source::TifScan | Source::LinearScan => FilmBase::Off,
            _ => FilmBase::Auto,
        };
    }
    ui.add(
        egui::Label::new(
            RichText::new(source_instructions(s.source))
                .small()
                .color(theme::FAINT),
        )
        .wrap(),
    );
    ui.add_space(6.0);
    widgets::fl_combo(
        ui,
        "Colour model",
        &mut s.color_model,
        ColorModel::ALL,
        ColorModel::label,
        "Basic: neutral · Frontier: teal-blues, golden yellows, warm skin · Noritsu: cleaner and cooler · B+W: monochrome · None: no calibration",
    );
    crosstalk_ui(ui, s);
    widgets::fl_combo(
        ui,
        "Film base",
        &mut s.film_base,
        FilmBase::ALL,
        FilmBase::label,
        "Neutralises the orange mask before analysis. Sampled uses each frame's own border sample.",
    );
    flat_field_ui(ui, &mut s.flat_field, start);
    ui.add_space(4.0);
    widgets::fl_slider_u8(
        ui,
        "Pre-saturation",
        &mut s.pre_saturation,
        1..=5,
        3,
        "1–2: fewer distinct hues, warmer · 4–5: more separation, cooler",
    );
    widgets::fl_slider(
        ui,
        "Border buffer",
        &mut s.border_buffer,
        0.0..=25.0,
        5.0,
        0,
        "Percent of the frame edge ignored by the analysis",
        None,
    );
    widgets::fl_combo(
        ui,
        "White balance",
        &mut s.wb_mode,
        WbMode::ALL,
        WbMode::label,
        "",
    );
    ui.add_space(6.0);
    widgets::checkbox(ui, &mut s.auto_crop, "Auto-crop frames").on_hover_text(
        "Find each uncropped frame inside the film rebate and holder, and straighten it",
    );
}

impl NlpApp {
    pub fn open_convert_dialog(&mut self, targets: Vec<usize>, as_roll: bool) {
        let Some(&first) = targets.first() else {
            return;
        };
        let settings = self
            .frames
            .get(first)
            .map_or_else(|| self.defaults.settings.clone(), |f| f.sc.settings.clone());
        self.convert_dialog = Some(ConvertDialog {
            as_roll: as_roll && targets.len() > 1,
            targets,
            settings,
        });
    }

    /// The Convert dialog for `targets`, balancing them as a roll by default
    /// only when they are the whole roll: a few frames make a poor roll.
    pub fn open_convert_for(&mut self, targets: Vec<usize>) {
        let whole = targets.len() == self.frames.len();
        self.open_convert_dialog(targets, whole);
    }

    pub fn convert_dialog_ui(&mut self, ctx: &egui::Context) {
        let Some(mut d) = self.convert_dialog.take() else {
            return;
        };
        let (mut go, mut cancel) = (false, false);
        let count = d.targets.len();
        let frame = egui::Frame::new()
            .fill(theme::PANEL)
            .stroke(egui::Stroke::new(1.0, theme::LINE))
            .corner_radius(12.0)
            .inner_margin(egui::Margin::same(22))
            .shadow(ctx.global_style().visuals.window_shadow);
        let r = egui::Modal::new(egui::Id::new("convert_modal"))
            .backdrop_color(Color32::from_black_alpha(150))
            .frame(frame)
            .show(ctx, |ui| {
                ui.set_width(420.0);
                let title = match (d.as_roll, count) {
                    (true, n) => format!("Convert {n} frames as a roll"),
                    (false, 1) => "Convert 1 frame".to_string(),
                    (false, n) => format!("Convert {n} frames"),
                };
                ui.label(RichText::new(title).font(theme::medium(16.0)).color(theme::TEXT));
                ui.add_space(16.0);
                let start = d
                    .targets
                    .first()
                    .and_then(|&i| self.frames[i].path.parent().map(|p| p.to_path_buf()));
                conversion_fields(ui, &mut d.settings, start.as_deref());
                if count > 1 {
                    ui.add_space(4.0);
                    widgets::checkbox(ui, &mut d.as_roll, "Balance as a roll").on_hover_text(
                        "Analyse the frames together for one consistent roll balance",
                    );
                }
                if d.as_roll {
                    widgets::fl_combo(
                        ui,
                        "Roll process",
                        &mut d.settings.roll_mode,
                        RollMode::ALL,
                        RollMode::label,
                        "Both use the film's channel slopes from the roll. Lab: each frame's own colour and density, like a lab scanner · Darkroom: the whole roll at one colour balance and one exposure",
                    );
                }
                ui.add_space(22.0);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(widgets::primary("Convert").min_size(vec2(96.0, 30.0))).clicked()
                        || (!ui.ctx().egui_wants_keyboard_input()
                            && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                    {
                        go = true;
                    }
                    if ui.add(widgets::ghost("Cancel").min_size(vec2(80.0, 30.0))).clicked()
                        || ui.input(|i| i.key_pressed(egui::Key::Escape))
                    {
                        cancel = true;
                    }
                });
            });
        if go {
            for &i in &d.targets {
                self.apply_conversion_options(i, &d.settings);
            }
            self.convert(d.targets, d.as_roll);
        } else if !(cancel || r.should_close()) {
            self.convert_dialog = Some(d);
        }
    }
}

impl NlpApp {
    /// Copies the Convert-dialog options into frame `i`'s settings.
    pub fn apply_conversion_options(&mut self, i: usize, o: &Settings) {
        let s = &mut self.frames[i].sc.settings;
        let (was_type, was_model) = (s.image_type, s.color_model);
        (s.image_type, s.source, s.color_model, s.film_base) =
            (o.image_type, o.source, o.color_model, o.film_base);
        s.flat_field = o.flat_field.clone();
        (s.crosstalk, s.crosstalk_strength) = (o.crosstalk.clone(), o.crosstalk_strength);
        (
            s.pre_saturation,
            s.border_buffer,
            s.wb_mode,
            s.roll_mode,
            s.auto_crop,
        ) = (
            o.pre_saturation,
            o.border_buffer,
            o.wb_mode,
            o.roll_mode,
            o.auto_crop,
        );
        if s.image_type != was_type && s.image_type != ImageType::Negative {
            s.apply_positive_defaults();
            s.wb_mode = o.wb_mode;
        }
        if s.color_model != was_model && s.is_monochrome() {
            s.apply_mono_defaults();
            s.wb_mode = o.wb_mode;
        }
        self.mark_changed(i);
    }
}

impl NlpApp {
    /// The plugin's "Large Exposure Variance Detected" prompt (`L22_1`).
    pub fn exposure_prompt_ui(&mut self, ctx: &egui::Context) {
        let Some((items, spread)) = self.exposure_prompt.take() else {
            return;
        };
        let (mut go, mut cancel) = (false, false);
        let frame = egui::Frame::new()
            .fill(theme::PANEL)
            .stroke(egui::Stroke::new(1.0, theme::LINE))
            .corner_radius(12.0)
            .inner_margin(egui::Margin::same(22))
            .shadow(ctx.global_style().visuals.window_shadow);
        let r = egui::Modal::new(egui::Id::new("exposure_variance_modal"))
            .backdrop_color(Color32::from_black_alpha(150))
            .frame(frame)
            .show(ctx, |ui| {
                ui.set_width(420.0);
                ui.label(
                    RichText::new("Large exposure variance detected")
                        .font(theme::medium(16.0))
                        .color(theme::TEXT),
                );
                ui.add_space(12.0);
                ui.add(
                    egui::Label::new(
                        RichText::new(format!(
                            "The exposure metadata of these frames differs by up to {spread:.1} EV. \
                             This happens when exposure wasn't fixed during digital capture, or when \
                             other tools overwrote the scans' EXIF exposure data.\n\n\
                             You can continue without adjusting for exposure, but the results may \
                             not be as accurate."
                        ))
                        .color(theme::MUTED),
                    )
                    .wrap(),
                );
                ui.add_space(22.0);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(widgets::primary("Continue without exposure data"))
                        .clicked()
                    {
                        go = true;
                    }
                    if ui
                        .add(widgets::ghost("Cancel").min_size(vec2(80.0, 30.0)))
                        .clicked()
                        || ui.input(|i| i.key_pressed(egui::Key::Escape))
                    {
                        cancel = true;
                    }
                });
            });
        if go {
            // The paused pass was counted done when it stopped: count this one in.
            self.convert_started(items.len());
            self.jobs
                .spawn(move || super::jobs::convert(items, true, false));
        } else if !(cancel || r.should_close()) {
            self.exposure_prompt = Some((items, spread));
        }
    }
}

/// A filter tab in the top bar: text that sits on a soft pill when active.
fn filter_tab(ui: &mut egui::Ui, on: bool, label: &str) -> egui::Response {
    let g = ui.painter().layout_no_wrap(
        label.to_string(),
        egui::FontId::proportional(12.5),
        if on { theme::TEXT } else { theme::MUTED },
    );
    let (rect, resp) = ui.allocate_exact_size(g.size() + vec2(20.0, 10.0), Sense::click());
    let p = ui.painter();
    if on {
        p.rect_filled(rect, rect.height() / 2.0, theme::SURFACE);
    } else if resp.hovered() {
        p.rect_filled(rect, rect.height() / 2.0, theme::HOVER.gamma_multiply(0.5));
    }
    let col = if on || resp.hovered() {
        theme::TEXT
    } else {
        theme::MUTED
    };
    p.galley_with_override_text_color(
        pos2(rect.left() + 10.0, rect.center().y - g.size().y / 2.0),
        g,
        col,
    );
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}
