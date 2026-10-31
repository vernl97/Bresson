// SPDX-License-Identifier: GPL-3.0-or-later
//! Lab-only multi-exposure bracket diagnostics (not part of `ScanImage`).

use ndarray::Array3;

use super::exposure_merge::MergeStats;

/// Bracket planes and IVW stats from an ME scan (`Scanner::last_me_debug`).
#[derive(Debug, Clone)]
pub struct MeScanDebug {
    pub rgb_short: Array3<u16>,
    pub rgb_long: Array3<u16>,
    pub exposure_short: i64,
    pub exposure_long: i64,
    pub fusion_stats: Option<MergeStats>,
    pub align_shift_long: Option<(f64, f64)>,
    pub align_shift_ir: Option<(f64, f64)>,
    /// Adaptive proposal before safety clamp (`None` for fixed / manual).
    pub exposure_proposed: Option<i64>,
    /// Why `exposure_long` was chosen.
    pub exposure_reason: Option<String>,
}

/// Multi-Pass stacking diagnostics for one exposure slot (short or long).
#[derive(Debug, Clone)]
pub struct SlotStackDebug {
    pub n_passes: i64,
    /// Repeat i → repeat 0 alignment shift, for repeats 2..N (empty when unaligned).
    pub align_shifts: Vec<(f64, f64)>,
    pub stack_stats: Option<MergeStats>,
}

/// Per-slot Multi-Pass stacking diagnostics (`Scanner::last_multi_pass_debug`).
#[derive(Debug, Clone)]
pub struct MultiPassDebug {
    pub short: SlotStackDebug,
    pub long: Option<SlotStackDebug>,
}
