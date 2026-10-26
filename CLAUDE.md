# NLP Studio (nlp_studio)

## Overview & Purpose
**NLP Studio** is a standalone Rust desktop application and headless CLI tool that ports the Lightroom **Negative Lab Pro** (Negative Conversion Plugin) curve engine. It ingests camera RAW files or TIFF/JPEG/PNG scans of film negatives, neutralises film base masks (orange mask for C-41, or clear base for B&W/slides), applies parametric tone curve inversions, white balance, toning/LUT looks, and exports positive copies (16-bit TIFF or JPEG) with embedded ICC color profiles.

## Tech Stack & Frameworks
- **Language**: Rust (Edition 2024)
- **GUI Framework**: `eframe` / `egui` (v0.35.0) with custom widgets, themes, and canvas zoom/pan/crop controls.
- **Image & RAW Decoding**:
  - `rawler` (v0.8.0) for camera RAW decoding (DNG, CR2, NEF, ARW, etc.) preserving linear camera RGB.
  - `image` (v0.25.10) & `tiff` (v0.11.3) for reading/writing TIFF, JPEG, PNG bitmaps.
- **Concurrency & Parallelism**: `rayon` (v1.12.0) for multi-threaded pixel operations, analysis, and job queues.
- **File Dialogs**: Native macOS Cocoa panels via `objc2`, `objc2-foundation`, `objc2-app-kit` (macOS), with fallback to `rfd` (cross-platform).
- **Serialization**: `serde` / `serde_json` for `<image>.nlp.json` sidecars and user defaults.
- **Testing**: Built-in Rust unit tests + `egui_kittest` (v0.35) for headless UI harness and snapshot testing.

---

## Commands

### Build & Check
```bash
cargo check                     # Fast syntax & type checking
cargo build                     # Debug build
cargo build --release           # Optimized release build
```

### Run
```bash
cargo run --release                     # Launch GUI (reopens last folder or welcome view)
cargo run --release -- /path/to/scans   # Launch GUI directly on a folder or file
```

### CLI Batch & Utility Commands
```bash
cargo run --release -- convert [options] <files...>   # Headless batch negative inversion
cargo run --release -- convert --roll *.dng           # Batch convert as a unified roll
cargo run --release -- scan-prep <files...>           # Gamma 2.2 encode linear (gamma 1.0) TIFF scans
cargo run --release -- synth [output.tif] [variant]   # Generate synthetic colour negative for testing
cargo run --release -- help                           # Display CLI help
```

#### CLI Options for `convert`:
- `--roll`: Aggregate all frames as a single roll (outlier rejection, unified base/origins).
- `--tone <NAME>`: Tone profile (e.g. `"LAB - Standard"`, `"Cinematic - Rich"`).
- `--wb <NAME>`: White balance preset (`Auto-Neutral`, `Kodak`, `Fuji`, etc.).
- `--lut <NAME>`: Enhanced LUT look (`None`, `Natural`, `Frontier`, `Crystal`, `Pakon`).
- `--border <PCT>`: Border buffer percent (default `5.0`).
- `--jpg`: Export JPEG instead of default 16-bit TIFF.
- `--out <DIR>`: Output directory (defaults to `<source>/Positives`).

### Testing
```bash
cargo test                                # Run all unit tests
cargo test --release                      # Run unit tests in release mode
# Run headless UI snapshot tests (ignored by default):
NLP_UI_TEST_DIR=<scans> NLP_UI_OUT=<dir> cargo test --release ui_screens -- --ignored
```

---

## Codebase Structure & Key Modules

```
src/
├── main.rs                 # Entry point; dispatches to CLI or GUI depending on arguments
├── cli.rs                  # Headless batch processing, scan preparation, synth negative generator
├── app/                    # GUI application (eframe / egui)
│   ├── mod.rs              # NlpApp state, Frame model, event loop, library loading, undo/redo
│   ├── folder.rs           # Folder grid view, batch controls, roll conversion panel
│   ├── editor.rs           # Editor view: canvas zoom/pan, crop/straighten, Convert/Edit/Advanced tabs
│   ├── widgets.rs          # UI widgets (plugin-styled sliders, dropdowns, curves, gradients, theme)
│   ├── jobs.rs             # Rayon background job pool (proxy loading, conversion, exports)
│   └── picker.rs           # Platform file/folder dialog (macOS NSOpenPanel / rfd)
└── engine/                 # Negative conversion & image processing engine
    ├── mod.rs              # Engine sub-modules declaration
    ├── decode.rs           # Rawler & bitmap decoders, downscaling, geometry transformations
    ├── pipeline.rs         # Conversion orchestration: base resolution, 600px analysis, roll aggregation
    ├── curve.rs            # Tone curve engine: sigmoids, multi-pass curves, gamma, WB, clipping
    ├── dcp.rs              # DNG Camera Profile (.dcp) parser, camera matching, HSV tables, matrices
    ├── look.rs             # Creative look styles (Frontier, Noritsu, Pakon, etc.) & sharpening
    ├── lut.rs              # Adobe XMP 3D LUT loader (Adobe base85 + zlib 32^3 tables)
    ├── color.rs            # MelissaRGB, ProPhoto, AdobeRGB, sRGB matrices, Bradford adapt, ICC generator
    ├── presets.rs          # NegativePresets: tone profiles, calibration tables, WB zones
    ├── settings.rs         # Inversion settings, export configuration, metadata definitions
    └── sidecar.rs          # Persistence: <image>.nlp.json sidecars and app defaults
```

---

## Processing Data Flow

1. **Ingestion (`decode.rs`)**:
   - Camera RAW: `rawler` extracts raw Bayer/demosaiced linear data in camera RGB space.
   - Bitmaps (TIFF/JPEG/PNG): Loaded as linear RGB.
2. **Profile Matching (`dcp.rs`)**:
   - Matches camera make/model or DNG `UniqueCameraModel` against `.dcp` profiles in `NLP Lightroom Plugin/LR Files/CameraRaw/CameraProfiles`.
   - Generates camera-to-MelissaRGB color transform matrix with hue/sat map and look table.
3. **Film Base Neutralisation (`pipeline.rs`)**:
   - Samples 98.5%–99.5% brightest pixels across the entire uncropped image (or user eyedropper sample) to isolate unexposed film base mask (so film borders remain available for sampling even if cropped). Once converted, this base is fixed.
4. **Analysis Buffer (`pipeline.rs`, `presets.rs`)**:
   - Downscales frame to 600px, crops border buffer, applies $\sigma=2$ Gaussian blur.
   - Computes CDF (Cumulative Distribution Function) histograms to find white/black/mean origin points.
   - Analyzes HSB zones for Auto-Neutral, Auto-Warm, and Auto-Cool white balance candidates.
   - In roll mode, aggregates frame origins with $2.5\sigma$ outlier rejection.
5. **Curve Inversion (`curve.rs`)**:
   - Builds parametric sigmoid curves (tanh) on 8-bit MelissaRGB nodes for Red, Green, and Blue channels.
   - Inverts negative densities into positive light values, applying auto-tone gamma, exposure, contrast, highlights, shadows, and color protection.
6. **Post-Inversion Looks & Sharpening (`look.rs`, `lut.rs`)**:
   - Applies 3D RGB LUTs (from `NLP Enhanced Settings` or mathematical approximations) and unsharp masking.
7. **Export & Sidecar (`export.rs`, `sidecar.rs`)**:
   - Transforms MelissaRGB $\to$ target color space (sRGB, AdobeRGB, ProPhoto RGB).
   - Generates and embeds valid ICC profile and EXIF metadata into 16-bit TIFF or JPEG.
   - Saves settings & analysis cache to `<filename>.nlp.json`.
