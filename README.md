# NLP Studio

A standalone Rust port of the **Negative Conversion Plugin** for Lightroom (see `../SPEC.md` and `../manual.md`). It reads camera RAW negatives or TIFF/JPEG scans, inverts them with the plugin's curve engine, and writes positive copies.

## Build & run

```bash
cargo run --release                     # GUI, reopens the last folder
cargo run --release -- /path/to/scans   # GUI on a folder (or a single file)
cargo run --release -- convert --roll *.dng   # headless batch conversion
cargo run --release -- help
```

## Windows

**Folder View**
- Browse a folder as a thumbnail grid. Select with click, ⌘/Ctrl-click, Shift-click or ⌘A.
- The right panel works on the whole selection:
  - **Convert**: set the conversion options, convert frames individually or *as a roll*, or unconvert.
  - **Edit**: batch-edit tone, white balance and LUT; copy, paste or sync settings; save defaults.
  - **Export**: positive copies as TIF (8/16-bit, ZIP) or JPG, in sRGB, AdobeRGB or ProPhoto, with an embedded ICC profile.
  - **Metadata**: film camera, lens, stock, ISO, shutter, aperture and developer. These go into EXIF / TIFF tags.
  - **Tools**: TIF Scan Prep for gamma-1.0 flatbed scans, and the LUT folder location.

**Editor View** (double-click a frame, or press Enter)
- Light toolbar:
  - Left: back (Space), previous/next (←/→, wraps; skips unconverted frames when the current one is converted), zoom (+/−, ⌘-scroll or double-click), undo/redo.
  - Centre: file name and tone profile.
  - Right: before/after (B), rotate, flip, crop/straighten (C), export.
- Crop mode: drag the frame edges and corners on the canvas, or use the straighten dial. Reset Crop / Cancel (Esc) / Apply (Enter).
- **Convert / Edit / Advanced** tabs follow the manual's layout:
  - Tone profiles; Exposure, Brightness, Contrast, Lights, Darks, Whites, Blacks, Lab Glow and Lab Fade; Soft Highs/Lows; WhiteClip and BlackClip.
  - White balance: presets, a neutral eyedropper and Temp/Tint.
  - Mids, highs and shadows toning; LUT with amount and saturation; sharpening.
  - Advanced: engine presets v2.1 to v3.1, curve points, processing order, WB density and WB type, toning, clipping, plus a curve and analysis readout. Frames converted as a roll get *Use roll analysis* and the *Roll process*: Pro Lab, Basic Lab or Darkroom Paper.
- Film-base eyedropper, rotate, flip and crop.
- Histogram, export dialog, undo/redo (⌘Z / ⇧⌘Z).

Settings are saved next to each scan as `<file>.nlp.json`.

## Engine (`src/engine`)

| Module | Ported from |
|---|---|
| `curve.rs` | `CurveEngineV3.lua`: tanh sigmoids, auto-tone gamma, every tone and colour layer, colour-protected and linear clipping, node inversion, single- and multi-pass curves |
| `pipeline.rs` | `NLPImageProcessorV3.lua`: 600 px analysis, border buffer, σ=2 blur, CDF clipping (0.01 % / 0.05 %), HSB-zone auto WB; `NLPRolls.lua`: roll aggregation (min/max origins, 2.2 power-mean means, 2σ colour-ratio outliers) and the Pro Lab / Basic Lab / Darkroom Paper roll processes |
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
- **Roll exposure normalisation.** The plugin reads each frame's aperture, shutter and ISO and offsets frames that were digitised at a different exposure; this isn't ported, so digitise a roll with fixed settings.
- **LUT folder.** The app finds the LUTs automatically in `../NLP Lightroom Plugin/...` or in Lightroom's CameraRaw settings folder. If they are missing, it uses built-in approximations of each look.
- **Not ported:** licensing and the Lightroom catalog/DNG-profile plumbing, which a standalone app doesn't need.

## Tests

```bash
cargo test --release
NLP_UI_TEST_DIR=<scans> NLP_UI_OUT=<dir> cargo test --release ui_screens -- --ignored   # headless UI screenshots
```
