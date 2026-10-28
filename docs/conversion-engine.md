# How the conversion engine works

This document walks through `src/engine/` from the file on disk to the
pixels of a positive copy. Each stage names the function that implements
it and gives the numbers it uses, so it can be read side by side with the
code.

The short version: the engine does **not** invert pixel values. It turns
the scan into **film density** per channel, measures the density range and
the neutral axis of the frame (or the roll), and then "prints" that
density onto a modelled paper with real printing controls: filtration in
CC, print time in stops, and a paper/tone curve. Every channel ends up as a
pure function of its own density, so the whole print is baked into three
lookup tables.

```
file ──decode──▶ SourceImage (linear camera RGB)
                    │  resolve_base: film base colour (roll base at the frame's scan exposure)
                    │  frame::detect: auto crop / straighten (optional, roll fallback)
                    ▼
            prepare_negative ──▶ Negative (density above base, per channel)
                    │
          analyze ──┴──▶ Analysis (Levels, neutral axis, auto WB, skin)
                    │      └─ aggregate_roll ──▶ RollBalance + roll Levels
                    ▼
     levels_for ──▶ Levels the frame prints with
                    ▼
     Renderer:  [dodge offset] → CurveLut (print engine) → Look
                → output colour space → [sharpening]  ──▶ export.rs
```

---

## 1. Decoding — `decode.rs`, `demosaic.rs`

`decode::load(path, max_long_edge)` returns a `SourceImage`:

| field | meaning |
|---|---|
| `data` | linear pixels, **camera RGB** for RAW, linear (sRGB‑primaried) RGB for bitmaps |
| `to_melissa` | 3×3 matrix source space → linear MelissaRGB |
| `as_shot_wb` | the camera's WB multipliers, normalised to green = 1 |
| `info` | make/model, full size, EXIF exposure (shutter, f‑number, ISO), `linear_raw`, TIFF `Software` tag |

**RAW files** (via `rawler`):

- The camera matrix comes from the first colour matrix found for D65, D55,
  Daylight, D50, D75, Flash or A (falling back to sRGB). It is turned into a
  MelissaRGB → camera matrix whose rows are normalised so RGB white maps to
  camera white, then inverted. This normalisation matters later: a pixel
  that is neutral in camera space stays neutral in MelissaRGB.
- **With a size limit** (proxies, thumbnails, analysis) a Bayer/X‑Trans CFA is
  *binned*, not demosaiced: after black/white scaling, each `block × block`
  tile (even size, ≥ 2) is averaged per CFA colour (`bin_cfa`).
- **At full resolution** (exports), Bayer goes through `demosaic::rcd`, an
  RCD‑style directional demosaic (direction from vertical/horizontal
  activity, ratio‑corrected green, colour‑difference red/blue, 4‑px border
  averaged). X‑Trans uses rawler's Markesteijn pass 1. Anything else
  (LinearRaw scanner DNGs, 4‑colour, monochrome) goes through rawler's own
  develop. The default crop is applied relative to the active area.
- EXIF orientation is applied to the pixels.

**Bitmaps** (TIFF/JPEG/PNG) are decoded to float, run through the sRGB
decode, and treated as sRGB primaries (`to_melissa` = sRGB → MelissaRGB with
Bradford adaptation).

The working space throughout is **MelissaRGB**: ProPhoto primaries, D50
white, sRGB tone response (`color.rs`).

`pipeline::detect_source` classifies the file: a RAW that is LinearRaw or
whose Software tag mentions VueScan/SilverFast is a *VueScan / SilverFast
DNG*, other RAWs are *Digital Camera*, bitmaps are *TIF Scan*. *Linear Scan*
(gamma 1.0 TIFFs) is chosen by hand.

---

## 2. Film base — `pipeline::resolve_base`, `base_multipliers`

The film base (unexposed orange mask) is the reference everything is
measured against.

- **`auto_base`**: sort all pixels of the **uncropped** source by R+G+B and
  average those between the 98.5th and 99.5th percentile. The crop is
  deliberately ignored so the rebate is always available.
- `FilmBase::Auto` uses the roll base when there is one, otherwise
  `auto_base`. The roll base (`roll_base`, used by the app's roll convert
  and the CLI's `convert --roll`) is the per‑channel median of the frames'
  auto bases, but a frame does not print against it as is:
  `base_at_exposure` scales it to the frame's own scan exposure. A scanner
  that exposes every frame for itself (VueScan does) reads the same film
  base up to 3× brighter on one frame than on the next; against the plain
  median, a thin night frame read thinner than the base and its pixels were
  dropped as flare (5.2), printing it black. The scale is the largest of the
  three ratios `own_c / roll_c`, so the base is at least as bright as the
  frame's own reading in every channel and no image reads thinner than it;
  the small per‑channel residue (up to 0.06 D on real rolls) is a constant
  density offset the levels absorb.
- `FilmBase::Sampled` uses the eyedropper sample (`film_base_sample`).
- `AsShot` and `Off` still compute `auto_base` but use it differently.

`base_multipliers` turns the base into per‑channel gains in source space:

| mode | multipliers |
|---|---|
| Auto, Sampled | `0.97 / base[c]` — the base lands at `BASE_LEVEL = 0.97` in every channel |
| AsShot | the camera WB, scaled so the brightest WB‑corrected base channel is 0.97 |
| Off | `[1, 1, 1]` |

Because the gains are applied in **camera space** and the camera matrix maps
camera‑neutral to RGB‑neutral, the base becomes a neutral 0.97 grey — the
same thing as clicking Lightroom's WB eyedropper on the rebate.

---

## 3. Optional frame detection — `frame.rs`

When auto‑crop is on, `frame::detect(src, settings, base)` proposes a crop
and a straighten angle on a 600‑px copy (after quarter turns / flip):

1. **Classify** every pixel against the base (`r = p / base`, `m` = mean of r):
   - *Outside* (holder or light past the film): `m ≥ 1.2`, or any channel
     below 5 % of the base.
   - *Rebate*: `m > 0.85` and all channels within 12 % of `m`.
   - *Image*: everything else.
2. **Angle**: on each of the four sides, for the middle 40 % of lines find
   where the border ends (3 image pixels in a row, within 2/5 of the frame),
   fit a straight line, keep it if RMS ≤ 1.5 px. The median of the side
   angles is used if it is between 0.1° and 5° (rounded to 0.01°). The
   buffer is then straightened and reclassified.
3. **Insets**: walking in from each side, a line is border if ≥ 25 % of its
   middle half is *Outside*, or ≥ 50 % is border *and* (mostly outside, or
   the rebate part is flat: s.d. of `m` < 0.03). The image starts at the
   first run of 6 non‑border lines (at most 40 % in); 1 % safety margin is
   added per detected side. Fails if less than 30 % of the area remains.
4. `fit_aspect` shrinks the result around its centre to the crop aspect
   (`Settings::crop_aspect`; *Original* = the frame's own ratio) when one is
   set. The app detects **free‑form** (`CropAspect::Free`: auto‑convert, the
   folder's auto‑crop action and the editor's detect) and leaves the frame's
   aspect unlocked, so the crop follows the frame; the CLI keeps the
   `--aspect` it is given.

**Roll fallback** (`roll_geometry`, `settle`). A night scene is mostly as
clear as the rebate, so its edge disappears into it: detection finds
nothing, or a wrong frame. When frames are converted together, their
geometry is pooled: `roll_geometry` is the median of every side and of the
angle over at least 3 auto‑cropped frames (`ROLL_FRAMES`). A frame whose
detection failed, or put any side more than 10 % from the roll's
(`ROLL_DEVIATION`), takes the roll's geometry — a scanner holds every frame
in the same place. Only frames detected in that conversion are replaced, so
a crop set by hand is kept.

Geometry is not part of `AnalysisKey`; it is applied in step 4.

---

## 4. The negative in density — `pipeline::prepare_negative`

For every pixel, in this order:

1. **Flat field** (`flat.rs`, optional). A frame of the bare light source is
   downscaled to 256 px, blurred (σ = 6), reduced to a 64‑px gain map and
   normalised to the mean of its central tenth: `gain = centre / pixel`. It
   is sampled bilinearly and multiplied in at source resolution. (Uneven
   light is multiplicative, i.e. a spatially varying density offset the
   levels cannot absorb.)
2. **Geometry** (`decode::geometry`): quarter turns → flip → straighten
   (bilinear, cropped to the largest border‑free rectangle of the same
   aspect) → crop insets.
3. *Linear Scan only*: values are sRGB‑encoded again (undoing the bitmap
   decode, so the file's values are used as linear), and so is the base.
4. **Base gains** in source space, then **`to_melissa`** to linear MelissaRGB.
5. **Density above the base**: `D = log10(0.97 / max(v, 1e‑5))`.
   The base is ≈ 0; denser film is positive; anything thinner than the base
   (flare, light past the film) is negative. Nothing is clamped. A wrong base
   is only a constant per‑channel offset in density, which the levels absorb.
6. **Density matrix** (`density_matrix`), applied as `D' = M · D`:
   - **C‑41 crosstalk removal** (colour negatives only): the inverse of
     ```
     C41_CROSSTALK = [[1.00, 0.06, 0.02],   R
                      [0.10, 1.00, 0.05],   G
                      [0.04, 0.15, 1.00]]   B   (columns: C, M, Y dye)
     ```
     with each row normalised to sum 1, so neutral density stays neutral.
   - **Colour model calibration** (`calibration_matrix`, values from
     `presets::calibration`): Lightroom‑style red/green/blue *hue* and
     *saturation* per model (Basic / Frontier / Noritsu, different for camera
     scans and scanner files). A hue shift moves a primary's column towards a
     neighbour by `0.25 % × hue`; saturation scales its distance from the
     weighted grey (`DENSITY_WEIGHTS = [0.30, 0.55, 0.15]`). Rows are
     normalised to sum 1.
   - **Saturation in density**: `f = (1 + pre_saturation%) × 0.85`, matrix
     `yw·(1 − f) + f·I`. Pre‑saturation steps 1..5 map to −20, −10, 0, +5,
     +8 %. The fixed `DENSITY_SATURATION = 0.85` compensates for the ~20 %
     extra chroma that crosstalk removal plus steep per‑channel curves would
     otherwise print.
7. **Black & white** (colour model B+W): all three channels become the
   weighted density `0.30 R + 0.55 G + 0.15 B`.

*Edit Only* frames (already positive) skip gains, matrix and crosstalk:
`D = −log10(v)`.

The result is a `Negative { width, height, data: Vec<[f32; 3]> }`.

---

## 5. Analysis — `pipeline::analyze`

Analysis runs on the proxy (1024 px in the app, 1200 px in the CLI) and
produces an `Analysis`: `levels`, `auto_wb`, `base`, the `AnalysisKey` it was
made with, the capture `ev`, the fitted `axis` and the `skin` found.

### 5.1 Analysis buffer — `analysis_buffer`

1. Downscale to a 600‑px long edge.
2. **Trim non‑image edges** (`frame_insets`). A pixel is *not image* if all
   channels are within ±0.05 D of the base (rebate), any channel is below
   −0.15 D (light past the film), or all channels exceed the *opaque*
   density (holder: 0.2 D below the frame's 99.9th‑percentile minimum‑channel
   density, at least 1.4 D). Edge rows/columns where at least half the
   pixels are not image are trimmed (up to 20 % per side) plus 2 extra lines.
3. Crop the **border buffer** (`border_buffer`, default 5 %) from every side.
4. Gaussian blur, σ = 2 px.

### 5.2 Levels — `levels_from_buffer`

`Levels` is one `ChannelLevels { lo, hi, mid, bow }` per channel, in density:
`lo` prints black, `hi` prints white, `mid` is the midtone key, `bow` the
crossover curvature (5.3).

**Per‑channel quantiles** (`channel_quantiles`). Pixels thinner than the base
by more than 0.03 D in any channel are ignored on negatives. For a negative,
`lo` is the 0.05 % quantile (`clip_thresholds`: black 5·10⁻⁴), `hi` the
99.99 % quantile (white 10⁻⁴), `mid` the median; `hi ≥ lo + 0.05`. Positives
use 10⁻⁴ at both ends and are flipped later.

**Highlight cap** (negatives). A light source or specular is far denser than
the scene and would print everything dark, so green's white end is capped at

```
cap = max(mid + 2·(mid − lo), mid + 0.1)
```

*Exception — a lit subject in a dark frame* (flash at night): if the median
sits in the bottom 25 % of the range (`DARK_SHARE`), the 97th percentile of
green is above the cap, and the top 3 % still spans more than 0.1 D
(`LIT_SPREAD`), the cap moves to the 97th percentile instead. Holder pixels
(above the opaque density) are excluded from these percentiles. Every
channel's range is then shortened by the same fraction
`f = (cap − lo_g) / (hi_g − lo_g)` (≤ 1) so neutrals stay neutral.

**Minimum range** (negatives). A nearly unexposed frame has almost no
density above the base (0.1 D); stretched over the paper's whole range,
its fog and grain print as a full‑range image of noise. If green's range is
under `MIN_RANGE = 0.3 D`, every channel's range is lengthened by the same
factor `0.3 / (hi_g − lo_g)`, so what little the frame holds prints dark.
On 189 real scans, frames span 0.6–1.0 D (10th–90th percentile) and
underexposed night scenes reach down to 0.3 D, so only nearly blank frames
change; they never look right, they look like dark frames.

**Neutral axis**, iterated three times. Each time `neutral_axis` is fitted
under the current levels; for red and blue:

- the slope `b` against green is clamped to `[0.8, 1.25] ×` the channel's own
  range ratio (`(hi_c − lo_c)/(hi_g − lo_g)`);
- `lo_c`, `hi_c` become the axis' density at green's `lo` and `hi`
  (`hi ≥ lo + 0.05`);
- `mid_c` keeps the channel's own median;
- `bow_c` comes from the axis' bend (`bow_for`, see below).

Green keeps its own (capped) range. If there is no axis, every channel keeps
its own quantile range. Matching red and blue to the axis is what makes the
layers' different H&D slopes cancel, so a grey prints grey from shadows to
highlights.

### 5.3 The neutral axis — `neutral_axis`, `neutral_cluster`

1. Normalise every buffer pixel under the current levels (0 = black end …
   1 = white end, see 6.1). Keep *near‑neutral* pixels: channel spread < 0.1
   and green within 0.03..0.97.
2. Bin them by green into **10 tonal steps**. A step needs at least
   `max(N/500, 8)` pixels.
3. Each step's grey is **`neutral_cluster`**: in the (R−G, B−G) offset plane,
   a mean‑shift window starts on the current axis (0, 0) with half‑widths
   0.03, 0.03, 0.02, 0.02. If fewer than `max(min_count/2, 8)` pixels lie
   within 0.02 of the result, it restarts from the step's median offset
   (0.05, 0.03, 0.02, 0.02). The step's grey is the per‑channel median
   density of that cluster, or the plain median if no cluster is found. This
   keeps a skin, sand or wood surface lit across many tones from pulling the
   grey onto itself.
4. At least **4** steps are required, otherwise there is no axis.
5. Least‑squares lines `D_c = a + b·D_g` through the steps (each step weighs
   the same). **Outlier steps**: while more than 4 remain, the step furthest
   off the line (max of red/blue residual) is dropped if it is more than
   `max(3 × 1.4826 × MAD, 0.02)` D off, and the lines are refitted.
6. **Bend (crossover)**: with ≥ 6 steps (`BEND_STEPS`) a quadratic is added,
   `bend_c · p(g)` with `p(g) = (g − m)² − u(g − m) − v` built orthogonal to
   the line over the steps, so the line fit is untouched. The bend is scaled
   by a *trust* that rises from 0 to 1 as the steps cover 50 → 80 % of
   green's range (no extrapolating a curve fitted to part of the range).
   This mirrors the per‑film gradation balance curves of Fuji lab scanners
   (US 6,160,634, EP 0 612 183).

`confidence = steps / 10`.

`bow_for` converts the bend into `ChannelLevels::bow`, a quadratic in
normalised green: a grey at normalised green `x` has the channel's density at
`lo + (hi − lo)·((1 + bow)·x − bow·x²)`. `|bow|` is limited to
`MAX_BOW = 0.5` and to a departure of at most 0.05 D from the straight line.

### 5.4 Auto white balance — `auto_wb`

The grey density used:

- with an axis: the axis at green's median (`axis.at(mid_g)`), confidence
  `steps/10`;
- without: mean of near‑neutral pixels, confidence 0.2;
- with neither: the channel medians, confidence 0.

`wb_for_density` normalises that grey per channel and asks
`curve::balance_wb` for the filtration that prints it neutral (paper exposure
`u = (1 − x)·1.05`; temp = `(u_R − u_B)/0.01`, tint = `(u_R − u_G)/0.01`, in
CC). The candidates stored in `AutoWb`:

| mode | (temp, tint) |
|---|---|
| Auto‑Neutral | the neutral balance |
| Auto‑Warm | neutral + (6, 1.5) |
| Auto‑Cool | neutral − (6, 0) |
| Auto‑Mix | temp halfway between neutral and warm, neutral tint |
| Auto‑AVG | the balance that prints the gray‑world mean of all in‑range pixels neutral |

Static film‑character presets (`presets::film_character`, magenta/yellow in
CC): Standard 3/5.5, Kodak 2/5.5, Fuji 1/5.5, Cine‑D 3/5.5, Cine‑T 2/7.5,
None 0/0. `wb_for_mode` writes the chosen pair into `wb_temp` / `wb_tint`;
from then on WB is ordinary printing filtration (6.2).

### 5.5 Skin‑keyed midtone — `skin.rs`

After the levels, colour negatives look for faces, like the Frontier's Auto
Setup:

1. If green's range is under 0.8 D (`SKIN_RANGE`), the white end is
   temporarily extended so it is 0.8 D (a flashed face at night can be far
   beyond a capped white end).
2. `is_skin` on normalised channels `x`: `(x_R − x_B)/max(x_G, 1)` in
   0.05..0.3, red holding 33–65 % of the red‑blue difference, green in
   0.15..2.0.
3. Morphological opening (3×3 erode, then dilate), then 4‑connected areas.
   An area counts as a face when it does not touch the frame edge, covers
   ≥ 0.2 % of the frame, fills ≥ 50 % of its bounding box, and its
   height/width is 0.6..2.5. Score = `min(share / 1 %, 1)`.
4. The skin tone is the score‑weighted median green of all face pixels;
   weight = sum of scores (≤ 1).

`skin::keyed` then moves the levels' midtone key:

- If the skin sits above normalised 0.9, the white end moves out (every
  channel by the same share, weighted by `weight`) until it would sit at 0.75.
- Two midtone gammas are computed: `g_mid = ln 0.5 / ln m` (the frame's median
  to 0.5) and `g_skin = ln 0.65 / ln s` (skin to `SKIN_TARGET = 0.65`), both
  clamped to 0.8..1.6, and blended in log with skin taking
  `0.6 × weight` (`SKIN_SHARE`).
- Each channel's `mid` is set to the density whose normalised value the auto
  midtone (6.2) turns into that blended gamma.

---

## 6. The print engine — `curve.rs`

### 6.1 Normalised density

`ChannelLevels::normalise(d)` maps density to 0 (`lo`, prints black) … 1
(`hi`, prints white). With a bow it inverts the quadratic neutral curve
(root of `q·x² − (1 + q)·x + y = 0`) so a grey has the *same* normalised
value in every channel; beyond the ends it continues straight with the
curve's end slopes. `density(x)` is the inverse.

Positives use `1 − normalise(d)`. Edit Only maps `d` back to the sRGB‑encoded
value `srgb_encode(10^−d)`.

### 6.2 One channel's curve — `ChannelCurve`

For each channel, density `d` goes through:

```
x  = normalised(d)                              # with clip:
x  = (x − bc) / (1 − bc − wc)                   # bc = black_clip·0.006, wc = white_clip·0.006
x  = x·(1 + whites'·0.006) + blacks'·0.004·(1 − x)
x  = pow_ext(x, gamma · mid_gamma_c)            # power on 0..1, straight outside
x += 0.10·highlights'/50 · highlight_bump(x, 5)
x += 0.10·shadows'/50    · shadow_bump(x, 5)
x += tone_high_c   · highlight_bump(x, high_range)
x += tone_shadow_c · shadow_bump(x, shadow_range)
x  = 0.5 + (x − 0.5)·(1 + contrast·0.015)       # tone(d)

u  = (1 − x)·PAPER_RANGE + exposure_c           # paper log exposure
y  = 1 − u / PAPER_RANGE                        # = x − exposure_c / 1.05
out = tone_curve(profile knots, y)
```

with

- `whites' = whites − glow`, `blacks' = blacks + fade`,
  `highlights' = highlights + glow/1.5`, `shadows' = shadows − fade/1.5`
  (Lab Glow / Lab Fade are combinations of the four).
- `gamma = auto × brightness`, `brightness = 2^(−brightness/50 · 0.7)`.
  **Auto midtone** (`auto_gamma`, only for LAB profiles, which have the auto
  flag, and only on negatives): `ln 0.5 / ln m`, where `m` is the mean
  normalised `mid` of the three channels, clamped to 0.8..1.6 — the frame's
  key prints at the middle, with bright frames allowed to come down further
  than dark ones go up.
- `mid_gamma_c = 2^(mid_c/75 · 0.5)` for the Mids colour sliders (cyan → red,
  tint → green, temp → blue). Highlight/shadow toning:
  `tone_* = −value/75 · 0.12`, as a bump whose width the range slider sets.
- `bump(x, a) = x^a·(1 − x)`, normalised to 1 at its peak `a/(a+1)`;
  `highlight_bump(x, r)` uses `a = 1 + (11 − r)·0.4`, `shadow_bump` mirrors it.
- **Paper**: `PAPER_RANGE = 1.05` log exposure between printed white and
  black (ISO R ≈ 105, grade 2). `CC = 0.01` log exposure.
  `exposure_c = (filtration_c + wb_c)·CC − print_exposure·log10 2`, where
  `wb = [0, wb_tint, wb_temp]`. So:
  - **Print exposure** is in stops: +1 EV removes 0.301 log E (a print
    ≈ 29 % of the paper range lighter).
  - **WB and the C·M·Y keys are subtractive filtration**: +Y (temp) adds
    blue‑channel exposure, i.e. less blue, warmer; +M (tint) takes green;
    +C takes red. The lab keys (`settings::lab`) step density by 1/6 EV and
    colour by 2.5 CC, within ±5 EV / ±50 CC.

### 6.3 Tone profiles

`profile()` holds the plugin's 14 tone profiles as **21 knots** each over
normalised density 0..1 (measured: the plugin's printed grey on the modelled
ColorChecker at −1, 0, +1 EV, regressed against this engine's normalised
density). The LAB profiles carry the auto‑midtone flag; Cinematic and Linear
ones do not.

`tone_curve` is a monotone Fritsch–Carlson cubic through the knots. Below 0
it decays exponentially towards black (or linearly to 0 if the first knot is
0); above 1 it approaches paper white exponentially with the end tangent. So
nothing hard‑clips inside the curve itself.

### 6.4 Lookup tables — `CurveLut`

Each channel is evaluated at **8192** densities from −0.5 to 4.5 D, clamped
to 0..1, then forced monotone (a running max, or min if the curve falls) so
extreme slider settings can never fold the curve back. `apply(c, d)`
interpolates linearly. The tables output **sRGB‑encoded MelissaRGB**.

`CurveLut::negative_view` is the unconverted view: `srgb_encode(0.97·10^−d)`,
the scan as captured with the base at 0.97.

### 6.5 Eyedropper WB — `eyedropper_wb`

With WB zeroed, the sampled density is run through `tone()`; samples above
0.97 or below 0.03 are rejected as too bright/dark. Otherwise temp and tint
are the blue and green paper‑exposure differences from red, in CC — the
filtration that gives all three channels equal exposure, i.e. grey.

---

## 7. Hyper Tone (dodge & burn) — `dodge.rs`

Off by default (`hyper_tone_highlights/shadows = 0`); only for negatives.
After the Frontier's Hyper Tone (US 6,807,316).

**Low‑pass** (`low_pass`, once per negative): green density on a 256‑px map,
blurred twice — wide σ = 12 % and narrow σ = 5 % of the long edge. They are
blended `s + (l − s)·exp(−((l − s)/0.1)²)`: the wide blur in smooth areas,
the narrow one where they disagree by ≳ 0.1 D (near strong edges), which
prevents halos. Upsampled bilinearly. (In the GPU preview it rides in the
density texture's alpha channel.)

**Offset** per pixel, with `u = low − key` (`key` = green `mid`):

```
offset = −g · u³ / (u² + 0.3²)          g = slider/100 · 0.4  (light side if u > 0, dark side if u < 0)
D_c   += offset · gain_c                gain_c = (hi_c − lo_c)/(hi_g − lo_g)
```

Near the key (|u| ≪ 0.3 D) almost nothing changes, so midtone contrast is
kept; far from it up to 40 % of the departure is taken away. The per‑channel
gain keeps greys grey. This is added to the density **before** the curve
LUTs.

---

## 8. Looks, output space and sharpening

### 8.1 Looks — `look.rs`, `assets/looks.json`

Applied to the encoded MelissaRGB positive (`Renderer::positive`) of
converted frames. Each look (Natural, Frontier, Crystal, Pakon) has
per‑channel grey curves (11 points), a 24 hue × 5 saturation × 8 value map of
(hue shift°, saturation scale, value shift) like a DNG look table, a maximum
strength, and 10 saturation levels (level 5 = 1).

For a pixel in HSV:

1. `sat = skin_kept(h, s, saturation_level)`: the saturation scale eases back
   to 1 on skin (hue 25° ± 20°, smoothstepped, fading out as `s` goes from
   0.5 to 0.75) — like the Frontier's Bright mode.
2. Without a look (or strength 0) only that saturation is applied (*None*
   uses the looks' average saturation levels).
3. With a look at strength `k` (`lut_strength/100`, capped at the look's
   maximum): trilinear lookup of (dh, ds, dv); the grey curves' mean `gv` and
   `dv` set value `v + (gv + dv − v)·k`, saturation
   `s·(1 + (ds − 1)·k)·sat`, hue `h + dh·k`; then each channel adds its grey
   curve's departure from the mean, `(grey_c − gv)·(1 − s)·k` (the tint the
   look gives to neutrals).

B+W frames get no look and saturation 1.

### 8.2 Output colour — `Renderer::output`, `color::output_transform`

sRGB‑decode → linear MelissaRGB → 3×3 matrix (with Bradford D50 → D65 for
sRGB / Adobe RGB) → encode: sRGB curve, Adobe gamma 563/256, or the ProPhoto
curve. `export.rs` embeds a matching ICC profile and writes TIFF (8/16‑bit,
optional ZIP) or JPEG with EXIF built from the film metadata.

### 8.3 Sharpening — `look::sharpen`

Only on CPU renders (exports) with Sharpening = Lab or Scanner, after the
output conversion, on encoded values. After the Frontier's Hyper Sharpness:

- Per channel, a fine band (`p − blur(radius)`) and a wider band
  (`p − blur(2.5·radius)`, the grain's scale).
- The local R‑G, G‑B, R‑B correlation of the wider bands (blurred products,
  σ = 2.5·radius) separates **edges** (present in all three dye layers at
  once) from **grain** (each layer has its own). `edge = smoothstep((r − 0.3)/0.5)`.
- Each pixel gets `amount · edge · luma(fine)` added to all channels
  (unsharp mask on luminance, with a soft threshold), and in grain‑only
  areas 25 % of the middle band removed: `− 0.25·(1 − edge)·(band − fine)`.

Presets: Lab = amount 0.9, radius 1.3, threshold 0.012; Scanner = 0.6, 1.0,
0. `with_megapixels` scales them to the file: factor `1 + (MP − 24)/48`,
strength capped at the equivalent of Lightroom 150, radius 1–3 px; the radius
also scales with any export downsizing.

---

## 9. Rolls — `aggregate_roll`, `levels_for`

A roll is printed like a lab prints one: the film's characteristics are
shared, the light per frame may not be.

### 9.1 Exposure matching

`frame_ev` = `log2(N²/t) + log2(ISO/100)`, rounded to thirds. `common_ev` is
the most frequent EV. One stop less light reads `log10 2 ≈ 0.301 D` denser
against a shared base, so with a common EV every frame's densities are moved
by `−(ev − common)·0.301` before comparing. (The app asks before continuing
when frames differ by more than 1 EV; the CLI then drops exposure data.)

### 9.2 Aggregation

From each frame's analysis, `frame_axis` gives its slopes, a grey on its axis
and its bends (or, without an axis, the slopes of its channel ranges and its
medians). *Reliable* frames have confidence ≥ 0.4 (all frames if fewer than
two qualify).

- **Bend** per channel: median of the reliable frames' bends (from ≥ 6 steps).
  **Pivot**: median green `mid`. **Reach**: median half green range; beyond
  it the bend levels off (`RollBalance::bent`).
- **Slopes** — the film's H&D ratios: median of each reliable frame's slope,
  carried along the roll's bend to the pivot.
- **Offsets** — the roll's filtration: each frame's offset that puts the roll
  axis through its grey (`through`). With ≥ 3 reliable frames, red and blue
  offsets further than `max(3.5 × 1.4826 × MAD, 0.05 D)` from the median mark
  a frame as an outlier (excluded and reported). The roll offset is the median
  of the remaining reliable frames.
- **Roll levels**: green `lo/hi/mid` = medians over non‑excluded frames
  (exposure‑shifted); red and blue on the roll axis (`RollBalance::levels`).
- **Roll auto WB**: grey on the roll axis at green's mid, the mean of the
  frames' means, confidence = the best frame's.

### 9.3 What a frame prints with — `levels_for`, `auto_wb_for`

- `use_roll` off, or no roll: the frame's own levels.
- **Darkroom** mode: the roll's levels and balance for every frame, shifted
  only by its EV difference — one exposure, one filtration.
- **Lab** mode: trust `t = clamp((confidence − 0.4)/0.2, 0, 1)`. The roll's
  balance is evaluated at the frame's **own green range**, and each of `lo`,
  `hi`, `mid`, `bow` is blended `roll + t·(own − roll)`. A frame with ≥ 6
  neutral steps prints exactly as it converts alone; one with ≤ 4 takes the
  film's slopes and the roll's filtration at its own density. `auto_wb_for`
  blends the grey the same way, so Auto‑Neutral matches the chosen levels.

---

## 10. Rendering — `pipeline::Renderer`

`Renderer::new(settings, levels, output_space)` builds:

- `CurveLut` from the levels (or `negative_view` if the frame is unconverted),
- the `Look`,
- the output matrix and encoding LUT,
- the sharpening preset,
- the `Dodge`, if Hyper Tone is on.

`render(neg, sharpen_scale)`, in parallel per pixel:

```
d    = density (+ dodge offset · gain_c)
p    = [lut_R(d_R), lut_G(d_G), lut_B(d_B)]        # encoded MelissaRGB
p    = look(p)                                     # converted frames only
out  = encode_out(M_out · srgb_decode(p))
```

then sharpening over the whole buffer.

Where it runs:

- **Editor preview**: `app/gpu.rs` + `preview.wgsl` run dodge, curve LUTs,
  look and output in a shader (no sharpening) on the 2048‑px proxy;
  `gpu_preview_matches_the_cpu_renderer` keeps it in step. The CPU renders a
  1024‑px copy for the histogram, readout and clipping, and is the fallback.
- **Export** (`export::export_frame`): decode at full resolution, the
  analysis' stored base, `prepare_negative`, optional downscale to the
  export's long edge, `levels_for`, `Renderer` with `with_megapixels`, render,
  write.

---

## 11. What triggers a re‑analysis

`Settings::analysis_key()` → `AnalysisKey`: engine version
(`ENGINE_VERSION = 4`), source, image type, colour model, pre‑saturation,
border buffer, film base mode and sample, flat field. These change the
density image or how it is measured, so changing any of them needs
**Convert** again. Everything else — tone, WB, filtration, print exposure,
looks, Hyper Tone, sharpening, geometry, roll mode — only rebuilds the
`Renderer`.

---

## 12. How quality is checked

`testchart.rs` models a ColorChecker exposed onto C‑41‑like film (per‑layer
speed and H&D slope with a soft toe, dye crosstalk, an orange mask) and
scanned by a camera with overlapping channels. Because the scene is known,
`pipeline::tests::chart_quality` can gate the conversion against it:

| metric | bound |
|---|---|
| colour ΔE00 against the chart's true colours | < 5.1 |
| tone‑matched ΔE00 | < 4.5 |
| mean / max neutral‑patch chroma C* | < 0.4 / < 1.0 |
| frame‑to‑frame spread across −1, 0, +1 EV (single / roll) | < 0.65 / < 0.65 |
| roll neutral C* | < 0.5 |

`dominated_scenes_stay_neutral` checks that scenes filled with one colour
still find their greys, and `roll_lab_matches_single_frames` that Lab mode
never prints a confident frame worse than converting it alone. The bounds
only tighten; run

```bash
cargo test --release chart_quality -- --nocapture
```

to print the metrics. `cargo run --release -- synth out.tif [ev]` writes the
modelled negative so it can be opened like a real scan.

The chart is a 3‑channel model whose assumptions match the engine's own, so
it can mislead on colour questions (removing the scanner's colour matrix
scored 6 ΔE00 better on the chart and looked clearly worse on real scans).
Engine changes are also checked on **real scans** with `scan_check.rs`, an
ignored test:

```bash
NLP_SCAN_DIRS=<roll>:<folder of rolls> NLP_SCAN_OUT=<dir> [NLP_SCAN_BASELINE=<old stats.json>] \
  cargo test --release real_scans -- --ignored --nocapture
```

Every folder with scans directly inside is a roll (a folder without any is
searched one level down). Each roll is converted the way the app's automatic
convert does — auto‑crop with the roll fallback, roll base at each frame's
exposure, roll in Lab mode, auto WB — with default settings and without
reading or writing sidecars. Real scans have no reference colours, so the
check reports health and consistency per roll: median axis confidence and
frames below 0.4, roll outliers, median L* and its spread, white clipping,
the cast (C*) of near‑neutral areas and its spread across the roll, the
highlight‑against‑shadow cast (crossover), mean and 95th‑percentile chroma,
and frames more than half clipped to black. `NLP_SCAN_OUT` saves
`stats.json` and a JPEG of every print; with `NLP_SCAN_BASELINE` the frames
whose cast moved more than 1.5 C* or median L* more than 2 are listed. Keep
the baseline outside the repo, and compare against it rather than gating on
values: stock‑typical casts are film character.
