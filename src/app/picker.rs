//! One file chooser that accepts either image files or a folder.

use std::path::{Path, PathBuf};

/// Let the user choose one or more images, or a folder. Returns the chosen
/// paths (empty when cancelled).
#[cfg(target_os = "macos")]
pub fn pick_files_or_folder(start: Option<&Path>) -> Vec<PathBuf> {
    use objc2::MainThreadMarker;
    use objc2_app_kit::NSOpenPanel;
    use objc2_foundation::{NSString, NSURL};

    let Some(mtm) = MainThreadMarker::new() else {
        return fallback(start);
    };
    let panel = NSOpenPanel::openPanel(mtm);
    panel.setCanChooseFiles(true);
    panel.setCanChooseDirectories(true);
    panel.setAllowsMultipleSelection(true);
    panel.setMessage(Some(&NSString::from_str(
        "Choose a folder of scans, or one or more images",
    )));
    panel.setPrompt(Some(&NSString::from_str("Open")));
    if let Some(dir) = start {
        let url = NSURL::fileURLWithPath(&NSString::from_str(&dir.to_string_lossy()));
        panel.setDirectoryURL(Some(&url));
    }
    // NSModalResponseOK == 1
    if panel.runModal() != 1 {
        return Vec::new();
    }
    panel
        .URLs()
        .iter()
        .filter_map(|u| u.path().map(|p| PathBuf::from(p.to_string())))
        .collect()
}

#[cfg(not(target_os = "macos"))]
pub fn pick_files_or_folder(start: Option<&Path>) -> Vec<PathBuf> {
    fallback(start)
}

/// Platforms without a combined chooser: pick files, or a folder if cancelled.
fn fallback(start: Option<&Path>) -> Vec<PathBuf> {
    let mut d = rfd::FileDialog::new().set_title("Open image(s) — cancel to choose a folder");
    if let Some(s) = start {
        d = d.set_directory(s);
    }
    if let Some(files) = d.pick_files() {
        return files;
    }
    let mut d = rfd::FileDialog::new().set_title("Open a folder of scans");
    if let Some(s) = start {
        d = d.set_directory(s);
    }
    d.pick_folder().into_iter().collect()
}
