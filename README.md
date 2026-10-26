# NLP Studio

A standalone Rust port of the **Negative Conversion Plugin** for Lightroom (see `../SPEC.md` and `../manual.md`). It reads camera RAW negatives or TIFF/JPEG scans, inverts them with the plugin's curve engine, and writes positive copies.

## Build & run

```bash
cargo run --release                     # GUI, reopens the last folder
cargo run --release -- /path/to/scans   # GUI on a folder (or a single file)
cargo run --release -- help
```

### Command line

```bash
cargo run --release -- convert [--roll] [--roll-process NAME] [--tone NAME] [--wb NAME] \
    [--lut NAME] [--border PCT] [--jpg] [--out DIR] FILES
cargo run --release -- scan-prep FILES              # level + gamma 2.2 encode linear (gamma 1.0) TIF scans
cargo run --release -- synth [out.tif] [variant]    # write a synthetic colour negative for testing
```

`--roll` analyses the files together (needs 2+ frames); `--roll-process` picks Pro Lab, Basic Lab or Darkroom Paper.

## GUI

**Folder View**
- Browse a folder of RAW and TIFF files as a thumbnail grid. Select with click, ⌘/Ctrl-click, Shift-click or ⌘A; double-click (or Enter) opens the editor.
- The top bar acts on the selection:
  - **Convert** / **Convert as roll** (2+ frames) open a dialog with the conversion inputs: source, colour model, film base, pre-saturation, border buffer, white balance and, for rolls, the roll process (Pro Lab, Basic Lab or Darkroom Paper).
  - **Export**: positive copies as TIF (8/16-bit, ZIP) or JPG, in sRGB, AdobeRGB or ProPhoto, with an embedded ICC profile and EXIF.
- Right-click a frame for open, convert, export, copy/paste settings and unconvert.

**Editor View**
- Toolbar:
  - Left: back (Space), previous/next (←/→, wraps; skips unconverted frames when the current one is converted), zoom (+/−, ⌘-scroll or double-click), undo/redo (⌘Z / ⇧⌘Z).
  - Centre: file name and tone profile.
  - Right: before/after (B), rotate, flip, crop/straighten (C), export.
- Crop mode: drag the frame edges and corners on the canvas, or use the straighten dial. Reset Crop / Cancel (Esc) / Apply (Enter).
- A single-column sidebar with a histogram on top:
  - Before conversion it shows the conversion inputs (Source, Analysis). Dragging the border buffer slider outlines the analysed area on the canvas. Afterwards they fold into a one-line **Conversion** summary with Re-convert.
  - **Tone**: tone profile, Exposure, Brightness, Contrast, Lights, Darks, Whites, Blacks; *Glow, fade & clipping* holds Lab Glow/Fade, Soft Highs/Lows and White/Black clip.
  - **White balance**: presets, a neutral eyedropper and Temp/Tint.
  - **Colour toning**: mids, highs and shadows.
  - **Look**: LUT with amount and saturation, sharpening.
  - **Engine** (collapsed): engine presets v2.1 to v3.1, curve points, processing order, WB density and type, plus a curve and analysis readout. Roll frames get *Use roll analysis* and the roll process.
  - The ⋯ menu beside Re-convert: save/apply default, reset edits, unconvert and the LUT folder.
- Film-base eyedropper for sampling the unexposed border.

Settings are saved next to each scan as `<file>.nlp.json`.

## Engine (`src/engine`)

| Module | Ported from |
|---|---|
| `curve.rs` | `CurveEngineV3.lua`: tanh sigmoids, auto-tone gamma, every tone and colour layer, colour-protected and linear clipping, node inversion, single- and multi-pass curves |
| `pipeline.rs` | `NLPImageProcessorV3.lua`: 600 px analysis, border buffer, σ=2 blur, CDF clipping (0.01 % / 0.05 %), HSB-zone auto WB; `NLPRolls.lua`: roll aggregation (min/max origins, 2.2 power-mean means, 2σ colour-ratio outliers) exposure matching by EXIF shutter/aperture/ISO (`commonEV`), and the Pro Lab / Basic Lab / Darkroom Paper roll processes |
| `presets.rs` | `NegativePresets.lua`: tone profiles, colour models, pre-saturation, film WB presets, engine presets |
| `lut.rs` | Reads the plugin's real 3D LUTs from `LR Files/CameraRaw/Settings/NLP Enhanced Settings/*.xmp` (Adobe base85 + zlib RGB tables, 32³) |
| `color.rs` | `NLPColorV3.lua`: MelissaRGB working space, output spaces, generated ICC profiles |
| `export.rs` | `NLPExportServiceProviderV3.lua`, `ExifV3.lua`, `GammaUtilityV3.lua` |

### Where it differs from the plugin
- **Raw processing.** `rawler` decodes the raw files.
- **Camera profiles.** The plugin's *Negative Lab v2.3* camera profiles (`LR Files/CameraRaw/CameraProfiles`) are applied automatically.
  - Matching follows Lightroom: the DNG UniqueCameraModel, or make + model. VueScan/SilverFast DNGs use the generic scanner profile, as the plugin's DNG updater arranges.
  - The pipeline follows the DNG spec: film base as the white balance, interpolated colour matrix, look table, baseline exposure offset and Adobe's hue-preserving tone curve.
  - Cameras without a profile fall back to `rawler`'s colour matrix.
  - Adobe's own raw processing (demosaic, highlight handling) still differs from `rawler`'s, so results are close to Lightroom but not identical.
- **Film base.** Lightroom's border-eyedropper white balance becomes the **Film base** option. *Auto* uses the brightest 1 % of the frame; *Sampled* uses the eyedropper.
- **Exposure.** The plugin's Exposure slider (±50 curve units, applied in node space before the other tone layers) is replaced by a print exposure in EV (±5). It models paper density and is applied to the finished curve. Sidecars from older versions are converted at `exposure / 10`.
- **LUT folder.** The app finds the LUTs automatically in `../NLP Lightroom Plugin/...` or in Lightroom's CameraRaw settings folder. If they are missing, it uses built-in approximations of each look.
- **Not ported:** licensing and the Lightroom catalog/DNG-profile plumbing, which a standalone app doesn't need.

## Tests

```bash
cargo test --release
NLP_UI_TEST_DIR=<scans> NLP_UI_OUT=<dir> cargo test --release ui_screens -- --ignored   # headless UI screenshots
```
