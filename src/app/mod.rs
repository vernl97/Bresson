//! NLP Studio GUI: a Folder View for browsing / batch-editing a set of scans
//! and an Editor View for fine-tuning a single frame.

mod editor;
mod folder;
mod jobs;
mod picker;
mod widgets;

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use eframe::egui;

use crate::engine::decode::{self, SourceImage};
use crate::engine::pipeline::{self, Renderer};
use crate::engine::settings::{OutputSpace, Settings, WbMode};
use crate::engine::sidecar::{self, Defaults, Sidecar};
use jobs::{JobResult, Jobs};

pub const THUMB_SIZE: usize = 360;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum View {
    Folder,
    Editor,
}

pub enum ProxyState {
    NotLoaded,
    Loading,
    Ready(Arc<SourceImage>),
    Failed(String),
}

pub struct Frame {
    pub path: PathBuf,
    pub name: String,
    pub sc: Sidecar,
    pub proxy: ProxyState,
    /// Small source used for thumbnails.
    pub thumb_src: Option<SourceImage>,
    pub thumb: Option<egui::TextureHandle>,
    pub thumb_dirty: bool,
    /// Time of the last unsaved change.
    pub dirty_since: Option<f64>,
    pub exporting: bool,
    /// No saved settings yet: the scan source is set from the file.
    pub source_auto: bool,
}

impl Frame {
    pub fn proxy(&self) -> Option<Arc<SourceImage>> {
        match &self.proxy {
            ProxyState::Ready(p) => Some(p.clone()),
            _ => None,
        }
    }

    pub fn is_roll(&self) -> bool {
        self.sc.roll.is_some() && self.sc.settings.use_roll
    }

    pub fn roll_origins(&self) -> Option<&crate::engine::curve::Origins> {
        self.sc.roll.as_ref().map(|r| &r.origins)
    }

    /// Base colour the frame renders with.
    pub fn base_for(&self, src: &SourceImage) -> [f32; 3] {
        match &self.sc.analysis {
            Some(a) => a.base,
            None => pipeline::resolve_base(
                src,
                &self.sc.settings,
                self.is_roll().then(|| self.sc.roll.as_ref().unwrap().base),
            ),
        }
    }

    /// Apply the WB dropdown choice to Temp / Tint (`updateFilm`).
    pub fn apply_wb_mode(&mut self) {
        let wb = if self.is_roll() {
            self.sc.roll.as_ref().and_then(|r| r.auto_wb)
        } else {
            None
        };
        let wb = wb.or_else(|| self.sc.analysis.as_ref().map(|a| a.auto_wb));
        if let Some(w) = wb {
            if let Some((t, n)) = pipeline::wb_for_mode(self.sc.settings.wb_mode, &w) {
                self.sc.settings.wb_temp = t;
                self.sc.settings.wb_tint = n;
            }
        } else if let Some((t, n)) =
            pipeline::wb_for_mode(self.sc.settings.wb_mode, &Default::default())
        {
            if !matches!(
                self.sc.settings.wb_mode,
                WbMode::AutoNeutral
                    | WbMode::AutoWarm
                    | WbMode::AutoCool
                    | WbMode::AutoAvg
                    | WbMode::AutoMix
            ) {
                self.sc.settings.wb_temp = t;
                self.sc.settings.wb_tint = n;
            }
        }
    }
}

pub struct Toast {
    pub text: String,
    pub until: f64,
    pub error: bool,
}

pub struct NlpApp {
    pub view: View,
    pub folder: Option<PathBuf>,
    pub frames: Vec<Frame>,
    pub selected: BTreeSet<usize>,
    pub anchor: Option<usize>,
    pub active: Option<usize>,
    pub jobs: Jobs,
    pub defaults: Defaults,
    pub clipboard: Option<Settings>,
    pub editor: editor::Editor,
    pub toasts: Vec<Toast>,
    pub export_total: usize,
    pub export_done: usize,
    pub thumb_size: f32,
    pub time: f64,
    /// Opened as a lone image: "back" returns to the welcome screen.
    pub single_mode: bool,
}

impl NlpApp {
    pub fn new(cc: &eframe::CreationContext<'_>, open: Option<PathBuf>) -> Self {
        widgets::theme::apply(&cc.egui_ctx);
        let defaults = sidecar::load_defaults();
        let mut app = Self {
            view: View::Folder,
            folder: None,
            frames: Vec::new(),
            selected: BTreeSet::new(),
            anchor: None,
            active: None,
            jobs: Jobs::new(cc.egui_ctx.clone()),
            defaults,
            clipboard: None,
            editor: editor::Editor::default(),
            toasts: Vec::new(),
            export_total: 0,
            export_done: 0,
            thumb_size: 180.0,
            time: 0.0,
            single_mode: false,
        };
        if let Some(p) = open {
            if p.is_dir() {
                app.open_folder(&p);
            } else if p.is_file() {
                app.open_files(vec![p]);
            }
        }
        app
    }

    pub fn toast(&mut self, text: impl Into<String>, error: bool) {
        let until = self.time + if error { 8.0 } else { 4.0 };
        self.toasts.push(Toast {
            text: text.into(),
            until,
            error,
        });
    }

    // ---- library --------------------------------------------------------------

    pub fn open_folder(&mut self, dir: &Path) {
        let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
            .map(|rd| {
                rd.filter_map(|e| e.ok().map(|e| e.path()))
                    .filter(|p| {
                        p.is_file()
                            && decode::is_supported(p)
                            && !p.to_string_lossy().ends_with("-prepped.tif")
                    })
                    .collect()
            })
            .unwrap_or_default();
        files.sort_by_key(|p| p.file_name().map(|n| n.to_string_lossy().to_lowercase()));
        self.load_frames(files, dir);
        self.single_mode = false;
        self.defaults.last_folder = Some(dir.to_path_buf());
        let _ = sidecar::save_defaults(&self.defaults);
    }

    /// Open individual image file(s) without the rest of their folder.
    /// A single image goes straight to the editor.
    pub fn open_files(&mut self, files: Vec<PathBuf>) {
        let files: Vec<PathBuf> = files
            .into_iter()
            .filter(|p| p.is_file() && decode::is_supported(p))
            .collect();
        let Some(first) = files.first() else {
            self.toast("Unsupported file type", true);
            return;
        };
        let dir = first.parent().unwrap_or(Path::new(".")).to_path_buf();
        let single = files.len() == 1;
        self.load_frames(files, &dir);
        self.single_mode = single;
        if single {
            self.open_editor(0);
        }
    }

    /// One chooser for files or a folder (welcome screen).
    pub fn choose_anything(&mut self) {
        let start = self
            .folder
            .clone()
            .or_else(|| self.defaults.last_folder.clone());
        let picked = picker::pick_files_or_folder(start.as_deref());
        if let Some(dir) = picked.iter().find(|p| p.is_dir()).cloned() {
            self.open_folder(&dir);
        } else if !picked.is_empty() {
            self.open_files(picked);
        }
    }

    pub fn choose_files(&mut self) {
        let mut exts: Vec<&str> = decode::RAW_EXTENSIONS.to_vec();
        exts.extend_from_slice(decode::BITMAP_EXTENSIONS);
        let upper: Vec<String> = exts.iter().map(|e| e.to_uppercase()).collect();
        let mut all: Vec<&str> = exts.clone();
        all.extend(upper.iter().map(|s| s.as_str()));
        let mut d = rfd::FileDialog::new()
            .set_title("Open image(s)")
            .add_filter("Scans", &all);
        if let Some(f) = &self.folder {
            d = d.set_directory(f);
        }
        if let Some(files) = d.pick_files() {
            self.open_files(files);
        }
    }

    fn load_frames(&mut self, files: Vec<PathBuf>, dir: &Path) {
        self.save_all();
        self.frames = files
            .into_iter()
            .map(|path| {
                let source_auto = sidecar::load(&path).is_none();
                let mut sc = sidecar::load(&path).unwrap_or_else(|| Sidecar {
                    settings: self.defaults.settings.clone(),
                    ..Default::default()
                });
                if decode::is_raw(&path) || sc.analysis.is_some() {
                } else if sidecar::load(&path).is_none() {
                    // Bitmap scans: Lightroom WB must not be applied; mirror that default.
                    sc.settings.film_base = crate::engine::settings::FilmBase::Off;
                    sc.settings.source = crate::engine::settings::Source::TifScan;
                }
                Frame {
                    name: path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default(),
                    path,
                    sc,
                    proxy: ProxyState::NotLoaded,
                    thumb_src: None,
                    thumb: None,
                    thumb_dirty: true,
                    dirty_since: None,
                    exporting: false,
                    source_auto,
                }
            })
            .collect();
        self.folder = Some(dir.to_path_buf());
        self.selected.clear();
        self.active = None;
        self.anchor = None;
        self.view = View::Folder;
        self.editor = editor::Editor::default();
        for i in 0..self.frames.len() {
            self.request_proxy(i);
        }
    }

    pub fn request_proxy(&mut self, i: usize) {
        let f = &mut self.frames[i];
        if !matches!(f.proxy, ProxyState::NotLoaded) {
            return;
        }
        f.proxy = ProxyState::Loading;
        let path = f.path.clone();
        self.jobs.spawn(move || {
            let result = jobs::load_proxy(&path)
                .map(Arc::new)
                .map_err(|e| format!("{e:#}"));
            JobResult::Proxy { path, result }
        });
    }

    /// Set the scan source of a frame without saved settings from its file
    /// (scanner profile → VueScan / SilverFast DNG, else digital camera).
    fn auto_source(&mut self, i: usize, src: &SourceImage) {
        use crate::engine::settings::{FilmBase, Source};
        let f = &mut self.frames[i];
        if !f.source_auto {
            return;
        }
        f.source_auto = false;
        let detected = pipeline::detect_source(src);
        if f.sc.settings.source != detected {
            f.sc.settings.source = detected;
            f.sc.settings.film_base = match detected {
                Source::TifScan | Source::LinearScan => FilmBase::Off,
                _ => FilmBase::Auto,
            };
            f.thumb_dirty = true;
            if self.editor.path.as_ref() == Some(&f.path) {
                self.editor.invalidate();
            }
        }
    }

    pub fn index_of(&self, p: &Path) -> Option<usize> {
        self.frames.iter().position(|f| f.path == p)
    }

    pub fn mark_changed(&mut self, i: usize) {
        let t = self.time;
        let f = &mut self.frames[i];
        f.dirty_since = Some(t);
        f.thumb_dirty = true;
    }

    pub fn save_frame(&mut self, i: usize) {
        let f = &mut self.frames[i];
        f.dirty_since = None;
        if let Err(e) = sidecar::save(&f.path, &f.sc) {
            let msg = format!("Could not save settings for {}: {e:#}", f.name);
            self.toast(msg, true);
        }
    }

    pub fn save_all(&mut self) {
        for i in 0..self.frames.len() {
            if self.frames[i].dirty_since.is_some() {
                self.save_frame(i);
            }
        }
    }

    fn autosave(&mut self) {
        for i in 0..self.frames.len() {
            if let Some(t) = self.frames[i].dirty_since {
                if self.time - t > 0.8 {
                    self.save_frame(i);
                }
            }
        }
    }

    // ---- conversion -----------------------------------------------------------

    pub fn convert(&mut self, idx: Vec<usize>, as_roll: bool) {
        if idx.is_empty() {
            return;
        }
        let items = idx
            .iter()
            .map(|&i| jobs::ConvertItem {
                path: self.frames[i].path.clone(),
                proxy: self.frames[i].proxy(),
                settings: self.frames[i].sc.settings.clone(),
            })
            .collect();
        let n = idx.len();
        self.toast(
            if as_roll {
                format!("Converting {n} frame(s) as a roll…")
            } else {
                format!("Converting {n} frame(s)…")
            },
            false,
        );
        self.jobs.spawn(move || jobs::convert(items, as_roll));
    }

    /// Synchronous re-analysis of one frame (after Convert-tab changes).
    #[allow(dead_code)]
    pub fn reanalyze(&mut self, i: usize) {
        let src = self.frames[i].proxy().or_else(|| {
            self.editor
                .src
                .clone()
                .filter(|_| self.editor.path.as_ref() == Some(&self.frames[i].path))
        });
        let Some(src) = src else { return };
        let f = &mut self.frames[i];
        let s = f.sc.settings.clone();
        let base = pipeline::resolve_base(&src, &s, None);
        let neg = pipeline::prepare_negative(&src, &s, base);
        f.sc.analysis = Some(pipeline::analyze(&neg, &s, base));
        if f.sc.settings.wb_mode != WbMode::Custom {
            f.apply_wb_mode();
        }
        self.mark_changed(i);
    }

    pub fn unconvert(&mut self, idx: &[usize]) {
        for &i in idx {
            let f = &mut self.frames[i];
            f.sc.analysis = None;
            f.sc.roll = None;
            f.sc.settings.use_roll = false;
            self.mark_changed(i);
        }
        self.editor.invalidate();
    }

    // ---- export -------------------------------------------------------------------

    pub fn export(&mut self, idx: Vec<usize>) {
        let ex = self.defaults.export.clone();
        let mut n = 0;
        for i in idx {
            let f = &mut self.frames[i];
            if f.exporting {
                continue;
            }
            f.exporting = true;
            n += 1;
            let (path, s, a, roll, meta, ex) = (
                f.path.clone(),
                f.sc.settings.clone(),
                f.sc.analysis.clone(),
                f.sc.roll.as_ref().map(|r| r.origins),
                f.sc.metadata.clone(),
                ex.clone(),
            );
            self.jobs.spawn(move || {
                let result = crate::engine::export::export_frame(
                    &path,
                    &s,
                    a.as_ref(),
                    roll.as_ref(),
                    &meta,
                    &ex,
                )
                .map_err(|e| format!("{e:#}"));
                JobResult::Exported { path, result }
            });
        }
        if n > 0 {
            if self.export_done >= self.export_total {
                self.export_total = 0;
                self.export_done = 0;
            }
            self.export_total += n;
        }
    }

    // ---- job results ------------------------------------------------------------

    fn poll_jobs(&mut self) {
        while let Ok(r) = self.jobs.rx.try_recv() {
            match r {
                JobResult::Proxy { path, result } => {
                    let Some(i) = self.index_of(&path) else {
                        continue;
                    };
                    match result {
                        Ok(p) => {
                            self.auto_source(i, &p);
                            let (d, w, h) =
                                decode::downscale(&p.data, p.width, p.height, THUMB_SIZE);
                            self.frames[i].thumb_src = Some(SourceImage {
                                data: d,
                                width: w,
                                height: h,
                                ..(*p).clone()
                            });
                            self.frames[i].proxy = ProxyState::Ready(p);
                            self.frames[i].thumb_dirty = true;
                        }
                        Err(e) => {
                            self.frames[i].proxy = ProxyState::Failed(e.clone());
                            self.toast(e, true);
                        }
                    }
                }
                JobResult::Editor { path, result } => {
                    if self.editor.path.as_ref() == Some(&path) {
                        self.editor.loading = false;
                        match result {
                            Ok(src) => {
                                if let Some(i) = self.index_of(&path) {
                                    self.auto_source(i, &src);
                                }
                                self.editor.src = Some(src);
                                self.editor.invalidate();
                            }
                            Err(e) => self.toast(e, true),
                        }
                    }
                }
                JobResult::Converted {
                    frames,
                    roll,
                    errors,
                } => {
                    for e in errors {
                        self.toast(e, true);
                    }
                    let n = frames.len();
                    for (path, src, a) in frames {
                        let Some(i) = self.index_of(&path) else {
                            continue;
                        };
                        let f = &mut self.frames[i];
                        if !matches!(f.proxy, ProxyState::Ready(_)) {
                            let (d, w, h) =
                                decode::downscale(&src.data, src.width, src.height, THUMB_SIZE);
                            f.thumb_src = Some(SourceImage {
                                data: d,
                                width: w,
                                height: h,
                                ..(*src).clone()
                            });
                            f.proxy = ProxyState::Ready(src);
                        }
                        f.sc.analysis = Some(a);
                        f.sc.roll = roll.clone();
                        f.sc.settings.use_roll = roll.is_some();
                        if f.sc.settings.wb_mode != WbMode::Custom {
                            f.apply_wb_mode();
                        }
                        self.mark_changed(i);
                        self.save_frame(i);
                    }
                    if let Some(r) = &roll {
                        if !r.excluded.is_empty() {
                            self.toast(
                                format!(
                                    "Roll: excluded outlier frames from the roll balance: {}",
                                    r.excluded.join(", ")
                                ),
                                false,
                            );
                        }
                    }
                    self.toast(format!("Converted {n} frame(s)"), false);
                    self.editor.invalidate();
                }
                JobResult::Exported { path, result } => {
                    self.export_done += 1;
                    if let Some(i) = self.index_of(&path) {
                        self.frames[i].exporting = false;
                    }
                    match result {
                        Ok(p) => {
                            if self.export_done >= self.export_total {
                                let msg = format!(
                                    "Exported {} positive cop{} (last: {})",
                                    self.export_total,
                                    if self.export_total == 1 { "y" } else { "ies" },
                                    p.display()
                                );
                                self.toast(msg, false);
                            }
                        }
                        Err(e) => self.toast(format!("Export failed: {e}"), true),
                    }
                }
                JobResult::Message(m) => self.toast(m, true),
            }
        }
    }

    /// Render a few dirty thumbnails per frame.
    fn render_thumbs(&mut self, ctx: &egui::Context) {
        let mut budget = 4;
        for f in self.frames.iter_mut() {
            if budget == 0 {
                ctx.request_repaint();
                break;
            }
            if !f.thumb_dirty {
                continue;
            }
            let Some(src) = &f.thumb_src else { continue };
            f.thumb_dirty = false;
            budget -= 1;
            let s = &f.sc.settings;
            let base = f.base_for(src);
            let neg = pipeline::prepare_negative(src, s, base);
            let roll = f.sc.roll.as_ref().map(|r| r.origins);
            let origins = pipeline::origins_for(s, f.sc.analysis.as_ref(), roll.as_ref());
            let r = Renderer::new(s, origins, OutputSpace::Srgb);
            let px = r.render(&neg, 0.0);
            let img = to_color_image(&px, neg.width, neg.height);
            match &mut f.thumb {
                Some(t) => t.set(img, egui::TextureOptions::LINEAR),
                None => {
                    f.thumb = Some(ctx.load_texture(
                        format!("thumb-{}", f.path.display()),
                        img,
                        egui::TextureOptions::LINEAR,
                    ))
                }
            }
        }
    }

    pub fn open_editor(&mut self, i: usize) {
        self.active = Some(i);
        if !self.selected.contains(&i) {
            self.selected.clear();
            self.selected.insert(i);
        }
        let path = self.frames[i].path.clone();
        if self.editor.path.as_ref() != Some(&path) {
            self.editor.load(&path, self.frames[i].sc.settings.clone());
            let p = path.clone();
            self.jobs.spawn(move || {
                let result = decode::load(&p, Some(jobs::EDITOR_SIZE))
                    .map(Arc::new)
                    .map_err(|e| format!("{e:#}"));
                JobResult::Editor { path: p, result }
            });
        }
        self.request_proxy(i);
        self.view = View::Editor;
    }

    pub fn choose_folder(&mut self) {
        let mut d = rfd::FileDialog::new().set_title("Open a folder of negative scans");
        if let Some(f) = &self.folder {
            d = d.set_directory(f);
        }
        if let Some(p) = d.pick_folder() {
            self.open_folder(&p);
        }
    }

    fn toasts_ui(&mut self, ctx: &egui::Context) {
        let now = self.time;
        self.toasts.retain(|t| t.until > now);
        if self.toasts.is_empty() {
            return;
        }
        ctx.request_repaint_after(std::time::Duration::from_millis(500));
        egui::Area::new(egui::Id::new("toasts"))
            .anchor(egui::Align2::RIGHT_BOTTOM, [-16.0, -48.0])
            .show(ctx, |ui| {
                for t in self.toasts.iter().rev().take(5) {
                    egui::Frame::popup(ui.style()).show(ui, |ui| {
                        ui.set_max_width(420.0);
                        let c = if t.error {
                            egui::Color32::from_rgb(200, 50, 40)
                        } else {
                            ui.visuals().text_color()
                        };
                        ui.label(egui::RichText::new(&t.text).color(c));
                    });
                }
            });
    }
}

pub fn to_color_image(px: &[[f32; 3]], w: usize, h: usize) -> egui::ColorImage {
    let mut rgb = Vec::with_capacity(px.len() * 3);
    for p in px {
        for v in p {
            rgb.push((v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8);
        }
    }
    egui::ColorImage::from_rgb([w, h], &rgb)
}

impl eframe::App for NlpApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.time = ctx.input(|i| i.time);
        // Drag & drop of a folder or image files anywhere in the window.
        let dropped: Vec<PathBuf> = ctx.input(|i| {
            i.raw
                .dropped_files
                .iter()
                .filter_map(|f| f.path.clone())
                .collect()
        });
        if let Some(first) = dropped.first() {
            if first.is_dir() {
                let d = first.clone();
                self.open_folder(&d);
            } else {
                self.open_files(dropped);
            }
        }
        self.poll_jobs();
        self.render_thumbs(ctx);
        self.autosave();
        if self.jobs.busy() > 0 {
            ctx.request_repaint_after(std::time::Duration::from_millis(150));
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        match self.view {
            View::Folder => self.folder_ui(ui),
            View::Editor => self.editor_ui(ui),
        }
        self.toasts_ui(&ctx);
        if ctx.input(|i| i.viewport().close_requested()) {
            self.save_all();
        }
    }
}

pub fn run(open: Option<PathBuf>) -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1480.0, 940.0])
            .with_min_inner_size([980.0, 640.0])
            .with_title("NLP Studio"),
        ..Default::default()
    };
    eframe::run_native(
        "NLP Studio",
        options,
        Box::new(move |cc| Ok(Box::new(NlpApp::new(cc, open)))),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::curve::ChannelOrigin;
    use crate::engine::pipeline::{Analysis, AutoWb};

    #[test]
    fn converted_frame_base_is_fixed_on_crop() {
        let mut frame = Frame {
            path: PathBuf::from("test.dng"),
            name: "test.dng".into(),
            sc: Sidecar {
                settings: Settings::default(),
                analysis: Some(Analysis {
                    origins: [ChannelOrigin { white: 10.0, black: 240.0, mean: 0.5 }; 3],
                    auto_wb: AutoWb::default(),
                    base: [0.85, 0.60, 0.40],
                    key: Settings::default().analysis_key(),
                }),
                roll: None,
                metadata: Default::default(),
                version: 1,
            },
            proxy: ProxyState::NotLoaded,
            thumb_src: None,
            thumb: None,
            thumb_dirty: false,
            dirty_since: None,
            exporting: false,
            source_auto: false,
        };

        let dummy_src = SourceImage {
            width: 10,
            height: 10,
            data: vec![[0.1, 0.1, 0.1]; 100],
            to_melissa: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            info: Default::default(),
            as_shot_wb: [1.0; 3],
            is_raw: false,
        };

        // Initially base is the analysis base
        assert_eq!(frame.base_for(&dummy_src), [0.85, 0.60, 0.40]);

        // Change crop
        frame.sc.settings.crop = [10.0, 10.0, 10.0, 10.0];
        // The base must REMAIN the fixed analysis base
        assert_eq!(frame.base_for(&dummy_src), [0.85, 0.60, 0.40]);
    }
}

#[cfg(test)]
mod ui_tests {
    use super::*;
    use egui_kittest::Harness;

    fn settle(h: &mut Harness<'_, NlpApp>, secs: f32) {
        let t0 = std::time::Instant::now();
        loop {
            h.step();
            let busy = h.state().jobs.busy() > 0
                || h.state()
                    .frames
                    .iter()
                    .any(|f| f.thumb_dirty && f.thumb_src.is_some())
                || h.state().editor.loading;
            if (!busy && t0.elapsed().as_secs_f32() > 0.5) || t0.elapsed().as_secs_f32() > secs {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        h.run_steps(3);
    }

    fn shot(h: &mut Harness<'_, NlpApp>, out: &Path, name: &str) {
        let img = h.render().expect("render");
        img.save(out.join(format!("{name}.png"))).unwrap();
    }

    /// Renders screenshots of both views. Run with
    /// `NLP_UI_TEST_DIR=<folder> NLP_UI_OUT=<folder> cargo test ui_screens -- --ignored`.
    #[test]
    #[ignore]
    fn ui_screens() {
        let dir = PathBuf::from(std::env::var("NLP_UI_TEST_DIR").expect("NLP_UI_TEST_DIR"));
        let out = PathBuf::from(std::env::var("NLP_UI_OUT").expect("NLP_UI_OUT"));
        let mut h = Harness::builder()
            .with_size(egui::vec2(1480.0, 940.0))
            .wgpu()
            .build_eframe(|cc| NlpApp::new(cc, Some(dir.clone())));
        settle(&mut h, 60.0);
        shot(&mut h, &out, "1_folder_unconverted");
        {
            let app = h.state_mut();
            app.selected = (0..app.frames.len()).collect();
            app.active = Some(0);
            let idx = app.sel_vec();
            app.convert(idx, false);
        }
        settle(&mut h, 60.0);
        shot(&mut h, &out, "2_folder_converted");
        {
            let app = h.state_mut();
            let i = app
                .frames
                .iter()
                .position(|f| f.name.ends_with(".dng"))
                .unwrap_or(0);
            app.selected = [i].into_iter().collect();
            app.active = Some(i);
        }
        h.run_steps(3);
        shot(&mut h, &out, "3_folder_edit_tab");
        {
            let app = h.state_mut();
            let i = app.active.unwrap();
            app.open_editor(i);
        }
        settle(&mut h, 60.0);
        shot(&mut h, &out, "4_editor_edit");
        {
            let app = h.state_mut();
            let i = app.active.unwrap();
            app.frames[i].sc.settings.rotation = 3;
            app.frames[i].sc.settings.tone_profile =
                crate::engine::settings::ToneProfile::LogarithmicRich;
            app.editor.invalidate();
        }
        settle(&mut h, 30.0);
        shot(&mut h, &out, "5_editor_rotated");
        {
            let app = h.state_mut();
            let i = app.active.unwrap();
            app.enter_crop(i);
            app.editor.set_pending_crop([6.0, 4.0, 5.0, 8.0], 2.5);
        }
        settle(&mut h, 30.0);
        shot(&mut h, &out, "6_editor_crop");
    }

    /// Regression: hovering the sidebars must not make them grow.
    #[test]
    #[ignore]
    fn sidebars_do_not_grow() {
        let dir = PathBuf::from(std::env::var("NLP_UI_TEST_DIR").expect("NLP_UI_TEST_DIR"));
        let mut h = Harness::builder()
            .with_size(egui::vec2(1480.0, 940.0))
            .build_eframe(|cc| NlpApp::new(cc, Some(dir.clone())));
        settle(&mut h, 60.0);
        let width = |h: &Harness<'_, NlpApp>, id: &str| {
            egui::containers::panel::PanelState::load(&h.ctx, egui::Id::new(id))
                .map(|s| s.size().x)
                .unwrap_or(-1.0)
        };
        // Editor sidebar, every tab and colour sub-tab.
        h.state_mut().open_editor(0);
        settle(&mut h, 60.0);
        for tab in [
            editor::Tab::Convert,
            editor::Tab::Edit,
            editor::Tab::Advanced,
        ] {
            h.state_mut().editor.set_tab(tab);
            h.run_steps(3);
            let w0 = width(&h, "editor_sidebar");
            for k in 0..60 {
                h.hover_at(egui::pos2(1300.0, 150.0 + (k % 25) as f32 * 30.0));
                h.step();
            }
            let w1 = width(&h, "editor_sidebar");
            eprintln!("editor sidebar {w0} -> {w1}");
            assert!(
                w0 > 0.0 && w1 <= w0 + 0.5,
                "editor sidebar grew from {w0} to {w1}"
            );
        }
    }

    #[test]
    #[ignore]
    fn single_file_and_tones_dropdown() {
        use egui_kittest::kittest::Queryable;
        let dir = PathBuf::from(std::env::var("NLP_UI_TEST_DIR").expect("NLP_UI_TEST_DIR"));
        let file = dir.join("testphoto.dng");
        let mut h = Harness::builder()
            .with_size(egui::vec2(1480.0, 940.0))
            .build_eframe(|cc| NlpApp::new(cc, Some(file.clone())));
        settle(&mut h, 60.0);
        assert_eq!(h.state().frames.len(), 1, "only the chosen file is opened");
        assert!(
            h.state().view == View::Editor,
            "single file goes straight to the editor"
        );
        let label = h.state().frames[0].sc.settings.tone_profile.label();
        h.get_by_value(label).click();
        h.run_steps(4);
        assert!(
            h.query_by_label("Cinematic - Rich").is_some(),
            "Tones dropdown should be open"
        );
        h.get_by_label("Cinematic - Rich").click();
        h.run_steps(4);
        assert_eq!(
            h.state().frames[0].sc.settings.tone_profile,
            crate::engine::settings::ToneProfile::LogarithmicRich
        );
    }

    #[test]
    #[ignore]
    fn welcome_screen() {
        let out = PathBuf::from(std::env::var("NLP_UI_OUT").expect("NLP_UI_OUT"));
        let mut h = Harness::builder()
            .with_size(egui::vec2(1200.0, 800.0))
            .wgpu()
            .build_eframe(|cc| NlpApp::new(cc, None));
        h.run_steps(3);
        assert!(h.state().frames.is_empty());
        shot(&mut h, &out, "0_welcome");
    }

    #[test]
    #[ignore]
    fn single_image_back_goes_to_welcome() {
        let dir = PathBuf::from(std::env::var("NLP_UI_TEST_DIR").expect("NLP_UI_TEST_DIR"));
        let mut h = Harness::builder()
            .with_size(egui::vec2(1200.0, 800.0))
            .build_eframe(|cc| NlpApp::new(cc, Some(dir.join("testphoto.dng"))));
        settle(&mut h, 60.0);
        assert!(h.state().view == View::Editor);
        // Click the toolbar's back icon (top-left).
        let pos = egui::pos2(29.0, 27.0);
        h.hover_at(pos);
        h.step();
        h.input_mut().events.push(egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: Default::default(),
        });
        h.step();
        h.input_mut().events.push(egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: Default::default(),
        });
        h.run_steps(3);
        assert!(
            h.state().view == View::Folder && h.state().frames.is_empty(),
            "back from a lone image shows the welcome screen"
        );
        // Folder: back returns to the grid.
        h.state_mut().open_folder(&dir);
        settle(&mut h, 60.0);
        h.state_mut().open_editor(0);
        settle(&mut h, 60.0);
        h.key_press(egui::Key::Space);
        h.run_steps(3);
        assert!(h.state().view == View::Folder && !h.state().frames.is_empty());
    }

    #[test]
    #[ignore]
    fn unconverted_opens_convert_tab() {
        let dir = PathBuf::from(std::env::var("NLP_UI_TEST_DIR").expect("NLP_UI_TEST_DIR"));
        let out = PathBuf::from(std::env::var("NLP_UI_OUT").expect("NLP_UI_OUT"));
        let mut h = Harness::builder()
            .with_size(egui::vec2(1480.0, 940.0))
            .wgpu()
            .build_eframe(|cc| NlpApp::new(cc, Some(dir.clone())));
        settle(&mut h, 60.0);
        h.state_mut().unconvert(&[1]);
        h.state_mut().open_editor(1);
        settle(&mut h, 60.0);
        shot(&mut h, &out, "7_editor_unconverted");
        h.state_mut().convert(vec![1], false);
        settle(&mut h, 60.0);
        shot(&mut h, &out, "8_editor_after_convert");
    }

    #[test]
    #[ignore]
    fn double_click_zooms_into_point() {
        let dir = PathBuf::from(std::env::var("NLP_UI_TEST_DIR").expect("NLP_UI_TEST_DIR"));
        let out = PathBuf::from(std::env::var("NLP_UI_OUT").expect("NLP_UI_OUT"));
        let mut h = Harness::builder()
            .with_size(egui::vec2(1480.0, 940.0))
            .wgpu()
            .build_eframe(|cc| NlpApp::new(cc, Some(dir.join("testphoto.dng"))));
        settle(&mut h, 60.0);
        // Double-click on the bush (upper right of the image).
        let pos = egui::pos2(900.0, 250.0);
        h.hover_at(pos);
        h.state_mut().editor.double_click_at(pos);
        h.step();
        std::thread::sleep(std::time::Duration::from_millis(80));
        h.step();
        shot(&mut h, &out, "9_zoom_mid_animation");
        for _ in 0..40 {
            std::thread::sleep(std::time::Duration::from_millis(10));
            h.step();
        }
        shot(&mut h, &out, "9_zoom_done");
        assert!(h.state().editor.is_zoomed(), "double-click zooms in");
    }
}
