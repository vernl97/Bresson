# Crosstalk matrices

These matrices come from [NegPy](https://github.com/marcinz606/NegPy) by marcinz606,
licensed under GPL-3.0, and are copied unchanged from its `crosstalk/` gallery. They are
here for personal use in NLP Studio. Don't distribute them under another licence.

Each file holds one film's dye crosstalk correction: a 3×3 matrix applied to the
negative's density (rows are output R/G/B, columns input R/G/B). NegPy read the values
off published spectral-dye-density sheets, sampled at 650/550/450 nm, so every one is
*approx*:

- **Narrowband scanners.** They describe the dyes as a narrowband scanner reads them.
  Broader scanner channels mix more than these matrices undo.
- **Kodak C-41 sheets.** Gold, Portra, Ektar and Ultramax publish only a midscale-neutral
  curve, so their matrices are Vision3 dye curves fitted to it.
- **Fuji duplicates.** Several Fuji files share one matrix.

NLP Studio embeds every file at build time (`presets::crosstalk_profiles`). The
**Crosstalk** setting picks one by file name; empty means the engine's generic C-41
matrix. **Crosstalk strength** blends the matrix with the identity, where 0 % is off and
100 % is the matrix as written. Both are analysis inputs, so changing them needs Convert.

To add a matrix, drop in a `.toml` with `name = "…"` and `matrix = [[…], […], […]]`,
then add it to `CROSSTALK_FILES` in `src/engine/presets.rs`.
