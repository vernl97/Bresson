// SPDX-License-Identifier: GPL-3.0-or-later
//! Film scanning through SANE: flatbeds with a transparency unit.
//!
//! libsane is loaded at runtime ([`Sane::load`]); when it isn't installed SANE
//! is simply unavailable. [`Sane::simulated`] is a flatbed in Rust with the
//! options real backends have, for running without libsane or a scanner.
//!
//! SANE isn't thread-safe: keep a [`Sane`] and its [`Device`]s on one thread.

use std::sync::atomic::{AtomicBool, Ordering};

pub mod fake;
pub mod ffi;

pub use fake::{FakeConfig, FakeSane};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// libsane isn't installed or couldn't be loaded.
    #[error("{0}")]
    Unavailable(String),
    #[error("{call}: {status}")]
    Status { call: &'static str, status: Status },
    #[error("scan cancelled")]
    Cancelled,
    #[error("{0}")]
    Invalid(String),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// `SANE_Status`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Good,
    Unsupported,
    Cancelled,
    DeviceBusy,
    Inval,
    Eof,
    Jammed,
    NoDocs,
    CoverOpen,
    IoError,
    NoMem,
    AccessDenied,
    Other(i32),
}

impl Status {
    pub fn from_code(c: i32) -> Self {
        use Status::*;
        [
            Good,
            Unsupported,
            Cancelled,
            DeviceBusy,
            Inval,
            Eof,
            Jammed,
            NoDocs,
            CoverOpen,
            IoError,
            NoMem,
            AccessDenied,
        ]
        .get(c as usize)
        .copied()
        .unwrap_or(Other(c))
    }
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Status::Good => "success",
            Status::Unsupported => "operation not supported",
            Status::Cancelled => "operation was cancelled",
            Status::DeviceBusy => "device busy",
            Status::Inval => "invalid argument",
            Status::Eof => "no more data",
            Status::Jammed => "document feeder jammed",
            Status::NoDocs => "document feeder out of documents",
            Status::CoverOpen => "scanner cover is open",
            Status::IoError => "error during device I/O",
            Status::NoMem => "out of memory",
            Status::AccessDenied => "access to resource has been denied",
            Status::Other(c) => return write!(f, "status {c}"),
        };
        f.write_str(s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionType {
    Bool,
    Int,
    Fixed,
    String,
    Button,
    Group,
}

impl OptionType {
    pub fn from_code(c: i32) -> Self {
        match c {
            0 => Self::Bool,
            1 => Self::Int,
            2 => Self::Fixed,
            3 => Self::String,
            4 => Self::Button,
            _ => Self::Group,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    None,
    Pixel,
    Bit,
    Mm,
    Dpi,
    Percent,
    Microsecond,
}

impl Unit {
    pub fn from_code(c: i32) -> Self {
        [
            Self::None,
            Self::Pixel,
            Self::Bit,
            Self::Mm,
            Self::Dpi,
            Self::Percent,
            Self::Microsecond,
        ]
        .get(c as usize)
        .copied()
        .unwrap_or(Self::None)
    }
}

/// An option's allowed values; ranges and word lists are raw SANE words
/// (16.16 fixed point for `Fixed` options).
#[derive(Debug, Clone, PartialEq)]
pub enum Constraint {
    None,
    Range { min: i32, max: i32, quant: i32 },
    WordList(Vec<i32>),
    StringList(Vec<String>),
}

pub const CAP_SOFT_SELECT: u32 = 1;
pub const CAP_INACTIVE: u32 = 1 << 5;
pub const INFO_RELOAD_OPTIONS: u32 = 2;

#[derive(Debug, Clone, PartialEq)]
pub struct OptionInfo {
    pub index: i32,
    pub name: String,
    pub title: String,
    pub value_type: OptionType,
    pub unit: Unit,
    /// Value size in bytes.
    pub size: usize,
    pub words: usize,
    pub cap: u32,
    pub constraint: Constraint,
}

impl OptionInfo {
    pub fn active(&self) -> bool {
        self.cap & CAP_INACTIVE == 0
    }
    pub fn settable(&self) -> bool {
        self.active() && self.cap & CAP_SOFT_SELECT != 0
    }
    fn word_to_f64(&self, w: i32) -> f64 {
        if self.value_type == OptionType::Fixed {
            fixed_to_f64(w)
        } else {
            w as f64
        }
    }
    /// Range bounds in the option's unit.
    pub fn range(&self) -> Option<(f64, f64)> {
        match &self.constraint {
            Constraint::Range { min, max, .. } => {
                Some((self.word_to_f64(*min), self.word_to_f64(*max)))
            }
            Constraint::WordList(w) if !w.is_empty() => {
                let v: Vec<f64> = w.iter().map(|&x| self.word_to_f64(x)).collect();
                Some((
                    v.iter().cloned().fold(f64::MAX, f64::min),
                    v.iter().cloned().fold(f64::MIN, f64::max),
                ))
            }
            _ => None,
        }
    }
    pub fn strings(&self) -> &[String] {
        match &self.constraint {
            Constraint::StringList(v) => v,
            _ => &[],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Bool(bool),
    Int(i32),
    Fixed(f64),
    Str(String),
    Button,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Frame {
    Gray,
    Rgb,
    Red,
    Green,
    Blue,
}

impl Frame {
    pub fn from_code(c: i32) -> Self {
        [Self::Gray, Self::Rgb, Self::Red, Self::Green, Self::Blue]
            .get(c as usize)
            .copied()
            .unwrap_or(Self::Gray)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parameters {
    pub format: Frame,
    pub last_frame: bool,
    pub bytes_per_line: usize,
    pub pixels_per_line: usize,
    /// `None` when the scanner doesn't know in advance (hand scanners, feeders).
    pub lines: Option<usize>,
    pub depth: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceInfo {
    pub name: String,
    pub vendor: String,
    pub model: String,
    pub kind: String,
}

pub fn fixed_to_f64(w: i32) -> f64 {
    w as f64 / 65536.0
}

pub fn f64_to_fixed(v: f64) -> i32 {
    (v * 65536.0).round() as i32
}

/// The SANE entry points, as libsane ([`ffi::LibSane`]) or simulated ([`FakeSane`]).
pub trait Backend: Send {
    fn devices(&mut self) -> Result<Vec<DeviceInfo>>;
    fn open(&mut self, name: &str) -> Result<Box<dyn Handle>>;
}

/// An open device's entry points (`sane_control_option` & co).
pub trait Handle: Send {
    fn option_count(&mut self) -> Result<i32>;
    fn descriptor(&mut self, index: i32) -> Option<OptionInfo>;
    fn get(&mut self, opt: &OptionInfo) -> Result<Value>;
    /// Returns the `SANE_INFO_*` flags.
    fn set(&mut self, opt: &OptionInfo, value: &Value) -> Result<u32>;
    fn parameters(&mut self) -> Result<Parameters>;
    fn start(&mut self) -> Result<()>;
    /// Bytes read; end of frame is `Err(Status::Eof)`.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;
    fn cancel(&mut self);
}

/// libsane, or the simulated flatbed.
pub struct Sane {
    backend: Box<dyn Backend>,
}

impl Sane {
    /// Load libsane and `sane_init` it.
    pub fn load() -> Result<Self> {
        Ok(Self {
            backend: Box::new(ffi::LibSane::load()?),
        })
    }

    /// The simulated flatbed alone.
    pub fn simulated(config: FakeConfig) -> Self {
        Self::from_backend(Box::new(FakeSane::new(config)))
    }

    pub fn from_backend(backend: Box<dyn Backend>) -> Self {
        Self { backend }
    }

    pub fn devices(&mut self) -> Result<Vec<DeviceInfo>> {
        self.backend.devices()
    }

    pub fn open(&mut self, name: &str) -> Result<Device> {
        Device::new(self.backend.open(name)?)
    }
}

/// What a film scan asks for. `area` is `[x1, y1, x2, y2]` normalised to the
/// source's scan area.
#[derive(Debug, Clone, PartialEq)]
pub struct FilmScan {
    pub source: Option<String>,
    pub dpi: u32,
    pub area: [f64; 4],
}

/// A finished scan: row-major RGB, 16 bits per sample.
#[derive(Debug, Clone, PartialEq)]
pub struct SaneImage {
    pub width: usize,
    pub height: usize,
    pub rgb: Vec<u16>,
    pub dpi: u32,
}

/// Resolutions offered for a range constraint.
const COMMON_DPI: [u32; 15] = [
    75, 100, 150, 200, 300, 400, 600, 800, 1200, 1600, 2400, 3200, 4800, 6400, 9600,
];

/// Words in a source name that mean film (transparency unit, slide, negative).
const FILM_SOURCE_WORDS: [&str; 5] = ["transparen", "tpu", "film", "slide", "negative"];

/// An open SANE device and its option descriptors.
pub struct Device {
    handle: Box<dyn Handle>,
    options: Vec<OptionInfo>,
}

impl Device {
    fn new(handle: Box<dyn Handle>) -> Result<Self> {
        let mut d = Self {
            handle,
            options: Vec::new(),
        };
        d.reload()?;
        Ok(d)
    }

    fn reload(&mut self) -> Result<()> {
        let n = self.handle.option_count()?;
        self.options = (1..n).filter_map(|i| self.handle.descriptor(i)).collect();
        Ok(())
    }

    pub fn options(&self) -> &[OptionInfo] {
        &self.options
    }

    pub fn option(&self, name: &str) -> Option<&OptionInfo> {
        self.options.iter().find(|o| o.name == name)
    }

    pub fn get(&mut self, name: &str) -> Result<Value> {
        let opt = self
            .option(name)
            .cloned()
            .ok_or_else(|| Error::Invalid(format!("no option {name}")))?;
        self.handle.get(&opt)
    }

    /// Set an option by name, re-reading the descriptors when SANE asks.
    pub fn set(&mut self, name: &str, value: Value) -> Result<()> {
        let opt = self
            .option(name)
            .cloned()
            .ok_or_else(|| Error::Invalid(format!("no option {name}")))?;
        let value = match (&opt.value_type, value) {
            (OptionType::Fixed, Value::Int(v)) => Value::Fixed(v as f64),
            (OptionType::Int, Value::Fixed(v)) => Value::Int(v.round() as i32),
            (_, v) => v,
        };
        let info = self.handle.set(&opt, &value)?;
        if info & INFO_RELOAD_OPTIONS != 0 {
            self.reload()?;
        }
        Ok(())
    }

    /// Set `name` only when it exists and can be set now.
    fn set_if(&mut self, name: &str, value: Value) -> Result<bool> {
        if self.option(name).is_some_and(OptionInfo::settable) {
            self.set(name, value)?;
            return Ok(true);
        }
        Ok(false)
    }

    /// Scan sources (flatbed, transparency unit, …), as the device names them.
    pub fn sources(&self) -> Vec<String> {
        self.option("source")
            .map(|o| o.strings().to_vec())
            .unwrap_or_default()
    }

    /// The first source that reads film, if any.
    pub fn film_source(&self) -> Option<String> {
        self.sources().into_iter().find(|s| {
            let l = s.to_ascii_lowercase();
            FILM_SOURCE_WORDS.iter().any(|w| l.contains(w))
        })
    }

    /// Resolutions offered (for a range, the usual steps inside it).
    pub fn resolutions(&self) -> Vec<u32> {
        let Some(o) = self.option("resolution") else {
            return Vec::new();
        };
        let mut v: Vec<u32> = match &o.constraint {
            Constraint::WordList(w) => w.iter().map(|&x| o.word_to_f64(x).round() as u32).collect(),
            Constraint::Range { .. } => {
                let (lo, hi) = o.range().unwrap();
                COMMON_DPI
                    .iter()
                    .copied()
                    .filter(|&d| d as f64 >= lo && d as f64 <= hi)
                    .collect()
            }
            _ => Vec::new(),
        };
        v.sort_unstable();
        v.dedup();
        v
    }

    /// The scan area of the current source in mm: `[x1, y1, x2, y2]`.
    pub fn area_mm(&self) -> Option<[f64; 4]> {
        let r = |n: &str| self.option(n).and_then(OptionInfo::range);
        let ((x0, x1), (y0, y1)) = (r("tl-x")?, r("tl-y")?);
        let (bx0, bx1) = r("br-x").unwrap_or((x0, x1));
        let (by0, by1) = r("br-y").unwrap_or((y0, y1));
        Some([x0.min(bx0), y0.min(by0), x1.max(bx1), y1.max(by1)])
    }

    /// Select the source and set everything that would alter film data to its
    /// neutral value: colour, the deepest bit depth, positive film type (no
    /// inversion), gamma off.
    pub fn prepare_film(&mut self, source: Option<&str>) -> Result<()> {
        let source = source.map(str::to_owned).or_else(|| self.film_source());
        if let Some(s) = source {
            self.set_if("source", Value::Str(s))?;
        }
        if let Some(mode) = self.option("mode").and_then(|o| {
            o.strings()
                .iter()
                .find(|m| m.eq_ignore_ascii_case("color"))
                .cloned()
        }) {
            self.set_if("mode", Value::Str(mode))?;
        }
        if let Some(depth) = self
            .option("depth")
            .and_then(|o| o.range())
            .map(|(_, hi)| hi.min(16.0))
        {
            self.set_if("depth", Value::Int(depth as i32))?;
        }
        if let Some(pos) = self.option("film-type").and_then(|o| {
            o.strings()
                .iter()
                .find(|s| s.to_ascii_lowercase().contains("positive"))
                .cloned()
        }) {
            self.set_if("film-type", Value::Str(pos))?;
        }
        self.set_if("negative", Value::Bool(false))?;
        self.set_if("custom-gamma", Value::Bool(false))?;
        self.set_if("gamma-correction", Value::Bool(false))?;
        self.set_if("preview", Value::Bool(false))?;
        Ok(())
    }

    /// Set resolution and area for a film scan (after [`Device::prepare_film`]).
    pub fn set_window(&mut self, dpi: u32, area: [f64; 4]) -> Result<()> {
        self.set_if("resolution", Value::Int(dpi as i32))?;
        let Some([x0, y0, x1, y1]) = self.area_mm() else {
            return Ok(());
        };
        let n = |v: f64| v.clamp(0.0, 1.0);
        let (ax0, ax1) = (n(area[0].min(area[2])), n(area[0].max(area[2])));
        let (ay0, ay1) = (n(area[1].min(area[3])), n(area[1].max(area[3])));
        let mm = |a: f64, lo: f64, hi: f64| lo + a * (hi - lo);
        // Bottom-right out to the limit first, so a new top-left is never past it.
        self.set_if("br-x", Value::Fixed(x1))?;
        self.set_if("br-y", Value::Fixed(y1))?;
        self.set_if("tl-x", Value::Fixed(mm(ax0, x0, x1)))?;
        self.set_if("tl-y", Value::Fixed(mm(ay0, y0, y1)))?;
        self.set_if("br-x", Value::Fixed(mm(ax1, x0, x1)))?;
        self.set_if("br-y", Value::Fixed(mm(ay1, y0, y1)))?;
        Ok(())
    }

    /// Prepare, set the window and scan.
    pub fn scan_film(
        &mut self,
        job: &FilmScan,
        cancel: &AtomicBool,
        progress: &mut dyn FnMut(f64),
    ) -> Result<SaneImage> {
        self.prepare_film(job.source.as_deref())?;
        self.set_window(job.dpi, job.area)?;
        self.scan(cancel, progress)
    }

    /// Read one image with the current settings (all frames of a three-pass scan).
    pub fn scan(
        &mut self,
        cancel: &AtomicBool,
        progress: &mut dyn FnMut(f64),
    ) -> Result<SaneImage> {
        let dpi = match self.get("resolution") {
            Ok(Value::Int(v)) => v.max(0) as u32,
            Ok(Value::Fixed(v)) => v.round().max(0.0) as u32,
            _ => 0,
        };
        let r = self.read_frames(cancel, progress);
        // SANE ends a scan with `sane_cancel`, after the last frame or on error.
        self.handle.cancel();
        let (width, height, rgb) = r?;
        progress(1.0);
        Ok(SaneImage {
            width,
            height,
            rgb,
            dpi,
        })
    }

    fn read_frames(
        &mut self,
        cancel: &AtomicBool,
        progress: &mut dyn FnMut(f64),
    ) -> Result<(usize, usize, Vec<u16>)> {
        let mut planes: [Option<Vec<u16>>; 3] = [None, None, None];
        let mut rgb: Option<Vec<u16>> = None;
        let mut size: Option<(usize, usize)> = None;
        for frame in 0.. {
            if cancel.load(Ordering::SeqCst) {
                return Err(Error::Cancelled);
            }
            self.handle.start()?;
            let p = self.handle.parameters()?;
            let total = if matches!(p.format, Frame::Red | Frame::Green | Frame::Blue) {
                3.0
            } else {
                1.0
            };
            let (samples, w, h) =
                self.read_frame(&p, cancel, &mut |f| progress(((frame as f64) + f) / total))?;
            if size.is_some_and(|s| s != (w, h)) {
                let (sw, sh) = size.unwrap();
                return Err(Error::Invalid(format!(
                    "frame size changed from {sw}×{sh} to {w}×{h}"
                )));
            }
            size = Some((w, h));
            match p.format {
                Frame::Rgb => rgb = Some(samples),
                Frame::Gray => rgb = Some(samples.iter().flat_map(|&v| [v, v, v]).collect()),
                Frame::Red => planes[0] = Some(samples),
                Frame::Green => planes[1] = Some(samples),
                Frame::Blue => planes[2] = Some(samples),
            }
            if p.last_frame {
                break;
            }
        }
        let (width, height) = size.unwrap_or((0, 0));
        let rgb = match (rgb, planes) {
            (Some(rgb), _) => rgb,
            (None, [Some(r), Some(g), Some(b)]) => (0..width * height)
                .flat_map(|i| [r[i], g[i], b[i]])
                .collect(),
            _ => {
                return Err(Error::Invalid(
                    "scan ended without all three colour frames".into(),
                ));
            }
        };
        Ok((width, height, rgb))
    }

    /// One started frame as 16-bit samples (one per pixel, or three for RGB).
    fn read_frame(
        &mut self,
        p: &Parameters,
        cancel: &AtomicBool,
        progress: &mut dyn FnMut(f64),
    ) -> Result<(Vec<u16>, usize, usize)> {
        let channels = if p.format == Frame::Rgb { 3 } else { 1 };
        let bytes = match p.depth {
            8 => 1,
            16 => 2,
            d => {
                return Err(Error::Invalid(format!(
                    "{d}-bit scans are not supported (choose colour, 8 or 16 bit)"
                )));
            }
        };
        let row = p.pixels_per_line * channels * bytes;
        if p.bytes_per_line < row || row == 0 {
            return Err(Error::Invalid(format!(
                "line of {} bytes for {} pixels",
                p.bytes_per_line, p.pixels_per_line
            )));
        }
        let expected = p.lines.map(|l| l * p.bytes_per_line);
        let mut data = Vec::with_capacity(expected.unwrap_or(1 << 20));
        let mut chunk = vec![0u8; 256 * 1024];
        loop {
            if cancel.load(Ordering::SeqCst) {
                return Err(Error::Cancelled);
            }
            match self.handle.read(&mut chunk) {
                Ok(n) => data.extend_from_slice(&chunk[..n]),
                Err(Error::Status {
                    status: Status::Eof,
                    ..
                }) => break,
                Err(Error::Status {
                    status: Status::Cancelled,
                    ..
                }) => return Err(Error::Cancelled),
                Err(e) => return Err(e),
            }
            if let Some(e) = expected {
                progress((data.len() as f64 / e as f64).min(1.0));
            }
        }
        let lines = data.len() / p.bytes_per_line;
        let mut out = Vec::with_capacity(lines * p.pixels_per_line * channels);
        for l in 0..lines {
            let line = &data[l * p.bytes_per_line..l * p.bytes_per_line + row];
            if bytes == 2 {
                out.extend(
                    line.chunks_exact(2)
                        .map(|b| u16::from_ne_bytes([b[0], b[1]])),
                );
            } else {
                out.extend(line.iter().map(|&b| ((b as u16) << 8) | b as u16));
            }
        }
        Ok((out, p.pixels_per_line, lines))
    }
}
