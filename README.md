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
    [--lut NAME] [--border PCT] [--auto-crop] [--jpg] [--long-edge PX] [--name TEMPLATE] [--out DIR] FILES
cargo run --release -- scan-prep FILES              # level + gamma 2.2 encode linear (gamma 1.0) TIF scans
cargo run --release -- synth [out.tif] [ev]         # write the modelled ColorChecker negative (ev = stops over)
```

`--roll` analyses the files together (needs 2+ frames); `--roll-process` picks Lab (density per frame, default) or Darkroom (one exposure for the roll). `--auto-crop` crops and straightens each frame to its detected frame before analysis. `--name` takes the export file name template (see Export below).

## GUI

**Folder View**
- Browse a folder of RAW and TIFF files as a thumbnail grid. Select with click, ⌘/Ctrl-click, Shift-click or ⌘A; double-click (or Enter) opens the editor.
- The top bar acts on the selection:
  - **Convert** / **Convert as roll** (2+ frames) open a dialog with the conversion inputs: source, colour model, film base, pre-saturation, border buffer, white balance, *Auto-crop frames* and, for rolls, the roll process (Lab: density per frame, or Darkroom: one exposure).
  - **Export**: positive copies as TIF (8/16-bit, ZIP) or JPG, in sRGB, AdobeRGB or ProPhoto, with an embedded ICC profile and EXIF.
    - Presets: *Lab JPEG M/L* (3000/6000 px) and *Archive TIFF*, plus your own (*Save as…*).
    - Size: full or a long edge of 2000–6000 px.
    - File name template: `{name}`, `{suffix}`, `{n}` / `{n:03}` (position in the export), `{stock}`, `{iso}`, `{date}` (scan date). Clashing names get `-2`, `-3`, ….
- Right-click a frame for open, convert, export, detect frame, copy/paste settings and unconvert.
- Lab correction keys act on the selected converted frames; the offsets show under each thumbnail (see below).

**Editor View**
- Toolbar:
  - Left: back (Space), previous/next (←/→, wraps; skips unconverted frames when the current one is converted), zoom (+/−, ⌘-scroll or double-click), undo/redo (⌘Z / ⇧⌘Z).
  - Centre: file name, tone profile and the lab correction offsets (`D+2 C−1 M0 Y+1`).
  - Right: before/after (B; ⇧B for a split view with a draggable divider), rotate, flip, crop/straighten (C), export.
- Lab correction keys, like a Frontier/Noritsu keyboard (number row or numpad): 7 8 9 add cyan, magenta, yellow; 1 2 3 take them away; 4 / 6 print denser (darker) / lighter in 1/6 EV steps; 0 resets. They move the print exposure and the printing filtration (2.5 CC per click).
- J shows clipping: red where a channel is blown, blue where the shadows are blocked. The histogram marks clipped channels in its corners.
- Hovering the image shows the output RGB and the negative's density above the film base.
- Crop mode: drag the frame edges and corners on the canvas, or use the straighten dial. Detect finds the frame inside the film rebate and holder. Reset Crop / Cancel (Esc) / Apply (Enter).
- A single-column sidebar with a histogram on top:
  - Before conversion it shows the conversion inputs (Source, Analysis). Dragging the border buffer slider outlines the analysed area on the canvas. Afterwards they fold into a one-line **Conversion** summary with Re-convert.
  - **Tone**: tone profile, Exposure, Brightness, Contrast, Lights, Darks, Whites, Blacks; *Glow, fade & clipping* holds Lab Glow/Fade and White/Black clip.
  - **White balance**: presets, a neutral eyedropper and Temp/Tint (yellow and magenta filtration in CC).
  - **Colour toning**: mids, highs and shadows.
  - **Look**: LUT with amount and saturation, sharpening.
  - **Film** (collapsed): film stock, ISO, developer and camera details for the EXIF and the file name tokens; *Apply to roll/folder* copies them.
  - **Analysis** (collapsed): the print curves and each channel's analysed densities. Roll frames get *Use roll analysis* and the roll process.
  - The ⋯ menu beside Re-convert: save/apply default, reset edits, unconvert and the LUT folder.
- Film-base eyedropper for sampling the unexposed border.

Settings are saved inside the `NLP/` subfolder as `<file>.nlp.json`.

## Engine (`src/engine`)

The engine started as a port of the Lightroom plugin and now models the film itself: a scan becomes density, density is printed onto a paper curve.

1. **Density.** `pipeline::prepare_negative` divides out uneven light with an optional flat-field frame (`flat.rs`), takes linear camera RGB through the camera matrix, divides out the film base and stores `log10(base / transmittance)` per channel, unclamped. A C-41 dye crosstalk correction (the dyes' unwanted absorptions) and the colour model's calibration are applied in density.
2. **Levels.** `pipeline::analyze` trims film rebate, holder and light past the film off the edges (found by density), then measures each channel's density range (exact 0.05 % / 99.99 % quantiles and the median) on the 600 px, border-cropped, blurred analysis buffer. Red and blue are placed on the image's neutral axis. Normalising every channel to its own range matches the layers' different H&D slopes, so neutrals stay neutral from shadows to highlights.
3. **Print.** `curve.rs` shapes the normalised density (brightness, contrast, lights/darks, whites/blacks, toning), turns it into paper log exposure plus printing filtration (white balance and the C·M·Y keys, in CC) and print time (EV), and prints it on a logistic paper curve with a toe and shoulder. The result is baked into density-indexed LUTs.
4. **Auto colour.** Auto-Neutral is the filtration that prints the near-neutral pixels grey.
5. **Rolls.** A roll is printed with one colour balance: the film's channel slopes (the median of the frames' neutral-axis slopes; one frame's fit also follows its light) and the roll's filtration (the median axis offset, leaving out frames with an outlying balance). *Lab* sets each frame's density from the frame; *Darkroom* prints the whole roll at one exposure. EXIF exposure differences are exact density offsets (`log10 2` per stop).

`testchart.rs` models a ColorChecker shot on C-41 film; `chart_quality` measures colour error, neutrality and roll consistency against it.

| Module | Role |
|---|---|
| `curve.rs` | Print engine: normalisation, tone and colour shaping, paper curve, LUTs, eyedropper |
| `pipeline.rs` | Film base, density, crosstalk, analysis, auto colour, roll aggregation, renderer |
| `presets.rs` | Colour models, pre-saturation, film WB presets, sharpening |
| `lut.rs` | Reads the plugin's 3D looks from `LR Files/CameraRaw/Settings/NLP Enhanced Settings/*.xmp` (Adobe base85 + zlib RGB tables, 32³) |
| `color.rs` | Working and output spaces, CIELAB / CIEDE2000, generated ICC profiles |
| `flat.rs` | Flat-field gain maps |
| `testchart.rs` | Physically modelled test negative and quality metrics |
| `export.rs` | Positive copies with ICC and EXIF, TIF scan prep |

### Notes
- **Raw processing.** `rawler` decodes the raw files and supplies the camera matrix.
- **Camera profiles.** The plugin's *Negative Lab* DCPs only identify VueScan/SilverFast DNGs. Their tone curves and look tables are made for scenes, so the conversion doesn't apply them.
- **Film base.** *Auto* uses the brightest 1 % of the frame; *Sampled* uses the eyedropper. In density a wrong base is a constant per-channel offset that the levels absorb, so the conversion barely depends on it (tested on a chart cropped tight with no rebate).
- **Flat field.** A frame of the bare light source or clear film leader, shot with the same camera, lens and light (Convert dialog, the editor's Source section, or `--flat FILE`). Uneven light varies across the frame, which the levels can't absorb.
- **Sidecars** from before the density engine keep their settings and metadata but need a new Convert.
- **LUT folder.** The app finds the LUTs automatically in `../NLP Lightroom Plugin/...` or in Lightroom's CameraRaw settings folder. If they are missing, it uses built-in approximations of each look.
- **Not ported:** licensing and the Lightroom catalog/DNG-profile plumbing, which a standalone app doesn't need.

## Tests

```bash
cargo test --release
NLP_UI_TEST_DIR=<scans> NLP_UI_OUT=<dir> cargo test --release ui_screens -- --ignored   # headless UI screenshots
```
