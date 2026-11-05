//! Undo / redo of settings edits: each step holds the settings of every
//! frame one action changed, so a paste to the whole roll undoes in one go
//! from the folder. The editor undoes one photo at a time: it takes that
//! photo's part out of the latest step that touched it.

use std::path::{Path, PathBuf};

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
    /// Whether `path` has an edit to undo.
    pub fn can_undo_for(&self, path: &Path) -> bool {
        self.undo.iter().flatten().any(|(p, _)| p == path)
    }

    /// Whether `path` has an undone edit to redo.
    pub fn can_redo_for(&self, path: &Path) -> bool {
        self.redo.iter().flatten().any(|(p, _)| p == path)
    }

    /// Record a step: the frames' settings before an edit. A new edit ends
    /// the redo chain of the frames it touches, and only theirs.
    pub fn push(&mut self, step: Step) {
        if step.is_empty() {
            return;
        }
        self.redo.retain_mut(|r| {
            r.retain(|(p, _)| !step.iter().any(|(q, _)| q == p));
            !r.is_empty()
        });
        self.undo.push(step);
        if self.undo.len() > LIMIT {
            self.undo.remove(0);
        }
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

    /// Put `s` back as frame `path`'s settings; returns what it replaced.
    fn restore(&mut self, path: &Path, s: Settings) -> Option<Settings> {
        let i = self.index_of(path)?;
        let cur = std::mem::replace(&mut self.frames[i].sc.settings, s);
        self.mark_changed(i);
        if self.editor.path.as_ref() == Some(&self.frames[i].path) {
            self.editor
                .set_committed(self.frames[i].sc.settings.clone());
        }
        Some(cur)
    }

    /// Undo (or redo) the last step, for every frame it touched. Returns
    /// whether anything changed.
    pub fn history_step(&mut self, undo: bool) -> bool {
        let popped = if undo {
            self.history.undo.pop()
        } else {
            self.history.redo.pop()
        };
        let Some(step) = popped else {
            return false;
        };
        let inverse: Step = step
            .into_iter()
            .filter_map(|(path, s)| self.restore(&path, s).map(|cur| (path, cur)))
            .collect();
        if undo {
            self.history.redo.push(inverse);
        } else {
            self.history.undo.push(inverse);
        }
        true
    }

    /// Undo (or redo) the latest edit of the photo at `path` alone, leaving
    /// the other frames of a batch step as they are.
    pub fn history_step_for(&mut self, path: &Path, undo: bool) -> bool {
        let stack = if undo {
            &mut self.history.undo
        } else {
            &mut self.history.redo
        };
        let Some(si) = stack
            .iter()
            .rposition(|st| st.iter().any(|(p, _)| p == path))
        else {
            return false;
        };
        let k = stack[si].iter().position(|(p, _)| p == path).unwrap();
        let (p, s) = stack[si].remove(k);
        if stack[si].is_empty() {
            stack.remove(si);
        }
        let Some(cur) = self.restore(&p, s) else {
            return false;
        };
        let other = if undo {
            &mut self.history.redo
        } else {
            &mut self.history.undo
        };
        other.push(vec![(p, cur)]);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn step(paths: &[&str]) -> Step {
        paths
            .iter()
            .map(|p| (PathBuf::from(p), Settings::default()))
            .collect()
    }

    #[test]
    fn each_photo_has_its_own_undo_and_redo() {
        let (a, b) = (Path::new("a.dng"), Path::new("b.dng"));
        let mut h = History::default();
        h.push(step(&["a.dng"]));
        h.push(step(&["a.dng", "b.dng"]));
        assert!(h.can_undo_for(a) && h.can_undo_for(b));
        assert!(!h.can_undo_for(Path::new("c.dng")));

        // An edit to one photo leaves another's redo chain alone.
        h.redo.push(step(&["b.dng"]));
        assert!(h.can_redo_for(b) && !h.can_redo_for(a));
        h.push(step(&["a.dng"]));
        assert!(h.can_redo_for(b), "editing a keeps b's redo");
        h.push(step(&["b.dng"]));
        assert!(!h.can_redo_for(b), "editing b ends its own redo");
    }
}
