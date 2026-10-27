//! Negative inversion engine — a standalone port of the Negative Conversion
//! Plugin (Lightroom) processing pipeline.

pub mod color;
pub mod curve;
pub mod decode;
pub mod export;
pub mod flat;
pub mod frame;
pub mod look;
pub mod pipeline;
pub mod presets;
pub mod settings;
pub mod sidecar;
#[cfg_attr(not(test), allow(dead_code))]
pub mod testchart;
