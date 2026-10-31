// SPDX-License-Identifier: GPL-3.0-or-later
//! Motor slope tables (ported from SANE genesys motor.cpp / low.cpp).

use crate::device::model::{DEFAULT_GL845_MOTOR, MotorProfile};

pub const MOTOR_INITIAL_W: i64 = DEFAULT_GL845_MOTOR.initial_w;
pub const MOTOR_MAX_W: i64 = DEFAULT_GL845_MOTOR.max_w;
pub const MOTOR_SLOPE_STEPS: i64 = DEFAULT_GL845_MOTOR.slope_steps;
pub const STEP_TYPE_QUARTER: i64 = DEFAULT_GL845_MOTOR.step_type;
pub const MOTOR_VREF: i64 = DEFAULT_GL845_MOTOR.vref;
pub const SLOPE_TABLE_MAX: i64 = DEFAULT_GL845_MOTOR.slope_table_max;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MotorSlope {
    pub initial_speed_w: i64,
    pub max_speed_w: i64,
    pub acceleration: f64,
}

impl MotorSlope {
    pub fn create_from_steps(initial_w: i64, max_w: i64, steps: i64) -> Self {
        let initial_v = 1.0 / initial_w as f64;
        let max_v = 1.0 / max_w as f64;
        let accel = (max_v * max_v - initial_v * initial_v) / (2.0 * steps as f64);
        Self {
            initial_speed_w: initial_w,
            max_speed_w: max_w,
            acceleration: accel,
        }
    }

    pub fn get_table_step_shifted(&self, step: i64, step_type: i64) -> i64 {
        if step < 2 {
            return self.initial_speed_w >> step_type;
        }
        let step = step - 1;
        let initial_v = 1.0 / self.initial_speed_w as f64;
        let speed_v = (initial_v * initial_v + 2.0 * self.acceleration * step as f64).sqrt();
        ((1.0 / speed_v) as i64) >> step_type
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MotorSlopeTable {
    pub table: Vec<i64>,
}

impl MotorSlopeTable {
    pub fn pixeltime_sum(&self) -> i64 {
        self.table.iter().sum()
    }
}

pub fn create_slope_table_for_speed(
    slope: &MotorSlope,
    target_speed_w: i64,
    step_type: i64,
    steps_alignment: i64,
    min_size: i64,
    max_size: i64,
) -> MotorSlopeTable {
    let target_shifted = target_speed_w >> step_type;
    let max_shifted = slope.max_speed_w >> step_type;
    let final_speed = target_shifted.max(max_shifted);
    let mut table: Vec<i64> = Vec::new();
    while (table.len() as i64) < max_size - 1 {
        let current = slope.get_table_step_shifted(table.len() as i64, step_type);
        if current <= final_speed {
            break;
        }
        table.push(current);
    }
    table.push(final_speed);
    while (table.len() as i64) < max_size - 1
        && ((table.len() as i64) % steps_alignment != 0 || (table.len() as i64) < min_size)
    {
        table.push(*table.last().unwrap());
    }
    MotorSlopeTable { table }
}

pub fn create_scan_slope_table(
    ydpi: i64,
    exposure: i64,
    base_ydpi: i64,
    step_multiplier: i64,
    profile: Option<&MotorProfile>,
) -> MotorSlopeTable {
    let mp = profile.copied().unwrap_or(DEFAULT_GL845_MOTOR);
    let slope = MotorSlope::create_from_steps(mp.initial_w, mp.max_w, mp.slope_steps);
    let target = crate::num::floor_div(exposure * ydpi, base_ydpi);
    create_slope_table_for_speed(
        &slope,
        target,
        mp.step_type,
        step_multiplier,
        2 * step_multiplier,
        mp.slope_table_max,
    )
}

pub fn create_fast_slope_table(
    step_multiplier: i64,
    profile: Option<&MotorProfile>,
) -> MotorSlopeTable {
    let mp = profile.copied().unwrap_or(DEFAULT_GL845_MOTOR);
    let slope = MotorSlope::create_from_steps(mp.initial_w, mp.max_w, mp.slope_steps);
    create_slope_table_for_speed(
        &slope,
        slope.max_speed_w,
        mp.step_type,
        step_multiplier,
        2 * step_multiplier,
        mp.slope_table_max,
    )
}

pub fn slope_table_to_bytes(table: &[i64]) -> Vec<u8> {
    table
        .iter()
        .flat_map(|&v| [(v & 0xFF) as u8, ((v >> 8) & 0xFF) as u8])
        .collect()
}

/// `sanei_genesys_calculate_zmod` (single-table path).
pub fn calculate_zmod(
    exposure_time: i64,
    slope_table: &[i64],
    acceleration_steps: i64,
    move_steps: i64,
    buffer_acceleration_steps: i64,
) -> (i64, i64) {
    let steps = (acceleration_steps as usize).min(slope_table.len());
    let total: i64 = slope_table[..steps].iter().sum();
    // Python `slope_table[steps - 1]` wraps to the last entry when steps is 0.
    let cruise = if steps == 0 {
        *slope_table.last().unwrap()
    } else {
        slope_table[steps - 1]
    };
    let z1 = crate::num::py_mod(total + buffer_acceleration_steps * cruise, exposure_time);
    let z2 = crate::num::py_mod(total + move_steps * cruise, exposure_time);
    (z1, z2)
}

/// Slope table slots → AHB base (genesys scanner_send_slope_table GL845).
pub const SLOPE_TABLE_AHB: [(i64, u32); 5] = [
    (0, 0x1000_0000), // SCAN
    (1, 0x1000_4000), // BACKTRACK
    (2, 0x1000_8000), // STOP
    (3, 0x1000_C000), // FAST
    (4, 0x1001_0000), // HOME
];

pub fn slope_table_ahb(slot: i64) -> Option<u32> {
    SLOPE_TABLE_AHB.iter().find(|e| e.0 == slot).map(|e| e.1)
}
