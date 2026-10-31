// SPDX-License-Identifier: GPL-3.0-or-later
//! Validation for explicit (manual) `REG_EXPOSURE` overrides.
//!
//! The only hard limit is the 24-bit register; invalid values are rejected
//! rather than clamped or wrapped.

use crate::error::{Error, Result};

/// Largest value representable in the 24-bit REG_EXPOSURE register.
pub const MAX_EXPOSURE_REGISTER: i64 = 0xFF_FFFF;
/// Largest Multi-Pass repeat count.
pub const MAX_N_PASSES: i64 = 9;

pub fn validate_n_passes(value: i64) -> Result<i64> {
    if !(1..=MAX_N_PASSES).contains(&value) {
        return Err(Error::Value(format!(
            "n_passes must be between 1 and {MAX_N_PASSES}, got {value}"
        )));
    }
    Ok(value)
}

/// `None` (no override) is always valid; otherwise 1..=0xFFFFFF.
pub fn validate_manual_exposure(value: Option<i64>, label: &str) -> Result<Option<i64>> {
    let Some(v) = value else { return Ok(None) };
    if v <= 0 {
        return Err(Error::Value(format!(
            "{label} must be a positive integer, got {v}"
        )));
    }
    if v > MAX_EXPOSURE_REGISTER {
        return Err(Error::Value(format!(
            "{label}={v} exceeds the 24-bit REG_EXPOSURE maximum (0x{MAX_EXPOSURE_REGISTER:06X} = {MAX_EXPOSURE_REGISTER})"
        )));
    }
    Ok(Some(v))
}
