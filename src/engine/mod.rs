//! Negative inversion engine — a standalone port of the Negative Conversion
//! Plugin (Lightroom) processing pipeline.

// Per-channel and per-pixel index loops over parallel arrays read like the
// formulas in docs/conversion-engine.md; iterator chains would hide them.
#![allow(clippy::needless_range_loop)]

pub mod burn;
pub mod color;
pub mod contact;
pub mod curve;
pub mod decode;
pub mod demosaic;
pub mod denoise;
pub mod export;
pub mod flat;
pub mod frame;
pub mod gainmap;
pub mod heal;
pub mod icc;
pub mod look;
pub mod pipeline;
pub mod presets;
#[cfg(test)]
mod scan_check;
pub mod scan_dng;
pub mod settings;
pub mod sidecar;
pub mod skin;
#[cfg_attr(not(test), allow(dead_code))]
pub mod testchart;
