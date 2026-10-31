// SPDX-License-Identifier: GPL-3.0-or-later
//! Correlation-ID context for USB-level recording.
//!
//! Lets [`RecordingTransport`](super::trace::RecordingTransport) stamp every
//! transaction with which high-level operation caused it. Nesting is allowed;
//! the innermost value set for each field wins, and dropping the guard
//! restores whatever was set before it.

use std::cell::RefCell;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Correlation {
    pub investigation_id: Option<String>,
    pub experiment_id: Option<String>,
    pub run_id: Option<String>,
    pub scan_id: Option<String>,
    pub operation_id: Option<String>,
}

thread_local! {
    static CURRENT: RefCell<Correlation> = RefCell::new(Correlation::default());
}

/// Snapshot of the correlation context active right now.
pub fn current() -> Correlation {
    CURRENT.with(|c| c.borrow().clone())
}

/// Restores the previous correlation when dropped.
pub struct CorrelationGuard {
    previous: Correlation,
}

impl Drop for CorrelationGuard {
    fn drop(&mut self) {
        let prev = std::mem::take(&mut self.previous);
        CURRENT.with(|c| *c.borrow_mut() = prev);
    }
}

/// Set any subset of correlation fields until the guard drops.
pub fn correlate(fields: Correlation) -> CorrelationGuard {
    CURRENT.with(|c| {
        let mut cur = c.borrow_mut();
        let previous = cur.clone();
        macro_rules! set {
            ($f:ident) => {
                if fields.$f.is_some() {
                    cur.$f = fields.$f;
                }
            };
        }
        set!(investigation_id);
        set!(experiment_id);
        set!(run_id);
        set!(scan_id);
        set!(operation_id);
        CorrelationGuard { previous }
    })
}

/// Same run/scan, new operation_id.
pub fn replace_operation(corr: &Correlation, operation_id: &str) -> Correlation {
    Correlation { operation_id: Some(operation_id.to_string()), ..corr.clone() }
}
