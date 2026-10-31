// SPDX-License-Identifier: GPL-3.0-or-later
//! Real libsane with the system's full backend list. Its own test binary (so
//! its own process): SANE is initialised once per process and never exited,
//! so the backends loaded here would leak into other tests.

use sane::Sane;

/// Listing devices in a thread that then exits, and dropping SANE, must not
/// crash. The backends leave thread-exit destructors, which ran from unloaded
/// code when libsane called `sane_exit` (the dll backend `dlclose`s them) or
/// was itself unloaded. Run with `cargo test -p sane --release -- --ignored`
/// (needs sane-backends).
#[test]
#[ignore]
fn libsane_system_config_survives_thread_exit() {
    let t = std::thread::spawn(|| {
        let mut sane = Sane::load().expect("libsane");
        let _ = sane.devices();
        drop(sane);
    });
    t.join().expect("listing thread");
    std::thread::sleep(std::time::Duration::from_millis(300));
}
