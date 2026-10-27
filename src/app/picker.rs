//! Native file and folder choosers (NSOpenPanel on macOS, `rfd` elsewhere).

use std::path::{Path, PathBuf};

/// Let the user choose one or more images, or a folder. Returns the chosen
/// paths (empty when cancelled).
#[cfg(target_os = "macos")]
pub fn pick_files_or_folder(start: Option<&Path>) -> Vec<PathBuf> {
    use objc2::MainThreadMarker;
    use objc2_app_kit::NSOpenPanel;
    use objc2_foundation::{NSString, NSURL};

    let Some(mtm) = MainThreadMarker::new() else {
        return rfd_pick_folder(start, None).into_iter().collect();
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

/// Choose an output directory or scan folder.
#[cfg(target_os = "macos")]
pub fn pick_folder(start: Option<&Path>, prompt: Option<&str>) -> Option<PathBuf> {
    use objc2::MainThreadMarker;
    use objc2_app_kit::NSOpenPanel;
    use objc2_foundation::{NSString, NSURL};

    let Some(mtm) = MainThreadMarker::new() else {
        return rfd_pick_folder(start, prompt);
    };
    let panel = NSOpenPanel::openPanel(mtm);
    panel.setCanChooseFiles(false);
    panel.setCanChooseDirectories(true);
    panel.setAllowsMultipleSelection(false);
    panel.setCanCreateDirectories(true);

    let prompt_str = prompt.unwrap_or("Open");
    let msg = if prompt_str.eq_ignore_ascii_case("choose") {
        "Choose Output Directory"
    } else {
        "Open a folder of negative scans"
    };
    panel.setMessage(Some(&NSString::from_str(msg)));
    panel.setPrompt(Some(&NSString::from_str(prompt_str)));

    if let Some(dir) = start {
        let url = NSURL::fileURLWithPath(&NSString::from_str(&dir.to_string_lossy()));
        panel.setDirectoryURL(Some(&url));
    }
    if panel.runModal() != 1 {
        return None;
    }
    panel
        .URLs()
        .iter()
        .next()
        .and_then(|u| u.path().map(|p| PathBuf::from(p.to_string())))
}

#[cfg(not(target_os = "macos"))]
pub fn pick_folder(start: Option<&Path>, prompt: Option<&str>) -> Option<PathBuf> {
    rfd_pick_folder(start, prompt)
}

fn rfd_pick_folder(start: Option<&Path>, prompt: Option<&str>) -> Option<PathBuf> {
    let title = if prompt.unwrap_or("").eq_ignore_ascii_case("choose") {
        "Choose Output Directory"
    } else {
        "Open a folder of negative scans"
    };
    let mut d = rfd::FileDialog::new().set_title(title);
    if let Some(s) = start {
        d = d.set_directory(s);
    }
    d.pick_folder()
}

/// Choose one or more scan image files.
#[cfg(target_os = "macos")]
pub fn pick_files(start: Option<&Path>) -> Vec<PathBuf> {
    use objc2::MainThreadMarker;
    use objc2_app_kit::NSOpenPanel;
    use objc2_foundation::{NSString, NSURL};

    let Some(mtm) = MainThreadMarker::new() else {
        return rfd_pick_files(start);
    };
    let panel = NSOpenPanel::openPanel(mtm);
    panel.setCanChooseFiles(true);
    panel.setCanChooseDirectories(false);
    panel.setAllowsMultipleSelection(true);
    panel.setMessage(Some(&NSString::from_str("Choose one or more image scans")));
    panel.setPrompt(Some(&NSString::from_str("Open")));
    if let Some(dir) = start {
        let url = NSURL::fileURLWithPath(&NSString::from_str(&dir.to_string_lossy()));
        panel.setDirectoryURL(Some(&url));
    }
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
pub fn pick_files(start: Option<&Path>) -> Vec<PathBuf> {
    rfd_pick_files(start)
}

fn rfd_pick_files(start: Option<&Path>) -> Vec<PathBuf> {
    let mut exts: Vec<&str> = crate::engine::decode::RAW_EXTENSIONS.to_vec();
    exts.extend_from_slice(crate::engine::decode::BITMAP_EXTENSIONS);
    let upper: Vec<String> = exts.iter().map(|e| e.to_uppercase()).collect();
    let mut all: Vec<&str> = exts.clone();
    all.extend(upper.iter().map(|s| s.as_str()));
    let mut d = rfd::FileDialog::new()
        .set_title("Open image(s)")
        .add_filter("Scans", &all);
    if let Some(s) = start {
        d = d.set_directory(s);
    }
    d.pick_files().unwrap_or_default()
}

/// Other platforms have no combined file-or-folder chooser; the welcome
/// screen offers "Open images…" separately, so choose a folder here.
#[cfg(not(target_os = "macos"))]
pub fn pick_files_or_folder(start: Option<&Path>) -> Vec<PathBuf> {
    rfd_pick_folder(start, None).into_iter().collect()
}

/// Show `path` in the system file manager (Finder, Explorer), selected
/// where the platform can.
pub fn reveal(path: &Path) {
    use std::process::Command;
    let r = if cfg!(target_os = "macos") {
        Command::new("open").arg("-R").arg(path).spawn()
    } else if cfg!(target_os = "windows") {
        Command::new("explorer")
            .arg(format!("/select,{}", path.display()))
            .spawn()
    } else {
        Command::new("xdg-open")
            .arg(path.parent().unwrap_or(path))
            .spawn()
    };
    let _ = r;
}
