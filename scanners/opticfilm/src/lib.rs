// SPDX-License-Identifier: GPL-3.0-or-later
//! Driver for Plustek OpticFilm USB film scanners (Genesys ASIC).
//!
//! A port of the Python `pyopticfilm` driver. See `README.md`.

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
