//! The Scan panel: pick a scanner, prescan, frame the crop and scan into the
//! open roll. The scanner itself runs on the worker in `scanner.rs`.

use eframe::egui::{self, RichText, Sense, pos2, vec2};
use opticfilm::scan::bringup;
use opticfilm::{FilmModel, ScanRequest};

use super::scanner::{self, ScanCmd, ScanEvent, ScanWorker, ScannerChoice, ScannerInfo};
use super::widgets::{self, theme};
use super::NlpApp;

#[derive(Default)]
pub struct ScanState {
    pub open: bool,
    worker: Option<ScanWorker>,
    /// Connected scanners: (device id, label), from the last refresh.
    devices: Option<Vec<(String, String)>>,
    pub info: Option<ScannerInfo>,
    /// A command is running on the worker.
    pub busy: bool,
    pub status: String,
    pub progress: Option<f64>,
    prescan: Option<egui::TextureHandle>,
    /// Crop on the prescan, normalised `[x1, y1, x2, y2]`.
    crop: [f32; 4],
    /// Corner where the current crop drag started.
    drag_from: Option<egui::Pos2>,
    pub error: Option<String>,
    /// Frames saved this session, for the status line.
    pub saved: usize,
}

/// Multi-exposure needs the model's long-exposure envelope (GL128 has its own).
fn supports_me(m: &FilmModel) -> bool {
    bringup::is_gl128_opticfilm(m) || m.me_long_clamp_max.is_some()
}

fn label_for(choice: &ScannerChoice, devices: &[(String, String)]) -> String {
    match choice {
        ScannerChoice::Device(id) => devices.iter().find(|d| &d.0 == id).map_or_else(|| id.clone(), |d| d.1.clone()),
        ScannerChoice::Simulated(name) => opticfilm::model_by_name(name).map_or_else(|| name.clone(), |m| format!("Simulated {}", m.model)),
    }
}

impl NlpApp {
    pub fn toggle_scan_panel(&mut self) {
        if self.scan.open {
            self.close_scan_panel();
            return;
        }
        self.scan.open = true;
        if self.scan.devices.is_none() {
            self.scan.devices = Some(scanner::list_scanners());
        }
    }

    /// Hide the panel and release the scanner: a running scan is cancelled,
    /// the lamp goes off.
    pub fn close_scan_panel(&mut self) {
        self.scan.open = false;
        self.scan.worker = None;
        self.scan.info = None;
        self.scan.prescan = None;
        self.scan.busy = false;
        self.scan.progress = None;
        self.scan.status.clear();
    }

    fn scan_send(&mut self, cmd: ScanCmd) {
        let worker = self
            .scan
            .worker
            .get_or_insert_with(|| ScanWorker::spawn(self.jobs.sender(), self.jobs.ctx().clone(), self.defaults.settings.clone()));
        self.scan.error = None;
        self.scan.busy = true;
        self.scan.progress = None;
        worker.send(cmd);
    }

    pub fn open_scanner(&mut self, choice: ScannerChoice) {
        self.scan.info = None;
        self.scan.prescan = None;
        self.scan_send(ScanCmd::Open(choice));
    }

    pub fn prescan(&mut self) {
        self.scan_send(ScanCmd::Prescan);
    }

    /// Scan the cropped frame into the open roll with the panel's settings.
    pub fn scan_frame(&mut self) {
        let (Some(info), Some(dir)) = (&self.scan.info, self.folder.clone()) else { return };
        let model = &info.model;
        let d = &self.defaults.scan;
        let dpi = if model.resolutions_dpi.contains(&d.dpi) { d.dpi } else { *model.resolutions_dpi.iter().max().unwrap() };
        let c = self.scan.crop.map(f64::from);
        let req = ScanRequest {
            resolution: dpi,
            area: Some(bringup::image_crop_to_scan_area(model, c)),
            infrared: d.infrared && model.supports_infrared,
            multi_exposure: d.multi_exposure && supports_me(model),
            n_passes: if bringup::is_gl128_opticfilm(model) { d.passes.max(1) } else { 1 },
            ..Default::default()
        };
        let path = scanner::next_scan_path(&dir);
        let ir_path = scanner::ir_path_for(&path);
        self.scan_send(ScanCmd::Scan { req: Box::new(req), path, ir_path });
    }

    pub fn on_scan_event(&mut self, ctx: &egui::Context, ev: ScanEvent) {
        let s = &mut self.scan;
        match ev {
            ScanEvent::Status(t) => s.status = t,
            ScanEvent::Progress(p) => s.progress = Some(p),
            ScanEvent::Opened(info) => {
                let info = *info;
                let m = &info.model;
                let area = bringup::scan_area_to_image_crop(m, bringup::default_frame_crop_norm(m));
                s.crop = area.map(|v| v as f32);
                s.status = if info.scan_ready {
                    format!("{} ready", m.model)
                } else {
                    format!("{} is not validated for scanning yet; it opens for status only.", m.model)
                };
                s.info = Some(info);
                s.busy = false;
            }
            ScanEvent::Prescan(img) => {
                s.prescan = Some(ctx.load_texture("scan-prescan", img, egui::TextureOptions::LINEAR));
                s.status = "Drag on the prescan to frame the scan.".into();
                s.busy = false;
                s.progress = None;
            }
            ScanEvent::Saved { path, ir_path } => {
                s.saved += 1;
                let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                s.status = if ir_path.is_some() { format!("Saved {name} with its IR plane") } else { format!("Saved {name}") };
                s.busy = false;
                s.progress = None;
                self.add_scanned_frame(path);
            }
            ScanEvent::Cancelled => {
                s.status = "Cancelled".into();
                s.busy = false;
                s.progress = None;
            }
            ScanEvent::Failed(m) => {
                s.status.clear();
                s.error = Some(m.clone());
                s.busy = false;
                s.progress = None;
                self.toast(m, true);
            }
            ScanEvent::Closed => {
                s.info = None;
                s.busy = false;
            }
        }
    }

    /// The panel (folder view, right side).
    pub fn scan_panel_ui(&mut self, ui: &mut egui::Ui) {
        if !self.scan.open {
            return;
        }
        egui::Panel::right("scan_panel")
            .resizable(true)
            .default_size(340.0)
            .min_size(300.0)
            .max_size(520.0)
            .frame(egui::Frame::new().fill(theme::PANEL).inner_margin(egui::Margin { left: 16, right: 12, top: 12, bottom: 8 }))
            .show(ui, |ui| self.scan_panel_body(ui));
    }

    fn scan_panel_body(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Scan").font(theme::medium(15.0)));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.add(widgets::ghost("Close")).on_hover_text("Close the panel and switch the scanner's lamp off").clicked() {
                    self.close_scan_panel();
                }
            });
        });
        ui.add_space(8.0);
        self.scanner_picker(ui);
        let Some(info) = self.scan.info.clone() else {
            self.scan_status(ui);
            return;
        };
        ui.add_space(10.0);
        self.scan_options(ui, &info.model);
        ui.add_space(10.0);
        self.prescan_view(ui);
        ui.add_space(10.0);

        let busy = self.scan.busy;
        let ready = info.scan_ready && !busy;
        let next_key = widgets::shortcut(ui.ctx(), egui::Modifiers::COMMAND, egui::Key::R);
        ui.horizontal(|ui| {
            if ui
                .add_enabled(ready, widgets::ghost("Prescan"))
                .on_hover_text("Scan the whole holder at low resolution to frame the shot")
                .clicked()
            {
                self.prescan();
            }
            let label = if self.scan.saved > 0 { "Scan next" } else { "Scan" };
            let scan = ui
                .add_enabled(ready && self.folder.is_some(), widgets::primary(label))
                .on_hover_text(format!("Scan the framed area into this roll ({next_key}). Advance the holder between frames."));
            let key = ui.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::R));
            if scan.clicked() || (key && ready && self.folder.is_some()) {
                self.scan_frame();
            }
            if busy
                && ui.add(widgets::ghost("Cancel")).clicked()
                && let Some(w) = &self.scan.worker
            {
                w.cancel();
            }
        });
        self.scan_status(ui);
    }

    fn scanner_picker(&mut self, ui: &mut egui::Ui) {
        let devices = self.scan.devices.clone().unwrap_or_default();
        let mut choices: Vec<ScannerChoice> = devices.iter().map(|d| ScannerChoice::Device(d.0.clone())).collect();
        if scanner::simulated_scanners_enabled() {
            choices.extend(opticfilm::known_models().iter().map(|m| ScannerChoice::Simulated(m.name.to_string())));
        }
        let current = self.scan.info.as_ref().map(|i| i.choice.clone());
        let text = current.as_ref().map_or_else(
            || if choices.is_empty() { "No scanner found".to_string() } else { "Choose a scanner".to_string() },
            |c| label_for(c, &devices),
        );
        let mut picked = None;
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt("scan_scanner").width(220.0).selected_text(text).show_ui(ui, |ui| {
                for c in &choices {
                    if ui.selectable_label(current.as_ref() == Some(c), label_for(c, &devices)).clicked() {
                        picked = Some(c.clone());
                    }
                }
            });
            if ui.add_enabled(!self.scan.busy, widgets::ghost("Refresh")).on_hover_text("Look for connected scanners").clicked() {
                self.scan.devices = Some(scanner::list_scanners());
            }
        });
        if let Some(c) = picked.filter(|c| Some(c) != current.as_ref()) {
            self.open_scanner(c);
        }
    }

    fn scan_options(&mut self, ui: &mut egui::Ui, model: &FilmModel) {
        let mut d = self.defaults.scan.clone();
        let mut dpis = model.resolutions_dpi.clone();
        dpis.sort_unstable();
        if !dpis.contains(&d.dpi) {
            d.dpi = *dpis.iter().find(|&&v| v >= d.dpi).unwrap_or(dpis.last().unwrap());
        }
        egui::Grid::new("scan_opts").num_columns(2).spacing(vec2(12.0, 8.0)).show(ui, |ui| {
            ui.label(RichText::new("Resolution").color(theme::MUTED));
            egui::ComboBox::from_id_salt("scan_dpi").selected_text(format!("{} dpi", d.dpi)).show_ui(ui, |ui| {
                for v in &dpis {
                    ui.selectable_value(&mut d.dpi, *v, format!("{v} dpi"));
                }
            });
            ui.end_row();
            if model.supports_infrared {
                ui.label(RichText::new("Infrared").color(theme::MUTED));
                ui.checkbox(&mut d.infrared, "Dust plane")
                    .on_hover_text("Also scan under the IR lamp, for dust removal (colour and chromogenic film)");
                ui.end_row();
            }
            if supports_me(model) {
                ui.label(RichText::new("Exposure").color(theme::MUTED));
                ui.checkbox(&mut d.multi_exposure, "Multi-exposure")
                    .on_hover_text("A second, longer pass merged in, for more shadow detail on dense negatives");
                ui.end_row();
            }
            if bringup::is_gl128_opticfilm(model) {
                ui.label(RichText::new("Passes").color(theme::MUTED));
                ui.add(egui::Slider::new(&mut d.passes, 1..=4)).on_hover_text("Average several aligned passes to lower noise");
                ui.end_row();
            }
        });
        if d != self.defaults.scan {
            self.defaults.scan = d;
            let _ = crate::engine::sidecar::save_defaults(&self.defaults);
        }
    }

    /// The prescan with its crop; a drag frames a new crop.
    fn prescan_view(&mut self, ui: &mut egui::Ui) {
        let Some(tex) = self.scan.prescan.clone() else {
            ui.label(RichText::new("Prescan to see the holder and frame the shot.").color(theme::FAINT));
            return;
        };
        let size = tex.size_vec2();
        let w = ui.available_width();
        let scale = (w / size.x).min(360.0 / size.y);
        let (rect, resp) = ui.allocate_exact_size(size * scale, Sense::drag());
        ui.painter().image(tex.id(), rect, egui::Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), egui::Color32::WHITE);
        let to_norm = |p: egui::Pos2| ((p - rect.min) / rect.size()).clamp(vec2(0.0, 0.0), vec2(1.0, 1.0));
        if resp.drag_started()
            && let Some(p) = resp.interact_pointer_pos()
        {
            self.scan.drag_from = Some(to_norm(p).to_pos2());
        }
        if let (Some(a), Some(p)) = (self.scan.drag_from, resp.interact_pointer_pos())
            && resp.dragged()
        {
            let b = to_norm(p);
            if (b.x - a.x).abs() > 0.01 && (b.y - a.y).abs() > 0.01 {
                self.scan.crop = [a.x.min(b.x), a.y.min(b.y), a.x.max(b.x), a.y.max(b.y)];
            }
        }
        if resp.drag_stopped() {
            self.scan.drag_from = None;
        }
        let [x1, y1, x2, y2] = self.scan.crop;
        let crop = egui::Rect::from_min_max(rect.min + vec2(x1, y1) * rect.size(), rect.min + vec2(x2, y2) * rect.size());
        let shade = egui::Color32::from_black_alpha(140);
        let p = ui.painter();
        for r in [
            egui::Rect::from_min_max(rect.min, pos2(rect.max.x, crop.min.y)),
            egui::Rect::from_min_max(pos2(rect.min.x, crop.max.y), rect.max),
            egui::Rect::from_min_max(pos2(rect.min.x, crop.min.y), pos2(crop.min.x, crop.max.y)),
            egui::Rect::from_min_max(pos2(crop.max.x, crop.min.y), pos2(rect.max.x, crop.max.y)),
        ] {
            p.rect_filled(r, 0.0, shade);
        }
        p.rect_stroke(crop, 0.0, egui::Stroke::new(1.5, theme::ACCENT), egui::StrokeKind::Inside);
    }

    fn scan_status(&self, ui: &mut egui::Ui) {
        ui.add_space(6.0);
        if let Some(p) = self.scan.progress.filter(|_| self.scan.busy) {
            ui.add(egui::ProgressBar::new(p as f32).desired_height(6.0));
        } else if self.scan.busy {
            ui.spinner();
        }
        if !self.scan.status.is_empty() {
            ui.label(RichText::new(&self.scan.status).color(theme::MUTED));
        }
        if let Some(e) = &self.scan.error {
            ui.label(RichText::new(e).color(theme::ERR));
        }
        if self.folder.is_none() {
            ui.label(RichText::new("Open or create a roll folder to scan into.").color(theme::WARN));
        }
    }
}
