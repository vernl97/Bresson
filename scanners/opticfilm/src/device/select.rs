// SPDX-License-Identifier: GPL-3.0-or-later
//! Select model from USB product id + bcdDevice.

use std::sync::OnceLock;

use super::generated as g;
use super::model::FilmModel;
use crate::error::{Error, Result};
use crate::usb::device::*;

/// 7600i v2 shares 8200i tables (SANE alias), with its hardware-validated knobs.
pub fn model_7600i_v2() -> FilmModel {
    // Generated from the Python object (MODEL_7600I_V2 is in KNOWN_MODELS).
    g::model_7600i_v2()
}

/// Every known model, in the Python `KNOWN_MODELS` order.
pub fn known_models() -> &'static [FilmModel] {
    static M: OnceLock<Vec<FilmModel>> = OnceLock::new();
    M.get_or_init(g::known_models)
}

/// The model named `name` (e.g. `plustek-opticfilm-8200i-se`).
pub fn model_by_name(name: &str) -> Option<FilmModel> {
    known_models().iter().find(|m| m.name == name).cloned()
}

/// Resolve model from product id alone (bcdDevice defaults to 0).
pub fn model_for_pid(product_id: u16) -> Result<FilmModel> {
    model_for_device(product_id, 0)
}

/// Resolve OpticFilm model from USB product id and bcdDevice.
///
/// Matches SANE `UsbDeviceEntry` rules: exact bcd when required for
/// `0x0c3a` / `0x0c3b` disambiguation.
pub fn model_for_device(product_id: u16, bcd_device: u16) -> Result<FilmModel> {
    Ok(match product_id {
        PID_OPTICFILM_8200I => g::model_8200i(),
        PID_OPTICFILM_8200I_SE => g::model_8200i_se(),
        PID_OPTICFILM_7200 => g::model_7200(),
        PID_OPTICFILM_7200I => g::model_7200i(),
        PID_OPTICFILM_7200_V2 => g::model_7200_v2(),
        PID_OPTICFILM_7300 => g::model_7300(),
        PID_OPTICFILM_7500I => g::model_7500i(),
        PID_OPTICFILM_8100 => g::model_8100(),
        // bcd 0x0400 → 7400-v1 (7300 tables); bcd 0x0605 → 7400-v2
        PID_OPTICFILM_7400 if bcd_device == 0x0400 => g::model_7400_v1(),
        PID_OPTICFILM_7400 => g::model_7400_v2(),
        // bcd 0x0400 → 7600i-v1 (captures); bcd 0x0605 → 7600i-v2 (8200i)
        PID_OPTICFILM_7600I if bcd_device == 0x0400 => g::model_7600i_v1(),
        PID_OPTICFILM_7600I => g::model_7600i_v2(),
        PID_OPTICFILM_8100_V2 => g::model_8100_v2(),
        PID_OPTICFILM_8300I_SE => g::model_8300i_se(),
        _ => return Err(Error::UnsupportedDevice(format!("No model mapping for product_id=0x{product_id:04x}"))),
    })
}

/// True for models validated for scan.
pub fn model_is_scan_ready(model: &FilmModel) -> bool {
    model.scan_ready
}
