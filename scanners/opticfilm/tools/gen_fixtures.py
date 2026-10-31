# SPDX-License-Identifier: GPL-3.0-or-later
"""Record pyopticfilm reference results for the Rust parity tests.

Run from scanners/pyopticfilm with its environment (numpy, pyusb):

    python ../opticfilm/tools/gen_fixtures.py ../opticfilm/tests/fixtures

Time is virtual (sleep advances monotonic), as in the Rust tests, so poll
loops and timeouts replay identically and nothing waits.
"""

from __future__ import annotations

import time

_VT = [1000.0]


def _sleep(s):
    _VT[0] += max(0.0, float(s))


time.sleep = _sleep
time.monotonic = lambda: _VT[0]

import base64  # noqa: E402
import dataclasses  # noqa: E402
import gzip  # noqa: E402
import hashlib  # noqa: E402
import json  # noqa: E402
import sys  # noqa: E402
import tempfile  # noqa: E402
import zlib  # noqa: E402
from pathlib import Path  # noqa: E402

import numpy as np  # noqa: E402

from pyopticfilm.device.select import KNOWN_MODELS  # noqa: E402
from pyopticfilm.scan.calibrate import calib_fingerprint  # noqa: E402
from pyopticfilm.scan.geometry import compute_calib_geometry, compute_geometry  # noqa: E402
from pyopticfilm.scanner import Scanner  # noqa: E402

MODELS = {m.name: m for m in KNOWN_MODELS}
TINY = [0.0, 0.0, 0.08, 0.08]
CROP = [0.2, 0.3, 0.45, 0.5]


def txn_row(t):
    if t.operation in ("control_read", "control_write"):
        d = t.data.hex() if t.data is not None else None
        return [t.operation, t.request_type, t.request, t.value, t.index, d, t.length]
    if t.operation == "bulk_write":
        data = t.data or b""
        return ["bulk_write", len(data), hashlib.sha1(data).hexdigest()]
    return ["bulk_read", t.length]


def arr_entry(a, keep_limit=1 << 20):
    if a is None:
        return None
    a = np.ascontiguousarray(a, dtype="<u2")
    raw = a.tobytes()
    out = {"shape": list(a.shape), "sha256": hashlib.sha256(raw).hexdigest()}
    if len(raw) <= keep_limit:
        out["data"] = base64.b64encode(zlib.compress(raw, 6)).decode()
    return out


def run_scenario(spec, out_dir):
    model = MODELS[spec["model"]]
    _VT[0] = 1000.0
    with tempfile.TemporaryDirectory() as td:
        scanner = Scanner.open_fake(model, calib_cache=Path(td) / "calib.json")
        transport = scanner.protocol._transport if hasattr(scanner.protocol, "_transport") else None
        if model.asic == "GL843" and hasattr(scanner.asic, "usb"):
            transport = scanner.asic.usb.transport
        results = []
        for step in spec["steps"]:
            op = step["op"]
            res = {"op": op}
            try:
                if op == "warmup":
                    scanner.warmup()
                elif op == "home":
                    scanner.home()
                elif op == "calibrate":
                    e = scanner.calibrate(resolution=step["resolution"], mode=step.get("mode", "color"),
                                          force=step.get("force", False), area=tuple(step["area"]) if step.get("area") else None)
                    res["entry"] = {"method": e.method, "resolution": e.resolution, "startx": e.startx,
                                    "pixels": e.pixels, "asic_shading": e.asic_shading,
                                    "dark": arr_entry(e.dark), "white": arr_entry(e.white),
                                    "blob_sha1": hashlib.sha1(e.shading_blob).hexdigest() if e.shading_blob else None,
                                    "afe_offsets": list(e.afe_offsets) if e.afe_offsets else None,
                                    "afe_gains": list(e.afe_gains) if e.afe_gains else None}
                elif op == "scan":
                    kw = {k: step[k] for k in ("resolution", "mode", "apply_calib", "multi_exposure", "infrared",
                                               "align_passes", "me_long_mode", "n_passes", "single_pass_exposure",
                                               "me_short_exposure", "me_long_exposure") if k in step}
                    if step.get("area"):
                        kw["area"] = tuple(step["area"])
                    prog = []
                    img = scanner.scan(progress=prog.append, **kw)
                    res["rgb"] = arr_entry(img.rgb)
                    res["ir"] = arr_entry(img.ir)
                    res["dpi"] = img.dpi
                    res["device_model"] = img.device_model
                    res["progress_last"] = prog[-1] if prog else None
                    res["progress_n"] = len(prog)
                    dbg = scanner.last_me_debug
                    if dbg is not None:
                        res["me"] = {"exposure_short": dbg.exposure_short, "exposure_long": dbg.exposure_long,
                                     "exposure_proposed": dbg.exposure_proposed, "exposure_reason": dbg.exposure_reason,
                                     "align_shift_long": dbg.align_shift_long}
                    res["align_shift_ir"] = scanner.last_align_shift_ir
            except Exception as exc:  # noqa: BLE001
                res["error"] = type(exc).__name__
                res["message"] = str(exc)
            results.append(res)
        txns = [txn_row(t) for t in transport.transactions] if transport is not None else []
        scanner.close()
    payload = {"spec": spec, "results": results, "transactions": txns}
    path = out_dir / "scans" / f"{spec['name']}.json.gz"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(gzip.compress(json.dumps(payload).encode(), 6))
    errs = [r.get("error") for r in results if r.get("error")]
    print(f"{spec['name']}: {len(txns)} txns {errs or ''}", flush=True)


def scan(**kw):
    return {"op": "scan", **kw}


SCENARIOS = [
    # GL128 (scan-ready)
    {"name": "se_150_tiny_nocalib", "model": "plustek-opticfilm-8200i-se",
     "steps": [scan(resolution=150, area=TINY, apply_calib=False)]},
    {"name": "se_1800_tiny_calib", "model": "plustek-opticfilm-8200i-se",
     "steps": [{"op": "warmup"}, scan(resolution=1800, area=TINY)]},
    {"name": "se_1800_crop_calib", "model": "plustek-opticfilm-8200i-se",
     "steps": [scan(resolution=1800, area=CROP)]},
    {"name": "se_1200_ir_only", "model": "plustek-opticfilm-8200i-se",
     "steps": [scan(resolution=1200, area=TINY, mode="infrared")]},
    {"name": "se_1800_me", "model": "plustek-opticfilm-8200i-se",
     "steps": [scan(resolution=1800, area=TINY, multi_exposure=True)]},
    {"name": "se_1800_me_fixed", "model": "plustek-opticfilm-8200i-se",
     "steps": [scan(resolution=1800, area=TINY, multi_exposure=True, me_long_mode="fixed")]},
    {"name": "se_1800_color_ir", "model": "plustek-opticfilm-8200i-se",
     "steps": [scan(resolution=1800, area=TINY, infrared=True)]},
    {"name": "se_1800_multipass3", "model": "plustek-opticfilm-8200i-se",
     "steps": [scan(resolution=1800, area=TINY, n_passes=3)]},
    {"name": "se_7200_tiny", "model": "plustek-opticfilm-8200i-se",
     "steps": [scan(resolution=7200, area=[0.0, 0.0, 0.02, 0.02], apply_calib=False)]},
    {"name": "se_7200_me_refused", "model": "plustek-opticfilm-8200i-se",
     "steps": [scan(resolution=7200, area=TINY, multi_exposure=True)]},
    {"name": "se_calibrate_1800", "model": "plustek-opticfilm-8200i-se",
     "steps": [{"op": "calibrate", "resolution": 1800}, {"op": "calibrate", "resolution": 1800, "mode": "infrared"}]},
    {"name": "se_1800_me_nocalib", "model": "plustek-opticfilm-8200i-se",
     "steps": [scan(resolution=1800, area=CROP, multi_exposure=True, apply_calib=False)]},
    {"name": "se_1800_color_ir_nocalib", "model": "plustek-opticfilm-8200i-se",
     "steps": [scan(resolution=1800, area=CROP, infrared=True, apply_calib=False)]},
    {"name": "se_1800_multipass2_nocalib", "model": "plustek-opticfilm-8200i-se",
     "steps": [scan(resolution=1800, area=TINY, n_passes=2, apply_calib=False)]},
    {"name": "s83_1200_me_fixed_nocalib", "model": "plustek-opticfilm-8300i-se",
     "steps": [scan(resolution=1200, area=TINY, multi_exposure=True, me_long_mode="fixed", apply_calib=False)]},
    {"name": "v2_600_nocalib", "model": "plustek-opticfilm-8100-v2",
     "steps": [scan(resolution=600, area=CROP, apply_calib=False)]},
    {"name": "se_manual_exposure", "model": "plustek-opticfilm-8200i-se",
     "steps": [scan(resolution=900, area=TINY, single_pass_exposure=20000, apply_calib=False)]},
    {"name": "v2_1800_calib", "model": "plustek-opticfilm-8100-v2",
     "steps": [scan(resolution=1800, area=TINY)]},
    {"name": "v2_3600_me", "model": "plustek-opticfilm-8100-v2",
     "steps": [scan(resolution=3600, area=[0.0, 0.0, 0.04, 0.04], multi_exposure=True)]},
    {"name": "v2_ir_refused", "model": "plustek-opticfilm-8100-v2",
     "steps": [scan(resolution=1800, area=TINY, infrared=True)]},
    {"name": "s83_1800_calib", "model": "plustek-opticfilm-8300i-se",
     "steps": [scan(resolution=1800, area=TINY)]},
    {"name": "s83_600_color_ir", "model": "plustek-opticfilm-8300i-se",
     "steps": [scan(resolution=600, area=CROP, infrared=True)]},
    {"name": "s83_2400_me", "model": "plustek-opticfilm-8300i-se",
     "steps": [scan(resolution=2400, area=[0.0, 0.0, 0.05, 0.05], multi_exposure=True)]},
    # GL845 family
    {"name": "v76_900_calib", "model": "plustek-opticfilm-7600i-v2",
     "steps": [{"op": "warmup"}, scan(resolution=900, area=TINY)]},
    {"name": "v76_1800_me", "model": "plustek-opticfilm-7600i-v2",
     "steps": [scan(resolution=1800, area=TINY, multi_exposure=True)]},
    {"name": "v76_900_color_ir", "model": "plustek-opticfilm-7600i-v2",
     "steps": [scan(resolution=900, area=TINY, infrared=True)]},
    {"name": "v76_900_ir_only", "model": "plustek-opticfilm-7600i-v2",
     "steps": [scan(resolution=900, area=TINY, mode="infrared")]},
    {"name": "v76_3600_prefeed", "model": "plustek-opticfilm-7600i-v2",
     "steps": [scan(resolution=3600, area=[0.3, 0.4, 0.33, 0.43], apply_calib=False)]},
    {"name": "v76_home", "model": "plustek-opticfilm-7600i-v2",
     "steps": [{"op": "warmup"}, {"op": "home"}]},
    {"name": "8200i_900_tiny", "model": "plustek-opticfilm-8200i",
     "steps": [scan(resolution=900, area=TINY, apply_calib=False)]},
    {"name": "8100_calib", "model": "plustek-opticfilm-8100",
     "steps": [scan(resolution=1800, area=TINY)]},
    {"name": "7400v2_tiny", "model": "plustek-opticfilm-7400-v2",
     "steps": [scan(resolution=1800, area=TINY, apply_calib=False)]},
    {"name": "7200i_calib", "model": "plustek-opticfilm-7200i",
     "steps": [scan(resolution=1800, area=TINY)]},
    {"name": "7200v2_tiny", "model": "plustek-opticfilm-7200-v2",
     "steps": [scan(resolution=1800, area=TINY, apply_calib=False)]},
    {"name": "7300_calib", "model": "plustek-opticfilm-7300",
     "steps": [scan(resolution=1800, area=TINY)]},
    {"name": "7400v1_tiny", "model": "plustek-opticfilm-7400-v1",
     "steps": [scan(resolution=1800, area=TINY, apply_calib=False)]},
    {"name": "7500i_calib", "model": "plustek-opticfilm-7500i",
     "steps": [scan(resolution=1800, area=TINY)]},
    {"name": "7200_calib", "model": "plustek-opticfilm-7200",
     "steps": [scan(resolution=1800, area=TINY)]},
    # GL843 v1 (vendor job replay on the simulated chip)
    {"name": "v1_1440_color", "model": "plustek-opticfilm-7600i-v1",
     "steps": [{"op": "warmup"}, scan(resolution=1440, area=[0.4, 0.4, 0.45, 0.45])]},
    {"name": "v1_3600_ir_recorded", "model": "plustek-opticfilm-7600i-v1",
     "steps": [scan(resolution=3600, mode="infrared", apply_calib=False, area=[0.4, 0.4, 0.42, 0.42])]},
    {"name": "v1_calibrate_refused", "model": "plustek-opticfilm-7600i-v1",
     "steps": [{"op": "calibrate", "resolution": 1440}]},
]


def geometry_fixture(out_dir):
    rows = []
    areas = [None, TINY, CROP, [0.0, 0.5, 1.0, 1.0], [0.1, 0.0, 0.9, 0.3]]
    for m in KNOWN_MODELS:
        for dpi in m.resolutions_dpi:
            for area in areas:
                try:
                    g = compute_geometry(dpi, model=m, area=tuple(area) if area else None)
                    rows.append({"model": m.name, "dpi": dpi, "area": area, "geometry": dataclasses.asdict(g)})
                except Exception as exc:  # noqa: BLE001
                    rows.append({"model": m.name, "dpi": dpi, "area": area, "error": type(exc).__name__})
            try:
                g = compute_calib_geometry(dpi, model=m)
                rows.append({"model": m.name, "dpi": dpi, "calib": True, "geometry": dataclasses.asdict(g)})
            except Exception as exc:  # noqa: BLE001
                rows.append({"model": m.name, "dpi": dpi, "calib": True, "error": type(exc).__name__})
    (out_dir / "geometry.json").write_text(json.dumps(rows))
    fps = {m.name: calib_fingerprint(m) for m in KNOWN_MODELS}
    (out_dir / "fingerprints.json").write_text(json.dumps(fps, indent=1))
    print(f"geometry: {len(rows)} rows")


def noise(shape, seed):
    """Deterministic integer-hash noise in [-1, 1) (reproduced in the Rust tests)."""
    n = int(np.prod(shape))
    i = np.arange(n, dtype=np.uint64) + np.uint64(seed) * np.uint64(0x9E3779B97F4A7C15)
    x = i * np.uint64(6364136223846793005) + np.uint64(1442695040888963407)
    x ^= x >> np.uint64(33)
    x *= np.uint64(0xFF51AFD7ED558CCD)
    x ^= x >> np.uint64(33)
    return ((x >> np.uint64(11)).astype(np.float64) / float(1 << 53) * 2.0 - 1.0).reshape(shape)


def synth_scene(h, w):
    yy, xx = np.mgrid[0:h, 0:w].astype(np.float64)
    base = 9000 + 7000 * np.sin(xx / 37.0) * np.cos(yy / 53.0) + 4000 * ((xx // 60 + yy // 45) % 2)
    return np.stack([base, base * 0.8 + 2000, base * 0.6 + 4000], axis=2)


def out_entry(a, step=5):
    """sha256 of the whole array plus a strided sample for tolerance checks."""
    a = np.ascontiguousarray(a, dtype="<u2")
    sample = np.ascontiguousarray(a[::step, ::step])
    return {"shape": list(a.shape), "sha256": hashlib.sha256(a.tobytes()).hexdigest(), "step": step,
            "sample": base64.b64encode(zlib.compress(sample.tobytes(), 6)).decode()}


def u16(a):
    return np.clip(np.rint(a), 0, 65535).astype(np.uint16)


def numerics_fixture(out_dir):
    from pyopticfilm.pass_align import align_pass_to_reference, align_pass_to_reference_banded, estimate_pass_shift
    from pyopticfilm.registration.resample import warp_cubic, warp_linear
    from pyopticfilm.scan.calib_gl128 import flatten_ir_columns, flatten_ir_image_columns
    from pyopticfilm.scan.exposure_merge import merge_exposures_result, merge_n_passes
    from pyopticfilm.scan.me_exposure import select_long_exposure

    h, w = 600, 700
    scene = synth_scene(h, w)
    short = u16(scene + 60 * noise(scene.shape, 1))
    long_ = u16(scene * 3.0 + 120 * noise(scene.shape, 2))
    shifted = np.roll(np.roll(long_, 2, axis=1), 1, axis=0)
    cases = {}
    m = merge_exposures_result(short, long_, exposure_short=14000, exposure_long=42000, align_shift=(0.0, 0.0))
    st = m.fusion_stats
    cases["merge_noshift"] = {"rgb": out_entry(m.rgb), "ratio": list(st.exposure_ratio_per_channel),
                              "alpha": list(st.noise_alpha), "beta": list(st.noise_beta),
                              "fitted": list(st.noise_fitted), "zero": st.zero_weight_pixels}
    frames = [u16(scene + 80 * noise(scene.shape, 10 + k)) for k in range(3)]
    mp = merge_n_passes(frames)
    cases["merge_n3"] = {"rgb": out_entry(mp.rgb), "outliers": mp.fusion_stats.outlier_pixels}
    cases["shift_est"] = list(estimate_pass_shift(short, np.roll(np.roll(short, 3, axis=1), -2, axis=0)))
    cases["shift_est_long"] = list(estimate_pass_shift(short, shifted))
    al, s = align_pass_to_reference(short, shifted)
    cases["align"] = {"rgb": out_entry(al), "shift": list(s)}
    tall = np.concatenate([short] * 4, axis=0)
    alb, sb = align_pass_to_reference_banded(tall, np.roll(tall, 1, axis=1))
    cases["banded"] = {"rgb": out_entry(alb), "shift": list(sb)}
    cases["warp_linear"] = out_entry(warp_linear(short, 1.3, -0.6))
    cases["warp_cubic"] = out_entry(warp_cubic(short, -2.25, 0.4))
    d = select_long_exposure(short, 14000)
    cases["select_long"] = {"proposed": d.proposed, "selected": d.selected, "reason": d.reason,
                            "p05": list(d.dense_p05), "clip": list(d.predicted_clip)}
    ir = short[:, :, 1]
    white = [int(v) for v in 20000 + 3000 * np.sin(np.arange(w) / 90.0)]
    cases["flatten_ir_columns"] = out_entry(flatten_ir_columns(ir, white))
    cases["flatten_ir_image"] = out_entry(flatten_ir_image_columns(ir))
    probe = noise((4,), 3)
    (out_dir / "numerics.json").write_text(json.dumps({"noise_probe": list(probe), "cases": cases}))
    print("numerics written")


def replay_fixture(out_dir):
    from pyopticfilm.device.model_7600i_v1 import MODEL_7600I_V1

    out = {}
    for mode, dpi in [("color", 1440), ("color", 3600), ("color", 7200), ("infrared", 3600), ("infrared", 7200)]:
        p = MODEL_7600I_V1.replay_profile(mode, dpi)
        out[f"{mode}_{dpi}"] = hashlib.sha256(json.dumps(p, sort_keys=True, separators=(",", ":")).encode()).hexdigest()
    (out_dir / "replay_profiles.json").write_text(json.dumps(out, indent=1))
    print("replay profiles hashed")


def main():
    out_dir = Path(sys.argv[1] if len(sys.argv) > 1 else "fixtures")
    out_dir.mkdir(parents=True, exist_ok=True)
    only = set(sys.argv[2:])
    if not only or "geometry" in only:
        geometry_fixture(out_dir)
    if not only or "numerics" in only:
        numerics_fixture(out_dir)
    if not only or "replay" in only:
        replay_fixture(out_dir)
    for spec in SCENARIOS:
        if not only or spec["name"] in only or "scans" in only:
            run_scenario(spec, out_dir)


if __name__ == "__main__":
    main()
