// SPDX-License-Identifier: GPL-3.0-or-later
//! `time.sleep` / `time.monotonic` with a virtual clock for hardwareless runs.
//!
//! The Python tests patch `time.sleep`; here a thread can switch to a virtual
//! clock, where `sleep` only advances `monotonic`, so poll loops and timeouts
//! behave exactly as on hardware but run instantly.

use std::cell::Cell;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

thread_local! {
    static VIRTUAL: Cell<Option<f64>> = const { Cell::new(None) };
}

fn start() -> Instant {
    static S: OnceLock<Instant> = OnceLock::new();
    *S.get_or_init(Instant::now)
}

/// Switch this thread to a virtual clock starting at `t0` seconds.
pub fn use_virtual(t0: f64) {
    VIRTUAL.with(|v| v.set(Some(t0)));
}

/// Back to wall-clock time on this thread.
pub fn use_real() {
    VIRTUAL.with(|v| v.set(None));
}

pub fn is_virtual() -> bool {
    VIRTUAL.with(|v| v.get().is_some())
}

/// Seconds since an arbitrary epoch (Python `time.monotonic`).
pub fn monotonic() -> f64 {
    VIRTUAL.with(|v| v.get()).unwrap_or_else(|| start().elapsed().as_secs_f64())
}

/// Python `time.sleep`.
pub fn sleep(secs: f64) {
    if secs <= 0.0 {
        return;
    }
    let virt = VIRTUAL.with(|v| {
        let cur = v.get();
        if let Some(t) = cur {
            v.set(Some(t + secs));
        }
        cur.is_some()
    });
    if !virt {
        std::thread::sleep(Duration::from_secs_f64(secs));
    }
}
