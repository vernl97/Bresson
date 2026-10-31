# opticfilm

A Rust driver for Plustek OpticFilm USB film scanners (Genesys GL128, GL845,
GL843 and GL842 ASICs). It is a port of the Python
[pyopticfilm](../pyopticfilm) driver (v1.3.8), kept 1:1 so the two can be
diffed transaction by transaction. NLP Studio will use it to scan straight
into the app.

GPL-3.0-or-later, like pyopticfilm (which carries SANE genesys tables).

## Support

Same gate as pyopticfilm: `Scanner::scan`, `calibrate`, `home` and `park`
refuse models that are not hardware tested. `Scanner::open_fake` lifts the
gate for the mock transports only.

| Model | USB ID | ASIC | Support |
|-------|--------|------|---------|
| OpticFilm 8200i SE | `07b3:1825` | GL128 | Hardware tested (in pyopticfilm) |
| OpticFilm 8100 (V2) | `07b3:1824` | GL128 | Hardware tested (no IR) |
| OpticFilm 8300i SE | `07b3:181f` | GL128 | Hardware tested (IR) |
| OpticFilm 7600i v1 | `07b3:0c3b` bcd 0400 | GL843 (vendor job replay) | Hardware tested |
| OpticFilm 7600i v2 | `07b3:0c3b` | GL845 | Hardware tested (IR, ME) |
| OpticFilm 8200i | `07b3:130d` | GL845 | Protocol validated, scan locked |
| 8100, 7500i, 7400 (v1/v2), 7300, 7200i, 7200 | | GL845 / GL843 / GL842 | Experimental, scan locked |

"Hardware tested" refers to the Python driver. **The Rust port itself has not
yet run on a physical scanner**; see [Validation](#validation).

## Use

```rust
use opticfilm::{Scanner, ScanControl, ScanRequest};

let mut scanner = Scanner::open(None, None)?;          // first scan-ready OpticFilm
scanner.warmup(true, true)?;                            // init, home, lamp
let image = scanner.scan(&ScanRequest { resolution: 3600, multi_exposure: true, ..Default::default() },
                         &mut ScanControl::default())?;
image.save_tiff("frame.tif")?;                         // 16-bit linear negative
```

`ScanImage::rgb` is HxWx3 `u16`, linear film-negative values (bright means
dense). `ScanImage::ir` holds the infrared dust plane when one was scanned;
check `ir_tracks_image()` before using it on silver black-and-white film.

The CLI wraps the same calls:

```bash
cargo run -p opticfilm --release -- list
cargo run -p opticfilm --release -- scan --dpi 3600 --me --out frame.tif
cargo run -p opticfilm --release -- scan --fake 8200i-se --dpi 1800 --ir --out mock.tif
```

## Layout

| Module | pyopticfilm |
|--------|-------------|
| `usb` | `usb/` — nusb device (no libusb), Genesys framing, trace JSON, fake / mock / playback transports |
| `device` | `device/` — models (`generated.rs` is written by `tools/gen_models.py`), selection, sensor lookups, 7600i v1 job data |
| `asic` | `asic/` — GL845/843/842 (one driver, `Gl84xVariant`), GL128, GL843 v1 |
| `scan` | `scan/` — geometry, pipeline, sessions, calibration + cache, ME/Multi-Pass merge, vendor-job replay |
| `registration`, `pass_align` | `registration/`, `pass_align.py` |
| `image`, `scanner` | `image.py`, `scanner.py` |
| `clock` | `time.sleep` / `time.monotonic`, with a virtual clock for mock runs |

Model tables are generated, not hand-copied:

```bash
cd ../pyopticfilm && python ../opticfilm/tools/gen_models.py > ../opticfilm/src/device/generated.rs
```

## Validation

Parity with pyopticfilm is the test oracle. `tools/gen_fixtures.py` runs the
Python driver on its mock transports with a virtual clock and records USB
traffic and outputs into `tests/fixtures/`. The tests:

- `parity_scans` — 59 scenarios over all 14 models (colour, IR, ME adaptive and
  fixed, Multi-Pass, calibration, home, error paths, 7600i v1 job replay):
  **byte-identical USB traffic and pixel-identical images**. The 18 `sim_*`
  scenarios run calibrated GL128 scans on a mock with a simulated AD9826
  front end (`tools/gl128_sim.py`, ported as `usb/fake_gl128_sim.rs`), where
  pyopticfilm's own mock can only reach calibration's error paths: AFE
  search, ASIC shading, IR host flatten, explicit and forced calibration, dpi
  changes, on all three GL128 models.
- `parity_geometry` — `compute_geometry` / `compute_calib_geometry` for every
  model, resolution and five crops (468 cases), plus calibration-cache
  fingerprints: exact.
- `parity_numerics` — ME merge, Multi-Pass stacking, registration, warps, IR
  flatten, long-exposure selection: bit-exact except banded Multi-Pass
  alignment (≤ 1 LSB on 0.01 % of samples, `np.polyfit` vs normal equations).
  The 7600i v1 vendor jobs, including the derived 7200 dpi infrared job, are
  identical.
- `parity_decode` — trace JSON, log lines and decoded events of mock scans.
- The other test files port pyopticfilm's unit tests where they reach code
  the parity scenarios miss (golden register programs, model-lock oracles,
  GL128 motor safety and image read, calibration cache, scanner façade,
  selection, registration, exposure, USB framing, TIFF I/O).

```bash
cargo test -p opticfilm --release
cd ../pyopticfilm && python ../opticfilm/tools/gen_fixtures.py ../opticfilm/tests/fixtures   # regenerate
```

The calibration cache uses pyopticfilm's format and version (`calib_v2.json`,
v22) at `dirs::cache_dir()/opticfilm/`.

Not yet covered:

- A scan on real hardware. Start with `status`, then `lamp`, then a small
  `scan --area` before full frames.
- Queued bulk reads on hardware. The 7600i v1 replay keeps one bulk read
  queued, as pyopticfilm's worker thread does, through nusb's own transfer
  queue; on the mocks it matches the plain reads byte for byte
  (`tests/queued_bulk_read.rs`), but it has not run on a scanner.
- The simulated front end is a model, not a capture: it proves both drivers
  calibrate the same way, not that either matches a real sensor. Its strips
  are flat-field and DVDSET shading is not applied to image reads.
- Unit tests not ported: those built on a mock ASIC object (their paths now
  run through the `sim_*` scenarios) and the numpy input-type tests.
- Scan Lab (`tools/scanlab`) and the forensic tools are out of scope.
