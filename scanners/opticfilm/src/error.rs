// SPDX-License-Identifier: GPL-3.0-or-later
//! Public error type for the Plustek USB driver (port of `exceptions.py`).

/// Every driver failure. The Python hierarchy is flat except that
/// `ScannerStuck` is a kind of `Scan`; [`Error::is_scan_error`] keeps that.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// No matching OpticFilm was found on the bus.
    #[error("{0}")]
    DeviceNotFound(String),
    /// A Plustek USB device was found but is not a supported model/chipset.
    #[error("{0}")]
    UnsupportedDevice(String),
    /// The OS driver does not allow raw USB access (e.g. vendor driver still bound).
    #[error("{0}")]
    DriverBinding(String),
    /// USB control/bulk transfer failure or link loss.
    #[error("{0}")]
    Usb(String),
    /// Scan sequencing or image assembly failed.
    #[error("{0}")]
    Scan(String),
    /// The scanner is in a state only a power-cycle clears; do not keep driving it.
    #[error("{0}")]
    ScannerStuck(String),
    /// Scan aborted because a cancel flag was set.
    #[error("{0}")]
    ScanCancelled(String),
    /// Dark/white/shading calibration failed or cache is invalid.
    #[error("{0}")]
    Calibration(String),
    /// Motor failed to reach home/park within the timeout.
    #[error("{0}")]
    MotorTimeout(String),
    /// ASIC initialization or control sequence failed.
    #[error("{0}")]
    Asic(String),
    /// Invalid argument (Python `ValueError`).
    #[error("{0}")]
    Value(String),
    /// Operation not available on this model (Python `NotImplementedError`).
    #[error("{0}")]
    NotImplemented(String),
}

impl Error {
    /// True for `Scan` and its subclass `ScannerStuck`.
    pub fn is_scan_error(&self) -> bool {
        matches!(self, Error::Scan(_) | Error::ScannerStuck(_))
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// `bail!(Kind, "fmt", args..)` returns `Err(Error::Kind(format!(..)))`.
#[macro_export]
macro_rules! bail {
    ($kind:ident, $($arg:tt)*) => {
        return Err($crate::error::Error::$kind(format!($($arg)*)))
    };
}

/// `err!(Kind, "fmt", args..)` builds an `Error::Kind(format!(..))`.
#[macro_export]
macro_rules! err {
    ($kind:ident, $($arg:tt)*) => {
        $crate::error::Error::$kind(format!($($arg)*))
    };
}
