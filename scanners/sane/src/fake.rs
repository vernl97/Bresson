// SPDX-License-Identifier: GPL-3.0-or-later
//! A simulated flatbed with a transparency unit, behaving like a SANE backend.
//!
//! Options: `mode` (Lineart/Gray/Color), `depth` (8/16), `resolution` (word
//! list), `source` (Flatbed / Transparency Unit: changing it reloads the
//! options, as the area changes), `film-type` (Positive/Negative Film, active
//! on the transparency unit; Negative inverts the data), `custom-gamma`
//! (on applies gamma 2.2), `preview`, and the `tl-x`…`br-y` area in mm.
//!
//! The transparency unit holds a 35 mm strip running down the bed: an orange
//! mask with frames of a modelled negative between frame gaps, opaque holder
//! around it. The flatbed reads a grey reflective card.

use std::sync::{Arc, Mutex};

use crate::{
    Backend, CAP_INACTIVE, CAP_SOFT_SELECT, Constraint, DeviceInfo, Error, Frame, Handle, INFO_RELOAD_OPTIONS, OptionInfo,
    OptionType, Parameters, Result, Status, Unit, Value, f64_to_fixed,
};

pub const DEVICE_NAME: &str = "nlp:simulated-flatbed";

const SOURCES: [&str; 2] = ["Flatbed", "Transparency Unit"];
const MODES: [&str; 3] = ["Lineart", "Gray", "Color"];
const FILM_TYPES: [&str; 2] = ["Positive Film", "Negative Film"];
const RESOLUTIONS: [i32; 9] = [75, 100, 150, 300, 600, 1200, 2400, 3200, 6400];
/// Scan area per source in mm (width, height).
const FLATBED_MM: (f64, f64) = (215.9, 297.2);
const TPU_MM: (f64, f64) = (63.5, 228.6);
/// The strip: 35 mm film centred across the unit, 24 × 36 mm frames, 2 mm gaps.
const FILM_X: (f64, f64) = (14.25, 49.25);
const FRAME_X: (f64, f64) = (19.75, 43.75);
const FRAME_PITCH: f64 = 38.0;
const FRAME_H: f64 = 36.0;
const STRIP_START: f64 = 10.0;
/// Orange-mask transmittance of the film base (R, G, B).
const BASE: [f64; 3] = [0.78, 0.47, 0.26];

/// How the simulated device delivers data.
#[derive(Debug, Clone, Default)]
pub struct FakeConfig {
    /// Three single-channel frames (R, G, B) instead of interleaved RGB.
    pub three_pass: bool,
    /// Report `lines = -1` (unknown) in the parameters.
    pub unknown_lines: bool,
    /// Bytes of padding at the end of each line.
    pub line_padding: usize,
    /// Largest `sane_read` (0 = whole buffer).
    pub max_read: usize,
}

/// What the simulated device saw, for tests.
#[derive(Debug, Default, Clone)]
pub struct FakeLog {
    pub starts: usize,
    pub cancels: usize,
    pub reloads: usize,
    pub sets: Vec<(String, Value)>,
}

/// The simulated flatbed as a [`Backend`].
pub struct FakeSane {
    config: FakeConfig,
    log: Arc<Mutex<FakeLog>>,
}

impl FakeSane {
    pub fn new(config: FakeConfig) -> Self {
        Self { config, log: Arc::default() }
    }

    /// Shared view of what the device saw.
    pub fn log(&self) -> Arc<Mutex<FakeLog>> {
        self.log.clone()
    }
}

impl Backend for FakeSane {
    fn devices(&mut self) -> Result<Vec<DeviceInfo>> {
        Ok(vec![DeviceInfo {
            name: DEVICE_NAME.into(),
            vendor: "NLP Studio".into(),
            model: "Simulated flatbed".into(),
            kind: "flatbed scanner".into(),
        }])
    }

    fn open(&mut self, name: &str) -> Result<Box<dyn Handle>> {
        if name != DEVICE_NAME {
            return Err(Error::Status { call: "sane_open", status: Status::Inval });
        }
        Ok(Box::new(FakeHandle::new(self.config.clone(), self.log.clone())))
    }
}

struct FakeHandle {
    config: FakeConfig,
    log: Arc<Mutex<FakeLog>>,
    mode: String,
    depth: i32,
    resolution: i32,
    source: String,
    film_type: String,
    custom_gamma: bool,
    preview: bool,
    area: [f64; 4],
    /// Next frame to start (three-pass), and the data being read.
    frame: usize,
    data: Vec<u8>,
    pos: usize,
    params: Option<Parameters>,
}

impl FakeHandle {
    fn new(config: FakeConfig, log: Arc<Mutex<FakeLog>>) -> Self {
        Self {
            config,
            log,
            mode: "Lineart".into(),
            depth: 8,
            resolution: 300,
            source: SOURCES[0].into(),
            film_type: FILM_TYPES[1].into(),
            custom_gamma: true,
            preview: false,
            area: [0.0, 0.0, FLATBED_MM.0, FLATBED_MM.1],
            frame: 0,
            data: Vec::new(),
            pos: 0,
            params: None,
        }
    }

    fn tpu(&self) -> bool {
        self.source == SOURCES[1]
    }

    fn bed(&self) -> (f64, f64) {
        if self.tpu() { TPU_MM } else { FLATBED_MM }
    }

    fn descriptors(&self) -> Vec<OptionInfo> {
        let strings = |v: &[&str]| Constraint::StringList(v.iter().map(|s| s.to_string()).collect());
        let opt = |index: i32, name: &str, value_type: OptionType, unit: Unit, constraint: Constraint, active: bool| OptionInfo {
            index,
            name: name.into(),
            title: name.into(),
            value_type,
            unit,
            size: if value_type == OptionType::String { 32 } else { 4 },
            words: 1,
            cap: CAP_SOFT_SELECT | if active { 0 } else { CAP_INACTIVE },
            constraint,
        };
        let (w, h) = self.bed();
        let range = |max: f64| Constraint::Range { min: 0, max: f64_to_fixed(max), quant: 0 };
        vec![
            opt(1, "mode", OptionType::String, Unit::None, strings(&MODES), true),
            opt(2, "depth", OptionType::Int, Unit::Bit, Constraint::WordList(vec![8, 16]), true),
            opt(3, "resolution", OptionType::Int, Unit::Dpi, Constraint::WordList(RESOLUTIONS.to_vec()), true),
            opt(4, "source", OptionType::String, Unit::None, strings(&SOURCES), true),
            opt(5, "film-type", OptionType::String, Unit::None, strings(&FILM_TYPES), self.tpu()),
            opt(6, "custom-gamma", OptionType::Bool, Unit::None, Constraint::None, true),
            opt(7, "preview", OptionType::Bool, Unit::None, Constraint::None, true),
            opt(8, "tl-x", OptionType::Fixed, Unit::Mm, range(w), true),
            opt(9, "tl-y", OptionType::Fixed, Unit::Mm, range(h), true),
            opt(10, "br-x", OptionType::Fixed, Unit::Mm, range(w), true),
            opt(11, "br-y", OptionType::Fixed, Unit::Mm, range(h), true),
        ]
    }

    fn size(&self) -> (usize, usize) {
        let px = |mm: f64| ((mm / 25.4) * self.resolution as f64).round().max(1.0) as usize;
        (px(self.area[2] - self.area[0]), px(self.area[3] - self.area[1]))
    }

    /// Transmittance (or reflectance) at a bed position, per channel, 0..1.
    fn sample(&self, x: f64, y: f64) -> [f64; 3] {
        if !self.tpu() {
            let g = 0.55 + 0.25 * (x / FLATBED_MM.0);
            return [g, g, g];
        }
        if x < FILM_X.0 || x > FILM_X.1 || y < STRIP_START {
            return [0.004; 3]; // holder
        }
        let k = ((y - STRIP_START) / FRAME_PITCH).floor();
        let fy = y - STRIP_START - k * FRAME_PITCH;
        if x < FRAME_X.0 || x > FRAME_X.1 || fy > FRAME_H {
            return BASE; // unexposed base: rebate and frame gaps
        }
        // A scene: horizontal light falloff, a bright disc, per-frame exposure.
        let (u, v) = ((x - FRAME_X.0) / (FRAME_X.1 - FRAME_X.0), fy / FRAME_H);
        let disc = if (u - 0.5).powi(2) + (v - 0.45).powi(2) < 0.06 { 0.6 } else { 0.0 };
        let d = 0.25 + 0.9 * u + disc + 0.1 * k;
        let dye = [d, d * 0.92, d * 0.85];
        [0, 1, 2].map(|c| BASE[c] * 10f64.powf(-dye[c]))
    }

    fn value(&self, t: f64) -> u16 {
        let mut v = t.clamp(0.0, 1.0);
        if self.tpu() && self.film_type == FILM_TYPES[1] {
            v = 1.0 - v; // the backend's own negative inversion
        }
        if self.custom_gamma {
            v = v.powf(1.0 / 2.2);
        }
        (v * 65535.0).round() as u16
    }

    fn render(&mut self) -> Parameters {
        let (w, h) = self.size();
        let color = self.mode == "Color";
        let three = color && self.config.three_pass;
        let format = match (color, three, self.frame) {
            (false, _, _) => Frame::Gray,
            (true, false, _) => Frame::Rgb,
            (true, true, 0) => Frame::Red,
            (true, true, 1) => Frame::Green,
            (true, true, _) => Frame::Blue,
        };
        let channels: Vec<usize> = match format {
            Frame::Rgb => vec![0, 1, 2],
            Frame::Red => vec![0],
            Frame::Green => vec![1],
            Frame::Blue => vec![2],
            Frame::Gray => vec![1],
        };
        let bytes = if self.depth == 16 { 2 } else { 1 };
        let bpl = w * channels.len() * bytes + self.config.line_padding;
        let mut data = Vec::with_capacity(bpl * h);
        let mm = 25.4 / self.resolution as f64;
        for row in 0..h {
            let y = self.area[1] + (row as f64 + 0.5) * mm;
            for col in 0..w {
                let s = self.sample(self.area[0] + (col as f64 + 0.5) * mm, y);
                for &c in &channels {
                    let v = self.value(s[c]);
                    if bytes == 2 {
                        data.extend_from_slice(&v.to_ne_bytes());
                    } else {
                        data.push((v >> 8) as u8);
                    }
                }
            }
            data.extend(std::iter::repeat_n(0u8, self.config.line_padding));
        }
        self.data = data;
        self.pos = 0;
        Parameters {
            format,
            last_frame: !three || self.frame == 2,
            bytes_per_line: bpl,
            pixels_per_line: w,
            lines: (!self.config.unknown_lines).then_some(h),
            depth: self.depth as u32,
        }
    }
}

impl Handle for FakeHandle {
    fn option_count(&mut self) -> Result<i32> {
        Ok(12)
    }

    fn descriptor(&mut self, index: i32) -> Option<OptionInfo> {
        self.descriptors().into_iter().find(|o| o.index == index)
    }

    fn get(&mut self, opt: &OptionInfo) -> Result<Value> {
        Ok(match opt.name.as_str() {
            "mode" => Value::Str(self.mode.clone()),
            "depth" => Value::Int(self.depth),
            "resolution" => Value::Int(self.resolution),
            "source" => Value::Str(self.source.clone()),
            "film-type" => Value::Str(self.film_type.clone()),
            "custom-gamma" => Value::Bool(self.custom_gamma),
            "preview" => Value::Bool(self.preview),
            "tl-x" => Value::Fixed(self.area[0]),
            "tl-y" => Value::Fixed(self.area[1]),
            "br-x" => Value::Fixed(self.area[2]),
            "br-y" => Value::Fixed(self.area[3]),
            _ => return Err(Error::Status { call: "sane_control_option", status: Status::Inval }),
        })
    }

    fn set(&mut self, opt: &OptionInfo, value: &Value) -> Result<u32> {
        let inval = || Error::Status { call: "sane_control_option", status: Status::Inval };
        let desc = self.descriptor(opt.index).ok_or_else(inval)?;
        if !desc.settable() {
            return Err(inval());
        }
        self.log.lock().unwrap().sets.push((opt.name.clone(), value.clone()));
        let in_list = |s: &str| desc.strings().iter().any(|x| x == s);
        let (w, h) = self.bed();
        let mut info = 0;
        match (opt.name.as_str(), value) {
            ("mode", Value::Str(s)) if in_list(s) => self.mode = s.clone(),
            ("depth", Value::Int(d)) if [8, 16].contains(d) => self.depth = *d,
            ("resolution", Value::Int(d)) => {
                // Snap to the nearest offered resolution (SANE_INFO_INEXACT).
                self.resolution = *RESOLUTIONS.iter().min_by_key(|r| (*r - d).abs()).unwrap();
            }
            ("source", Value::Str(s)) if in_list(s) => {
                if *s != self.source {
                    self.source = s.clone();
                    let (w, h) = self.bed();
                    self.area = [0.0, 0.0, w, h];
                    info |= INFO_RELOAD_OPTIONS;
                    self.log.lock().unwrap().reloads += 1;
                }
            }
            ("film-type", Value::Str(s)) if in_list(s) => self.film_type = s.clone(),
            ("custom-gamma", Value::Bool(b)) => self.custom_gamma = *b,
            ("preview", Value::Bool(b)) => self.preview = *b,
            ("tl-x", Value::Fixed(v)) if *v <= self.area[2] => self.area[0] = v.clamp(0.0, w),
            ("tl-y", Value::Fixed(v)) if *v <= self.area[3] => self.area[1] = v.clamp(0.0, h),
            ("br-x", Value::Fixed(v)) if *v >= self.area[0] => self.area[2] = v.clamp(0.0, w),
            ("br-y", Value::Fixed(v)) if *v >= self.area[1] => self.area[3] = v.clamp(0.0, h),
            _ => return Err(inval()),
        }
        Ok(info)
    }

    fn parameters(&mut self) -> Result<Parameters> {
        self.params.clone().ok_or(Error::Status { call: "sane_get_parameters", status: Status::Inval })
    }

    fn start(&mut self) -> Result<()> {
        if self.mode == "Lineart" {
            // 1-bit data is out of scope; real backends would deliver it.
            self.depth = 1;
        }
        self.log.lock().unwrap().starts += 1;
        let p = self.render();
        if self.depth == 1 {
            self.params = Some(Parameters { depth: 1, ..p });
            return Ok(());
        }
        self.params = Some(p);
        Ok(())
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if self.pos >= self.data.len() {
            if self.params.as_ref().is_some_and(|p| !p.last_frame) {
                self.frame += 1;
            } else {
                self.frame = 0;
            }
            return Err(Error::Status { call: "sane_read", status: Status::Eof });
        }
        let max = if self.config.max_read == 0 { buf.len() } else { self.config.max_read.min(buf.len()) };
        let n = max.min(self.data.len() - self.pos);
        buf[..n].copy_from_slice(&self.data[self.pos..self.pos + n]);
        self.pos += n;
        Ok(n)
    }

    fn cancel(&mut self) {
        self.log.lock().unwrap().cancels += 1;
        self.frame = 0;
        self.data.clear();
        self.pos = 0;
    }
}
