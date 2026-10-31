// SPDX-License-Identifier: GPL-3.0-or-later
//! The OpticFilm model: tables (generated from Python) plus their behaviour.
//!
//! Python models are frozen dataclasses with per-class methods. Here every
//! model is one [`FilmModel`]: shared fields in [`ModelFields`], GL128-only
//! knobs in [`Gl128Data`], and [`ModelKind`] picking the leaf overrides.

use std::ops::{Deref, DerefMut};

pub use super::generated::{Gl128Data, ModelFields};
use super::generated::{
    gl128_common as g, model_7600i_v1 as v1, model_8100_v2, model_8200i_se, model_8300i_se,
    tables_8200i_se, tables_8300i_se,
};
use super::tables;
use crate::error::{Error, Result};
use crate::num::{ceil_i, nearest_key, py_int, py_round};

/// Light path of a scan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScanMethod {
    Transparency,
    Infrared,
}

impl ScanMethod {
    pub fn as_str(self) -> &'static str {
        match self {
            ScanMethod::Transparency => "transparency",
            ScanMethod::Infrared => "infrared",
        }
    }

    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "transparency" => Ok(ScanMethod::Transparency),
            "infrared" => Ok(ScanMethod::Infrared),
            _ => Err(Error::Value(format!("unknown scan method {s:?}"))),
        }
    }
}

/// SANE-style overlay key: `(method, dpi)`, `dpi` or `(method,)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MdKey(pub Option<ScanMethod>, pub Option<i64>);

/// Insertion-ordered map (Python `dict` semantics).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OrdMap<K, V>(pub Vec<(K, V)>);

impl<K: PartialEq, V> OrdMap<K, V> {
    pub fn from_vec(v: Vec<(K, V)>) -> Self {
        Self(v)
    }
    pub fn get(&self, k: &K) -> Option<&V> {
        self.0.iter().find(|e| &e.0 == k).map(|e| &e.1)
    }
    pub fn contains_key(&self, k: &K) -> bool {
        self.get(k).is_some()
    }
    pub fn insert(&mut self, k: K, v: V) {
        match self.0.iter_mut().find(|e| e.0 == k) {
            Some(e) => e.1 = v,
            None => self.0.push((k, v)),
        }
    }
    pub fn remove(&mut self, k: &K) -> Option<V> {
        let i = self.0.iter().position(|e| &e.0 == k)?;
        Some(self.0.remove(i).1)
    }
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.0.iter().map(|e| (&e.0, &e.1))
    }
    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.0.iter().map(|e| &e.0)
    }
    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.0.iter().map(|e| &e.1)
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<K: PartialEq + Copy, V: Copy> OrdMap<K, V> {
    pub fn get_or(&self, k: K, default: V) -> V {
        self.get(&k).copied().unwrap_or(default)
    }
}

impl OrdMap<i64, i64> {
    /// Python `min(table, key=lambda k: abs(k - x))` then lookup.
    pub fn nearest(&self, x: i64) -> Option<i64> {
        nearest_key(&self.0, x).map(|e| e.1)
    }
}

/// Register map: address → value, in Python dict insertion order.
pub type RegMap = OrdMap<u16, u16>;

impl RegMap {
    pub fn from_pairs(p: &[(u16, u16)]) -> Self {
        Self(p.to_vec())
    }

    /// Python `dict.update`.
    pub fn update(&mut self, other: &RegMap) {
        for (&k, &v) in other.iter() {
            self.insert(k, v);
        }
    }

    pub fn reg(&self, k: u16) -> u16 {
        self.get_or(k, 0)
    }

    /// Addresses in ascending order with byte values.
    pub fn sorted_bytes(&self) -> Vec<(u16, u8)> {
        let mut v: Vec<(u16, u8)> = self.iter().map(|(&k, &v)| (k, v as u8)).collect();
        v.sort_by_key(|e| e.0);
        v
    }
}

/// Per-model motor slope parameters (SANE `tables_motor.cpp`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MotorProfile {
    pub initial_w: i64,
    pub max_w: i64,
    pub slope_steps: i64,
    /// StepType::QUARTER = 2.
    pub step_type: i64,
    pub vref: i64,
    pub slope_table_max: i64,
}

/// Default GL845 OpticFilm profile (8200i / 7400 / …).
pub const DEFAULT_GL845_MOTOR: MotorProfile = MotorProfile {
    initial_w: 64102 * 4,
    max_w: 400 * 4,
    slope_steps: 100,
    step_type: 2,
    vref: 3,
    slope_table_max: 1024,
};

/// Which Python class a model is: picks the leaf method overrides.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum ModelKind {
    Sane8200i,
    Sane7400,
    Sane7300,
    Sane7500i,
    Sane7200i,
    Sane7200,
    Opticfilm7600iV1,
    Gl128_8200iSe,
    Gl128_8100V2,
    Gl128_8300iSe,
}

/// An OpticFilm model.
#[derive(Debug, Clone, PartialEq)]
pub struct FilmModel {
    pub kind: ModelKind,
    pub f: ModelFields,
    pub gl128: Option<Box<Gl128Data>>,
}

impl Deref for FilmModel {
    type Target = ModelFields;
    fn deref(&self) -> &ModelFields {
        &self.f
    }
}

impl DerefMut for FilmModel {
    fn deref_mut(&mut self) -> &mut ModelFields {
        &mut self.f
    }
}

pub const MM_PER_INCH: f64 = 25.4;
const NATIVE_DPI: i64 = 7200;

/// `LPERIOD` for `pixels` read per line in a scan at ASIC `scan_dpi`.
pub fn lperiod_for_line(pixels: f64, scan_dpi: i64) -> i64 {
    let divisor = if scan_dpi >= g::LPERIOD_HIGH_FROM_DPI {
        2.0
    } else {
        4.0
    };
    py_int(g::LPERIOD_BASE as f64 + pixels / divisor)
}

/// `LPERIOD` for an image or shading window `span_native` (`END - STR`) wide.
pub fn lperiod_for_window(asic_dpi: i64, span_native: i64) -> i64 {
    lperiod_for_line(
        (span_native * asic_dpi) as f64 / NATIVE_DPI as f64,
        asic_dpi,
    )
}

/// `(0x2B, 0xA5, 0xAB)` SilverFast writes for a shading strip `span_native` wide.
pub fn shading_clocks_for_window(asic_dpi: i64, span_native: i64, dvdset: bool) -> (i64, i64, i64) {
    let pixels = (span_native * asic_dpi) as f64 / NATIVE_DPI as f64;
    let line_bytes = 6.0 * pixels;
    if !dvdset {
        return (ceil_i(3.0 * line_bytes / 8192.0), 0x01, 0x30);
    }
    let high = asic_dpi >= 3600;
    let clk = if high { 0x01 } else { 0x02 };
    (
        ceil_i(if high { 2.0 } else { 1.0 } * line_bytes / 8192.0),
        clk,
        clk,
    )
}

/// USB serial-number string of current GL128 firmware.
pub const GL128_FIRMWARE_CURRENT: &str = g::GL128_FIRMWARE_CURRENT;
/// `0x36` from the plain (older firmware) register set.
pub const GL128_REG36_LEGACY_FIRMWARE: u16 = g::GL128_REG36_LEGACY_FIRMWARE as u16;

/// Return `model` with the vendor's per-firmware `0x36` for this unit.
pub fn gl128_firmware_variant(model: &FilmModel, firmware: Option<&str>) -> FilmModel {
    let mut m = model.clone();
    match firmware {
        Some(fw) if model.is_gl128() && fw.trim() != GL128_FIRMWARE_CURRENT => {
            m.init_regs.insert(0x36, GL128_REG36_LEGACY_FIRMWARE);
            m
        }
        _ => m,
    }
}

/// R/G/B line delays at `yres` lines/inch (7600i v1), scaled from twice it if needed.
pub fn shifts_for_7600i_v1(yres: i64) -> [f64; 3] {
    let r = if v1::CALIBRATED_SHIFTS.iter().any(|e| e.0 == yres) {
        yres
    } else {
        2 * yres
    };
    let s = v1::CALIBRATED_SHIFTS
        .iter()
        .find(|e| e.0 == r)
        .map(|e| e.1)
        .expect("calibrated 7600i v1 shift");
    [
        s[0] * yres as f64 / r as f64,
        s[1] * yres as f64 / r as f64,
        s[2] * yres as f64 / r as f64,
    ]
}

fn stagger_table(kind: ModelKind) -> &'static [(i64, &'static [i64])] {
    match kind {
        ModelKind::Gl128_8200iSe => model_8200i_se::STAGGER_BY_DPI_8200I_SE,
        ModelKind::Gl128_8100V2 => model_8100_v2::STAGGER_BY_DPI_8100_V2,
        _ => model_8300i_se::STAGGER_BY_DPI_8300I,
    }
}

impl FilmModel {
    pub fn is_gl128(&self) -> bool {
        self.gl128.is_some()
    }

    /// GL128-only fields. Panics on other models (Python would raise AttributeError).
    pub fn g(&self) -> &Gl128Data {
        self.gl128
            .as_deref()
            .unwrap_or_else(|| panic!("{} is not a GL128 model", self.name))
    }

    pub fn g_mut(&mut self) -> &mut Gl128Data {
        let name = self.name;
        self.gl128
            .as_deref_mut()
            .unwrap_or_else(|| panic!("{name} is not a GL128 model"))
    }

    pub fn max_area_mm(&self) -> (f64, f64) {
        (self.x_size_ta_mm, self.y_size_ta_mm)
    }

    pub fn mirror_x(&self) -> bool {
        self.f.mirror_x.unwrap_or(false)
    }

    pub fn supports_multi_exposure(&self) -> bool {
        self.f.supports_multi_exposure.unwrap_or(self.is_gl128())
    }

    /// Resolutions where `scan(multi_exposure=true)` is accepted.
    pub fn multi_exposure_resolutions_dpi(&self) -> Vec<i64> {
        if self.is_gl128() {
            return self
                .resolutions_dpi
                .iter()
                .copied()
                .filter(|&d| self.multi_pass_allowed_at(d))
                .collect();
        }
        match self.kind {
            ModelKind::Sane8200i if self.supports_multi_exposure() => self.resolutions_dpi.clone(),
            _ => Vec::new(),
        }
    }

    /// Resolutions where `scan(n_passes > 1)` is accepted (GL128).
    pub fn multi_pass_resolutions_dpi(&self) -> Vec<i64> {
        self.resolutions_dpi
            .iter()
            .copied()
            .filter(|&d| self.multi_pass_allowed_at(d))
            .collect()
    }

    /// Boot register map.
    pub fn boot_register_map(&self) -> RegMap {
        if self.is_gl128() {
            // The init blast plus the memory layout block.
            let mut regs = self.init_regs.clone();
            regs.update(&self.memory_layout_regs);
            return regs;
        }
        if self.kind == ModelKind::Opticfilm7600iV1 {
            let mut regs = RegMap::default();
            for group in super::replay_data::replay_data()["boot"]
                .as_array()
                .into_iter()
                .flatten()
            {
                let g = group.as_array().expect("boot group");
                if g.first().and_then(|v| v.as_str()) == Some("8c") {
                    continue;
                }
                for p in g.chunks_exact(2) {
                    regs.insert(p[0].as_u64().unwrap() as u16, p[1].as_u64().unwrap() as u16);
                }
            }
            return regs;
        }
        // Init regs with sensor custom overlay and dpihw=1200 applied.
        let mut regs = self.init_regs.clone();
        regs.update(&self.sensor_custom_regs);
        let r5 = regs.reg(0x05);
        regs.insert(0x05, (r5 & !0xC0) | 0x40);
        regs
    }

    // --- GL128 helpers (Gl128Common and leaf overrides) ---

    /// PPI used for ASIC geometry / tables (floors at `min_asic_dpi`).
    pub fn asic_dpi_for(&self, resolution: i64) -> i64 {
        match &self.gl128 {
            Some(g) => resolution.max(g.min_asic_dpi),
            None => resolution,
        }
    }

    /// Native lines the ASIC returns per *ASIC* output line.
    pub fn oversample_for(&self, resolution: i64) -> i64 {
        (self.optical_resolution / self.asic_dpi_for(resolution)).max(1)
    }

    /// Lines SilverFast scans past the frame or crop height on an image pass.
    pub fn image_line_margin(&self, resolution: i64) -> i64 {
        let dpi = self.asic_dpi_for(resolution);
        let shift = py_round((self.ld_shift_b * dpi) as f64 / self.motor_base_ydpi as f64);
        shift
            + if self.oversample_for(resolution) == 1 {
                5
            } else {
                1
            }
    }

    /// Whether multi-exposure and Multi-Pass may run at `resolution` (not at native 7200).
    pub fn multi_pass_allowed_at(&self, resolution: i64) -> bool {
        self.oversample_for(resolution) != 1
    }

    /// `LPERIOD` (`0x28`) at `resolution` for a window `span` native pixels wide.
    pub fn line_period_for(&self, resolution: i64, span: Option<i64>) -> i64 {
        lperiod_for_window(
            self.asic_dpi_for(resolution),
            span.unwrap_or(g::FULL_FRAME_SPAN_NATIVE),
        )
    }

    /// Motor ramp for feed2 / scan positioning (AHB slope windows).
    pub fn slope_table_fast(&self) -> &'static [i64] {
        match self.kind {
            ModelKind::Gl128_8300iSe => tables_8300i_se::SLOPE_TABLE_FAST,
            _ => tables_8200i_se::SLOPE_TABLE_FAST,
        }
    }

    /// Motor ramp for the reference feed / shading acquires.
    pub fn slope_table_slow(&self) -> &'static [i64] {
        match self.kind {
            ModelKind::Gl128_8300iSe => tables_8300i_se::SLOPE_TABLE_SLOW,
            _ => tables_8200i_se::SLOPE_TABLE_SLOW,
        }
    }

    /// `(0x2B, 0xA5, 0xAB)` for a shading strip `span` native pixels wide.
    pub fn shading_strip_clocks(
        &self,
        resolution: i64,
        dvdset: bool,
        span: Option<i64>,
    ) -> (i64, i64, i64) {
        if self.kind == ModelKind::Gl128_8300iSe && dvdset {
            let key = self.asic_dpi_for(resolution);
            let t = model_8300i_se::SHADING_WHITE_BY_ASIC_DPI;
            let row = t
                .iter()
                .find(|e| e.0 == key)
                .or_else(|| nearest_key(t, key))
                .expect("8300i white clocks")
                .1;
            return (row[0], row[1], row[2]);
        }
        shading_clocks_for_window(
            self.asic_dpi_for(resolution),
            span.unwrap_or(g::FULL_FRAME_SPAN_NATIVE),
            dvdset,
        )
    }

    /// `0x1D` low bits (LINESEL) for the dark and white shading strips.
    pub fn shading_linesel_for(&self, resolution: i64) -> i64 {
        self.g()
            .shading_linesel_by_dpi
            .get_or(self.asic_dpi_for(resolution), 0)
    }

    /// Scan-motor table period for the white shading strip.
    pub fn shading_white_motor_period(&self, resolution: i64) -> i64 {
        self.exposure_lperiod * (self.shading_linesel_for(resolution) + 1) / 4
    }

    /// Scan motor table period.
    pub fn channel_exposure_for(&self, resolution: i64, exposure: Option<i64>) -> i64 {
        let base = exposure.unwrap_or(self.exposure_lperiod);
        let oversample = self.oversample_for(resolution);
        if self.kind == ModelKind::Gl128_8300iSe {
            // exposure × ((0x1D & 0x1F) + 1) × asic_dpi / 14400.
            let reg_1d_lines = if oversample == 1 { 2 } else { 1 };
            return base * reg_1d_lines / (2 * oversample);
        }
        base / oversample
    }

    /// Words for the AHB scan motor windows.
    pub fn scan_motor_table(&self, period: i64) -> Result<Vec<i64>> {
        match self.kind {
            ModelKind::Gl128_8300iSe => tables::scan_motor_table_8300i(period),
            _ => tables::exposure_table(period),
        }
    }

    /// `0xA5`/`0xAB` for an image pass at `resolution`.
    pub fn pixel_clock_for_image(&self, resolution: i64, long_exposure: bool) -> i64 {
        let key = self.asic_dpi_for(resolution);
        let g = self.g();
        let m = if long_exposure {
            &g.pixel_clock_long_by_dpi
        } else {
            &g.pixel_clock_by_dpi
        };
        m.get_or(key, 0x02)
    }

    /// `REG_EXPOSURE` for short or long ME bracket.
    pub fn image_exposure(&self, long_exposure: bool) -> i64 {
        if long_exposure {
            self.exposure_long.unwrap_or(42000)
        } else {
            self.g().exposure_short
        }
    }

    /// Full-frame second-feed steps at `resolution`.
    pub fn feed_to_scan_steps_for_dpi(&self, resolution: i64, long_exposure: bool) -> i64 {
        if self.kind == ModelKind::Gl128_8300iSe {
            if long_exposure {
                return 13128;
            }
            let key = self.asic_dpi_for(resolution);
            let t = model_8300i_se::FEED2_BY_ASIC_DPI_8300I;
            return t
                .iter()
                .find(|e| e.0 == key)
                .or_else(|| nearest_key(t, key))
                .expect("8300i feed2")
                .1;
        }
        self.g().feed_to_scan_steps
    }

    /// Second-feed steps for a normalized TA crop `(x1,y1,x2,y2)`.
    pub fn feed_to_scan_steps_for_area(&self, area: Option<[f64; 4]>) -> i64 {
        let g = self.g();
        let Some(area) = area else {
            return g.feed_to_scan_steps;
        };
        let y1 = area[1].clamp(0.0, 1.0);
        let top = g.feed_to_scan_top_steps;
        let end = g.scan_window_end_steps;
        py_round(top as f64 + y1 * (end - top) as f64)
    }

    /// Image `LINCNT` captured at this second-feed distance (regression fixture).
    pub fn max_lincnt_for_feed2(&self, feed2: i64) -> Option<i64> {
        let table = &self.g().max_image_lincnt_by_feed2;
        if table.is_empty() {
            return None;
        }
        if let Some(&v) = table.get(&feed2) {
            return Some(v);
        }
        let e = nearest_key(&table.0, feed2)?;
        if (e.0 - feed2).abs() > 16 {
            None
        } else {
            Some(e.1)
        }
    }

    /// Steps left between `feed2` and the scan-window end.
    pub fn max_travel_steps_for_feed2(&self, feed2: i64) -> i64 {
        (self.g().scan_window_end_steps - feed2).max(0)
    }

    /// Largest image `LINCNT` that still stops at the scan-window end.
    pub fn max_lincnt_for(&self, feed2: i64, resolution: i64) -> i64 {
        self.lincnt_for_travel_steps(self.max_travel_steps_for_feed2(feed2), resolution)
    }

    /// Largest image `LINCNT` within the travel SilverFast itself uses.
    pub fn max_safe_lincnt_for(&self, feed2: i64, resolution: i64) -> i64 {
        let steps = (self.g().max_travel_end_steps - feed2).max(0);
        self.lincnt_for_travel_steps(steps, resolution)
    }

    fn lincnt_for_travel_steps(&self, steps: i64, resolution: i64) -> i64 {
        let g = self.g();
        let asic_dpi = self.asic_dpi_for(resolution);
        let raw = steps * asic_dpi * g.image_lincnt_per_line / g.feed_steps_per_inch;
        let per_line = g.image_lincnt_per_line.max(1);
        per_line.max((raw / per_line) * per_line)
    }

    /// Physical Y travel of an image pass with `lincnt` at `resolution`.
    pub fn travel_mm_for_lincnt(&self, lincnt: i64, resolution: i64) -> f64 {
        let asic_dpi = self.asic_dpi_for(resolution);
        lincnt as f64 * MM_PER_INCH / (self.g().image_lincnt_per_line * asic_dpi) as f64
    }

    /// Image `LINCNT` needed to cover `travel_mm` at `resolution`.
    pub fn lincnt_for_travel_mm(&self, travel_mm: f64, resolution: i64) -> i64 {
        let asic_dpi = self.asic_dpi_for(resolution);
        let lines = py_round(travel_mm * asic_dpi as f64 / MM_PER_INCH);
        lines.max(1) * self.g().image_lincnt_per_line
    }

    /// Session-13 image `LINCNT` for `resolution` (exact or nearest PPI).
    pub fn ladder_lincnt_for(&self, resolution: i64) -> i64 {
        let t = &self.g().ladder_lincnt_by_dpi;
        t.get(&resolution)
            .copied()
            .or_else(|| t.nearest(resolution))
            .expect("ladder table")
    }

    /// Rounded line-distance shift (GL128 leaves); `None` where the model has no override.
    pub fn scaled_ld_shift(&self, ld: i64, asic_dpi: i64) -> Option<i64> {
        self.is_gl128()
            .then(|| py_round((ld * asic_dpi) as f64 / self.motor_base_ydpi as f64))
    }

    /// Host column unstagger (GL128 leaves); `None` where the model has no override.
    pub fn stagger_y_for(&self, resolution: i64) -> Option<Vec<i64>> {
        if !self.is_gl128() {
            return None;
        }
        let key = self.asic_dpi_for(resolution);
        Some(
            stagger_table(self.kind)
                .iter()
                .find(|e| e.0 == key)
                .map(|e| e.1.to_vec())
                .unwrap_or_default(),
        )
    }

    /// 7600i v1: infrared resolutions (others: all resolutions when IR is supported).
    pub fn infrared_resolutions(&self) -> Vec<i64> {
        self.f
            .infrared_resolutions_dpi
            .clone()
            .unwrap_or_else(|| self.resolutions_dpi.clone())
    }
}
