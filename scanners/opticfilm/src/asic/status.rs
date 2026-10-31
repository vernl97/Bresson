// SPDX-License-Identifier: GPL-3.0-or-later
//! ASIC status decoded from register 0x41.

use super::registers::gl845 as r;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScannerStatus {
    pub raw: u8,
    pub is_replugged: bool,
    pub is_buffer_empty: bool,
    pub is_feeding_finished: bool,
    pub is_scanning_finished: bool,
    pub is_at_home: bool,
    pub is_lamp_on: bool,
    pub is_front_end_busy: bool,
    pub is_motor_enabled: bool,
}

impl ScannerStatus {
    pub fn from_reg41(value: u8) -> Self {
        let v = value;
        Self {
            raw: v,
            is_replugged: v & r::STATUS_PWRBIT == 0,
            is_buffer_empty: v & r::STATUS_BUFEMPTY != 0,
            is_feeding_finished: v & r::STATUS_FEEDFSH != 0,
            is_scanning_finished: v & r::STATUS_SCANFSH != 0,
            is_at_home: v & r::STATUS_HOMESNR != 0,
            is_lamp_on: v & r::STATUS_LAMPSTS != 0,
            is_front_end_busy: v & r::STATUS_FEBUSY != 0,
            is_motor_enabled: v & r::STATUS_MOTORENB != 0,
        }
    }
}
