# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

NLP Studio (`nlp_studio` crate): a standalone Rust port of the Negative Lab Pro Lightroom plugin. It reads camera RAW negatives or TIFF/JPEG/PNG scans, inverts them with the plugin's curve engine and writes positive copies. A single binary provides an egui GUI and a headless CLI.

The original Lua plugin sources live in `../NLP_decompile/` (e.g. `CurveEngineV3.lua`, `NLPImageProcessorV3.lua`, `NegativePresets.lua`, `NLPColorV3.lua`). Engine code is a deliberate port of them: doc comments cite the Lua names (`LR_size_maxHeight`, `updateSharpening`, …). When you change engine behaviour, check the matching Lua first and keep parity unless the README lists a known difference.

## Commands

```bash
cargo run --release                          # GUI (reopens last folder)
cargo run --release -- /path/to/scans        # GUI on a folder or single file
cargo run --release -- convert [--roll] [--tone NAME] [--wb NAME] [--lut NAME] [--border PCT] [--jpg] [--out DIR] FILES
cargo run --release -- scan-prep FILES       # level + gamma 2.2 encode linear (gamma 1.0) TIF scans
cargo run --release -- synth [out.tif] [variant]   # write a synthetic colour negative for testing

cargo test --release                         # unit tests (engine + app)
cargo test --release <name>                  # single test, e.g. cargo test --release roll_aggregation_extremes
NLP_UI_TEST_DIR=<scans> NLP_UI_OUT=<dir> cargo test --release ui_screens -- --ignored   # headless egui_kittest screenshots
```

`main.rs` sends `convert | scan-prep | synth | help` to `cli.rs`. Any other argument is treated as a path to open in the GUI.

## Architecture

**`src/engine/`**: pure image processing, no UI. The pipeline for one frame:

1. `decode::load(path, max_long_edge)` → `SourceImage` (linear camera RGB). `rawler` handles RAW files; with a size limit, the CFA is binned instead of fully demosaiced.
2. `pipeline::resolve_base` finds the film base: the 98.5–99.5 percentile of the *uncropped* frame, or an eyedropper sample, or the roll base.
3. `pipeline::prepare_negative` neutralises the base and applies the camera profile (`dcp.rs`: `.dcp` files matched like Lightroom, by DNG UniqueCameraModel or make+model). The result is a `Negative` in encoded MelissaRGB, which is the space the plugin's curves operate on.
4. `pipeline::analyze` builds the 600 px analysis buffer (border crop, σ=2 blur), reads curve `Origins` from the CDF histograms, and computes the HSB-zone auto WB. The output is an `Analysis`. In roll mode, `aggregate_roll` merges the frames' origins with 2.5σ outlier rejection.
5. `pipeline::Renderer` is built from `Settings` and `Origins` (`origins_for` chooses roll or frame origins). It applies the `curve.rs` tone curves, then `look.rs`/`lut.rs` looks, then output-space conversion (`color.rs`), then sharpening. `export.rs` writes TIFF/JPEG with ICC and EXIF.

Key data types live in `settings.rs`: `Settings` holds the per-image edit state, `ExportSettings`, `FilmMetadata` and `AnalysisKey`. `AnalysisKey` is the subset of settings the analysis depends on. If you change one of those fields, the frame needs re-analysis ("Convert"). Any other edit only rebuilds the `Renderer`. A new setting that affects analysis must go into `AnalysisKey`.

Persistence (`sidecar.rs`):
- `<image>.nlp.json` next to each scan holds settings, analysis and roll data.
- App defaults and last folder: `dirs::config_dir()/nlp-studio/defaults.json`.
- Decoded proxy cache: `dirs::cache_dir()/nlp-studio/proxies/`, keyed by path, size and mtime.

New `Settings` fields need `#[serde(default)]` compatibility so existing sidecars still load.

**`src/app/`**: the eframe GUI.
- `mod.rs` holds `NlpApp`: the `Frame` list, Folder/Editor `View`, undo/redo and toasts. It also contains the kittest UI tests.
- `folder.rs` holds the welcome screen and the thumbnail grid; selection-dependent Convert / Convert as roll / Export actions sit in its top bar. `editor.rs` is the single-frame editor with canvas, crop/straighten and the Convert/Edit/Advanced tabs. `widgets.rs` holds the dark graphite theme (colour tokens in `theme`, Inter embedded from `assets/fonts/`), the shared controls and the vector icons. Use `theme::*` tokens and `widgets::primary`/`ghost` rather than ad-hoc colours, and `widgets::shortcut` for OS-correct shortcut labels.
- `jobs.rs` runs all heavy work on the rayon pool and returns `JobResult`s over an mpsc channel, which the UI polls each frame. Frames load at `PROXY_SIZE` (1024 px) for thumbnails and analysis, and at `EDITOR_SIZE` (2048 px) for the editor preview. Exports decode at full resolution.
- `picker.rs`: native macOS NSOpenPanel via `objc2`, with `rfd` elsewhere.

## External assets

`NLP Lightroom Plugin/LR Files/CameraRaw/` holds the plugin's real resources:
- `CameraProfiles/…/*.dcp` camera profiles.
- `Settings/NLP Enhanced Settings/*.xmp` 3D LUTs: Adobe base85 + zlib, 32³.

These are found at runtime by searching upward from the executable and Lightroom's CameraRaw folder. `NLP_DCP_DIR` overrides the profile directory. If no profile matches, the app falls back to `rawler`'s colour matrix. If no LUTs are found, `look.rs` uses built-in approximations. The folder is listed in `.gitignore` but its files are already tracked.
