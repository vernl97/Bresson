// SPDX-License-Identifier: GPL-3.0-or-later
//! Scan arguments and cooperative cancel / progress.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use super::geometry::{Area, ScanGeometry};
use super::me_exposure::MeLongMode;
use crate::device::ScanMethod;
use crate::error::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScanMode {
    #[default]
    Color,
    Infrared,
    Gray,
}

impl ScanMode {
    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "color" => Ok(Self::Color),
            "infrared" => Ok(Self::Infrared),
            "gray" => Ok(Self::Gray),
            _ => Err(Error::Value(format!("Unsupported mode {s:?} (color|infrared)"))),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Color => "color",
            Self::Infrared => "infrared",
            Self::Gray => "gray",
        }
    }

    pub fn method(self) -> ScanMethod {
        if self == Self::Infrared { ScanMethod::Infrared } else { ScanMethod::Transparency }
    }
}

/// Everything `Scanner::scan` accepts (Python keyword defaults).
#[derive(Debug, Clone, PartialEq)]
pub struct ScanRequest {
    pub resolution: i64,
    pub mode: ScanMode,
    pub area: Option<Area>,
    pub geometry: Option<ScanGeometry>,
    pub apply_calib: bool,
    pub multi_exposure: bool,
    pub infrared: bool,
    pub align_passes: bool,
    pub single_pass_exposure: Option<i64>,
    pub me_short_exposure: Option<i64>,
    pub me_long_exposure: Option<i64>,
    pub me_long_mode: MeLongMode,
    pub n_passes: i64,
}

impl Default for ScanRequest {
    fn default() -> Self {
        Self {
            resolution: 1800,
            mode: ScanMode::Color,
            area: None,
            geometry: None,
            apply_calib: true,
            multi_exposure: false,
            infrared: false,
            align_passes: true,
            single_pass_exposure: None,
            me_short_exposure: None,
            me_long_exposure: None,
            me_long_mode: MeLongMode::Adaptive,
            n_passes: 1,
        }
    }
}

impl ScanRequest {
    pub fn resolution(&self) -> i64 {
        self.geometry.as_ref().map_or(self.resolution, |g| g.resolution)
    }
}

/// Cooperative cancel flag and progress callback (0..1).
#[derive(Default)]
pub struct ScanControl {
    pub cancel: Option<Arc<AtomicBool>>,
    pub progress: Option<Box<dyn FnMut(f64) + Send>>,
    /// Called with `"scanning"` when the scan starts.
    pub on_status: Option<Box<dyn FnMut(&str) + Send>>,
}

impl ScanControl {
    pub fn is_cancelled(&self) -> bool {
        self.cancel.as_ref().is_some_and(|c| c.load(Ordering::SeqCst))
    }

    pub fn check(&self, what: &str) -> Result<()> {
        if self.is_cancelled() {
            return Err(Error::ScanCancelled(what.into()));
        }
        Ok(())
    }

    pub fn report(&mut self, p: f64) {
        if let Some(f) = self.progress.as_mut() {
            f(p);
        }
    }
}

/// Progress mapped into a sub-range: `report(p)` → `outer((idx + p) / n)`.
pub struct SubProgress<'a> {
    pub ctl: &'a mut ScanControl,
    pub idx: f64,
    pub n: f64,
}

/// What a capture loop needs: cancel checks and progress.
pub trait Progress {
    fn cancelled(&self) -> bool;
    fn report(&mut self, p: f64);
    fn check(&self, what: &str) -> Result<()> {
        if self.cancelled() {
            return Err(Error::ScanCancelled(what.into()));
        }
        Ok(())
    }
}

impl Progress for ScanControl {
    fn cancelled(&self) -> bool {
        self.is_cancelled()
    }
    fn report(&mut self, p: f64) {
        ScanControl::report(self, p);
    }
}

impl Progress for SubProgress<'_> {
    fn cancelled(&self) -> bool {
        self.ctl.is_cancelled()
    }
    fn report(&mut self, p: f64) {
        self.ctl.report(((self.idx + p) / self.n).min(1.0));
    }
}

/// No cancel, no progress (calibration captures).
pub struct NoProgress;

impl Progress for NoProgress {
    fn cancelled(&self) -> bool {
        false
    }
    fn report(&mut self, _p: f64) {}
}
