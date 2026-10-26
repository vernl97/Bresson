//! Folder View: browse a folder of scans, convert (individually or as a roll)
//! and batch-edit / export the selection.

use eframe::egui::{self, Color32, RichText, Sense};

use super::widgets::{self, section};
use super::{NlpApp, ProxyState};
use crate::engine::settings::*;

pub fn source_instructions(s: Source) -> &'static str {
    match s {
        Source::DigitalCamera => {
            "DIGITAL CAMERA SCANS\n1. Film base is neutralised automatically (or sample the border in the Editor's Convert tab).\n2. Crop, or set the border buffer to exclude non-film areas.\n3. Click CONVERT."
        }
        Source::VueScanRaw => {
            "VUESCAN / SILVERFAST DNG\n1. Sample the film border (Editor › Convert › Sample film base) or use Auto.\n2. Exclude non-film areas with crop / border buffer.\n3. Click CONVERT."
        }
        Source::TifScan => {
            "TIF SCANS (gamma 1.8 / 2.2)\n1. Do NOT white balance TIF scans — film base is set to Off.\n2. Exclude non-film areas with crop / border buffer.\n3. Click CONVERT."
        }
        Source::LinearScan => {
            "LINEAR TIF SCANS (gamma 1.0)\n1. Run Tools › TIF Scan Prep first and convert the -prepped files.\n2. Do NOT white balance.\n3. Click CONVERT."
        }
    }
}

impl NlpApp {
    pub fn sel_vec(&self) -> Vec<usize> {
        self.selected.iter().copied().collect()
    }

    /// Start screen shown before anything is opened.
    fn welcome_ui(&mut self, ui: &mut egui::Ui) {
        use widgets::{Icon, icon_button, theme};
        egui::Panel::top("welcome_toolbar")
            .exact_size(54.0)
            .frame(
                egui::Frame::new()
                    .fill(theme::TOOLBAR)
                    .stroke(egui::Stroke::new(1.0, theme::LINE))
                    .inner_margin(egui::Margin::symmetric(12, 0)),
            )
            .show(ui, |ui| {
                let rect = ui.max_rect();
                let p = ui.painter();
                p.text(
                    egui::pos2(rect.center().x, rect.center().y - 8.0),
                    egui::Align2::CENTER_CENTER,
                    "Welcome to NLP Studio",
                    egui::FontId::proportional(15.0),
                    theme::TEXT,
                );
                p.text(
                    egui::pos2(rect.center().x, rect.center().y + 10.0),
                    egui::Align2::CENTER_CENTER,
                    "Select a photo or folder to begin.",
                    egui::FontId::proportional(12.5),
                    theme::MUTED,
                );
                ui.horizontal_centered(|ui| {
                    if icon_button(ui, Icon::Folder, "Open photo or folder…", false, true).clicked() {
                        self.choose_anything();
                    }
                });
            });
        let hovering = ui.ctx().input(|i| !i.raw.hovered_files.is_empty());
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(theme::SIDEBAR))
            .show(ui, |ui| {
                let avail = ui.max_rect();
                let resp = ui.allocate_rect(avail, Sense::click());
                let p = ui.painter();
                let zone = avail.shrink(40.0);
                if hovering {
                    p.rect_filled(zone, 16.0, Color32::from_rgb(226, 236, 252));
                    p.rect_stroke(
                        zone,
                        16.0,
                        egui::Stroke::new(2.0, theme::ACCENT),
                        egui::StrokeKind::Inside,
                    );
                } else if resp.hovered() {
                    p.rect_filled(zone, 16.0, Color32::from_rgb(240, 240, 242));
                }
                let text = if hovering {
                    "Drop to open"
                } else {
                    "Click to choose a photo or folder, or drag here"
                };
                p.text(
                    avail.center(),
                    egui::Align2::CENTER_CENTER,
                    text,
                    egui::FontId::proportional(16.0),
                    if hovering {
                        theme::ACCENT
                    } else {
                        theme::MUTED
                    },
                );
                if resp.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                if resp.clicked() {
                    self.choose_anything();
                }
            });
    }

    pub fn folder_ui(&mut self, ui: &mut egui::Ui) {
        if self.frames.is_empty() {
            self.welcome_ui(ui);
            return;
        }
        let ctx = ui.ctx().clone();
        self.folder_shortcuts(&ctx);
        egui::Panel::top("folder_top").show(ui, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.heading("NLP Studio");
                ui.separator();
                if ui.button("📂 Open Folder…").clicked() {
                    self.choose_folder();
                }
                if ui
                    .button("Open Image…")
                    .on_hover_text("Open one or more pictures without the rest of the folder")
                    .clicked()
                {
                    self.choose_files();
                }
                if let Some(f) = &self.folder {
                    ui.label(RichText::new(f.display().to_string()).weak());
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let can_open = self.selected.first().is_some();
                    if ui
                        .add_enabled(can_open, egui::Button::new("Open in Editor ▶"))
                        .on_hover_text("Enter / double-click")
                        .clicked()
                    {
                        if let Some(&i) = self.selected.first() {
                            self.open_editor(i);
                        }
                    }
                    ui.add(
                        egui::Slider::new(&mut self.thumb_size, 110.0..=340.0).show_value(false),
                    )
                    .on_hover_text("Thumbnail size");
                    ui.label("🔍");
                    if self.jobs.busy() > 0 {
                        ui.add(egui::Spinner::new());
                        ui.label(RichText::new(format!("{} task(s)", self.jobs.busy())).weak());
                    }
                });
            });
            ui.add_space(4.0);
        });

        egui::Panel::bottom("folder_status").show(ui, |ui| {
            ui.horizontal(|ui| {
                let conv = self.frames.iter().filter(|f| f.sc.is_converted()).count();
                ui.label(format!(
                    "{} frames · {} converted · {} selected",
                    self.frames.len(),
                    conv,
                    self.selected.len()
                ));
                ui.separator();
                if ui.small_button("Select all").clicked() {
                    self.selected = (0..self.frames.len()).collect();
                }
                if ui.small_button("Select none").clicked() {
                    self.selected.clear();
                }
                let has_sel = !self.selected.is_empty();
                if ui.add_enabled(has_sel, egui::Button::new("Export…").small()).clicked() {
                    self.open_export_dialog(self.sel_vec());
                }
                if self.export_total > 0 && self.export_done < self.export_total {
                    ui.separator();
                    ui.add(
                        egui::ProgressBar::new(self.export_done as f32 / self.export_total as f32)
                            .desired_width(180.0)
                            .text(format!(
                                "Exporting {}/{}",
                                self.export_done, self.export_total
                            )),
                    );
                }
            });
        });

        egui::CentralPanel::default().show(ui, |ui| self.grid(ui));
    }

    fn folder_shortcuts(&mut self, ctx: &egui::Context) {
        if ctx.egui_wants_keyboard_input() {
            return;
        }
        let (all, enter, copy, paste) = ctx.input(|i| {
            (
                i.modifiers.command && i.key_pressed(egui::Key::A),
                i.key_pressed(egui::Key::Enter),
                i.modifiers.command && i.key_pressed(egui::Key::C),
                i.modifiers.command && i.key_pressed(egui::Key::V),
            )
        });
        if all {
            self.selected = (0..self.frames.len()).collect();
        }
        if enter {
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
            let was_roll = self.frames[i].is_roll();
            self.frames[i].sc.settings.copy_edit_from(&c);
            self.frames[i].sc.settings.use_roll = was_roll;
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
        if self.frames.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(ui.available_height() * 0.35);
                    ui.heading("Open a folder of negative scans");
                    ui.label(
                        RichText::new(
                            "Camera RAW (DNG, CR2/CR3, NEF, ARW, RAF …), TIFF, JPEG and PNG",
                        )
                        .weak(),
                    );
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button("📂 Open Folder…").clicked() {
                            self.choose_folder();
                        }
                        if ui.button("Open Image…").clicked() {
                            self.choose_files();
                        }
                    });
                });
            });
            return;
        }
        let cell = self.thumb_size;
        let mut clicked: Option<(usize, egui::Modifiers)> = None;
        let mut double: Option<usize> = None;
        let mut ctx_action: Option<(usize, &'static str)> = None;
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(10.0, 10.0);
                    for (i, f) in self.frames.iter().enumerate() {
                        let sel = self.selected.contains(&i);
                        let (rect, resp) =
                            ui.allocate_exact_size(egui::vec2(cell, cell + 22.0), Sense::click());
                        if !ui.is_rect_visible(rect) {
                            continue;
                        }
                        let p = ui.painter();
                        let bg = if sel {
                            Color32::from_rgb(206, 224, 252)
                        } else if resp.hovered() {
                            Color32::from_rgb(228, 228, 232)
                        } else {
                            Color32::from_rgb(238, 238, 240)
                        };
                        p.rect_filled(rect, 8.0, bg);
                        if sel {
                            p.rect_stroke(
                                rect,
                                8.0,
                                egui::Stroke::new(1.5, widgets::theme::ACCENT),
                                egui::StrokeKind::Inside,
                            );
                        }
                        let img_rect = egui::Rect::from_min_size(
                            rect.min + egui::vec2(6.0, 6.0),
                            egui::vec2(cell - 12.0, cell - 12.0),
                        );
                        match (&f.thumb, &f.proxy) {
                            (Some(t), _) => {
                                let ts = t.size_vec2();
                                let s = (img_rect.width() / ts.x).min(img_rect.height() / ts.y);
                                let r = egui::Rect::from_center_size(img_rect.center(), ts * s);
                                p.image(
                                    t.id(),
                                    r,
                                    egui::Rect::from_min_max(
                                        egui::pos2(0.0, 0.0),
                                        egui::pos2(1.0, 1.0),
                                    ),
                                    Color32::WHITE,
                                );
                            }
                            (None, ProxyState::Failed(e)) => {
                                resp.clone().on_hover_text(e);
                                p.text(
                                    img_rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    "cannot read",
                                    egui::FontId::proportional(13.0),
                                    Color32::from_rgb(200, 50, 40),
                                );
                            }
                            _ => {
                                p.text(
                                    img_rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    "loading…",
                                    egui::FontId::proportional(13.0),
                                    Color32::GRAY,
                                );
                            }
                        }
                        let name_pos = egui::pos2(rect.left() + 8.0, rect.bottom() - 13.0);
                        p.text(
                            name_pos,
                            egui::Align2::LEFT_CENTER,
                            &f.name,
                            egui::FontId::proportional(12.0),
                            ui.visuals().text_color(),
                        );
                        let mut badges = Vec::new();
                        if f.sc.is_converted() {
                            badges.push(("✔", widgets::theme::OK));
                        }
                        if f.is_roll() {
                            badges.push(("ROLL", widgets::theme::ACCENT));
                        }
                        if f.exporting {
                            badges.push(("EXPORT", widgets::theme::WARN));
                        }
                        let mut x = rect.right() - 8.0;
                        for (b, c) in badges {
                            let g = p.text(
                                egui::pos2(x, rect.bottom() - 13.0),
                                egui::Align2::RIGHT_CENTER,
                                b,
                                egui::FontId::proportional(11.0),
                                c,
                            );
                            x = g.left() - 6.0;
                        }
                        if resp.double_clicked() {
                            double = Some(i);
                        } else if resp.clicked() {
                            clicked = Some((i, ui.input(|inp| inp.modifiers)));
                        }
                        resp.context_menu(|ui| {
                            if ui.button("Open in Editor").clicked() {
                                ctx_action = Some((i, "open"));
                                ui.close();
                            }
                            if ui.button("Convert").clicked() {
                                ctx_action = Some((i, "convert"));
                                ui.close();
                            }
                            if ui.button("Export positive copies").clicked() {
                                ctx_action = Some((i, "export"));
                                ui.close();
                            }
                            ui.separator();
                            if ui.button("Copy settings").clicked() {
                                ctx_action = Some((i, "copy"));
                                ui.close();
                            }
                            if ui
                                .add_enabled(
                                    self.clipboard.is_some(),
                                    egui::Button::new("Paste settings"),
                                )
                                .clicked()
                            {
                                ctx_action = Some((i, "paste"));
                                ui.close();
                            }
                            if ui.button("Unconvert").clicked() {
                                ctx_action = Some((i, "unconvert"));
                                ui.close();
                            }
                        });
                    }
                });
            });
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
                "open" => self.open_editor(i),
                "convert" => self.convert(self.sel_vec(), false),
                "roll" => self.convert(self.sel_vec(), true),
                "export" => self.open_export_dialog(self.sel_vec()),
                "copy" => self.copy_settings(),
                "paste" => self.paste_settings(),
                "unconvert" => self.unconvert(&self.sel_vec()),
                _ => {}
            }
        }
    }
}

pub fn lut_folder_ui(app: &mut NlpApp, ui: &mut egui::Ui) {
    use crate::engine::lut::LutLibrary;
    section(ui, "ENHANCED PROFILE LUTS", |ui| {
        let dir = LutLibrary::global()
            .read()
            .ok()
            .and_then(|l| l.dir().map(|d| d.to_path_buf()));
        match &dir {
            Some(d) => {
                ui.label(
                    RichText::new("✔ Using the plugin's 3D LUTs from:")
                        .small()
                        .color(widgets::theme::OK),
                );
                ui.label(RichText::new(d.display().to_string()).small().weak());
            }
            None => {
                ui.label(RichText::new("⚠ 'NLP Enhanced Settings' folder not found — using built-in approximations of the looks.").small().color(widgets::theme::WARN));
            }
        }
        if ui
            .button("Choose LUT folder…")
            .on_hover_text("The plugin's LR Files/CameraRaw/Settings/NLP Enhanced Settings folder")
            .clicked()
        {
            if let Some(p) = super::picker::pick_folder(None, Some("Choose")) {
                let r = LutLibrary::global().write().map(|mut l| l.set_dir(p));
                match r {
                    Ok(Ok(())) => {
                        app.toast("LUT folder set", false);
                        for f in app.frames.iter_mut() {
                            f.thumb_dirty = true;
                        }
                        app.editor.invalidate();
                    }
                    Ok(Err(e)) => app.toast(format!("{e:#}"), true),
                    Err(_) => {}
                }
            }
        }
    });
}
