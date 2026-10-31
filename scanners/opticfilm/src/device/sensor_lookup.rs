// SPDX-License-Identifier: GPL-3.0-or-later
//! DPI / method keyed sensor lookups (SANE `tables_sensor.cpp` shape).
//!
//! Fallbacks use the flat `exposure_lperiod` / `sensor_custom_regs` /
//! `frontend_regs` fields every model has.

use super::model::{FilmModel, MdKey, OrdMap, RegMap, ScanMethod};

fn lookup_method_dpi<V: Clone>(table: Option<&OrdMap<MdKey, V>>, method: ScanMethod, resolution: i64) -> Option<V> {
    let table = table.filter(|t| !t.is_empty())?;
    for key in [MdKey(Some(method), Some(resolution)), MdKey(None, Some(resolution)), MdKey(Some(method), None)] {
        if let Some(v) = table.get(&key) {
            return Some(v.clone());
        }
    }
    None
}

/// `LPERIOD` exposure for `resolution` / `method`.
pub fn exposure_lperiod_for(model: &FilmModel, resolution: i64, method: ScanMethod) -> i64 {
    lookup_method_dpi(model.lperiod_by_dpi.as_ref(), method, resolution).unwrap_or(model.exposure_lperiod)
}

/// Shading/scan DUMMY register value.
pub fn dummy_pixel_for(model: &FilmModel, _resolution: i64) -> i64 {
    // No model carries `dummy_pixel_by_dpi`.
    model.dummy_pixel.unwrap_or(20)
}

/// Sensor custom register overlay for `resolution`.
pub fn sensor_regs_for(model: &FilmModel, _resolution: i64) -> RegMap {
    // No model carries `sensor_regs_by_dpi`.
    model.sensor_custom_regs.clone()
}

/// AFE frontend register map (base + optional dpi overlay).
pub fn frontend_regs_for(model: &FilmModel, resolution: i64, method: ScanMethod) -> RegMap {
    let mut base = model.frontend_regs.clone();
    if let Some(hit) = lookup_method_dpi(model.frontend_regs_by_dpi.as_ref(), method, resolution) {
        base.update(&hit);
    }
    base
}

/// ASIC `MAXWD` from SANE per-chip quirks.
///
/// - GL845/GL846: `(line_bytes * channels) >> 2` (4-word units; genesys bug preserved)
/// - GL843: `line_bytes >> 1` for OpticFilm CCD
/// - GL842 CCD: `line_bytes`
pub fn maxwd_register_value(model: &FilmModel, line_bytes: i64, channels: i64) -> i64 {
    match model.asic {
        "GL843" => line_bytes >> 1,
        "GL842" => line_bytes,
        _ => (line_bytes * channels) >> 2,
    }
}
