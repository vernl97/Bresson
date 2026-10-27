//! Folder View: browse a folder of scans, convert (individually or as a roll)
//! and batch-edit / export the selection.

use eframe::egui::{self, Color32, RichText, Sense, pos2, vec2};

use super::widgets::{self, Icon, icon_button, theme};
use super::{NlpApp, ProxyState, View};
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
}

impl NlpApp {
    pub fn sel_vec(&self) -> Vec<usize> {
        self.selected.iter().copied().collect()
    }

    /// Save and close the open folder, then show the Projects (rolls) list.
    fn close_to_projects(&mut self) {
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
        let hovering = ui.ctx().input(|i| !i.raw.hovered_files.is_empty());
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(theme::BG))
            .show(ui, |ui| {
                let avail = ui.max_rect();
                let zone_w = (avail.width() - 64.0).clamp(280.0, 560.0);
                let zone_h = (avail.height() * 0.36).clamp(180.0, 280.0);
                let content_h = 64.0 + 28.0 + zone_h + 18.0 + 44.0;
                ui.add_space(((avail.height() - content_h) * 0.45).max(24.0));
                ui.vertical_centered(|ui| {
                    ui.label(
                        RichText::new("NLP Studio")
                            .font(theme::medium(26.0))
                            .color(theme::TEXT),
                    );
                    ui.label(RichText::new("Film negative conversion").color(theme::MUTED));
                    ui.add_space(28.0);

                    let (zone, resp) = ui.allocate_exact_size(vec2(zone_w, zone_h), Sense::click());
                    let p = ui.painter();
                    let (fill, stroke, fg) = if hovering {
                        (
                            theme::ACCENT.gamma_multiply(0.12),
                            egui::Stroke::new(1.5, theme::ACCENT),
                            theme::ACCENT,
                        )
                    } else if resp.hovered() {
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
                        if hovering {
                            "Drop to open"
                        } else {
                            "Drop a folder or scans here"
                        },
                        theme::medium(15.0),
                        if hovering { theme::ACCENT } else { theme::TEXT },
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

                    ui.add_space(18.0);
                    ui.label(
                        RichText::new("Camera RAW · TIFF · JPEG · PNG")
                            .small()
                            .color(theme::FAINT),
                    );
                });
            });
    }

    pub fn folder_ui(&mut self, ui: &mut egui::Ui) {
        if self.frames.is_empty() {
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
        let export_key = widgets::shortcut(&ctx, cmd, egui::Key::E);
        let select_key = widgets::shortcut(&ctx, cmd, egui::Key::A);
        let paste_key = widgets::shortcut(&ctx, cmd, egui::Key::V);
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
                    // Breadcrumb: Rolls / <roll name>.
                    let back = icon_button(ui, Icon::Back, "Back to all rolls", false, true);
                    let rolls = ui
                        .add(
                            egui::Label::new(RichText::new("Rolls").color(theme::MUTED))
                                .sense(Sense::click()),
                        )
                        .on_hover_cursor(egui::CursorIcon::PointingHand);
                    if back.clicked() || rolls.clicked() {
                        self.close_to_projects();
                    }
                    if let Some(f) = self.folder.clone() {
                        ui.label(RichText::new("/").color(theme::FAINT));
                        let name = f
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| f.display().to_string());
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
                    }
                    ui.add_space(2.0);
                    let r = icon_button(ui, Icon::More, "More", false, true);
                    egui::Popup::menu(&r).show(|ui| {
                        let key = |k: &str| RichText::new(k.to_owned()).color(theme::FAINT);
                        if ui.button("Open folder…").clicked() {
                            self.choose_folder();
                        }
                        if ui.button("Open images…").clicked() {
                            self.choose_files();
                        }
                        ui.separator();
                        if !self.single_mode {
                            if let Some(f) = self.folder.clone() {
                                if ui.button("Roll info…").clicked() {
                                    self.open_roll_dialog(&f, false);
                                }
                            }
                        }
                        if ui
                            .button("Contact sheet")
                            .on_hover_text("Save a JPEG contact sheet in the NLP folder (selection, or all frames)")
                            .clicked()
                        {
                            self.make_contact_sheet(self.sel_vec());
                        }
                        if ui
                            .add(egui::Button::new("Select all").shortcut_text(key(&select_key)))
                            .clicked()
                        {
                            self.selected = (0..self.frames.len()).collect();
                        }
                        let paste = egui::Button::new("Paste settings").shortcut_text(key(&paste_key));
                        if ui
                            .add_enabled(self.clipboard.is_some() && !self.selected.is_empty(), paste)
                            .clicked()
                        {
                            self.paste_settings();
                        }
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let n = self.selected.len();
                        if n > 0 {
                            if ui
                                .add(widgets::primary("Export"))
                                .on_hover_text(format!("Export positive copies ({export_key})"))
                                .clicked()
                            {
                                self.open_export_dialog(self.sel_vec());
                            }
                            // Split Convert button: main action + ▾ for roll mode.
                            ui.spacing_mut().item_spacing.x = 1.0;
                            let more = ui
                                .add(widgets::ghost("").min_size(vec2(22.0, 28.0)))
                                .on_hover_text("Convert options");
                            let c = more.rect.center();
                            ui.painter().line(
                                vec![pos2(c.x - 4.0, c.y - 2.0), pos2(c.x, c.y + 2.0), pos2(c.x + 4.0, c.y - 2.0)],
                                egui::Stroke::new(1.5, theme::MUTED),
                            );
                            egui::Popup::menu(&more).show(|ui| {
                                if ui.button("Convert each frame").clicked() {
                                    self.open_convert_dialog(self.sel_vec(), false);
                                }
                                if ui
                                    .add_enabled(n >= 2, egui::Button::new("Convert as roll"))
                                    .on_hover_text(
                                        "Analyse the selection together for one consistent roll balance",
                                    )
                                    .on_disabled_hover_text("Select two or more frames")
                                    .clicked()
                                {
                                    self.open_convert_dialog(self.sel_vec(), true);
                                }
                            });
                            if ui
                                .add(widgets::ghost("Convert"))
                                .on_hover_text("Analyse each selected frame on its own")
                                .clicked()
                            {
                                self.open_convert_dialog(self.sel_vec(), false);
                            }
                            ui.spacing_mut().item_spacing.x = 6.0;
                            ui.add_space(6.0);
                            ui.separator();
                            ui.add_space(6.0);
                            if ui
                                .add(
                                    egui::Button::new(RichText::new("Clear").small().color(theme::MUTED))
                                        .frame(false),
                                )
                                .on_hover_text("Clear selection (Esc)")
                                .clicked()
                            {
                                self.selected.clear();
                            }
                            ui.label(
                                RichText::new(format!("{n} selected")).small().color(theme::MUTED),
                            ).on_hover_text(super::LAB_KEYS_HELP);
                        }
                        // Fixed slot so the buttons don't shift when jobs start/finish.
                        ui.add_space(4.0);
                        let (slot, _) = ui.allocate_exact_size(vec2(16.0, 16.0), Sense::hover());
                        if self.jobs.busy() > 0 {
                            ui.put(slot, egui::Spinner::new().size(14.0).color(theme::MUTED));
                        }
                    });
                });
            });

        egui::Panel::bottom("folder_status")
            .exact_size(28.0)
            .frame(
                egui::Frame::new()
                    .fill(theme::PANEL)
                    .inner_margin(egui::Margin::symmetric(14, 0)),
            )
            .show(ui, |ui| {
                ui.horizontal_centered(|ui| {
                    let conv = self.frames.iter().filter(|f| f.sc.is_converted()).count();
                    let status = format!("{} frames · {} converted", self.frames.len(), conv);
                    ui.label(RichText::new(status).small().color(theme::MUTED));
                    if self.selected.is_empty() {
                        ui.label(
                            RichText::new("· Click to select · double-click to edit")
                                .small()
                                .color(theme::FAINT),
                        );
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.spacing_mut().item_spacing.x = 6.0;
                        ui.spacing_mut().slider_width = 90.0;
                        let tiny =
                            |t: &str| RichText::new(t.to_owned()).small().color(theme::FAINT);
                        ui.label(tiny("Large"));
                        ui.add(
                            egui::Slider::new(&mut self.thumb_size, 110.0..=340.0)
                                .show_value(false),
                        )
                        .on_hover_text("Thumbnail size");
                        ui.label(tiny("Small"));
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

        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(theme::BG)
                    .inner_margin(egui::Margin::same(16)),
            )
            .show(ui, |ui| self.grid(ui));
    }

    fn folder_shortcuts(&mut self, ctx: &egui::Context) {
        if ctx.egui_wants_keyboard_input() {
            return;
        }
        let (all, none, enter, copy, paste, export) = ctx.input(|i| {
            (
                i.modifiers.command && i.key_pressed(egui::Key::A),
                i.key_pressed(egui::Key::Escape),
                i.key_pressed(egui::Key::Enter),
                i.modifiers.command && i.key_pressed(egui::Key::C),
                i.modifiers.command && i.key_pressed(egui::Key::V),
                i.modifiers.command && i.key_pressed(egui::Key::E),
            )
        });
        if all {
            self.selected = (0..self.frames.len()).collect();
        }
        if none && self.export_targets.is_none() {
            self.selected.clear();
        }
        if enter && self.export_targets.is_none() {
            if let Some(&i) = self.selected.first() {
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
            self.open_export_dialog(self.sel_vec());
        }
        if let Some(key) = super::lab_key(ctx).filter(|_| self.export_targets.is_none()) {
            let sel = self.sel_vec();
            let n = sel.iter().filter(|&&i| self.apply_lab_key(i, key)).count();
            if n < sel.len() {
                self.toast("Only converted frames can be corrected", false);
            }
        }
    }

    /// Crop and straighten each frame to its detected frame.
    pub fn detect_frames(&mut self, idx: &[usize]) {
        let mut missed = 0;
        for &i in idx {
            let f = &self.frames[i];
            let Some(src) = f.proxy() else {
                missed += 1;
                continue;
            };
            match crate::engine::frame::detect(&src, &f.sc.settings, f.base_for(&src)) {
                Some(d) => {
                    let s = &mut self.frames[i].sc.settings;
                    (s.crop, s.straighten) = (d.crop, d.straighten);
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
        let Some(c) = self.clipboard.clone() else {
            return;
        };
        for i in self.sel_vec() {
            self.frames[i].sc.settings.copy_edit_from(&c);
            if c.wb_mode != WbMode::Custom {
                self.frames[i].apply_wb_mode();
            }
            self.mark_changed(i);
        }
        self.editor.invalidate();
        let n = self.selected.len();
        self.toast(format!("Pasted settings to {n} frame(s)"), false);
    }

    fn grid(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        let cmd = egui::Modifiers::COMMAND;
        let keys = [
            widgets::shortcut(&ctx, egui::Modifiers::NONE, egui::Key::Enter),
            widgets::shortcut(&ctx, cmd, egui::Key::E),
            widgets::shortcut(&ctx, cmd, egui::Key::C),
            widgets::shortcut(&ctx, cmd, egui::Key::V),
        ];
        let cell = self.thumb_size;
        let name_h = 22.0;
        let now = ui.input(|i| i.time);
        let mut clicked: Option<(usize, egui::Modifiers)> = None;
        let mut double: Option<usize> = None;
        let mut ctx_action: Option<(usize, CellAction)> = None;
        let mut bg_click = false;
        let multi = self.selected.len() >= 2;
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = vec2(14.0, 14.0);
                    for (i, f) in self.frames.iter().enumerate() {
                        let sel = self.selected.contains(&i);
                        let (rect, resp) =
                            ui.allocate_exact_size(vec2(cell, cell + name_h), Sense::click());
                        if !ui.is_rect_visible(rect) {
                            continue;
                        }
                        let p = ui.painter();
                        let square = egui::Rect::from_min_size(rect.min, vec2(cell, cell));
                        if resp.hovered() && !sel {
                            p.rect_filled(square, 6.0, Color32::from_white_alpha(5));
                        }
                        let img_area = square.shrink(6.0);
                        let mut frame_rect = img_area;
                        match (&f.thumb, &f.proxy) {
                            (Some(t), _) => {
                                let ts = t.size_vec2();
                                let s = (img_area.width() / ts.x).min(img_area.height() / ts.y);
                                // Bottom-aligned so the name sits right under the image.
                                let size = ts * s;
                                frame_rect = egui::Rect::from_min_size(
                                    pos2(
                                        img_area.center().x - size.x / 2.0,
                                        img_area.bottom() - size.y,
                                    ),
                                    size,
                                );
                                egui::Image::new((t.id(), ts))
                                    .corner_radius(3.0)
                                    .paint_at(ui, frame_rect);
                            }
                            (None, ProxyState::Failed(e)) => {
                                resp.clone().on_hover_text(e);
                                p.rect_filled(img_area, 3.0, theme::PANEL);
                                widgets::paint_icon(
                                    p,
                                    egui::Rect::from_center_size(
                                        img_area.center(),
                                        vec2(24.0, 24.0),
                                    ),
                                    Icon::Warn,
                                    theme::ERR,
                                );
                            }
                            _ => {
                                // Loading shimmer.
                                let a = (((now * 2.4 + i as f64 * 0.35).sin() * 0.5 + 0.5) * 10.0)
                                    as u8
                                    + 6;
                                p.rect_filled(img_area, 3.0, Color32::from_white_alpha(a));
                                ui.ctx().request_repaint();
                            }
                        }
                        if sel {
                            p.rect_stroke(
                                frame_rect.expand(3.0),
                                5.0,
                                egui::Stroke::new(2.0, theme::ACCENT),
                                egui::StrokeKind::Outside,
                            );
                        }

                        // Name row with status markers on the right.
                        let row_y = rect.bottom() - name_h / 2.0 + 1.0;
                        let mut x = rect.right() - 8.0;
                        if f.exporting {
                            let a = ((now * 4.0).sin() * 0.35 + 0.65) as f32;
                            p.circle_filled(
                                pos2(x - 3.0, row_y),
                                3.0,
                                theme::WARN.gamma_multiply(a),
                            );
                            x -= 12.0;
                            ui.ctx().request_repaint();
                        }
                        if f.is_roll() {
                            let g = p.layout_no_wrap(
                                "Roll".into(),
                                egui::FontId::proportional(10.0),
                                theme::MUTED,
                            );
                            let chip = egui::Rect::from_min_max(
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
                        if f.sc.is_converted() {
                            p.circle_filled(pos2(x - 3.0, row_y), 3.0, theme::OK);
                            x -= 12.0;
                            if let Some(r) = lab::readout(&f.sc.settings) {
                                let g =
                                    p.layout_no_wrap(r, egui::FontId::monospace(9.5), theme::MUTED);
                                let sz = g.size();
                                let r = egui::Rect::from_min_size(
                                    pos2(x - sz.x, row_y - sz.y / 2.0),
                                    sz,
                                );
                                ui.interact(r, ui.id().with(("lab_readout", i)), Sense::hover())
                                    .on_hover_text(super::LAB_KEYS_HELP);
                                p.galley(r.min, g, theme::MUTED);
                                x -= sz.x + 6.0;
                            }
                        }
                        let name_x = rect.left() + 6.0;
                        let mut job = egui::text::LayoutJob::simple_singleline(
                            f.name.clone(),
                            egui::FontId::proportional(11.5),
                            if sel { theme::TEXT } else { theme::MUTED },
                        );
                        job.wrap = egui::text::TextWrapping::truncate_at_width(
                            (x - name_x - 4.0).max(10.0),
                        );
                        let g = p.layout_job(job);
                        p.galley(pos2(name_x, row_y - g.size().y / 2.0), g, theme::MUTED);

                        if resp.double_clicked() {
                            double = Some(i);
                        } else if resp.clicked() {
                            clicked = Some((i, ui.input(|inp| inp.modifiers)));
                        }
                        let has_clip = self.clipboard.is_some();
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
                            item(ui, "Convert", "", CellAction::Convert, true);
                            if multi && sel {
                                item(ui, "Convert as roll", "", CellAction::ConvertRoll, true);
                            }
                            item(ui, "Export…", &keys[1], CellAction::Export, true);
                            ui.separator();
                            item(ui, "Detect frame", "", CellAction::DetectFrame, true);
                            ui.separator();
                            item(ui, "Copy settings", &keys[2], CellAction::Copy, true);
                            item(ui, "Paste settings", &keys[3], CellAction::Paste, has_clip);
                            ui.separator();
                            item(ui, "Unconvert", "", CellAction::Unconvert, true);
                        });
                    }
                });
                // Clicking empty space below the grid clears the selection.
                let rest = ui.available_rect_before_wrap();
                if rest.height() > 0.0 && ui.allocate_rect(rest, Sense::click()).clicked() {
                    bg_click = true;
                }
            });
        if bg_click && clicked.is_none() {
            self.selected.clear();
        }
        if let Some((i, m)) = clicked {
            if m.shift {
                let a = self.anchor.unwrap_or(i);
                let (lo, hi) = (a.min(i), a.max(i));
                if !m.command {
                    self.selected.clear();
                }
                self.selected.extend(lo..=hi);
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
            }
        }
    }
}

/// State of the Convert dialog shown before a folder-view conversion.
pub struct ConvertDialog {
    pub targets: Vec<usize>,
    pub as_roll: bool,
    /// Conversion options, seeded from the first selected frame.
    pub settings: Settings,
}

/// The conversion options shared by the folder Convert dialog and the editor wizard.
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
    ui.checkbox(&mut s.auto_crop, "Auto-crop frames")
        .on_hover_text(
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
                    ui.add_space(6.0);
                    ui.checkbox(&mut d.as_roll, "Convert as roll")
                        .on_hover_text("Analyse the selection together for one consistent roll balance");
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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WizardStep {
    Crop,
    Settings,
    Convert,
}

/// The guided first conversion of an unconverted frame.
pub struct ConvertWizard {
    pub frame: usize,
    pub step: WizardStep,
    pub settings: Settings,
}

#[derive(Default)]
pub struct WizardState {
    pub active: Option<ConvertWizard>,
    /// Frames the wizard was already shown for this session.
    pub seen: std::collections::HashSet<std::path::PathBuf>,
}

impl NlpApp {
    /// Opens the wizard for an unconverted frame that hasn't had it yet.
    pub fn start_wizard(&mut self, i: usize) {
        let f = &self.frames[i];
        if f.sc.is_converted() || self.wizard.seen.contains(&f.path) {
            return;
        }
        self.begin_wizard(i);
    }

    /// Opens the wizard at the crop step, whether or not it was seen before.
    pub fn begin_wizard(&mut self, i: usize) {
        let path = self.frames[i].path.clone();
        self.wizard.seen.insert(path);
        let settings = self.frames[i].sc.settings.clone();
        self.enter_crop(i);
        self.wizard.active = Some(ConvertWizard {
            frame: i,
            step: WizardStep::Crop,
            settings,
        });
    }

    pub fn wizard_ui(&mut self, ctx: &egui::Context) {
        let Some(mut w) = self.wizard.active.take() else {
            return;
        };
        // Leaving the editor or switching frames abandons the wizard.
        if self.view != View::Editor || self.active != Some(w.frame) {
            return;
        }
        let mut go = false;
        let mut cancel = ctx.input(|i| i.key_pressed(egui::Key::Escape)) && w.step != WizardStep::Crop;
        match w.step {
            WizardStep::Crop => {
                // The editor's own crop tool runs; Done or Cancel there moves on.
                if self.editor.crop.is_none() {
                    w.step = WizardStep::Settings;
                    w.settings = self.frames[w.frame].sc.settings.clone();
                } else {
                    egui::Area::new(egui::Id::new("wizard_hint"))
                        .anchor(egui::Align2::CENTER_TOP, vec2(0.0, 52.0))
                        .show(ctx, |ui| {
                            egui::Frame::new()
                                .fill(theme::PANEL)
                                .stroke(egui::Stroke::new(1.0, theme::LINE))
                                .corner_radius(10.0)
                                .inner_margin(egui::Margin::symmetric(14, 8))
                                .show(ui, |ui| {
                                    ui.label(
                                        RichText::new("Step 1 of 3 · Crop the frame, then press Done")
                                            .color(theme::TEXT),
                                    );
                                });
                        });
                }
            }
            WizardStep::Settings | WizardStep::Convert => {
                let frame = egui::Frame::new()
                    .fill(theme::PANEL)
                    .stroke(egui::Stroke::new(1.0, theme::LINE))
                    .corner_radius(12.0)
                    .inner_margin(egui::Margin::same(22))
                    .shadow(ctx.global_style().visuals.window_shadow);
                let settings_step = w.step == WizardStep::Settings;
                let mut back = false;
                let r = egui::Modal::new(egui::Id::new("wizard_modal"))
                    .backdrop_color(Color32::from_black_alpha(150))
                    .frame(frame)
                    .show(ctx, |ui| {
                        ui.set_width(420.0);
                        let title = if settings_step {
                            "Step 2 of 3 · Conversion settings"
                        } else {
                            "Step 3 of 3 · Convert"
                        };
                        ui.label(RichText::new(title).font(theme::medium(16.0)).color(theme::TEXT));
                        ui.add_space(16.0);
                        if settings_step {
                            let start = self.frames[w.frame].path.parent().map(|p| p.to_path_buf());
                            conversion_fields(ui, &mut w.settings, start.as_deref());
                        } else {
                            let s = &w.settings;
                            for line in [
                                format!("Type: {}", s.image_type.label()),
                                format!("Source: {}", s.source.label()),
                                format!("Colour model: {}", s.color_model.label()),
                                format!("Film base: {}", s.film_base.label()),
                                format!("White balance: {}", s.wb_mode.label()),
                            ] {
                                ui.label(RichText::new(line).color(theme::TEXT));
                            }
                        }
                        ui.add_space(22.0);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let label = if settings_step { "Next" } else { "Convert" };
                            if ui.add(widgets::primary(label).min_size(vec2(96.0, 30.0))).clicked()
                                || (!ui.ctx().egui_wants_keyboard_input()
                                    && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                            {
                                go = true;
                            }
                            if ui.add(widgets::ghost("Back").min_size(vec2(80.0, 30.0))).clicked() {
                                back = true;
                            }
                            if ui.add(widgets::ghost("Cancel").min_size(vec2(80.0, 30.0))).clicked() {
                                cancel = true;
                            }
                        });
                    });
                if r.should_close() {
                    cancel = true;
                }
                if back {
                    if settings_step {
                        self.enter_crop(w.frame);
                        w.step = WizardStep::Crop;
                    } else {
                        w.step = WizardStep::Settings;
                    }
                } else if go {
                    if settings_step {
                        w.step = WizardStep::Convert;
                    } else {
                        let i = w.frame;
                        self.apply_conversion_options(i, &w.settings);
                        self.convert(vec![i], false);
                        return;
                    }
                }
            }
        }
        if !cancel {
            self.wizard.active = Some(w);
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
            self.jobs
                .spawn(move || super::jobs::convert(items, true, false));
        } else if !(cancel || r.should_close()) {
            self.exposure_prompt = Some((items, spread));
        }
    }
}
