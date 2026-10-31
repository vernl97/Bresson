// SPDX-License-Identifier: GPL-3.0-or-later
//! Device models and selection.

pub mod generated;
pub mod model;
pub mod replay_data;
pub mod select;
pub mod sensor_lookup;
pub mod tables;

pub use model::{DEFAULT_GL845_MOTOR, FilmModel, MdKey, ModelKind, MotorProfile, OrdMap, RegMap, ScanMethod};
pub use select::{known_models, model_by_name, model_for_device, model_for_pid, model_is_scan_ready};
