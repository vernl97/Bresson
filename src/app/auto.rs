//! Work the app does by itself: new frames are cropped and converted as
//! their proxies load, then balanced together as a roll, and exports wait
//! for their frame's conversion.

use std::collections::HashSet;
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

#[derive(Default)]
pub struct AutoState {
    /// Frames waiting for their automatic conversion, oldest first.
    pub queue: Vec<PathBuf>,
    pub queued_since: Option<f64>,
    /// Frames sent to an automatic conversion that hasn't come back yet.
    pub in_flight: HashSet<PathBuf>,
    pub roll_pass: RollPass,
    /// The roll pass ignored the exposure metadata: the frames' spread in EV.
    pub exposure_note: Option<f64>,
    /// Frames sent to conversion and come back, for the progress bar.
    pub convert_total: usize,
    pub convert_done: usize,
    /// Files of this folder that couldn't be read, for the shared toast.
    pub read_failures: Vec<String>,
    /// Exports waiting for their frame to be converted: frame, target, settings.
    pub export_waiting: Vec<(PathBuf, PathBuf, ExportSettings)>,
}

impl AutoState {
    pub fn busy(&self) -> bool {
        !self.queue.is_empty() || !self.in_flight.is_empty() || self.roll_pass == RollPass::Running
    }
}

impl NlpApp {
    /// A proxy arrived: queue the frame if it converts automatically.
    pub fn auto_queue(&mut self, i: usize) {
        let f = &self.frames[i];
        if self.defaults.manual_convert || !f.auto_convert || f.sc.is_converted() {
            return;
        }
        if !self.auto.queue.contains(&f.path) && !self.auto.in_flight.contains(&f.path) {
            self.auto.queue.push(f.path.clone());
            self.auto.queued_since.get_or_insert(self.time);
        }
    }

    /// Run queued conversions in batches, then the roll pass once the
    /// whole folder has loaded.
    pub fn auto_tick(&mut self) {
        let due = self
            .auto
            .queued_since
            .is_some_and(|t| self.time - t > BATCH_WAIT || self.auto.queue.len() >= BATCH_SIZE);
        if due {
            self.auto.queued_since = None;
            let paths = std::mem::take(&mut self.auto.queue);
            let idx: Vec<usize> = paths.iter().filter_map(|p| self.index_of(p)).collect();
            self.spawn_auto_convert(idx, false);
        }
        if self.auto.roll_pass != RollPass::Waiting
            || self.auto.busy()
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
            .filter(|&i| self.frames[i].auto_convert && self.frames[i].sc.is_converted())
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
                self.auto.in_flight.insert(f.path.clone());
                ConvertItem {
                    path: f.path.clone(),
                    proxy: f.proxy(),
                    settings: f.sc.settings.clone(),
                }
            })
            .collect();
        self.convert_started(idx.len());
        self.jobs.spawn(move || jobs::convert(items, as_roll, true));
    }

    /// Count frames into the conversion progress.
    pub fn convert_started(&mut self, n: usize) {
        if self.auto.convert_done >= self.auto.convert_total {
            (self.auto.convert_total, self.auto.convert_done) = (0, 0);
        }
        self.auto.convert_total += n;
    }

    /// Stop converting by itself: queued frames stay as they are.
    pub fn stop_auto(&mut self) {
        self.auto.convert_total -= self.auto.queue.len().min(self.auto.convert_total);
        self.auto.queue.clear();
        self.auto.queued_since = None;
        self.auto.roll_pass = RollPass::Done;
        for f in &mut self.frames {
            if !f.sc.is_converted() {
                f.auto_convert = false;
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
        if self.auto.roll_pass != RollPass::Running
            || !items
                .iter()
                .all(|it| self.auto.in_flight.contains(&it.path))
        {
            return Some(items);
        }
        self.auto.exposure_note = Some(spread);
        self.convert_started(items.len());
        self.jobs.spawn(move || jobs::convert(items, true, false));
        None
    }

    /// A conversion came back: returns whether it was an automatic one
    /// (no toast for those).
    pub fn auto_converted(&mut self, paths: &[PathBuf], roll: bool) -> bool {
        let mut auto = !paths.is_empty();
        for p in paths {
            auto &= self.auto.in_flight.remove(p);
        }
        if auto && roll && self.auto.roll_pass == RollPass::Running {
            self.auto.roll_pass = RollPass::Done;
        }
        auto
    }

    /// The user converted or unconverted these frames: they no longer
    /// convert by themselves.
    pub fn auto_release(&mut self, idx: &[usize]) {
        for &i in idx {
            self.frames[i].auto_convert = false;
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
            f.auto_convert = true;
            self.mark_changed(i);
            if self.frames[i].proxy().is_some() {
                self.auto_queue(i);
            }
        }
        self.auto.roll_pass = RollPass::Waiting;
        self.auto.exposure_note = None;
        self.editor.invalidate();
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
                    self.export_done += 1;
                }
                Some(_) => self.auto.export_waiting.push((path, target, ex)),
                None => {}
            }
        }
    }
}
