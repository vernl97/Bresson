# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

NLP Studio (`nlp_studio` crate): a standalone Rust application for converting, editing, and managing film negatives. While originally inspired by the Negative Lab Pro Lightroom plugin, **NLP Studio is an independent application designed to modernize, streamline, and meaningfully improve upon the entire film negative workflow**.

It reads camera RAW negatives or TIFF/JPEG/PNG scans, converts them with a density-based print engine, and exports positive copies. A single fast, cross-platform binary provides both an interactive egui GUI and a headless CLI, completely free of Lightroom, catalog databases, or Adobe subscriptions.

### Product Vision & Design Philosophy

- **Modern Photo App UX**: The workflow, layout, and UI/UX are inspired by modern photo editors like Lightroom and Capture One—featuring smooth thumbnail browsing, responsive image previews, non-destructive editing, side-by-side / split before-and-after views, and professional color grading.
- **Purpose-Built for Film Workflows**: Rather than forcing digital camera paradigms onto digitized film, the app delivers tailored quality-of-life improvements specifically for film photographers and scanning labs:
  - **First-class roll handling**: Seamless roll-level workflows, batch roll analysis, cross-frame consistency (outlier-rejected roll curve aggregation), and synchronized roll metadata (film stock, ISO, push/pull, lab, developer).
  - **Photographic print exposure**: Real photographic units (EV-calibrated print exposure modeling paper density) rather than arbitrary digital slider units.
  - **Lab & darkroom shortcuts**: Dedicated Frontier/Noritsu-style lab density and color balance keys (number row / numpad: C, M, Y, Density) for ultra-fast proofing and correction.
  - **Automated frame prep**: Intelligent frame detection, crop, and straightening to isolate the active negative from film borders, rebates, and scanner holders.
  - **Transparent, open sidecars**: Clean, human-readable JSON sidecars (`NLP/<file>.nlp.json`) and shared roll metadata (`NLP/roll.json`) keeping scan folders portable and non-destructive without catalog lock-in.
- **Native Performance & Independence**: Native compiled Rust binary with multi-threaded pipelines (Rayon), fast proxy generation for instant visual feedback, and zero reliance on external hosts or legacy plugin runtimes.

## Commands

```bash
cargo run --release                          # GUI (reopens last folder)
cargo run --release -- /path/to/scans        # GUI on a folder or single file
cargo run --release -- convert [--roll] [--roll-process NAME] [--tone NAME] [--wb NAME] [--lut NAME] [--border PCT] [--auto-crop] [--flat FILE] [--jpg] [--out DIR] FILES
cargo run --release -- scan-prep FILES       # gamma-encode linear TIF scans (Linear Scan source reads them directly)
cargo run --release -- synth [out.tif] [ev]  # write the modelled ColorChecker negative (testchart.rs)

cargo test --release                         # unit tests (engine + app)
cargo test --release <name>                  # single test, e.g. cargo test --release roll_process_modes
cargo test --release chart_quality -- --nocapture   # print the conversion quality metrics
NLP_UI_TEST_DIR=<scans> NLP_UI_OUT=<dir> cargo test --release ui_screens -- --ignored   # headless egui_kittest screenshots
```

`main.rs` sends `convert | scan-prep | synth | help` to `cli.rs`. Any other argument is treated as a path to open in the GUI.

## Architecture

**`src/engine/`**: pure image processing, no UI. The pipeline for one frame:

1. `decode::load(path, max_long_edge)` → `SourceImage` (linear camera RGB). `rawler` handles RAW files; with a size limit, the CFA is binned instead of demosaiced. Full resolution uses `demosaic.rs` (RCD-style) for Bayer and rawler's Markesteijn for X-Trans; scanner DNGs (LinearRaw) need none.
2. `pipeline::resolve_base` finds the film base: the 98.5–99.5 percentile of the *uncropped* frame, or an eyedropper sample, or the roll base.
3. `pipeline::prepare_negative` applies the optional flat field (`flat.rs`), the camera matrix and the base, and stores per-channel **density above the base** (`log10(base / T)`, unclamped), with C-41 dye crosstalk removed and the colour model's calibration applied in density. In density a wrong base is only a constant offset that the levels absorb. No camera profiles (DCP) are used.
4. `pipeline::analyze` builds the 600 px analysis buffer (film border and holder trimmed by density, then the border buffer, σ=2 blur) and finds per-channel `Levels` (density lo/hi/mid; the white end is capped relative to the median so light sources can't darken the print, except that a mostly dark frame with a many-toned bright area clips only its top 3 %). Red and blue are placed on the image's neutral axis, fitted through ten equally weighted tonal steps. Each step's grey is the cluster of near-grey pixels on the current axis (`neutral_cluster`), so a skin, sand or wood surface lit across many tones can't pull it, and steps far off the line are dropped. With enough steps the axis bends: a quadratic stored as `ChannelLevels::bow`, which corrects crossover (Fuji US 6,160,634, EP 0 612 183). Auto-Neutral prints that axis grey. `skin.rs` then finds compact skin areas away from the frame edge and moves the midtone key (`mid`) towards the skin, like the Frontier's face-density Auto Setup. The output is an `Analysis`. In roll mode, `aggregate_roll` builds a `RollBalance`: the film's slopes and bends (medians of the frames') and the roll's filtration (median offset, robust outliers left out). Lab mode uses the roll's slopes with each frame's own balance, bend and density; the frame's balance fades towards the roll's as its confidence drops. Darkroom uses the roll's balance and exposure for every frame.
5. `pipeline::Renderer` is built from `Settings` and `Levels` (`levels_for` chooses roll or frame levels). `curve.rs` is the print engine: normalised density → tone/colour shaping → paper log exposure plus filtration (WB, C·M·Y keys, in CC) and print time (EV) → the tone profile (the plugin's 14 profiles, fitted as 21-knot curves over normalised density), baked into density-indexed LUTs. A base saturation (`DENSITY_SATURATION`) in density keeps chroma near the scene's. Hyper Tone (`dodge.rs`, off by default) adds a density offset before the curves, from a two-scale low-pass of green density, compressing large areas away from the key. Then come the native `look.rs` looks (saturation spares skin tones), output-space conversion (`color.rs`) and grain-aware sharpening. Sharpening weighs edges against grain by the local R/G/B correlation of the detail bands; it runs only on CPU renders (exports) with Sharpening set to Lab or Scanner. `export.rs` writes TIFF/JPEG with ICC and EXIF.

Conversion quality is measured on `testchart.rs`, a ColorChecker modelled on C-41 film. `pipeline::tests::chart_quality` and `dominated_scenes_stay_neutral` hold the metrics; their bounds only tighten. Bump `settings::ENGINE_VERSION` when analyses change meaning, and `sidecar::VERSION` when stored analyses or rolls change shape.

Key data types live in `settings.rs`: `Settings` holds the per-image edit state, `ExportSettings`, `FilmMetadata` and `AnalysisKey`. `AnalysisKey` is the subset of settings the analysis depends on. If you change one of those fields, the frame needs re-analysis ("Convert"). Any other edit only rebuilds the `Renderer`. A new setting that affects analysis must go into `AnalysisKey`.

Persistence (`sidecar.rs`):
- `<image>.nlp.json` in the `NLP/` subfolder of each scan folder holds settings, analysis and roll data.
- `NLP/roll.json` holds the roll info (`RollInfo`: name, date, film metadata, notes). It is asked for the first time a folder is opened (`app/roll.rs`), and frames without their own film details inherit it.
- App defaults, last folder and the Projects list (`projects`): `dirs::config_dir()/nlp-studio/defaults.json`.
- Decoded proxy cache: `dirs::cache_dir()/nlp-studio/proxies/`, keyed by path, size and mtime.

New `Settings` fields need `#[serde(default)]` compatibility so existing sidecars still load.

**`src/app/`**: the eframe GUI.
- `mod.rs` holds `NlpApp`: the `Frame` list, Folder/Editor `View`, undo/redo and toasts. It also contains the kittest UI tests.
- `folder.rs` holds the welcome screen and the thumbnail grid; selection-dependent Convert / Convert as roll / Export actions sit in its top bar. `editor.rs` is the single-frame editor with canvas, crop/straighten and the sidebar sections (conversion, tone, white balance, lab correction, toning, look, film, analysis). `widgets.rs` holds the dark graphite theme (colour tokens in `theme`, Inter embedded from `assets/fonts/`), the shared controls and the vector icons. Use `theme::*` tokens and `widgets::primary`/`ghost` rather than ad-hoc colours, and `widgets::shortcut` for OS-correct shortcut labels.
- `jobs.rs` runs all heavy work on the rayon pool and returns `JobResult`s over an mpsc channel, which the UI polls each frame. Frames load at `PROXY_SIZE` (1024 px) for thumbnails and analysis, and at `EDITOR_SIZE` (2048 px) for the editor preview. Exports decode at full resolution.
- `gpu.rs` + `preview.wgsl`: with wgpu, the editor preview is a paint callback running the renderer's dodge, curves, look and output conversion in a shader (no sharpening). The density texture's alpha holds the dodge low-pass. `gpu_preview_matches_the_cpu_renderer` checks it against `Renderer::render`; keep the shader in step with `curve.rs`/`look.rs`. The CPU renders a 1024 px copy for histogram, readout and clipping, and remains the fallback.
- `picker.rs`: native macOS NSOpenPanel via `objc2`, with `rfd` elsewhere.

## External assets

None at runtime. The looks are native data in `assets/looks.json` (embedded with `include_str!`), measured from the plugin's enhanced looks; the plugin's tone profiles are fitted curves in `curve.rs`. VueScan / SilverFast DNGs are set as the scan source by hand.
