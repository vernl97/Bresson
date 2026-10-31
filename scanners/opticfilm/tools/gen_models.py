# SPDX-License-Identifier: GPL-3.0-or-later
"""Emit src/device/generated.rs from the live pyopticfilm model objects.

Run from scanners/pyopticfilm with its environment:
    python ../opticfilm/tools/gen_models.py > ../opticfilm/src/device/generated.rs

Every register map and table comes straight from Python, so the Rust port
programs byte-identical values. Model *behaviour* (methods) is hand-ported in
src/device/model.rs.
"""

from __future__ import annotations

import dataclasses
import importlib
import inspect

from pyopticfilm.device.protocol import MotorProfile
from pyopticfilm.device.select import KNOWN_MODELS

GL128 = {"plustek-opticfilm-8200i-se", "plustek-opticfilm-8100-v2", "plustek-opticfilm-8300i-se"}
REG_FIELDS = {"init_regs", "sensor_custom_regs", "frontend_regs", "gpo_regs", "memory_layout_regs"}
CONST_MODULES = [
    "gl128_common", "tables_8200i_se", "tables_8300i_se",
    "model_8200i_se", "model_8100_v2", "model_8300i_se", "model_7600i_v1",
]
KIND = {
    "Model8200i": "Sane8200i", "Model7400": "Sane7400", "Model7300": "Sane7300",
    "Model7500i": "Sane7500i", "Model7200i": "Sane7200i", "Model7200": "Sane7200",
    "Model7600iV1": "Opticfilm7600iV1", "Model8200iSE": "Gl128_8200iSe",
    "Model8100V2": "Gl128_8100V2", "Model8300iSE": "Gl128_8300iSe",
}


def rs_str(s: str) -> str:
    return '"' + s.replace("\\", "\\\\").replace('"', '\\"') + '"'


def rs_float(v: float) -> str:
    r = repr(float(v))
    return r if ("." in r or "e" in r or "inf" in r) else r + ".0"


def key_rs(k) -> str:
    """lperiod/frontend overlay keys: (method, dpi), dpi or (method,)."""
    if isinstance(k, tuple):
        method = f"Some(ScanMethod::{'Infrared' if k[0] == 'infrared' else 'Transparency'})"
        dpi = f"Some({k[1]})" if len(k) > 1 else "None"
        return f"MdKey({method}, {dpi})"
    return f"MdKey(None, Some({k}))"


def val_rs(name: str, v) -> tuple[str, str]:
    """(rust type, rust expr) for a model field value."""
    if isinstance(v, bool):
        return "bool", "true" if v else "false"
    if isinstance(v, int):
        return "i64", str(v)
    if isinstance(v, float):
        return "f64", rs_float(v)
    if isinstance(v, str):
        return "&'static str", rs_str(v)
    if isinstance(v, MotorProfile):
        f = ", ".join(f"{x.name}: {getattr(v, x.name)}" for x in dataclasses.fields(v))
        return "MotorProfile", f"MotorProfile {{ {f} }}"
    if isinstance(v, tuple):
        return "Vec<i64>", "vec![" + ", ".join(str(int(x)) for x in v) + "]"
    if isinstance(v, dict):
        if name in REG_FIELDS:
            items = ", ".join(f"(0x{k:02X}, 0x{val:02X})" for k, val in v.items())
            return "RegMap", f"RegMap::from_pairs(&[{items}])"
        if name in ("lperiod_by_dpi",):
            items = ", ".join(f"({key_rs(k)}, {val})" for k, val in v.items())
            return "OrdMap<MdKey, i64>", f"OrdMap::from_vec(vec![{items}])"
        if name == "frontend_regs_by_dpi":
            items = ", ".join(
                f"({key_rs(k)}, RegMap::from_pairs(&[" + ", ".join(f"(0x{a:02X}, 0x{b:02X})" for a, b in d.items()) + "]))"
                for k, d in v.items()
            )
            return "OrdMap<MdKey, RegMap>", f"OrdMap::from_vec(vec![{items}])"
        if name == "stagger_y_by_dpi":
            items = ", ".join(f"({k}, vec![{', '.join(map(str, t))}])" for k, t in v.items())
            return "OrdMap<i64, Vec<i64>>", f"OrdMap::from_vec(vec![{items}])"
        items = ", ".join(f"({k}, {val})" for k, val in v.items())
        return "OrdMap<i64, i64>", f"OrdMap::from_vec(vec![{items}])"
    raise TypeError(f"{name}: {type(v)}")


# Field types are taken from the first non-None, non-empty value seen.
FORCE_TYPES = {
    "calib_white_home_y_mm": "f64", "afe_white_target": "f64", "auto_exposure_target": "f64",
    "window_start": "i64", "lperiod_by_dpi": "OrdMap<MdKey, i64>",
    "register_dpiset_by_dpi": "OrdMap<i64, i64>", "output_pixel_offset_by_dpi": "OrdMap<i64, i64>",
    "stagger_y_by_dpi": "OrdMap<i64, Vec<i64>>",
}
EMPTY = {
    "Vec<i64>": "vec![]", "RegMap": "RegMap::default()", "OrdMap<i64, i64>": "OrdMap::default()",
    "OrdMap<MdKey, i64>": "OrdMap::default()", "OrdMap<i64, Vec<i64>>": "OrdMap::default()",
}


def main() -> None:
    order: list[str] = []
    types: dict[str, str] = {}
    present: dict[str, set[str]] = {}
    for m in KNOWN_MODELS:
        for f in dataclasses.fields(m):
            v = getattr(m, f.name)
            if f.name not in order:
                order.append(f.name)
            present.setdefault(f.name, set()).add(m.name)
            if f.name in FORCE_TYPES:
                types[f.name] = FORCE_TYPES[f.name]
            elif f.name not in types and v is not None and not (isinstance(v, (dict, tuple)) and not v):
                types[f.name] = val_rs(f.name, v)[0]
    all_names = {m.name for m in KNOWN_MODELS}
    common = [n for n in order if present[n] == all_names]
    gl128 = [n for n in order if present[n] == GL128]
    optional = [n for n in order if n not in common and n not in gl128]

    out: list[str] = []
    w = out.append
    w("// SPDX-License-Identifier: GPL-3.0-or-later")
    w("// @generated by tools/gen_models.py from pyopticfilm; do not edit by hand.")
    w("#![allow(clippy::all, clippy::unreadable_literal)]\n")
    w("use super::model::{FilmModel, MdKey, ModelKind, MotorProfile, OrdMap, RegMap, ScanMethod};\n")

    # Struct definitions.
    w("/// Model fields present on every OpticFilm model.")
    w("#[derive(Debug, Clone, PartialEq)]")
    w("pub struct ModelFields {")
    for n in common:
        w(f"    pub {n}: {types[n]},")
    for n in optional:
        w(f"    pub {n}: Option<{types[n]}>,")
    w("}\n")
    w("/// Fields only GL128 models (8200i SE, 8100 V2, 8300i SE) carry.")
    w("#[derive(Debug, Clone, PartialEq)]")
    w("pub struct Gl128Data {")
    for n in gl128:
        w(f"    pub {n}: {types[n]},")
    w("}\n")

    def expr(n, v):
        if v is None:
            return "None"
        t = types[n]
        if isinstance(v, (dict, tuple)) and not v:
            return EMPTY[t]
        if t == "f64" and isinstance(v, int) and not isinstance(v, bool):
            return rs_float(v)
        return val_rs(n, v)[1]

    fn_names = []
    for m in KNOWN_MODELS:
        fn = m.name.replace("plustek-opticfilm-", "model_").replace("-", "_")
        fn_names.append((fn, m.name))
        kind = KIND[type(m).__name__]
        w(f"pub fn {fn}() -> FilmModel {{")
        w("    FilmModel {")
        w(f"        kind: ModelKind::{kind},")
        w("        f: ModelFields {")
        names = {f.name for f in dataclasses.fields(m)}
        for n in common:
            w(f"            {n}: {expr(n, getattr(m, n))},")
        for n in optional:
            if n in names and getattr(m, n) is not None:
                w(f"            {n}: Some({expr(n, getattr(m, n))}),")
            else:
                w(f"            {n}: None,")
        w("        },")
        if m.name in GL128:
            w("        gl128: Some(Box::new(Gl128Data {")
            for n in gl128:
                w(f"            {n}: {expr(n, getattr(m, n))},")
            w("        })),")
        else:
            w("        gl128: None,")
        w("    }")
        w("}\n")
    w("/// Every known model, in the Python `KNOWN_MODELS` order.")
    w("pub fn known_models() -> Vec<FilmModel> {")
    w("    vec![" + ", ".join(f"{fn}()" for fn, _ in fn_names) + "]")
    w("}\n")

    # Module constants.
    for mod in CONST_MODULES:
        mm = importlib.import_module("pyopticfilm.device." + mod)
        w(f"/// Constants from `pyopticfilm.device.{mod}`.")
        w(f"pub mod {mod} {{")
        for k, v in vars(mm).items():
            name = k.lstrip("_")
            if not name.isupper() or inspect.ismodule(v) or callable(v) or name == "TYPE_CHECKING":
                continue
            if isinstance(v, bool):
                continue
            if isinstance(v, int):
                w(f"    pub const {name}: i64 = {v};")
            elif isinstance(v, float):
                w(f"    pub const {name}: f64 = {rs_float(v)};")
            elif isinstance(v, str):
                w(f"    pub const {name}: &str = {rs_str(v)};")
            elif isinstance(v, frozenset):
                items = ", ".join(rs_str(s) for s in sorted(v))
                w(f"    pub static {name}: &[&str] = &[{items}];")
            elif isinstance(v, tuple):
                w(f"    pub static {name}: &[i64] = &[{', '.join(map(str, v))}];")
            elif isinstance(v, dict) and v:
                kk, vv = next(iter(v.items()))
                if isinstance(vv, tuple) and any(isinstance(x, float) for t in v.values() for x in t):
                    items = ", ".join(f"({a}, &[{', '.join(rs_float(x) for x in t)}])" for a, t in v.items())
                    w(f"    pub static {name}: &[(i64, &[f64])] = &[{items}];")
                elif isinstance(vv, tuple):
                    items = ", ".join(f"({a}, &[{', '.join(map(str, t))}])" for a, t in v.items())
                    w(f"    pub static {name}: &[(i64, &[i64])] = &[{items}];")
                else:
                    items = ", ".join(f"({a}, {b})" for a, b in v.items())
                    w(f"    pub static {name}: &[(i64, i64)] = &[{items}];")
        w("}\n")
    print("\n".join(out))


if __name__ == "__main__":
    main()
