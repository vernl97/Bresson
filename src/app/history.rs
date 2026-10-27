//! Undo / redo of settings edits across the folder: each step holds the
//! settings of every frame one action changed, so a paste to the whole
//! roll undoes in one go.

use std::path::PathBuf;

use crate::engine::settings::Settings;

use super::NlpApp;

const LIMIT: usize = 200;

type Step = Vec<(PathBuf, Settings)>;

#[derive(Default)]
pub struct History {
    undo: Vec<Step>,
    redo: Vec<Step>,
}

impl History {
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Record a step: the frames' settings before an edit.
    pub fn push(&mut self, step: Step) {
        if step.is_empty() {
            return;
        }
        self.undo.push(step);
        if self.undo.len() > LIMIT {
            self.undo.remove(0);
        }
        self.redo.clear();
    }
}

impl NlpApp {
    /// Record frames `idx` before a batch edit made outside the editor.
    pub fn record(&mut self, idx: &[usize]) {
        let step = idx
            .iter()
            .map(|&i| {
                (
                    self.frames[i].path.clone(),
                    self.frames[i].sc.settings.clone(),
                )
            })
            .collect();
        self.history.push(step);
    }

    /// Undo (or redo) the last step. Returns whether anything changed.
    pub fn history_step(&mut self, undo: bool) -> bool {
        let popped = if undo {
            self.history.undo.pop()
        } else {
            self.history.redo.pop()
        };
        let Some(step) = popped else {
            return false;
        };
        let mut inverse = Vec::with_capacity(step.len());
        for (path, s) in step {
            let Some(i) = self.index_of(&path) else {
                continue;
            };
            let cur = std::mem::replace(&mut self.frames[i].sc.settings, s);
            inverse.push((path, cur));
            self.mark_changed(i);
            if self.editor.path.as_ref() == Some(&self.frames[i].path) {
                self.editor
                    .set_committed(self.frames[i].sc.settings.clone());
            }
        }
        if undo {
            self.history.redo.push(inverse);
        } else {
            self.history.undo.push(inverse);
        }
        true
    }
}
