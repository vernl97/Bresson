//! Work the app does by itself: exports that wait for their frame's
//! conversion.

use std::path::PathBuf;

use crate::engine::settings::ExportSettings;

use super::{NlpApp, ProxyState};

#[derive(Default)]
pub struct AutoState {
    /// Exports waiting for their frame to be converted: frame, target, settings.
    pub export_waiting: Vec<(PathBuf, PathBuf, ExportSettings)>,
}

impl NlpApp {
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
