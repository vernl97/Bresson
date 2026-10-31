//! The Scan panel: pick a scanner, prescan, frame the shots and scan into the
//! open roll. Film scanners frame one area; flatbeds take a box per frame
//! and scan them in turn. The scanner runs on the worker in `scanner.rs`.

use eframe::egui::{self, RichText, Sense, pos2, vec2};

use super::NlpApp;
use super::scanner::{self, DeviceEntry, ScanCmd, ScanEvent, ScanJob, ScanWorker, ScannerChoice, ScannerInfo, ScannerKind};
use super::widgets::{self, theme};

/// Smallest box kept from a drag, as a fraction of the prescan.
const MIN_BOX: f32 = 0.01;
/// Corner grab radius in points.
const HANDLE: f32 = 8.0;

#[derive(Debug, Clone, Copy)]
enum Drag {
    New([f32; 2]),
    Move { i: usize, grab: [f32; 2], orig: [f32; 4] },
    /// Corner: 0 top-left, 1 top-right, 2 bottom-right, 3 bottom-left.
    Resize { i: usize, corner: usize },
}

#[derive(Default)]
pub struct ScanState {
    pub open: bool,
    worker: Option<ScanWorker>,
    /// Scanners from the last refresh, and why SANE isn't available.
    devices: Option<Vec<DeviceEntry>>,
    sane_note: Option<String>,
    pub info: Option<ScannerInfo>,
    /// A command is running on the worker.
    pub busy: bool,
    /// Frames of the running scan not saved yet.
    pending: usize,
    pub status: String,
    pub progress: Option<f64>,
    prescan: Option<egui::TextureHandle>,
    /// Framed areas on the prescan, normalised `[x1, y1, x2, y2]`, in scan order.
    pub crops: Vec<[f32; 4]>,
    selected: Option<usize>,
    drag: Option<Drag>,
    pub error: Option<String>,
    /// Frames saved since the panel opened.
    pub saved: usize,
}

fn order(a: [f32; 2], b: [f32; 2]) -> [f32; 4] {
    [a[0].min(b[0]), a[1].min(b[1]), a[0].max(b[0]), a[1].max(b[1])]
}

/// The corner of the box spanned by `fixed` and `p` that `p` is on.
fn corner_toward(fixed: [f32; 2], p: [f32; 2]) -> usize {
    match (p[0] < fixed[0], p[1] < fixed[1]) {
        (true, true) => 0,
        (false, true) => 1,
        (false, false) => 2,
        (true, false) => 3,
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
            self.scan_send(ScanCmd::Refresh);
        }
    }

    /// Hide the panel and release the scanner: a running scan is cancelled,
    /// the lamp goes off. The device list is kept.
    pub fn close_scan_panel(&mut self) {
        let devices = self.scan.devices.take();
        let sane_note = self.scan.sane_note.take();
        self.scan = ScanState { devices, sane_note, ..Default::default() };
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

    pub fn refresh_scanners(&mut self) {
        self.scan_send(ScanCmd::Refresh);
    }

    pub fn open_scanner(&mut self, choice: ScannerChoice) {
        self.scan.info = None;
        self.scan.prescan = None;
        self.scan.crops.clear();
        self.scan_send(ScanCmd::Open(choice));
    }

    pub fn prescan(&mut self) {
        self.scan_send(ScanCmd::Prescan);
    }

    fn set_scan_source(&mut self, source: String) {
        if let Some(info) = &self.scan.info {
            self.defaults.scan.source.insert(info.choice.key(), source.clone());
            let _ = crate::engine::sidecar::save_defaults(&self.defaults);
        }
        self.scan_send(ScanCmd::SetSource(source));
    }

    /// Scan every framed area into the open roll with the panel's settings.
    pub fn scan_frames(&mut self) {
        let (Some(info), Some(dir)) = (&self.scan.info, self.folder.clone()) else { return };
        if self.scan.crops.is_empty() {
            return;
        }
        let d = &self.defaults.scan;
        let dpi = if info.resolutions.contains(&d.dpi) { d.dpi } else { info.resolutions.iter().copied().max().unwrap_or(d.dpi) };
        let areas = self.scan.crops.clone();
        let job = ScanJob {
            dpi,
            paths: scanner::next_scan_paths(&dir, areas.len()),
            areas,
            infrared: d.infrared && info.supports_ir,
            multi_exposure: d.multi_exposure && info.supports_me,
            passes: if info.supports_passes { d.passes.max(1) } else { 1 },
            source: info.source.clone(),
            gamma_encoded: d.gamma_encoded.contains(&info.choice.key()),
        };
        let n = job.areas.len();
        self.scan_send(ScanCmd::Scan(Box::new(job)));
        self.scan.pending = n;
    }

    pub fn on_scan_event(&mut self, ctx: &egui::Context, ev: ScanEvent) {
        let s = &mut self.scan;
        match ev {
            ScanEvent::Status(t) => s.status = t,
            ScanEvent::Progress(p) => s.progress = Some(p),
            ScanEvent::Devices { devices, sane_note } => {
                s.devices = Some(devices);
                s.sane_note = sane_note;
                s.busy = false;
            }
            ScanEvent::Opened(info) => {
                let info = *info;
                s.crops = info.default_crop.into_iter().collect();
                s.selected = None;
                s.prescan = None;
                s.status = if info.scan_ready {
                    format!("{} ready", info.label)
                } else {
                    format!("{} is not validated for scanning yet; it opens for status only.", info.label)
                };
                s.busy = false;
                // A flatbed opens on its film source; switch to the one picked last time.
                let wanted = self.defaults.scan.source.get(&info.choice.key()).cloned();
                let switch = wanted.filter(|w| info.kind == ScannerKind::Flatbed && Some(w) != info.source.as_ref() && info.sources.contains(w));
                s.info = Some(info);
                if let Some(w) = switch {
                    self.scan_send(ScanCmd::SetSource(w));
                }
            }
            ScanEvent::Prescan(img) => {
                s.prescan = Some(ctx.load_texture("scan-prescan", img, egui::TextureOptions::LINEAR));
                let flatbed = s.info.as_ref().is_some_and(|i| i.kind == ScannerKind::Flatbed);
                s.status = if flatbed { "Draw a box around each frame." } else { "Drag on the prescan to frame the scan." }.into();
                s.busy = false;
                s.progress = None;
            }
            ScanEvent::Saved { path, ir_path } => {
                s.saved += 1;
                s.pending = s.pending.saturating_sub(1);
                let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                s.status = if ir_path.is_some() { format!("Saved {name} with its IR plane") } else { format!("Saved {name}") };
                if s.pending == 0 {
                    s.busy = false;
                    s.progress = None;
                }
                self.add_scanned_frame(path);
            }
            ScanEvent::Cancelled => {
                s.status = "Cancelled".into();
                s.busy = false;
                s.pending = 0;
                s.progress = None;
            }
            ScanEvent::Failed(m) => {
                s.status.clear();
                s.error = Some(m.clone());
                s.busy = false;
                s.pending = 0;
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
            .default_size(360.0)
            .min_size(300.0)
            .max_size(560.0)
            .frame(egui::Frame::new().fill(theme::PANEL).inner_margin(egui::Margin { left: 16, right: 12, top: 12, bottom: 8 }))
            .show(ui, |ui| {
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| self.scan_panel_body(ui));
            });
    }

    fn scan_panel_body(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Scan").font(theme::medium(15.0)));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.add(widgets::ghost("Close")).on_hover_text("Close the panel and release the scanner").clicked() {
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
        self.scan_options(ui, &info);
        ui.add_space(10.0);
        self.prescan_view(ui, &info);
        ui.add_space(10.0);

        let busy = self.scan.busy;
        let ready = info.scan_ready && !busy && self.folder.is_some();
        let n = self.scan.crops.len();
        let next_key = widgets::shortcut(ui.ctx(), egui::Modifiers::COMMAND, egui::Key::R);
        ui.horizontal(|ui| {
            if ui
                .add_enabled(info.scan_ready && !busy, widgets::ghost("Prescan"))
                .on_hover_text("Scan the whole holder at low resolution to frame the shots")
                .clicked()
            {
                self.prescan();
            }
            let label = match (info.kind, n) {
                (ScannerKind::Flatbed, n) if n > 1 => format!("Scan {n} frames"),
                (ScannerKind::Film, _) if self.scan.saved > 0 => "Scan next".to_string(),
                _ => "Scan".to_string(),
            };
            let hint = match info.kind {
                ScannerKind::Film => format!("Scan the framed area into this roll ({next_key}). Advance the holder between frames."),
                ScannerKind::Flatbed => format!("Scan each boxed frame into this roll, one file per frame ({next_key})"),
            };
            let scan = ui.add_enabled(ready && n > 0, widgets::primary(&label)).on_hover_text(hint);
            let key = ui.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::R));
            if scan.clicked() || (key && ready && n > 0) {
                self.scan_frames();
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
        let current = self.scan.info.as_ref().map(|i| i.choice.clone());
        let text = match &self.scan.info {
            Some(i) => i.label.clone(),
            None if self.scan.devices.is_none() => "Looking for scanners…".to_string(),
            None if devices.is_empty() => "No scanner found".to_string(),
            None => "Choose a scanner".to_string(),
        };
        let mut picked = None;
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt("scan_scanner").width(230.0).selected_text(text).show_ui(ui, |ui| {
                for d in &devices {
                    if ui.selectable_label(current.as_ref() == Some(&d.choice), &d.label).clicked() {
                        picked = Some(d.choice.clone());
                    }
                }
            });
            if ui.add_enabled(!self.scan.busy, widgets::ghost("Refresh")).on_hover_text("Look for connected scanners").clicked() {
                self.refresh_scanners();
            }
        });
        if let Some(note) = &self.scan.sane_note {
            ui.label(RichText::new(note).color(theme::FAINT).small());
        }
        if let Some(c) = picked.filter(|c| Some(c) != current.as_ref()) {
            self.open_scanner(c);
        }
    }

    fn scan_options(&mut self, ui: &mut egui::Ui, info: &ScannerInfo) {
        let key = info.choice.key();
        let mut d = self.defaults.scan.clone();
        let dpis = &info.resolutions;
        if !dpis.is_empty() && !dpis.contains(&d.dpi) {
            d.dpi = *dpis.iter().find(|&&v| v >= d.dpi).unwrap_or(dpis.last().unwrap());
        }
        let mut gamma = d.gamma_encoded.contains(&key);
        let mut new_source = None;
        egui::Grid::new("scan_opts").num_columns(2).spacing(vec2(12.0, 8.0)).show(ui, |ui| {
            if info.sources.len() > 1 {
                ui.label(RichText::new("Source").color(theme::MUTED));
                let current = info.source.clone().unwrap_or_default();
                egui::ComboBox::from_id_salt("scan_source").selected_text(&current).show_ui(ui, |ui| {
                    for s in &info.sources {
                        if ui.selectable_label(*s == current, s).clicked() && *s != current {
                            new_source = Some(s.clone());
                        }
                    }
                });
                ui.end_row();
            }
            ui.label(RichText::new("Resolution").color(theme::MUTED));
            egui::ComboBox::from_id_salt("scan_dpi").selected_text(format!("{} dpi", d.dpi)).show_ui(ui, |ui| {
                for v in dpis {
                    ui.selectable_value(&mut d.dpi, *v, format!("{v} dpi"));
                }
            });
            ui.end_row();
            if info.supports_ir {
                ui.label(RichText::new("Infrared").color(theme::MUTED));
                widgets::checkbox(ui, &mut d.infrared, "Dust plane")
                    .on_hover_text("Also scan under the IR lamp, for dust removal (colour and chromogenic film)");
                ui.end_row();
            }
            if info.supports_me {
                ui.label(RichText::new("Exposure").color(theme::MUTED));
                widgets::checkbox(ui, &mut d.multi_exposure, "Multi-exposure")
                    .on_hover_text("A second, longer pass merged in, for more shadow detail on dense negatives");
                ui.end_row();
            }
            if info.supports_passes {
                ui.label(RichText::new("Passes").color(theme::MUTED));
                ui.allocate_ui(egui::vec2(150.0, 28.0), |ui| {
                    widgets::segmented(ui, &mut d.passes, &[(1, "1"), (2, "2"), (3, "3"), (4, "4")]);
                })
                .response
                .on_hover_text("Average several aligned passes to lower noise");
                ui.end_row();
            }
            if info.kind == ScannerKind::Flatbed {
                ui.label(RichText::new("Data").color(theme::MUTED));
                widgets::checkbox(ui, &mut gamma, "Gamma-encoded").on_hover_text(
                    "Turn on if this scanner applies a tone curve even at 16 bit (scans look light and flat). \
                     Frames are then saved as TIFF scans instead of linear DNG.",
                );
                ui.end_row();
            }
        });
        if gamma {
            d.gamma_encoded.insert(key);
        } else {
            d.gamma_encoded.remove(&key);
        }
        if d != self.defaults.scan {
            self.defaults.scan = d;
            let _ = crate::engine::sidecar::save_defaults(&self.defaults);
        }
        if let Some(s) = new_source {
            self.set_scan_source(s);
        }
    }

    /// The prescan with its boxes. Film scanners frame one area; on a flatbed
    /// a drag on empty space adds a box, inside one moves it, on a corner
    /// resizes it; × or Delete removes the selected box.
    fn prescan_view(&mut self, ui: &mut egui::Ui, info: &ScannerInfo) {
        let Some(tex) = self.scan.prescan.clone() else {
            let hint = match info.kind {
                ScannerKind::Film => "Prescan to see the holder and frame the shot.",
                ScannerKind::Flatbed => "Prescan, then draw a box around each frame on the bed.",
            };
            ui.label(RichText::new(hint).color(theme::FAINT));
            return;
        };
        let multi = info.kind == ScannerKind::Flatbed;
        let size = tex.size_vec2();
        let scale = (ui.available_width() / size.x).min(520.0 / size.y);
        let (rect, resp) = ui.allocate_exact_size(size * scale, Sense::click_and_drag());
        ui.painter().image(tex.id(), rect, egui::Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), egui::Color32::WHITE);
        let to_norm = |p: egui::Pos2| {
            let v = ((p - rect.min) / rect.size()).clamp(vec2(0.0, 0.0), vec2(1.0, 1.0));
            [v.x, v.y]
        };
        let to_screen =
            |c: [f32; 4]| egui::Rect::from_min_max(rect.min + vec2(c[0], c[1]) * rect.size(), rect.min + vec2(c[2], c[3]) * rect.size());
        let close_box = |r: egui::Rect| egui::Rect::from_min_size(pos2(r.max.x - 18.0, r.min.y + 2.0), vec2(16.0, 16.0));
        let hit_corner = |p: egui::Pos2, crops: &[[f32; 4]]| {
            crops.iter().enumerate().rev().find_map(|(i, c)| {
                let r = to_screen(*c);
                [r.left_top(), r.right_top(), r.right_bottom(), r.left_bottom()].iter().position(|q| q.distance(p) <= HANDLE).map(|k| (i, k))
            })
        };
        let hit_box = |p: egui::Pos2, crops: &[[f32; 4]]| crops.iter().rposition(|c| to_screen(*c).contains(p));

        if resp.clicked()
            && let Some(p) = resp.interact_pointer_pos()
        {
            let close = self.scan.crops.iter().rposition(|c| multi && close_box(to_screen(*c)).contains(p));
            if let Some(i) = close.filter(|&i| self.scan.selected == Some(i) || hit_box(p, &self.scan.crops) == Some(i)) {
                self.scan.crops.remove(i);
                self.scan.selected = None;
            } else {
                self.scan.selected = hit_box(p, &self.scan.crops);
            }
        }
        if resp.drag_started()
            && let Some(p) = resp.interact_pointer_pos()
        {
            let crops = &self.scan.crops;
            self.scan.drag = Some(if let Some((i, corner)) = hit_corner(p, crops) {
                Drag::Resize { i, corner }
            } else if let Some(i) = hit_box(p, crops) {
                Drag::Move { i, grab: to_norm(p), orig: crops[i] }
            } else {
                Drag::New(to_norm(p))
            });
        }
        if resp.dragged()
            && let (Some(drag), Some(p)) = (self.scan.drag, resp.interact_pointer_pos())
        {
            let n = to_norm(p);
            match drag {
                Drag::New(a) => {
                    // Once big enough the box exists and the drag resizes its far corner.
                    let b = order(a, n);
                    if b[2] - b[0] > MIN_BOX && b[3] - b[1] > MIN_BOX {
                        if multi {
                            self.scan.crops.push(b);
                        } else {
                            self.scan.crops = vec![b];
                        }
                        let i = self.scan.crops.len() - 1;
                        self.scan.selected = Some(i);
                        self.scan.drag = Some(Drag::Resize { i, corner: corner_toward(a, n) });
                    }
                }
                Drag::Move { i, grab, orig } => {
                    let (w, h) = (orig[2] - orig[0], orig[3] - orig[1]);
                    let x = (orig[0] + n[0] - grab[0]).clamp(0.0, 1.0 - w);
                    let y = (orig[1] + n[1] - grab[1]).clamp(0.0, 1.0 - h);
                    if let Some(c) = self.scan.crops.get_mut(i) {
                        *c = [x, y, x + w, y + h];
                    }
                    self.scan.selected = Some(i);
                }
                Drag::Resize { i, corner } => {
                    if let Some(c) = self.scan.crops.get_mut(i) {
                        let fixed = match corner {
                            0 => [c[2], c[3]],
                            1 => [c[0], c[3]],
                            2 => [c[0], c[1]],
                            _ => [c[2], c[1]],
                        };
                        let b = order(fixed, n);
                        if b[2] - b[0] > MIN_BOX && b[3] - b[1] > MIN_BOX {
                            *c = b;
                            // Follow the pointer when it crosses the fixed corner.
                            self.scan.drag = Some(Drag::Resize { i, corner: corner_toward(fixed, n) });
                        }
                    }
                    self.scan.selected = Some(i);
                }
            }
        }
        if resp.drag_stopped() {
            self.scan.drag = None;
        }
        if multi
            && resp.hovered()
            && let Some(i) = self.scan.selected.filter(|&i| i < self.scan.crops.len())
            && ui.input_mut(|inp| inp.consume_key(egui::Modifiers::NONE, egui::Key::Delete) || inp.consume_key(egui::Modifiers::NONE, egui::Key::Backspace))
        {
            self.scan.crops.remove(i);
            self.scan.selected = None;
        }

        // Shade outside the frame (film) or dim until a box exists (flatbed), then outline.
        let p = ui.painter().with_clip_rect(rect);
        if self.scan.crops.is_empty() {
            p.rect_filled(rect, 0.0, egui::Color32::from_black_alpha(50));
        } else if !multi {
            let crop = to_screen(self.scan.crops[0]);
            for r in [
                egui::Rect::from_min_max(rect.min, pos2(rect.max.x, crop.min.y)),
                egui::Rect::from_min_max(pos2(rect.min.x, crop.max.y), rect.max),
                egui::Rect::from_min_max(pos2(rect.min.x, crop.min.y), pos2(crop.min.x, crop.max.y)),
                egui::Rect::from_min_max(pos2(crop.max.x, crop.min.y), pos2(rect.max.x, crop.max.y)),
            ] {
                p.rect_filled(r, 0.0, egui::Color32::from_black_alpha(130));
            }
        }
        let hover = resp.hover_pos();
        let hovered_box = hover.and_then(|h| hit_box(h, &self.scan.crops));
        for (i, c) in self.scan.crops.iter().enumerate() {
            let r = to_screen(*c);
            let active = self.scan.selected == Some(i) || hovered_box == Some(i);
            p.rect_stroke(r, 0.0, egui::Stroke::new(if active { 2.0 } else { 1.5 }, theme::ACCENT), egui::StrokeKind::Inside);
            if multi {
                let tag = egui::Rect::from_min_size(r.min + vec2(2.0, 2.0), vec2(18.0, 16.0));
                p.rect_filled(tag, 3.0, theme::ACCENT);
                p.text(tag.center(), egui::Align2::CENTER_CENTER, format!("{}", i + 1), egui::FontId::proportional(11.0), egui::Color32::WHITE);
                if active {
                    let x = close_box(r);
                    p.rect_filled(x, 3.0, egui::Color32::from_black_alpha(170));
                    p.text(x.center(), egui::Align2::CENTER_CENTER, "×", egui::FontId::proportional(13.0), theme::TEXT);
                }
            }
            if self.scan.selected == Some(i) {
                for q in [r.left_top(), r.right_top(), r.right_bottom(), r.left_bottom()] {
                    p.circle_filled(q, 3.5, theme::ACCENT);
                }
            }
        }
        if let Some(h) = hover {
            let cursor = match hit_corner(h, &self.scan.crops) {
                Some((_, k)) if k % 2 == 0 => egui::CursorIcon::ResizeNwSe,
                Some(_) => egui::CursorIcon::ResizeNeSw,
                None if hovered_box.is_some() => egui::CursorIcon::Grab,
                None => egui::CursorIcon::Crosshair,
            };
            ui.ctx().set_cursor_icon(cursor);
        }
        if multi {
            ui.horizontal(|ui| {
                let n = self.scan.crops.len();
                let text = match n {
                    0 => "Drag to draw a box around each frame.".to_string(),
                    1 => "1 frame boxed".to_string(),
                    n => format!("{n} frames boxed"),
                };
                ui.label(RichText::new(text).color(theme::MUTED));
                if n > 0 && ui.add(widgets::ghost("Clear boxes")).clicked() {
                    self.scan.crops.clear();
                    self.scan.selected = None;
                }
            });
        }
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
