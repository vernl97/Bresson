//! Work the app does by itself: new frames are cropped and converted as
//! their proxies load, then balanced together as a roll, and exports wait
//! for their frame's conversion.

use std::path::PathBuf;

use crate::engine::settings::{ExportSettings, FilmBase, Source};

use super::jobs::{self, ConvertItem};
use super::{NlpApp, ProxyState};

/// Frames queued for longer than this are converted even if the batch
/// isn't full.
const BATCH_WAIT: f64 = 0.25;
const BATCH_SIZE: usize = 8;

#[derive(Default, PartialEq, Eq, Clone, Copy, Debug)]
pub enum RollPass {
    /// Waiting for every frame to load and convert on its own.
    #[default]
    Waiting,
    Running,
    Done,
}

/// Whether a frame converts by itself. A frame is in flight while its
/// [`ConvertTicket`](super::ConvertTicket) is an automatic one.
#[derive(Default, PartialEq, Eq, Clone, Copy, Debug)]
pub enum AutoConvert {
    /// Converted when the folder opened, or converted or unconverted by
    /// hand: it waits for the user.
    #[default]
    Off,
    /// Converts by itself once its proxy loads, and stays so afterwards: the
    /// roll pass balances these frames together.
    On,
    /// On, and waiting for the next batch (`auto_tick`).
    Queued,
}

#[derive(Default)]
pub struct AutoState {
    /// When the oldest frame now [`AutoConvert::Queued`] was queued.
    pub queued_since: Option<f64>,
    pub roll_pass: RollPass,
    /// The roll pass ignored the exposure metadata: the frames' spread in EV.
    pub exposure_note: Option<f64>,
    /// Frames sent to conversion and come back, for the progress bar.
    pub progress: super::Progress,
    /// Files of this folder that couldn't be read, for the shared toast.
    pub read_failures: Vec<String>,
    /// Exports waiting for their frame to be converted: frame, target, settings.
    pub export_waiting: Vec<(PathBuf, PathBuf, ExportSettings)>,
}

impl NlpApp {
    /// Frames waiting for the next automatic batch.
    fn auto_queued(&self) -> Vec<usize> {
        (0..self.frames.len())
            .filter(|&i| self.frames[i].auto == AutoConvert::Queued)
            .collect()
    }

    /// Frames are queued for, or in, an automatic conversion, or the roll
    /// pass runs.
    pub fn auto_busy(&self) -> bool {
        self.auto.roll_pass == RollPass::Running
            || self.frames.iter().any(|f| {
                f.auto == AutoConvert::Queued || f.convert_ticket.is_some_and(|t| t.auto)
            })
    }

    /// The app has work of its own lined up that Stop would cancel.
    pub fn auto_pending(&self) -> bool {
        self.auto.roll_pass == RollPass::Running
            || self.frames.iter().any(|f| f.auto == AutoConvert::Queued)
    }

    /// A proxy arrived (or a conversion came back stale): queue the frame if
    /// it converts automatically and nothing is converting it already.
    pub fn auto_queue(&mut self, i: usize) {
        let f = &mut self.frames[i];
        if self.defaults.manual_convert
            || f.auto != AutoConvert::On
            || f.sc.is_converted()
            || f.convert_ticket.is_some()
            || matches!(f.proxy, ProxyState::Failed(_))
        {
            return;
        }
        f.auto = AutoConvert::Queued;
        self.auto.queued_since.get_or_insert(self.time);
    }

    /// Whether every frame the grid shows has its proxy (or failed to load).
    fn visible_loaded(&self) -> bool {
        let visible: Vec<usize> = self
            .jobs
            .ctx()
            .data(|d| d.get_temp(eframe::egui::Id::new(super::folder::GRID_VISIBLE)))
            .unwrap_or_default();
        visible.iter().all(|&i| {
            self.frames
                .get(i)
                .is_none_or(|f| !matches!(f.proxy, ProxyState::NotLoaded | ProxyState::Loading))
        })
    }

    /// Run queued conversions in batches, then the roll pass once the
    /// whole folder has loaded.
    pub fn auto_tick(&mut self) {
        if let Some(t) = self.auto.queued_since {
            let queued = self.auto_queued();
            let due = self.time - t > BATCH_WAIT || queued.len() >= BATCH_SIZE;
            // Conversion shares the pool with decoding: let the frames on
            // screen load first.
            if queued.is_empty() {
                self.auto.queued_since = None;
            } else if due && self.visible_loaded() {
                self.auto.queued_since = None;
                self.spawn_auto_convert(queued, false);
            }
        }
        if self.auto.roll_pass != RollPass::Waiting
            || self.auto_busy()
            || self.defaults.manual_convert
        {
            return;
        }
        let loading = self
            .frames
            .iter()
            .any(|f| matches!(f.proxy, ProxyState::NotLoaded | ProxyState::Loading));
        if loading {
            return;
        }
        let roll: Vec<usize> = (0..self.frames.len())
            .filter(|&i| self.frames[i].auto != AutoConvert::Off && self.frames[i].sc.is_converted())
            .collect();
        if roll.len() < 2 || self.single_mode {
            self.auto.roll_pass = RollPass::Done;
            return;
        }
        self.auto.roll_pass = RollPass::Running;
        self.spawn_auto_convert(roll, true);
    }

    fn spawn_auto_convert(&mut self, idx: Vec<usize>, as_roll: bool) {
        if idx.is_empty() {
            return;
        }
        let items: Vec<ConvertItem> = idx
            .iter()
            .map(|&i| {
                let f = &mut self.frames[i];
                f.sc.settings.auto_crop = true;
                f.auto = AutoConvert::On;
                self.convert_item(i, true)
            })
            .collect();
        self.convert_started(idx.len());
        self.jobs.spawn(move || jobs::convert(items, as_roll, true));
    }

    /// Count frames into the conversion progress.
    pub fn convert_started(&mut self, n: usize) {
        self.auto.progress.start(n);
    }

    /// Stop converting by itself: queued frames stay as they are. (They
    /// were never counted into the progress, so it stays as it is.)
    pub fn stop_auto(&mut self) {
        self.auto.queued_since = None;
        self.auto.roll_pass = RollPass::Done;
        for f in &mut self.frames {
            if !f.sc.is_converted() {
                f.auto = AutoConvert::Off;
            }
        }
    }

    /// A roll pass found exposures more than 1 EV apart: balance by the
    /// images instead of asking, and say so in the folder banner.
    pub fn auto_exposure_variance(
        &mut self,
        items: Vec<ConvertItem>,
        spread: f64,
    ) -> Option<Vec<ConvertItem>> {
        if self.auto.roll_pass != RollPass::Running || !items.iter().all(|it| it.auto) {
            return Some(items);
        }
        self.auto.exposure_note = Some(spread);
        self.convert_started(items.len());
        self.jobs.spawn(move || jobs::convert(items, true, false));
        None
    }

    /// The user converted or unconverted these frames: they no longer
    /// convert by themselves (a queued one leaves the queue).
    pub fn auto_release(&mut self, idx: &[usize]) {
        for &i in idx {
            self.frames[i].auto = AutoConvert::Off;
        }
    }

    /// Set the scan source of every frame and convert them again.
    pub fn set_roll_source(&mut self, source: Source) {
        for i in 0..self.frames.len() {
            let f = &mut self.frames[i];
            f.source_auto = false;
            f.sc.settings.source = source;
            f.sc.settings.film_base = match source {
                Source::TifScan | Source::LinearScan => FilmBase::Off,
                _ => FilmBase::Auto,
            };
            f.sc.analysis = None;
            f.sc.roll = None;
            f.sc.settings.use_roll = false;
            if f.auto == AutoConvert::Off {
                f.auto = AutoConvert::On;
            }
            // A conversion still running used the old source: its result
            // will be stale, and the frame queues again now.
            f.convert_ticket = None;
            self.mark_changed(i);
            if self.frames[i].proxy().is_some() {
                self.auto_queue(i);
            }
        }
        self.auto.roll_pass = RollPass::Waiting;
        self.auto.exposure_note = None;
        self.editor.invalidate();
    }

    /// A conversion of frame `i` came back after something made it stale. If
    /// no newer one is running, an automatic frame queues again; otherwise
    /// its waiting exports have nothing left to wait for and are cancelled.
    pub fn stale_conversion(&mut self, i: usize) {
        let f = &self.frames[i];
        if f.convert_ticket.is_some() || f.sc.is_converted() {
            return;
        }
        self.auto_queue(i);
        if self.frames[i].auto == AutoConvert::Queued {
            return;
        }
        let path = self.frames[i].path.clone();
        let before = self.auto.export_waiting.len();
        self.auto.export_waiting.retain(|(p, _, _)| *p != path);
        let dropped = before - self.auto.export_waiting.len();
        if dropped > 0 {
            self.frames[i].exporting = false;
            self.export_progress.finish(dropped);
            let name = self.frames[i].name.clone();
            self.toast(format!("Cancelled the export of {name}: it is no longer converted"), true);
        }
    }

    /// Start the exports whose frames are now converted.
    pub fn flush_waiting_exports(&mut self) {
        let waiting = std::mem::take(&mut self.auto.export_waiting);
        for (path, target, ex) in waiting {
            match self.index_of(&path) {
                Some(i) if self.frames[i].sc.is_converted() => self.spawn_export(i, target, ex),
                // The frame can't be read: the conversion reported it already.
                Some(i) if matches!(self.frames[i].proxy, ProxyState::Failed(_)) => {
                    self.frames[i].exporting = false;
                    self.export_progress.finish(1);
                }
                Some(_) => self.auto.export_waiting.push((path, target, ex)),
                // The frame left with its roll: count the export out.
                None => self.export_progress.finish(1),
            }
        }
    }
}
