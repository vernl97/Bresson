// SPDX-License-Identifier: GPL-3.0-or-later
//! GL128 scan motor (exposure) tables.

use super::generated::{tables_8200i_se as se, tables_8300i_se as t83};
use crate::error::{Error, Result};

/// Build an 8200i SE / 8100 V2 scan motor table for `line_period`: a flat body
/// with entry 0 raised by `EXPOSURE_FIRST_DELTA`.
///
/// `line_period` must fit the table's 16-bit words: letting it wrap uploads a
/// body inconsistent with `REG_EXPOSURE`, which reads on hardware as a motor jam.
pub fn exposure_table(line_period: i64) -> Result<Vec<i64>> {
    let body = line_period;
    if !(0..=0xFFFF).contains(&body) {
        return Err(Error::Value(format!(
            "channel exposure {body} does not fit the AHB scan motor table's 16-bit words (0-65535) \
             — check the caller's oversample-aware clamp"
        )));
    }
    let first = (body + se::EXPOSURE_FIRST_DELTA) & 0xFFFF;
    let mut v = vec![body; se::EXPOSURE_TABLE_LENGTH as usize];
    v[0] = first;
    Ok(v)
}

/// 8300i SE: SilverFast accelerates along `SCAN_RAMP` and holds `period` once
/// the ramp reaches it.
pub fn scan_motor_table_8300i(period: i64) -> Result<Vec<i64>> {
    if !(0..=0xFFFF).contains(&period) {
        return Err(Error::Value(format!("scan motor period {period} does not fit the AHB table's 16-bit words (0-65535)")));
    }
    Ok(t83::SCAN_RAMP.iter().map(|&v| v.max(period)).collect())
}
