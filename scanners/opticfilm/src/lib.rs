// SPDX-License-Identifier: GPL-3.0-or-later
//! Driver for Plustek OpticFilm USB film scanners (Genesys ASIC).
//!
//! A port of the Python `pyopticfilm` driver. See `README.md`.

// The port follows the Python line for line (parity is the test oracle), so
// its signatures, callback types, index loops and NaN-aware negated
// comparisons stay as the original has them.
#![allow(
    clippy::type_complexity,
    clippy::too_many_arguments,
    clippy::needless_range_loop,
    clippy::neg_cmp_op_on_partial_ord,
    clippy::nonminimal_bool
)]

pub mod asic;
pub mod clock;
pub mod device;
pub mod difflib;
pub mod error;
pub mod image;
pub mod num;
pub mod pass_align;
pub mod registration;
pub mod scan;
pub mod scanner;
pub mod usb;

pub use device::{FilmModel, ScanMethod, known_models, model_by_name};
pub use error::{Error, Result};
pub use image::ScanImage;
pub use scan::request::{ScanControl, ScanMode, ScanRequest};
pub use scanner::Scanner;
