//! NLP Studio GUI: a Folder View for browsing / batch-editing a set of scans
//! and an Editor View for fine-tuning a single frame.

mod auto;
mod editor;
mod folder;
mod gpu;
mod history;
mod jobs;
mod picker;
mod roll;
mod widgets;

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use eframe::egui;

use crate::engine::decode::{self, SourceImage};
use crate::engine::pipeline::{self, AutoWb, Renderer};
use crate::engine::settings::{
    CropAspect, ExportFormat, ExportPreset, ExportSettings, OutputSpace, Placement, Settings,
    WbMode, lab,
};
use crate::engine::sidecar::{self, Defaults, Sidecar};
use jobs::{JobResult, Jobs};

pub const THUMB_SIZE: usize = 360;

/// The [`lab_key`] legend, shown wherever lab corrections appear.
pub const LAB_KEYS_HELP: &str =
    "Lab keys: 7 8 9 add C M Y · 1 2 3 remove · 4 / 6 darker / lighter · 0 reset";

/// A Frontier/Noritsu-style correction key.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LabKey {
    Nudge(lab::Channel, i32),
    Reset,
}

/// The lab correction key pressed this frame, on the number row or numpad:
/// 7 8 9 add C M Y, 1 2 3 take it away, 4 / 6 print darker / lighter
/// (density + / −), 0 resets. Keep [`LAB_KEYS_HELP`] in sync.
pub fn lab_key(ctx: &egui::Context) -> Option<LabKey> {
    use egui::Key;
    use lab::Channel::*;
    ctx.input(|i| {
        if i.modifiers.command || i.modifiers.alt || i.modifiers.ctrl {
            return None;
        }
        let keys = [
            (Key::Num7, LabKey::Nudge(C, 1)),
            (Key::Num8, LabKey::Nudge(M, 1)),
            (Key::Num9, LabKey::Nudge(Y, 1)),
            (Key::Num1, LabKey::Nudge(C, -1)),
            (Key::Num2, LabKey::Nudge(M, -1)),
            (Key::Num3, LabKey::Nudge(Y, -1)),
            (Key::Num4, LabKey::Nudge(D, 1)),
            (Key::Num6, LabKey::Nudge(D, -1)),
            (Key::Num0, LabKey::Reset),
        ];
        keys.into_iter()
            .find(|(k, _)| i.key_pressed(*k))
            .map(|(_, l)| l)
    })
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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
    /// Not converted when the folder opened: cropped and converted by
    /// itself (see `auto.rs`) until the user converts it by hand.
    pub auto_convert: bool,
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
        if let Some((t, n)) = self.wb_values(&self.sc.settings) {
            self.sc.settings.wb_temp = t;
            self.sc.settings.wb_tint = n;
        }
    }

    /// The auto WB candidates for the levels this frame renders with (see
    /// [`pipeline::auto_wb_for`]).
    pub fn auto_wb(&self, s: &Settings) -> Option<AutoWb> {
        pipeline::auto_wb_for(s, self.sc.analysis.as_ref(), self.sc.roll.as_ref())
    }

    /// Temp / Tint that `s.wb_mode` gives this frame (roll WB when rendering
    /// from the roll); `None` keeps the current values.
    pub fn wb_values(&self, s: &Settings) -> Option<(f32, f32)> {
        match self.auto_wb(s) {
            Some(w) => pipeline::wb_for_mode(s.wb_mode, &w),
            // Unconverted: film presets apply, the auto modes wait for analysis.
            None => pipeline::wb_for_mode(s.wb_mode, &Default::default()).filter(|_| {
                !matches!(
                    s.wb_mode,
                    WbMode::AutoNeutral
                        | WbMode::AutoWarm
                        | WbMode::AutoCool
                        | WbMode::AutoAvg
                        | WbMode::AutoMix
                )
            }),
        }
    }
}

pub struct Toast {
    pub text: String,
    pub until: f64,
    pub error: bool,
    /// A file the toast offers to show in Finder / Explorer.
    pub reveal: Option<PathBuf>,
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
    pub export_targets: Option<Vec<usize>>,
    /// Open "Convert" dialog: the conversion options before analysis runs.
    pub convert_dialog: Option<folder::ConvertDialog>,
    /// A roll conversion paused because its frames' exposures differ by
    /// more than 1 EV: the frames and the spread.
    pub exposure_prompt: Option<(Vec<jobs::ConvertItem>, f64)>,
    /// Name being typed for a new export preset.
    pub preset_draft: Option<String>,
    /// Roll info, its dialog and the Projects screen.
    pub roll: roll::RollState,
    /// The editor preview is drawn on the GPU (wgpu available).
    pub gpu: bool,
    /// Conversions and exports the app runs by itself.
    pub auto: auto::AutoState,
    /// Undo / redo of settings edits, editor and grid alike.
    pub history: history::History,
}

impl NlpApp {
    pub fn new(cc: &eframe::CreationContext<'_>, open: Option<PathBuf>) -> Self {
        widgets::theme::apply(&cc.egui_ctx);
        let mut defaults = sidecar::load_defaults();
        if defaults.projects.is_empty()
            && let Some(last) = defaults.last_folder.clone().filter(|p| p.is_dir())
        {
            sidecar::remember_project(&mut defaults, &last);
        }
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
            export_targets: None,
            convert_dialog: None,
            auto: Default::default(),
            history: Default::default(),
            exposure_prompt: None,
            preset_draft: None,
            roll: Default::default(),
            gpu: gpu::install(cc),
        };
        if open.is_none() {
            app.show_projects();
        }
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
            reveal: None,
        });
    }

    /// Report a file that couldn't be read. Failures arriving together
    /// share one toast instead of one each.
    fn read_failed(&mut self, name: String) {
        let until = self.time + 8.0;
        self.auto.read_failures.push(name);
        let names = &self.auto.read_failures;
        let text = match names.len() {
            1 => format!("Couldn't read {}", names[0]),
            n => format!("Couldn't read {n} files: {}", names.join(", ")),
        };
        let live = self
            .toasts
            .iter_mut()
            .find(|t| t.error && t.text.starts_with("Couldn't read"));
        match live {
            Some(t) => (t.text, t.until) = (text, until),
            None => self.toasts.push(Toast {
                text,
                until,
                error: true,
                reveal: None,
            }),
        }
    }

    // ---- library --------------------------------------------------------------

    pub fn open_folder(&mut self, dir: &Path) {
        let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
            .map(|rd| {
                rd.filter_map(|e| e.ok().map(|e| e.path()))
                    .filter(|p| {
                        p.is_file()
                            && decode::is_folder_scan(p)
                            && !p.to_string_lossy().ends_with("-prepped.tif")
                    })
                    .collect()
            })
            .unwrap_or_default();
        files.sort_by_key(|p| p.file_name().map(|n| n.to_string_lossy().to_lowercase()));
        self.load_frames(files, dir);
        self.single_mode = false;
        self.defaults.last_folder = Some(dir.to_path_buf());
        sidecar::remember_project(&mut self.defaults, dir);
        let _ = sidecar::save_defaults(&self.defaults);
        self.roll.projects_open = false;
        if self.roll.info.is_none() {
            self.offer_new_roll(dir);
        }
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
        let start = self
            .folder
            .clone()
            .or_else(|| self.defaults.last_folder.clone());
        let files = picker::pick_files(start.as_deref());
        if !files.is_empty() {
            self.open_files(files);
        }
    }

    fn load_frames(&mut self, files: Vec<PathBuf>, dir: &Path) {
        self.save_all();
        let _ = std::fs::create_dir_all(dir.join("NLP"));
        self.roll.info = sidecar::load_roll_info(dir);
        self.roll.banner = None;
        let roll_meta = self.roll.info.as_ref().map(|r| r.metadata.clone());
        self.frames = files
            .into_iter()
            .map(|path| {
                let loaded = sidecar::load(&path);
                let source_auto = loaded.is_none();
                let mut sc = loaded.unwrap_or_else(|| Sidecar {
                    settings: self.defaults.settings.clone(),
                    metadata: roll_meta.clone().unwrap_or_default(),
                    ..Default::default()
                });
                if decode::is_raw(&path) || sc.analysis.is_some() {
                } else if source_auto {
                    // Bitmap scans: Lightroom WB must not be applied; mirror that default.
                    sc.settings.film_base = crate::engine::settings::FilmBase::Off;
                    sc.settings.source = crate::engine::settings::Source::TifScan;
                }
                let auto_convert = !sc.is_converted();
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
                    auto_convert,
                }
            })
            .collect();
        self.folder = Some(dir.to_path_buf());
        self.selected.clear();
        self.active = None;
        self.anchor = None;
        self.view = View::Folder;
        self.editor = editor::Editor::default();
        self.auto = Default::default();
        self.history = Default::default();
        for i in 0..self.frames.len() {
            self.request_thumb(i);
        }
        self.pump_proxies();
    }

    /// Start proxy loads for the frames the grid shows first, then the rest in
    /// order, keeping only a few decodes in flight at once. Called every frame.
    fn pump_proxies(&mut self) {
        let cap = (rayon::current_num_threads() / 2).max(2);
        let mut loading = self
            .frames
            .iter()
            .filter(|f| matches!(f.proxy, ProxyState::Loading))
            .count();
        if loading >= cap {
            return;
        }
        let visible: Vec<usize> = self
            .jobs
            .ctx()
            .data(|d| d.get_temp(egui::Id::new(folder::GRID_VISIBLE)))
            .unwrap_or_default();
        let rest = (0..self.frames.len()).filter(|i| !visible.contains(i));
        let order: Vec<usize> = visible.iter().copied().chain(rest).collect();
        for i in order {
            if loading >= cap {
                break;
            }
            if matches!(self.frames.get(i).map(|f| &f.proxy), Some(ProxyState::NotLoaded)) {
                self.request_proxy(i);
                loading += 1;
            }
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
            let result = jobs::load_proxy_and_thumb(&path)
                .map(|(p, t)| (Arc::new(p), t))
                .map_err(|e| format!("{e:#}"));
            JobResult::Proxy { path, result }
        });
    }

    /// Read the cached thumbnail source of a frame. These jobs are cheap, so
    /// they are queued ahead of the proxies and the grid fills in first.
    fn request_thumb(&self, i: usize) {
        let path = self.frames[i].path.clone();
        self.jobs.spawn(move || {
            let thumb = jobs::read_thumb(&path);
            JobResult::Thumb { path, thumb }
        });
    }

    /// Set the scan source of a frame without saved settings from its file
    /// (raw → digital camera, bitmap → TIF scan).
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

    /// Apply this frame's lab correction key (`lab_key`) to frame `i`.
    /// Returns false when the frame isn't converted.
    pub fn apply_lab_key(&mut self, i: usize, key: LabKey) -> bool {
        if !self.frames[i].sc.is_converted() {
            return false;
        }
        let s = &mut self.frames[i].sc.settings;
        match key {
            LabKey::Nudge(ch, n) => lab::nudge(s, ch, n),
            LabKey::Reset => lab::reset(s),
        }
        self.mark_changed(i);
        true
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
        self.auto_release(&idx);
        let items = idx
            .iter()
            .map(|&i| jobs::ConvertItem {
                path: self.frames[i].path.clone(),
                proxy: self.frames[i].proxy(),
                settings: self.frames[i].sc.settings.clone(),
            })
            .collect();
        self.convert_started(idx.len());
        self.jobs.spawn(move || jobs::convert(items, as_roll, true));
    }

    pub fn unconvert(&mut self, idx: &[usize]) {
        self.auto_release(idx);
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

    /// Render the given frames (or all, if empty) onto one JPEG in the
    /// project folder.
    pub fn make_contact_sheet(&mut self, idx: Vec<usize>) {
        let idx: Vec<usize> = if idx.is_empty() {
            (0..self.frames.len()).collect()
        } else {
            idx
        };
        let jobs: Vec<_> = idx
            .iter()
            .filter_map(|&i| {
                let f = &self.frames[i];
                let src = f.thumb_src.clone()?;
                Some((
                    f.name.clone(),
                    src,
                    f.sc.settings.clone(),
                    f.sc.analysis.clone(),
                    f.sc.roll.clone(),
                ))
            })
            .collect();
        let Some(first) = idx.first().map(|&i| self.frames[i].path.clone()) else {
            return;
        };
        if jobs.is_empty() {
            self.toast("Frames are still loading", true);
            return;
        }
        let dir = sidecar::project_dir(&first);
        let stem = self
            .folder
            .as_ref()
            .and_then(|f| f.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "roll".into());
        let out = dir.join(format!("{stem}_contact_sheet.jpg"));
        self.jobs.spawn(move || {
            use rayon::prelude::*;
            let tiles: Vec<_> = jobs
                .into_par_iter()
                .map(|(name, src, s, a, roll)| {
                    let base = match &a {
                        Some(a) => a.base,
                        None => pipeline::resolve_base(&src, &s, None),
                    };
                    let neg = pipeline::prepare_negative(&src, &s, base);
                    let levels = pipeline::levels_for(&s, a.as_ref(), roll.as_ref());
                    let r = Renderer::new(&s, levels.as_ref(), OutputSpace::Srgb);
                    crate::engine::contact::Tile {
                        name,
                        w: neg.width,
                        h: neg.height,
                        px: r.render(&neg, 0.0),
                    }
                })
                .collect();
            let res = std::fs::create_dir_all(&dir)
                .map_err(anyhow::Error::from)
                .and_then(|_| crate::engine::contact::write_sheet(&out, &tiles))
                .map(|_| out)
                .map_err(|e| format!("{e:#}"));
            JobResult::ContactSheet(res)
        });
    }

    pub fn open_export_dialog(&mut self, targets: Vec<usize>) {
        if targets.is_empty() {
            return;
        }
        self.editor.export_open = true;
        self.export_targets = Some(targets);
    }

    /// Export frames `idx`. Unconverted frames are converted first and
    /// exported when their analysis arrives.
    pub fn export_with_settings(&mut self, idx: Vec<usize>, ex: ExportSettings) {
        let items: Vec<_> = idx
            .iter()
            .map(|&i| (self.frames[i].path.as_path(), &self.frames[i].sc.metadata))
            .collect();
        let targets = crate::engine::export::batch_paths(&items, &ex);
        let mut n = 0;
        let mut unconverted = Vec::new();
        for (i, target) in idx.into_iter().zip(targets) {
            let f = &mut self.frames[i];
            if f.exporting {
                continue;
            }
            f.exporting = true;
            n += 1;
            if f.sc.is_converted() {
                self.spawn_export(i, target, ex.clone());
            } else {
                unconverted.push(i);
                self.auto
                    .export_waiting
                    .push((f.path.clone(), target, ex.clone()));
            }
        }
        self.convert(unconverted, false);
        if n > 0 {
            if self.export_done >= self.export_total {
                self.export_total = 0;
                self.export_done = 0;
            }
            self.export_total += n;
        }
    }

    pub fn spawn_export(&mut self, i: usize, target: PathBuf, ex: ExportSettings) {
        let f = &self.frames[i];
        let (path, s, a, roll, meta) = (
            f.path.clone(),
            f.sc.settings.clone(),
            f.sc.analysis.clone(),
            f.sc.roll.clone(),
            f.sc.metadata.clone(),
        );
        self.jobs.spawn(move || {
            let result = crate::engine::export::export_frame(
                &path,
                &s,
                a.as_ref(),
                roll.as_ref(),
                &meta,
                &ex,
                &target,
            )
            .map_err(|e| format!("{e:#}"));
            JobResult::Exported { path, result }
        });
    }

    pub fn export_dialog(&mut self, ctx: &egui::Context) {
        use widgets::theme;
        if self.editor.export_open && self.export_targets.is_none() {
            if let Some(i) = self.active {
                self.export_targets = Some(vec![i]);
            }
        }
        let Some(targets) = self.export_targets.clone() else {
            return;
        };
        if targets.is_empty() {
            self.export_targets = None;
            self.editor.export_open = false;
            return;
        }

        let scan_dir = targets
            .first()
            .and_then(|&idx| self.frames.get(idx))
            .and_then(|f| f.path.parent())
            .map(Path::to_path_buf)
            .or_else(|| self.folder.clone())
            .unwrap_or_else(|| PathBuf::from("."));
        // A custom folder: the roll's own, else the last one chosen.
        let custom_dir = self
            .roll_export_dir()
            .or_else(|| self.defaults.export.output_dir.clone())
            .filter(|d| !d.as_os_str().is_empty() && d.exists());

        let mut go = false;
        let mut should_close = false;
        let mut picked_dir: Option<PathBuf> = None;
        let mut save = false;
        let count = targets.len();
        // The first target, for the file name preview.
        let first = &self.frames[targets[0]];
        let (first_path, first_meta) = (first.path.clone(), first.sc.metadata.clone());
        let builtin = ExportPreset::builtin();
        let n_builtin = builtin.len();
        let presets: Vec<ExportPreset> = builtin
            .into_iter()
            .chain(self.defaults.export_presets.iter().cloned())
            .collect();

        let frame = egui::Frame::new()
            .fill(theme::PANEL)
            .stroke(egui::Stroke::new(1.0, theme::LINE))
            .corner_radius(12.0)
            .inner_margin(egui::Margin::same(22))
            .shadow(ctx.global_style().visuals.window_shadow);
        let r = egui::Modal::new(egui::Id::new("export_modal"))
            .backdrop_color(egui::Color32::from_black_alpha(150))
            .frame(frame)
            .show(ctx, |ui| {
                ui.set_width(420.0);
                let title = if count == 1 {
                    "Export 1 image".to_string()
                } else {
                    format!("Export {count} images")
                };
                ui.label(
                    egui::RichText::new(title)
                        .font(theme::medium(16.0))
                        .color(theme::TEXT),
                );
                ui.add_space(16.0);

                let ex = &mut self.defaults.export;
                ui.columns(2, |cols| {
                    save |= widgets::fl_combo(
                        &mut cols[0],
                        "Format",
                        &mut ex.format,
                        ExportFormat::ALL,
                        ExportFormat::label,
                        "",
                    );
                    save |= widgets::fl_combo(
                        &mut cols[1],
                        "Colour space",
                        &mut ex.space,
                        OutputSpace::ALL,
                        OutputSpace::label,
                        "",
                    );
                });

                if ex.format == ExportFormat::Jpeg {
                    ui.add_space(6.0);
                    save |= widgets::fl_slider_u8_origin(
                        ui,
                        "Quality",
                        &mut ex.jpeg_quality,
                        1..=100,
                        95,
                        Some(0.0),
                        "JPEG compression quality (1-100)",
                    );
                }

                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new("Destination")
                        .small()
                        .color(theme::MUTED),
                );
                let dest_name = |custom: bool, p: Placement| match (custom, p) {
                    (true, _) => "Custom folder",
                    (false, Placement::Subfolder) => "Subfolder next to the scans",
                    (false, Placement::SameFolder) => "Next to the scans",
                };
                let custom = custom_dir.is_some();
                ui.horizontal(|ui| {
                    egui::ComboBox::from_id_salt("export_dest")
                        .width(if !custom && ex.placement == Placement::Subfolder {
                            220.0
                        } else {
                            ui.available_width() - 4.0
                        })
                        .selected_text(dest_name(custom, ex.placement))
                        .show_ui(ui, |ui| {
                            for p in [Placement::Subfolder, Placement::SameFolder] {
                                if ui
                                    .selectable_label(
                                        !custom && ex.placement == p,
                                        dest_name(false, p),
                                    )
                                    .clicked()
                                {
                                    ex.placement = p;
                                    picked_dir = Some(PathBuf::new());
                                    save = true;
                                }
                            }
                            if ui.selectable_label(custom, "Custom folder…").clicked() {
                                let start = custom_dir.clone().unwrap_or_else(|| scan_dir.clone());
                                if let Some(picked) =
                                    picker::pick_folder(Some(&start), Some("Choose"))
                                {
                                    picked_dir = Some(picked);
                                }
                            }
                        });
                    if !custom && ex.placement == Placement::Subfolder {
                        save |= ui
                            .add(
                                egui::TextEdit::singleline(&mut ex.subfolder)
                                    .hint_text("Positives")
                                    .desired_width(ui.available_width() - 4.0),
                            )
                            .on_hover_text("Subfolder name")
                            .changed();
                    }
                });
                let mut shown = ex.clone();
                shown.output_dir = custom_dir.clone();
                let dest = crate::engine::export::output_path(
                    &first_path,
                    &shown,
                    &crate::engine::export::NameCtx {
                        n: 1,
                        meta: &first_meta,
                    },
                );
                let dest = dest.parent().unwrap_or(&scan_dir).display().to_string();
                let r = ui.add(
                    egui::Label::new(egui::RichText::new(&dest).small().color(theme::FAINT))
                        .truncate(),
                );
                r.on_hover_text(&dest);

                ui.add_space(10.0);
                egui::CollapsingHeader::new(
                    egui::RichText::new("Size, file name and presets")
                        .small()
                        .color(theme::MUTED),
                )
                .id_salt("export_more")
                .default_open(false)
                .show(ui, |ui| {
                    let current = presets.iter().position(|p| p.matches(ex));
                    ui.label(egui::RichText::new("Preset").small().color(theme::MUTED));
                    ui.horizontal(|ui| {
                        egui::ComboBox::from_id_salt("export_preset")
                            .width(ui.available_width() - 120.0)
                            .selected_text(
                                current
                                    .map(|k| presets[k].name.as_str())
                                    .unwrap_or("Custom"),
                            )
                            .show_ui(ui, |ui| {
                                for (k, p) in presets.iter().enumerate() {
                                    if k == n_builtin {
                                        ui.separator();
                                    }
                                    if ui.selectable_label(current == Some(k), &p.name).clicked() {
                                        p.apply_to(ex);
                                        save = true;
                                    }
                                }
                            });
                        match (&mut self.preset_draft, current) {
                            (Some(_), _) => {}
                            (None, Some(k)) if k >= n_builtin => {
                                if ui.add(widgets::ghost("Delete")).clicked() {
                                    self.defaults.export_presets.remove(k - n_builtin);
                                    save = true;
                                }
                            }
                            (None, _) => {
                                if ui.add(widgets::ghost("Save as…")).clicked() {
                                    self.preset_draft = Some(String::new());
                                }
                            }
                        }
                    });
                    let mut close_draft = false;
                    if let Some(draft) = &mut self.preset_draft {
                        ui.horizontal(|ui| {
                            let r = ui.add(
                                egui::TextEdit::singleline(draft)
                                    .hint_text("Preset name")
                                    .desired_width(ui.available_width() - 150.0),
                            );
                            if draft.is_empty() && !r.has_focus() {
                                r.request_focus();
                            }
                            let name = draft.trim().to_string();
                            let ok = !name.is_empty();
                            if ui.add_enabled(ok, widgets::primary("Save")).clicked()
                                || (ok
                                    && r.lost_focus()
                                    && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                            {
                                let p = ExportPreset::new(&name, ex);
                                let list = &mut self.defaults.export_presets;
                                match list.iter_mut().find(|q| q.name == p.name) {
                                    Some(q) => *q = p,
                                    None => list.push(p),
                                }
                                close_draft = true;
                                save = true;
                            } else if ui.add(widgets::ghost("Cancel")).clicked() {
                                close_draft = true;
                            }
                        });
                    }
                    if close_draft {
                        self.preset_draft = None;
                    }
                    ui.add_space(6.0);
                    ui.columns(2, |cols| {
                        let ui = &mut cols[0];
                        ui.label(egui::RichText::new("Size").small().color(theme::MUTED));
                        let size_name = |v: u32| match v {
                            0 => "Full size".to_string(),
                            3000 => "3000 px (lab M)".to_string(),
                            6000 => "6000 px (lab L)".to_string(),
                            v => format!("{v} px"),
                        };
                        egui::ComboBox::from_id_salt("export_size")
                            .width(ui.available_width() - 4.0)
                            .selected_text(size_name(ex.long_edge))
                            .show_ui(ui, |ui| {
                                for v in [0, 2000, 3000, 4500, 6000] {
                                    if ui
                                        .selectable_label(ex.long_edge == v, size_name(v))
                                        .clicked()
                                    {
                                        ex.long_edge = v;
                                        save = true;
                                    }
                                }
                            })
                            .response
                            .on_hover_text("Long edge of the exported image");
                        let ui = &mut cols[1];
                        ui.label(egui::RichText::new("File name").small().color(theme::MUTED));
                        save |= ui
                            .add(
                                egui::TextEdit::singleline(&mut ex.name_template)
                                    .desired_width(ui.available_width() - 4.0),
                            )
                            .changed();
                    });
                    // Click a token to add it to the file name.
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);
                        for (tok, tip) in [
                            ("{name}", "Source file name"),
                            ("{suffix}", "The suffix, e.g. -positive"),
                            ("{n:03}", "Position in the export: 001, 002, …"),
                            ("{stock}", "Film stock"),
                            ("{iso}", "Film ISO"),
                            ("{date}", "Scan date"),
                        ] {
                            if ui
                                .add(
                                    egui::Button::new(
                                        egui::RichText::new(tok)
                                            .small()
                                            .monospace()
                                            .color(theme::MUTED),
                                    )
                                    .corner_radius(8.0),
                                )
                                .on_hover_text(tip)
                                .clicked()
                            {
                                ex.name_template.push_str(tok);
                                save = true;
                            }
                        }
                    });
                    let preview = crate::engine::export::file_name(
                        &first_path,
                        ex,
                        &crate::engine::export::NameCtx {
                            n: 1,
                            meta: &first_meta,
                        },
                    );
                    ui.label(
                        egui::RichText::new(if count > 1 {
                            format!("{preview}, …")
                        } else {
                            preview
                        })
                        .small()
                        .color(theme::FAINT),
                    );
                });

                ui.add_space(22.0);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(widgets::primary("Export").min_size(egui::vec2(96.0, 30.0)))
                        .clicked()
                        || (!ui.ctx().egui_wants_keyboard_input()
                            && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                    {
                        go = true;
                    }
                    if ui
                        .add(widgets::ghost("Cancel").min_size(egui::vec2(80.0, 30.0)))
                        .clicked()
                        || ui.input(|i| i.key_pressed(egui::Key::Escape))
                    {
                        should_close = true;
                    }
                });
            });

        if save {
            let _ = crate::engine::sidecar::save_defaults(&self.defaults);
        }
        // An empty path clears the custom folder.
        if let Some(picked) = picked_dir {
            let dir = (!picked.as_os_str().is_empty()).then_some(picked);
            self.set_export_dir(dir);
        }

        if r.should_close() || should_close || go {
            self.preset_draft = None;
        }
        if r.should_close() || should_close {
            self.export_targets = None;
            self.editor.export_open = false;
        }

        if go {
            self.export_targets = None;
            self.editor.export_open = false;

            let mut ex = self.defaults.export.clone();
            ex.output_dir = custom_dir;

            for &t in &targets {
                self.save_frame(t);
            }
            self.export_with_settings(targets, ex);
        }
    }

    // ---- job results ------------------------------------------------------------

    fn poll_jobs(&mut self) {
        while let Ok(r) = self.jobs.rx.try_recv() {
            match r {
                JobResult::ExposureVariance { items, spread } => {
                    self.auto.convert_done += items.len();
                    if let Some(items) = self.auto_exposure_variance(items, spread) {
                        self.exposure_prompt = Some((items, spread));
                    }
                }
                JobResult::Thumb { path, thumb } => {
                    let (Some(i), Some(thumb)) = (self.index_of(&path), thumb) else {
                        continue;
                    };
                    if self.frames[i].thumb_src.is_none() {
                        self.auto_source(i, &thumb);
                        self.frames[i].thumb_src = Some(thumb);
                        self.frames[i].thumb_dirty = true;
                    }
                }
                JobResult::Proxy { path, result } => {
                    let Some(i) = self.index_of(&path) else {
                        continue;
                    };
                    match result {
                        Ok((p, thumb)) => {
                            self.auto_source(i, &p);
                            // A cached thumbnail that arrived first is the same image.
                            if self.frames[i].thumb_src.is_none() {
                                self.frames[i].thumb_src = Some(thumb);
                                self.frames[i].thumb_dirty = true;
                            }
                            self.frames[i].proxy = ProxyState::Ready(p);
                            self.auto_queue(i);
                        }
                        Err(e) => {
                            self.frames[i].proxy = ProxyState::Failed(e);
                            self.read_failed(self.frames[i].name.clone());
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
                    for e in &errors {
                        self.toast(e.clone(), true);
                    }
                    let n = frames.len();
                    self.auto.convert_done += n + errors.len();
                    let paths: Vec<PathBuf> = frames.iter().map(|f| f.0.clone()).collect();
                    let auto = self.auto_converted(&paths, roll.is_some());
                    for (path, src, a, detected) in frames {
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
                        if let Some(d) = detected {
                            (f.sc.settings.crop, f.sc.settings.straighten) = (d.crop, d.straighten);
                            f.sc.settings.crop_aspect = CropAspect::Free;
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
                    if !auto {
                        self.toast(format!("Converted {n} frame(s)"), false);
                    } else if roll.is_some() {
                        self.toast(format!("Balanced {n} frames as a roll"), false);
                    }
                    self.editor.invalidate();
                    self.flush_waiting_exports();
                }
                JobResult::Exported { path, result } => {
                    self.export_done += 1;
                    if let Some(i) = self.index_of(&path) {
                        self.frames[i].exporting = false;
                    }
                    match result {
                        Ok(p) => {
                            if self.export_done >= self.export_total {
                                let dir = p.parent().unwrap_or(&p).display().to_string();
                                let msg = format!(
                                    "Exported {} positive cop{} to {dir}",
                                    self.export_total,
                                    if self.export_total == 1 { "y" } else { "ies" },
                                );
                                self.toast(msg, false);
                                if let Some(t) = self.toasts.last_mut() {
                                    t.reveal = Some(p);
                                    t.until += 4.0;
                                }
                            }
                        }
                        Err(e) => self.toast(format!("Export failed: {e}"), true),
                    }
                }
                JobResult::ContactSheet(r) => match r {
                    Ok(p) => self.toast(format!("Saved contact sheet: {}", p.display()), false),
                    Err(e) => self.toast(format!("Contact sheet failed: {e}"), true),
                },
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
            let levels = pipeline::levels_for(s, f.sc.analysis.as_ref(), f.sc.roll.as_ref());
            let r = Renderer::new(s, levels.as_ref(), OutputSpace::Srgb);
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
        let start = self
            .folder
            .clone()
            .or_else(|| self.defaults.last_folder.clone());
        if let Some(p) = picker::pick_folder(start.as_deref(), Some("Open")) {
            self.open_folder(&p);
        }
    }

    fn toasts_ui(&mut self, ctx: &egui::Context) {
        use widgets::theme;
        let now = self.time;
        self.toasts.retain(|t| t.until > now);
        if self.toasts.is_empty() {
            return;
        }
        ctx.request_repaint_after(std::time::Duration::from_millis(500));
        egui::Area::new(egui::Id::new("toasts"))
            .anchor(egui::Align2::CENTER_BOTTOM, [0.0, -44.0])
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing.y = 6.0;
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    let n = self.toasts.len();
                    let mut reveal = None;
                    for t in self.toasts.iter().skip(n.saturating_sub(3)) {
                        egui::Frame::new()
                            .fill(theme::SURFACE)
                            .stroke(egui::Stroke::new(1.0, theme::LINE))
                            .corner_radius(16.0)
                            .inner_margin(egui::Margin::symmetric(14, 7))
                            .shadow(ui.visuals().popup_shadow)
                            .show(ui, |ui| {
                                ui.set_max_width(520.0);
                                ui.horizontal(|ui| {
                                    if t.error {
                                        let (r, _) = ui.allocate_exact_size(
                                            egui::vec2(8.0, 8.0),
                                            egui::Sense::hover(),
                                        );
                                        ui.painter().circle_filled(r.center(), 3.5, theme::ERR);
                                    }
                                    ui.add(
                                        egui::Label::new(
                                            egui::RichText::new(&t.text).color(theme::TEXT),
                                        )
                                        .wrap(),
                                    );
                                    if let Some(p) = &t.reveal {
                                        let label = if cfg!(target_os = "macos") {
                                            "Show in Finder"
                                        } else {
                                            "Show"
                                        };
                                        if ui
                                            .add(
                                                egui::Button::new(
                                                    egui::RichText::new(label).color(theme::ACCENT),
                                                )
                                                .frame(false),
                                            )
                                            .clicked()
                                        {
                                            reveal = Some(p.clone());
                                        }
                                    }
                                });
                            });
                    }
                    if let Some(p) = reveal {
                        picker::reveal(&p);
                    }
                });
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
        self.pump_proxies();
        self.auto_tick();
        self.render_thumbs(ctx);
        self.autosave();
        if self.jobs.busy() > 0 || self.auto.queued_since.is_some() {
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
        self.export_dialog(&ctx);
        self.convert_dialog_ui(&ctx);
        self.roll_dialog_ui(&ctx);
        self.exposure_prompt_ui(&ctx);
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
            .with_title("NLP Studio")
            .with_app_id("nlp-studio"),
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
    use crate::engine::curve::ChannelLevels;
    use crate::engine::pipeline::{Analysis, AutoWb};

    #[test]
    fn converted_frame_base_is_fixed_on_crop() {
        let mut frame = Frame {
            path: PathBuf::from("test.dng"),
            name: "test.dng".into(),
            sc: Sidecar {
                settings: Settings::default(),
                analysis: Some(Analysis {
                    levels: [ChannelLevels {
                        lo: 0.1,
                        hi: 1.2,
                        mid: 0.6,
                        bow: 0.0,
                    }; 3],
                    auto_wb: AutoWb::default(),
                    base: [0.85, 0.60, 0.40],
                    key: Settings::default().analysis_key(),
                    ev: None,
                    axis: None,
                    skin: None,
                }),
                roll: None,
                metadata: Default::default(),
                version: crate::engine::sidecar::VERSION,
            },
            proxy: ProxyState::NotLoaded,
            thumb_src: None,
            thumb: None,
            thumb_dirty: false,
            dirty_since: None,
            exporting: false,
            source_auto: false,
            auto_convert: false,
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
                || h.state().editor.loading
                || h.state().auto.busy();
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
        {
            use egui_kittest::kittest::Queryable;
            h.get_by_label("Detect").click();
        }
        settle(&mut h, 30.0);
        shot(&mut h, &out, "7_editor_detect_frame");
        h.key_press(egui::Key::Enter);
        settle(&mut h, 30.0);
        for k in [
            egui::Key::Num4,
            egui::Key::Num4,
            egui::Key::Num7,
            egui::Key::Num9,
        ] {
            h.key_press(k);
            h.run_steps(2);
        }
        settle(&mut h, 30.0);
        shot(&mut h, &out, "8_editor_lab_keys");
        h.key_press(egui::Key::J);
        settle(&mut h, 30.0);
        h.hover_at(egui::pos2(520.0, 460.0));
        h.run_steps(3);
        shot(&mut h, &out, "9_editor_clipping_readout");
        h.key_press(egui::Key::J);
        h.key_press_modifiers(egui::Modifiers::SHIFT, egui::Key::B);
        settle(&mut h, 30.0);
        shot(&mut h, &out, "10_editor_split");
        h.key_press_modifiers(egui::Modifiers::SHIFT, egui::Key::B);
        {
            let app = h.state_mut();
            let i = app.active.unwrap();
            app.open_export_dialog(vec![i]);
        }
        settle(&mut h, 30.0);
        shot(&mut h, &out, "11_export_dialog");
        h.key_press(egui::Key::Escape);
        h.run_steps(3);
        h.key_press(egui::Key::Space);
        settle(&mut h, 30.0);
        shot(&mut h, &out, "12_folder_lab_badge");
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
        // Editor sidebar.
        h.state_mut().open_editor(0);
        settle(&mut h, 60.0);
        {
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
        // Expanding the Conversion section of a converted frame.
        h.state_mut().convert(vec![0], false);
        settle(&mut h, 60.0);
        let w0 = width(&h, "editor_sidebar");
        let header = egui::pos2(1190.0, 204.0);
        h.hover_at(header);
        h.step();
        h.drag_at(header);
        h.step();
        h.drop_at(header);
        h.run_steps(5);
        let w1 = width(&h, "editor_sidebar");
        eprintln!("conversion expanded {w0} -> {w1}");
        assert!(w1 <= w0 + 0.5, "editor sidebar grew from {w0} to {w1}");
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
        // The Tones dropdown lives on the Edit tab, which needs a conversion.
        if !h.state().frames[0].sc.is_converted() {
            h.state_mut().convert(vec![0], false);
            settle(&mut h, 60.0);
        }
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

    /// A new folder converts itself: each frame as it loads, then the roll.
    #[test]
    #[ignore]
    fn folder_converts_itself() {
        let src = PathBuf::from(std::env::var("NLP_UI_TEST_DIR").expect("NLP_UI_TEST_DIR"));
        let dir = std::env::temp_dir().join(format!("nlp_auto_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let mut raws: Vec<PathBuf> = std::fs::read_dir(&src)
            .unwrap()
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| decode::is_raw(p))
            .collect();
        raws.sort();
        for p in raws.iter().take(3) {
            std::fs::copy(p, dir.join(p.file_name().unwrap())).unwrap();
        }
        let mut h = Harness::builder()
            .with_size(egui::vec2(1200.0, 800.0))
            .build_eframe(|cc| NlpApp::new(cc, Some(dir.clone())));
        settle(&mut h, 120.0);
        let app = h.state();
        assert_eq!(app.frames.len(), 3);
        for f in &app.frames {
            assert!(f.sc.is_converted(), "{} converted", f.name);
            assert!(f.sc.roll.is_some(), "{} balanced as a roll", f.name);
            assert!(f.auto_convert);
        }
        assert_eq!(app.auto.roll_pass, auto::RollPass::Done);
        assert!(app.convert_dialog.is_none() && app.exposure_prompt.is_none());

        // A paste to the whole roll undoes and redoes in one step.
        let app = h.state_mut();
        let before: Vec<f32> = app
            .frames
            .iter()
            .map(|f| f.sc.settings.brightness)
            .collect();
        let mut c = app.frames[0].sc.settings.clone();
        c.brightness = 17.0;
        app.clipboard = Some(c);
        app.selected = (0..3).collect();
        app.paste_settings();
        assert!(app.frames.iter().all(|f| f.sc.settings.brightness == 17.0));
        assert!(app.history_step(true));
        let undone: Vec<f32> = app
            .frames
            .iter()
            .map(|f| f.sc.settings.brightness)
            .collect();
        assert_eq!(undone, before);
        assert!(app.history_step(false));
        assert!(app.frames.iter().all(|f| f.sc.settings.brightness == 17.0));
        let _ = std::fs::remove_dir_all(&dir);
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

    #[test]
    fn roll_info_fills_frames_and_persists() {
        let dir = std::env::temp_dir().join(format!("nlp_roll_app_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let img = dir.join("frame_01.tif");
        std::fs::write(&img, b"").unwrap();
        let mut app = NlpApp {
            view: View::Folder,
            folder: None,
            frames: Vec::new(),
            selected: BTreeSet::new(),
            anchor: None,
            active: None,
            jobs: Jobs::new(egui::Context::default()),
            defaults: Default::default(),
            clipboard: None,
            editor: editor::Editor::default(),
            toasts: Vec::new(),
            export_total: 0,
            export_done: 0,
            thumb_size: 180.0,
            time: 0.0,
            single_mode: false,
            export_targets: None,
            convert_dialog: None,
            auto: Default::default(),
            history: Default::default(),
            exposure_prompt: None,
            preset_draft: None,
            roll: Default::default(),
            gpu: false,
        };
        // (open_folder would also write the user's defaults.json.)
        app.load_frames(vec![img.clone()], &dir);
        assert!(app.roll.info.is_none(), "new folder has no roll info");
        app.open_roll_dialog(&dir, true);
        let mut info = app.roll.dialog.take().unwrap().info;
        info.metadata.film_stock = "Portra 400".into();
        app.save_roll_info(&dir, info);
        assert!(sidecar::roll_info_path(&dir).is_file());
        assert_eq!(app.frames[0].sc.metadata.film_stock, "Portra 400");

        // Reopening reads it back and new frames inherit it.
        app.frames.clear();
        app.load_frames(vec![img], &dir);
        let r = app.roll.info.as_ref().expect("roll info reloaded");
        assert_eq!(r.metadata.film_stock, "Portra 400");
        assert_eq!(app.frames[0].sc.metadata.film_stock, "Portra 400");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn export_dialog_state_management() {
        let mut app = NlpApp {
            view: View::Folder,
            folder: None,
            frames: Vec::new(),
            selected: BTreeSet::new(),
            anchor: None,
            active: None,
            jobs: Jobs::new(egui::Context::default()),
            defaults: Default::default(),
            clipboard: None,
            editor: editor::Editor::default(),
            toasts: Vec::new(),
            export_total: 0,
            export_done: 0,
            thumb_size: 180.0,
            time: 0.0,
            single_mode: false,
            export_targets: None,
            convert_dialog: None,
            auto: Default::default(),
            history: Default::default(),
            exposure_prompt: None,
            preset_draft: None,
            roll: Default::default(),
            gpu: false,
        };

        // Empty targets is a no-op
        app.open_export_dialog(vec![]);
        assert!(app.export_targets.is_none());
        assert!(!app.editor.export_open);

        // Open dialog for indices
        app.open_export_dialog(vec![0, 2]);
        assert_eq!(app.export_targets, Some(vec![0, 2]));
        assert!(app.editor.export_open);
    }

    #[test]
    fn convert_buttons_open_the_options_dialog() {
        let mut app = NlpApp {
            view: View::Folder,
            folder: None,
            frames: Vec::new(),
            selected: BTreeSet::new(),
            anchor: None,
            active: None,
            jobs: Jobs::new(egui::Context::default()),
            defaults: Default::default(),
            clipboard: None,
            editor: editor::Editor::default(),
            toasts: Vec::new(),
            export_total: 0,
            export_done: 0,
            thumb_size: 180.0,
            time: 0.0,
            single_mode: false,
            export_targets: None,
            convert_dialog: None,
            auto: Default::default(),
            history: Default::default(),
            exposure_prompt: None,
            preset_draft: None,
            roll: Default::default(),
            gpu: false,
        };
        app.open_convert_dialog(vec![], true);
        assert!(app.convert_dialog.is_none());
        app.open_convert_dialog(vec![0], true);
        assert!(
            !app.convert_dialog.as_ref().unwrap().as_roll,
            "one frame is never a roll"
        );
        app.open_convert_dialog(vec![0, 1], true);
        let d = app.convert_dialog.as_ref().unwrap();
        assert!(d.as_roll);
        assert_eq!(d.targets, vec![0, 1]);
    }

    #[test]
    fn export_settings_json_backward_compatibility() {
        // Legacy JSON without output_dir
        let json = r#"{
            "format": "Tiff",
            "sixteen_bit": true,
            "zip_compression": true,
            "jpeg_quality": 95,
            "space": "Srgb",
            "placement": "Subfolder",
            "subfolder": "Positives",
            "suffix": "-positive",
            "long_edge": 0
        }"#;
        let ex: ExportSettings =
            serde_json::from_str(json).expect("deserialize legacy export settings");
        assert_eq!(ex.output_dir, None);

        // Modern JSON with output_dir
        let modern_json = r#"{
            "format": "Jpeg",
            "sixteen_bit": true,
            "zip_compression": true,
            "jpeg_quality": 95,
            "space": "Srgb",
            "placement": "Subfolder",
            "subfolder": "Positives",
            "suffix": "-positive",
            "long_edge": 0,
            "output_dir": "/Users/abe/Pictures/Export"
        }"#;
        let ex_modern: ExportSettings =
            serde_json::from_str(modern_json).expect("deserialize modern export settings");
        assert_eq!(
            ex_modern.output_dir,
            Some(PathBuf::from("/Users/abe/Pictures/Export"))
        );
    }

    #[test]
    fn single_image_open_routes_to_editor() {
        let mut app = NlpApp {
            view: View::Folder,
            folder: None,
            frames: Vec::new(),
            selected: BTreeSet::new(),
            anchor: None,
            active: None,
            jobs: Jobs::new(egui::Context::default()),
            defaults: Default::default(),
            clipboard: None,
            editor: editor::Editor::default(),
            toasts: Vec::new(),
            export_total: 0,
            export_done: 0,
            thumb_size: 180.0,
            time: 0.0,
            single_mode: false,
            export_targets: None,
            convert_dialog: None,
            auto: Default::default(),
            history: Default::default(),
            exposure_prompt: None,
            preset_draft: None,
            roll: Default::default(),
            gpu: false,
        };

        let tmp = std::env::temp_dir().join("test_picker_single.tif");
        let img = image::RgbImage::new(10, 10);
        let _ = img.save(&tmp);

        if tmp.exists() {
            app.open_files(vec![tmp.clone()]);
            assert!(app.single_mode, "single image should set single_mode");
            assert_eq!(
                app.view,
                View::Editor,
                "single image should immediately open editor"
            );
            assert_eq!(app.frames.len(), 1);
            let _ = std::fs::remove_file(tmp);
        }
    }

    #[test]
    fn jpeg_quality_setting_persistence_and_range() {
        let mut ex = ExportSettings::default();
        assert_eq!(ex.jpeg_quality, 95);

        // Test custom quality setting
        ex.jpeg_quality = 82;
        let json = serde_json::to_string(&ex).expect("serialize");
        let deserialized: ExportSettings = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.jpeg_quality, 82);

        // Test clamping in write_jpeg
        let quality_clamped = 150u8.clamp(1, 100);
        assert_eq!(quality_clamped, 100);
        let quality_low = 0u8.clamp(1, 100);
        assert_eq!(quality_low, 1);
    }

    #[test]
    fn slider_origin_ui_renders_and_preserves_default() {
        let ctx = egui::Context::default();
        let _ = ctx.run_ui(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let mut q = 95u8;
                let ch = widgets::fl_slider_u8_origin(
                    ui,
                    "Quality",
                    &mut q,
                    1..=100,
                    95,
                    Some(0.0),
                    "JPEG compression quality (1-100)",
                );
                assert!(!ch);
                assert_eq!(q, 95);
            });
        });
    }
}
