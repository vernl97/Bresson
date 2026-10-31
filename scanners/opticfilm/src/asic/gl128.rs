// SPDX-License-Identifier: GPL-3.0-or-later
//! GL128 driver for OpticFilm 8200i SE, 8100 (V2), and 8300i SE.
//!
//! SANE genesys has no GL128 command set, so none of this is ported from
//! SANE. Every register write replays what the Windows driver does in the
//! USB captures. Differences from GL845 that matter here:
//!
//! * status is at `0x101` (high-address read) rather than `0x41`;
//! * the analog frontend is written through `0x51`/`0x5D`/`0x5E`;
//! * the white lamp is `0x03` bit 4 and infrared is `0x37` bit 2;
//!   `0x03` bit 5 (`XPASEL`) stays set for every transparency operation;
//! * positioning is two capture-constant feeds from home, then the image pass
//!   runs with `FEEDL=1` and `AGOHOME` so the carriage parks afterwards.

use std::collections::HashSet;

use log::{debug, error, info, warn};

use super::buttons::{BUTTON_IDS, Button};
use super::registers::gl128 as r;
use super::status::ScannerStatus;
use crate::clock::{monotonic, sleep};
use crate::device::model::{FilmModel, RegMap, ScanMethod};
use crate::error::{Error, Result};
use crate::scan::calib_gl128::*;
use crate::usb::protocol::GenesysUsbProtocol;

pub const BRINGUP_HINT: &str = "GL128 (OpticFilm 8200i SE / 8100 V2 / 8300i SE) support is derived from \
USB captures of the Windows driver rather than from SANE — see docs/gl128-model-comparison.md.";

/// Raised when code deliberately disarms the motor (e.g. stationary shading).
pub const MOTOR_GATED_HINT: &str = "GL128 motor moves are temporarily disabled on this handle \
(disarmed for safety). Re-enable with Scanner.arm_bringup_motor() after stationary calib, or open a \
fresh Scanner (motor is on by default for scan-ready GL128).";

pub const MM_PER_INCH: f64 = 25.4;
pub const HOME_POLL_S: f64 = 0.05;

/// Stationary calib ready: buffer has data **and** carriage is at home.
const STATIONARY_DATA_READY: [u8; 4] = [0xBD, 0xA9, 0xAD, 0x9C];
const STATIONARY_IDLE_WAIT_S: f64 = 2.0;
const STATIONARY_START_RETRIES: usize = 1;
/// Vendor probe `wIndex` polled during fast feeds until it returns this.
const FEED_PROBE_INDEX: u8 = 0x21;
const FEED_PROBE_DONE: i32 = 0x04;
const MOTOR_IDLE_TIMEOUT_S: f64 = 2.0;
const MOTOR_IDLE_POLL_S: f64 = 0.015;
const REFERENCE_FEED_TIMEOUT_S: f64 = 2.5;
const FEED_PROBE_POLL_S: f64 = 0.002;

/// Register block written immediately before every captured fast feed.
const FEED_SETUP_REGS: [(u16, u16); 28] = [
    (0x01, 0x22),
    (0x04, 0x42),
    (0x05, 0x48),
    (0xA6, 0x00),
    (0xA7, 0x00),
    (0xA8, 0x00),
    (0xA9, 0x00),
    (0x7D, 0x00),
    (0x7E, 0x36),
    (0x7F, 0xB0), // exposure = 14000
    (0x80, 0x00),
    (0x81, 0x40),
    (0x82, 0x00),
    (0x83, 0x00),
    (0x84, 0xF2),
    (0x85, 0x00),
    (0x86, 0x29),
    (0x87, 0x72),
    (0x2C, 0x00),
    (0x2D, 0xC8), // DPISET = 200
    (0x1D, 0x80),
    (0x1C, 0x20),
    (0xA4, 0x00),
    (0xA5, 0x02),
    (0xAA, 0x00),
    (0xAB, 0x02),
    (0xAE, 0x00),
    (0xAF, 0x7F),
];
/// `0xAF` on the second (scan-start) positioning feed.
const FINAL_FEED_REG_AF: u16 = 0xFF;
/// `0xAF` on the reference feed.
const REFERENCE_FEED_REG_AF: u16 = 0x7F;
/// Registers a feed changes itself (motor / START / counters / FEEDL / LINCNT).
const FEED_OWN_REGS: [u16; 9] = [0x02, 0x0D, 0x0F, 0x25, 0x26, 0x27, 0x3D, 0x3E, 0x3F];
/// Feed-time values SilverFast writes that differ from the image pass's.
const FEED_FIXED_REGS: [u16; 2] = [0x01, 0x05];

/// Registers written before a positioning feed.
pub fn feed_setup_regs(scan_regs: Option<&RegMap>, final_positioning: bool) -> RegMap {
    let fixed = RegMap::from_pairs(&FEED_SETUP_REGS);
    let mut setup = match scan_regs {
        None => fixed.clone(),
        Some(sr) => {
            let mut s = RegMap::from_vec(
                sr.iter()
                    .filter(|(a, _)| !FEED_OWN_REGS.contains(a))
                    .map(|(&a, &v)| (a, v))
                    .collect(),
            );
            for a in FEED_FIXED_REGS {
                s.insert(a, fixed.reg(a));
            }
            s
        }
    };
    setup.insert(
        0xAF,
        if final_positioning {
            FINAL_FEED_REG_AF
        } else {
            REFERENCE_FEED_REG_AF
        },
    );
    setup
}

const BOOT_BLOB_ADDR_A: u32 = 0x000F_FF00;
const BOOT_BLOB_ADDR_B: u32 = 0x000F_FF01;
fn boot_blob_b() -> [u8; 34] {
    let mut b = [0u8; 34];
    b[32] = 0x33;
    b
}

const REG_CLK_DRAM: u16 = 0x0B;
const DRAM_ENABLE: u16 = 0x08;
const REG_0X13: u16 = 0x13;
const DRAM_INIT_0X13: u16 = 0x0F;
/// `0xA6`-`0xA9` on every calibration strip.
const CALIB_STRIP_A6_A9: [(u16, u16); 4] = [(0xA6, 0x00), (0xA7, 0x01), (0xA8, 0x00), (0xA9, 0x01)];
/// Image cancel / end-scan lamp writes on `0x03`.
const CANCEL_LAMP_PRE: [u8; 2] = [0x30, 0x20];
const CANCEL_LAMP_POST: [u8; 6] = [0x10, 0x00, 0x20, 0x30, 0x20, 0x30];
/// Clear `SCAN`, keep SHDAREA|DVDSET.
const CANCEL_REG01: u8 = 0x22;

/// Pack 16-bit table entries little-endian, as the AHB windows expect.
pub fn u16_table_bytes(words: &[i64]) -> Vec<u8> {
    words
        .iter()
        .flat_map(|&w| [(w & 0xFF) as u8, ((w >> 8) & 0xFF) as u8])
        .collect()
}

fn encode_offsets(v: [i64; 3]) -> [i64; 3] {
    v.map(encode_afe_offset)
}

/// GL128 chip operations.
pub struct Gl128 {
    pub protocol: GenesysUsbProtocol,
    pub model: FilmModel,
    pub initialized: bool,
    pub reg_cache: RegMap,
    pub scan_method: ScanMethod,
    /// On for scan-ready GL128; may be temporarily disarmed for stationary shading.
    pub motor_moves_enabled: bool,
    /// Whether the last AGOHOME park completed.
    pub park_ok: bool,
    /// Set after a successful `run_asic_shading` upload this session (never for IR).
    pub asic_shading_ready: bool,
    /// Equalized per-column IR white for host flatten.
    pub last_ir_host_white: Option<Vec<i64>>,
    pub ir_host_flatten_ready: bool,
    /// Last successful `search_afe` result; image configure re-applies it.
    pub last_afe: Option<AfeFrontend>,
    /// ASIC dpi whose AFE strip timing `last_afe` was measured under.
    pub last_afe_dpi: Option<i64>,
    /// An image pass ran since the last boot.
    pub image_pass_since_init: bool,
    strip_lperiod: i64,
    /// USB line layout learned from AFE strip means (true = planar).
    pub usb_planar_rgb: bool,
    pub last_color_shading_reject_reason: Option<String>,
    pub last_host_calib_dark: Option<Vec<Rgb>>,
    pub last_host_calib_white: Option<Vec<Rgb>>,
    pub last_color_shading_host_ok: bool,
    /// Lab acoustic A/B: use the slow ramp for non-shading table uploads.
    pub image_slope_slow: bool,
}

impl Gl128 {
    pub fn new(protocol: GenesysUsbProtocol, model: FilmModel) -> Self {
        let armed = model.scan_ready;
        Self {
            protocol,
            model,
            initialized: false,
            reg_cache: RegMap::default(),
            scan_method: ScanMethod::Transparency,
            motor_moves_enabled: armed,
            park_ok: true,
            asic_shading_ready: false,
            last_ir_host_white: None,
            ir_host_flatten_ready: false,
            last_afe: None,
            last_afe_dpi: None,
            image_pass_since_init: false,
            strip_lperiod: 0,
            usb_planar_rgb: false,
            last_color_shading_reject_reason: None,
            last_host_calib_dark: None,
            last_host_calib_white: None,
            last_color_shading_host_ok: false,
            image_slope_slow: false,
        }
    }

    fn require_motor_enabled(&self) -> Result<()> {
        if !self.motor_moves_enabled {
            return Err(Error::Asic(MOTOR_GATED_HINT.into()));
        }
        Ok(())
    }

    fn cached(&self, addr: u16, default: u8) -> u8 {
        self.reg_cache.get(&addr).map_or(default, |&v| v as u8)
    }

    // --- registers ---

    pub fn write(&mut self, address: u16, value: u8) -> Result<()> {
        self.protocol.write_register(address, value)?;
        self.reg_cache.insert(address, value as u16);
        Ok(())
    }

    pub fn write_many(&mut self, regs: &RegMap) -> Result<()> {
        let pairs = regs.sorted_bytes();
        self.protocol.write_registers_batched(&pairs, 32)?;
        for (a, v) in pairs {
            self.reg_cache.insert(a, v as u16);
        }
        Ok(())
    }

    /// Read-modify-write one register and return the value written.
    pub fn update_bits(&mut self, address: u16, set_bits: u8, clear_bits: u8) -> Result<u8> {
        let current = self.protocol.read_register(address)?;
        let value = (current & !clear_bits) | set_bits;
        self.write(address, value)?;
        Ok(value)
    }

    // --- status ---

    /// Read the status register at `0x101`.
    pub fn read_status(&mut self) -> Result<ScannerStatus> {
        let raw = self
            .protocol
            .read_register(r::REG_STATUS)
            .map_err(|e| Error::Asic(format!("GL128 status read failed: {e}")))?;
        Ok(ScannerStatus::from_reg41(raw))
    }

    /// Read status twice and keep the second value (the first can be stale).
    pub fn read_status_reliable(&mut self) -> Result<ScannerStatus> {
        self.read_status()?;
        self.read_status()
    }

    pub fn is_at_home(&mut self) -> Result<bool> {
        Ok(self.read_status_reliable()?.is_at_home)
    }

    /// True when the power bit is clear, meaning the ASIC lost its state.
    pub fn is_cold_boot(&mut self) -> Result<bool> {
        Ok(self.read_status()?.is_replugged)
    }

    // --- boot ---

    /// Load the analog frontend defaults through the GL124 write path.
    pub fn set_frontend_init(&mut self) -> Result<()> {
        let mut fe: Vec<(u16, u16)> = self
            .model
            .frontend_regs
            .iter()
            .map(|(&a, &v)| (a, v))
            .collect();
        fe.sort_by_key(|e| e.0);
        for (i, v) in &fe {
            self.protocol.write_fe_register_gl124(*i as u8, *v)?;
        }
        debug!("GL128 frontend initialised ({} regs)", fe.len());
        Ok(())
    }

    /// Write per-channel FE gains and/or offsets.
    pub fn set_frontend_channels(
        &mut self,
        offsets: Option<[i64; 3]>,
        gains: Option<[i64; 3]>,
    ) -> Result<()> {
        let fe = AfeFrontend {
            offsets: offsets.unwrap_or([0; 3]),
            gains: gains.unwrap_or([0; 3]),
        };
        for (index, value) in fe.as_fe_writes() {
            if gains.is_none() && FE_GAIN_INDICES.contains(&index) {
                continue;
            }
            if offsets.is_none() && FE_OFFSET_INDICES.contains(&index) {
                continue;
            }
            self.protocol.write_fe_register_gl124(index, value)?;
        }
        Ok(())
    }

    /// Program a full offset+gain FE state.
    pub fn apply_frontend(&mut self, fe: AfeFrontend) -> Result<()> {
        for (index, value) in fe.as_fe_writes() {
            self.protocol.write_fe_register_gl124(index, value)?;
        }
        Ok(())
    }

    /// `sensor_custom_regs` + memory layout for calib acquires.
    fn apply_stationary_scan_regs(&mut self, include_memory_layout: bool) -> Result<()> {
        if include_memory_layout {
            let ml = self.model.memory_layout_regs.clone();
            self.write_many(&ml)?;
        }
        let mut custom = self.model.sensor_custom_regs.clone();
        // Never clobber lamp / IR from a stale table copy.
        custom.remove(&r::REG_0X03);
        custom.remove(&r::REG_IR);
        self.write_many(&custom)
    }

    fn wait_stationary_data_ready(&mut self, timeout_s: f64, where_: &str) -> Result<()> {
        self.wait_shading_data_ready(timeout_s, where_, false)
    }

    /// Poll `0x101` until shading/AFE strip data is ready.
    fn wait_shading_data_ready(
        &mut self,
        timeout_s: f64,
        where_: &str,
        motorized: bool,
    ) -> Result<()> {
        self.read_status()?;
        let deadline = monotonic() + timeout_s;
        let mut last: i32 = -1;
        while monotonic() < deadline {
            let status = self.read_status()?;
            last = status.raw as i32;
            if motorized {
                if !status.is_buffer_empty {
                    return Ok(());
                }
            } else if STATIONARY_DATA_READY.contains(&status.raw)
                || (!status.is_buffer_empty && status.is_at_home)
            {
                return Ok(());
            }
            sleep(0.01);
        }
        let kind = if motorized { "motorized" } else { "stationary" };
        Err(Error::Scan(format!(
            "{where_}: no {kind} data ready within {timeout_s:.0}s (last status=0x{:02x})",
            last & 0xFF
        )))
    }

    /// Abort a leftover image SCAN so stationary geometry writes stick.
    #[doc(hidden)]
    pub fn reset_stationary_scan_engine(&mut self, where_: &str) -> Result<()> {
        let motor_mask = r::MTRPWR | r::AGOHOME | r::FASTFED;
        self.write(r::REG_START, 0x00)?;
        self.write(r::REG_0X01, CANCEL_REG01)?;
        self.write(r::REG_0X02, 0x00)?;
        self.write(r::REG_CLRCNT, r::CLRCNT_ALL)?;
        let deadline = monotonic() + STATIONARY_IDLE_WAIT_S;
        let (mut last01, mut last02, mut last_st) = (-1i32, -1i32, -1i32);
        let mut status: Option<ScannerStatus> = None;
        while monotonic() < deadline {
            let v01 = self.protocol.read_register(r::REG_0X01)?;
            let v02 = self.protocol.read_register(r::REG_0X02)?;
            let st = self.read_status()?;
            last01 = v01 as i32;
            last02 = v02 as i32;
            last_st = st.raw as i32;
            status = Some(st);
            let scan_clear = v01 & r::SCAN == 0;
            let motor_clear = v02 & motor_mask == 0;
            let idle = st.is_at_home && !st.is_motor_enabled;
            if scan_clear && motor_clear && idle {
                self.reg_cache.insert(r::REG_0X01, v01 as u16);
                self.reg_cache.insert(r::REG_0X02, v02 as u16);
                if v01 & r::DVDSET != 0 {
                    self.write(r::REG_0X01, (v01 | r::SHDAREA) & !r::SCAN & !r::DVDSET)?;
                }
                return Ok(());
            }
            if v01 & r::SCAN != 0 {
                self.write(r::REG_0X01, (v01 | r::SHDAREA) & !r::SCAN & !r::DVDSET)?;
            }
            if v02 & motor_mask != 0 {
                self.write(r::REG_0X02, 0x00)?;
            }
            sleep(0.02);
        }
        // Motor power off and at home, yet MOTORENB stays set: only a power-cycle clears it.
        if let Some(st) = status
            && (last02 as u8) & motor_mask == 0
            && st.is_at_home
            && st.is_motor_enabled
        {
            return Err(Error::ScannerStuck(format!(
                "GL128 {where_}: the scanner reports its motor running (status 0x{:02x}) with motor power off, \
                 and it did not clear in {STATIONARY_IDLE_WAIT_S:.0} s. Switch the scanner off and on, then retry the scan.",
                last_st & 0xFF
            )));
        }
        warn!(
            "GL128 {where_}: scan engine not idle (0x01={last01:#04x} 0x02={last02:#04x} status=0x{:02x})",
            last_st & 0xFF
        );
        Ok(())
    }

    /// Wait until HOME and motor off (two consecutive idle samples).
    fn wait_idle_at_home_for_stationary(&mut self, timeout_s: f64, where_: &str) -> Result<()> {
        let deadline = monotonic() + timeout_s;
        let mut last = -1i32;
        let mut idle_hits = 0;
        while monotonic() < deadline {
            let st = self.read_status()?;
            last = st.raw as i32;
            if st.is_at_home && !st.is_motor_enabled {
                idle_hits += 1;
                if idle_hits >= 2 {
                    return Ok(());
                }
            } else {
                idle_hits = 0;
            }
            sleep(0.01);
        }
        warn!(
            "GL128 {where_}: motor not idle at home within {timeout_s:.1}s (last status=0x{:02x})",
            last & 0xFF
        );
        Ok(())
    }

    /// Capture start recipe: motor off, `0x0d` → absolute SCAN → `0x0f`.
    fn start_stationary_strip(&mut self) -> Result<()> {
        let motor_mask = r::MTRPWR | r::AGOHOME | r::FASTFED;
        self.write(r::REG_0X02, 0x00)?;
        let reg02 = self.protocol.read_register(r::REG_0X02)?;
        if reg02 & motor_mask != 0 {
            warn!("GL128 AFE START 0x02={reg02:#04x} still armed — rewriting 0");
            self.write(r::REG_0X02, 0x00)?;
        }
        self.write(r::REG_CLRCNT, r::CLRCNT_ALL)?;
        let reg01 = (self.cached(r::REG_0X01, 0x22) | r::SHDAREA | r::SCAN) & !r::DVDSET;
        self.write(r::REG_0X01, reg01)?;
        self.write(r::REG_START, r::START_GO)
    }

    /// Stationary AFE strip geometry and timing (SF window and registers, motor off).
    fn setup_afe_strip_regs(&mut self, wide: bool, timing: Option<AfeStripTiming>) -> Result<()> {
        self.apply_stationary_scan_regs(true)?;
        let dpi_calib = self.model.optical_resolution / 6;
        let timing = timing.unwrap_or_else(|| afe_strip_timing(dpi_calib, wide, !wide));
        // AFE uses the 1200-dpi dark-strip pixel clocks (session 03).
        let (_dummy, clk_a, clk_b) =
            self.model
                .shading_strip_clocks(self.model.asic_dpi_for(1200), false, None);
        self.write(0x2B, timing.dummy as u8)?;
        self.write(0xA5, clk_a as u8)?;
        self.write(0xAB, clk_b as u8)?;
        self.write(0xA3, 0x00)?;
        self.write_many(&RegMap::from_pairs(&CALIB_STRIP_A6_A9))?;
        let end = if wide {
            AFE_WIDE_ENDPIXEL
        } else {
            AFE_ENDPIXEL
        };
        self.protocol.write_u24(r::REG_LINCNT, 1)?;
        self.protocol.write_u16(r::REG_DPISET, dpi_calib as u32)?;
        self.protocol
            .write_u24(r::REG_STRPIXEL, AFE_STRPIXEL as u32)?;
        self.protocol.write_u24(r::REG_ENDPIXEL, end as u32)?;
        self.protocol.write_u24(r::REG_FEEDL, 1)?;
        self.protocol
            .write_u24(r::REG_LPERIOD, timing.lperiod as u32)?;
        self.protocol
            .write_u24(r::REG_EXPOSURE, self.model.exposure_lperiod as u32)?;
        self.write(r::REG_DEPTH_A, r::DEPTH16_A)?;
        self.write(r::REG_DEPTH_B, r::DEPTH16_B)?;
        self.write(r::REG_0X02, 0x00)?;
        let reg01 = (self.cached(r::REG_0X01, 0x22) | r::SHDAREA) & !r::SCAN & !r::DVDSET;
        self.write(r::REG_0X01, reg01)
    }

    /// Read one stationary 16-bit AFE strip. Does not move the carriage.
    pub fn acquire_afe_strip(
        &mut self,
        size: usize,
        timeout_s: f64,
        timing: Option<AfeStripTiming>,
    ) -> Result<Vec<u8>> {
        if !self.initialized {
            self.init(false)?;
        }
        if size == 0 {
            return Err(Error::Value("AFE strip size must be positive".into()));
        }
        self.reset_stationary_scan_engine("AFE strip")?;
        self.setup_afe_strip_regs(size >= AFE_WIDE_BYTES, timing)?;
        let mut last_exc: Option<Error> = None;
        for attempt in 0..=STATIONARY_START_RETRIES {
            if attempt > 0 {
                warn!(
                    "GL128 AFE strip START retry after: {}",
                    last_exc.as_ref().map(|e| e.to_string()).unwrap_or_default()
                );
                self.reset_stationary_scan_engine("AFE strip retry")?;
            }
            self.start_stationary_strip()?;
            match self.wait_stationary_data_ready(timeout_s, "AFE strip") {
                Ok(()) => {
                    last_exc = None;
                    break;
                }
                Err(e) if e.is_scan_error() => {
                    last_exc = Some(e);
                    self.reset_stationary_scan_engine("AFE strip fail")?;
                }
                Err(e) => return Err(e),
            }
        }
        if let Some(e) = last_exc {
            return Err(e);
        }
        self.protocol
            .bulk_read_begin(size, r::BULK_INDEX_RAM, r::AHB_RAM_BASE)?;
        let mut buf = Vec::with_capacity(size);
        while buf.len() < size {
            let chunk = self.protocol.bulk_read_chunk(size - buf.len())?;
            if chunk.is_empty() {
                break;
            }
            buf.extend_from_slice(&chunk);
        }
        self.update_bits(r::REG_0X01, 0, r::SCAN)?;
        if buf.len() < size {
            return Err(Error::Scan(format!(
                "AFE strip short read: {} of {size} bytes",
                buf.len()
            )));
        }
        buf.truncate(size);
        Ok(buf)
    }

    /// SilverFast's coarse AFE calibration; leave the FE at the result.
    pub fn search_afe(
        &mut self,
        method: Option<ScanMethod>,
        resolution: Option<i64>,
    ) -> Result<AfeFrontend> {
        if !self.initialized {
            self.init(false)?;
        }
        self.reinit_before_stationary_strips()?;
        if let Some(m) = method {
            self.set_scan_method(m);
        }
        let infrared = self.scan_method == ScanMethod::Infrared;
        let scan_dpi = self.model.asic_dpi_for(resolution.unwrap_or(1200));
        let pre_timing = afe_strip_timing(scan_dpi, false, true);
        let timing = afe_strip_timing(scan_dpi, true, false);
        // Strips are decoded chunky; image/shading layout stays the model's.
        self.usb_planar_rgb = self.model.g().usb_planar_rgb;
        // Never lamp_off() here: in IR mode it clears the IR LED too.
        self.lamp_on()?;
        sleep(0.3);
        let zero = [0; 3];
        // Throwaway strip (#124): the first line after the CCD sat unclocked reads high.
        self.apply_frontend(AfeFrontend {
            gains: zero,
            offsets: [AFE_OFFSET_PROBE_LOW; 3],
        })?;
        self.acquire_afe_strip(AFE_STRIP_BYTES, 5.0, Some(pre_timing))?;
        let pre = self.offsets_for_black(
            zero,
            AFE_PRE_OFFSET_BLACK,
            pre_timing,
            AFE_FALLBACK_PRE_OFFSETS,
            "pre-offset",
        )?;
        self.apply_frontend(AfeFrontend {
            gains: zero,
            offsets: encode_offsets(pre),
        })?;
        let wide = self.acquire_afe_strip(AFE_WIDE_BYTES, 5.0, Some(timing))?;
        let wide_means = channel_means_u16(&wide, AFE_WIDE_PIXELS, false)?;
        let gains = coarse_gains_from_wide_means(
            wide_means,
            if infrared {
                AFE_WHITE_TARGETS_IR
            } else {
                AFE_WHITE_TARGETS_COLOR
            },
            AFE_GAIN_MAX,
        );
        let offsets = self.offsets_for_black(
            gains,
            if infrared {
                AFE_BLACK_TARGET_IR
            } else {
                AFE_BLACK_TARGET_COLOR
            },
            timing,
            if infrared {
                AFE_FALLBACK_OFFSETS_IR
            } else {
                AFE_FALLBACK_OFFSETS_COLOR
            },
            "black offset",
        )?;
        let result = AfeFrontend {
            gains,
            offsets: encode_offsets(offsets),
        };
        self.apply_frontend(result)?;
        self.last_afe = Some(result);
        self.last_afe_dpi = Some(scan_dpi);
        info!(
            "GL128 AFE {} dpi={scan_dpi} pre_offsets={pre:?} wide_means=({:.0},{:.0},{:.0}) gains={gains:?} offsets={offsets:?}",
            if infrared { "infrared" } else { "colour" },
            wide_means[0],
            wide_means[1],
            wide_means[2]
        );
        Ok(result)
    }

    /// Two-point optical-black probe at `gains`; signed offsets for `target`.
    fn offsets_for_black(
        &mut self,
        gains: [i64; 3],
        target: f64,
        timing: AfeStripTiming,
        fallback: [i64; 3],
        stage: &str,
    ) -> Result<[i64; 3]> {
        let mut black = Vec::with_capacity(2);
        for code in [AFE_OFFSET_PROBE_LOW, AFE_OFFSET_PROBE_HIGH] {
            self.apply_frontend(AfeFrontend {
                gains,
                offsets: [code; 3],
            })?;
            let strip = self.acquire_afe_strip(AFE_STRIP_BYTES, 5.0, Some(timing))?;
            black.push(optical_black_means(&strip)?);
        }
        let solved = offsets_for_black_level(
            black[0],
            black[1],
            target,
            AFE_OFFSET_PROBE_LOW,
            AFE_OFFSET_PROBE_HIGH,
        );
        let offsets = [0, 1, 2].map(|c| solved[c].unwrap_or(fallback[c]));
        if solved.iter().any(Option::is_none) {
            warn!(
                "GL128 AFE {stage}: optical black barely moved between offsets {AFE_OFFSET_PROBE_LOW:#x} and \
                 {AFE_OFFSET_PROBE_HIGH:#x} ({:?} -> {:?}); using SilverFast's codes {fallback:?} on those channels",
                black[0].map(crate::num::py_round),
                black[1].map(crate::num::py_round)
            );
        }
        Ok(offsets)
    }

    /// Write a packed shading coefficient blob to `0x10014000`.
    pub fn upload_shading_table(&mut self, blob: &[u8]) -> Result<()> {
        if blob.is_empty() {
            return Err(Error::Value("shading blob is empty".into()));
        }
        self.protocol.write_ahb(AHB_SHADING, blob)?;
        info!(
            "GL128 uploaded shading table ({} bytes) to 0x{AHB_SHADING:08x}",
            blob.len()
        );
        Ok(())
    }

    /// Resolve the `(STRPIXEL, ENDPIXEL, DPISET)` for a shading pass.
    pub fn shading_window(
        &self,
        pixels: i64,
        resolution: i64,
        strpixel: Option<i64>,
        endpixel: Option<i64>,
        dpiset: Option<i64>,
    ) -> (i64, i64, i64) {
        let dpi = resolution;
        let dpiset = dpiset.unwrap_or_else(|| {
            let by_dpi = &self.model.register_dpiset_by_dpi;
            by_dpi.get(&dpi).copied().unwrap_or((dpi / 6).max(1))
        });
        let factor = (self.model.optical_resolution / (dpiset * 6).max(1)).max(1);
        let start = strpixel.unwrap_or(240);
        let end = endpixel.unwrap_or(start + pixels * factor);
        (start, end, dpiset)
    }

    #[allow(clippy::too_many_arguments)]
    fn setup_shading_strip_regs(
        &mut self,
        pixels: i64,
        lines: i64,
        resolution: i64,
        strpixel: Option<i64>,
        endpixel: Option<i64>,
        dpiset: Option<i64>,
        dvdset: bool,
    ) -> Result<()> {
        let (start, end, dpi_calib) =
            self.shading_window(pixels, resolution, strpixel, endpixel, dpiset);
        let (dummy, clk_a, clk_b) =
            self.model
                .shading_strip_clocks(resolution, dvdset, Some(end - start));
        if !dvdset {
            self.apply_stationary_scan_regs(true)?;
        }
        self.write(0x2B, dummy as u8)?;
        self.write(0xA5, clk_a as u8)?;
        self.write(0xAB, clk_b as u8)?;
        let reg1d = self.cached(0x1D, 0x80) & !0x1F;
        self.write(
            0x1D,
            reg1d | self.model.shading_linesel_for(resolution) as u8,
        )?;
        self.write_many(&RegMap::from_pairs(&CALIB_STRIP_A6_A9))?;
        self.protocol.write_u24(r::REG_LINCNT, lines as u32)?;
        self.protocol.write_u16(r::REG_DPISET, dpi_calib as u32)?;
        self.protocol.write_u24(r::REG_STRPIXEL, start as u32)?;
        self.protocol.write_u24(r::REG_ENDPIXEL, end as u32)?;
        self.protocol.write_u24(r::REG_FEEDL, 1)?;
        self.strip_lperiod = self.model.line_period_for(resolution, Some(end - start));
        self.protocol
            .write_u24(r::REG_LPERIOD, self.strip_lperiod as u32)?;
        self.protocol
            .write_u24(r::REG_EXPOSURE, self.model.exposure_lperiod as u32)?;
        self.write(r::REG_DEPTH_A, r::DEPTH16_A)?;
        self.write(r::REG_DEPTH_B, r::DEPTH16_B)?;
        let reg01 = if dvdset {
            self.write(r::REG_0X02, r::MTRPWR | r::AGOHOME)?;
            self.write(0x3B, 0x00)?;
            self.write(0xA3, 0x00)?;
            (self.cached(r::REG_0X01, 0x22) | r::SHDAREA | r::DVDSET) & !r::SCAN
        } else {
            self.write(r::REG_0X02, 0x00)?;
            self.write(0xA3, 0x01)?;
            (self.cached(r::REG_0X01, 0x22) | r::SHDAREA) & !r::SCAN & !r::DVDSET
        };
        self.write(r::REG_0X01, reg01)?;
        if dvdset {
            let mut rb = Vec::new();
            for a in [
                r::REG_0X01,
                r::REG_0X02,
                0x3B,
                0xA3,
                0x04,
                0x05,
                0x2B,
                0xA5,
                0xAB,
            ] {
                rb.push(self.protocol.read_register(a)?);
            }
            info!(
                "GL128 shading strip DVDSET setup readback 0x01={:#04x} 0x02={:#04x} 0x3B={:#04x} 0xA3={:#04x} \
                 0x04={:#04x} 0x05={:#04x} 0x2B={:#04x} 0xA5={:#04x} 0xAB={:#04x}",
                rb[0], rb[1], rb[2], rb[3], rb[4], rb[5], rb[6], rb[7], rb[8]
            );
        }
        Ok(())
    }

    /// Refuse to continue if the head is off the home sensor (grind risk).
    fn require_carriage_at_home(&mut self, where_: &str) -> Result<()> {
        if !self.read_status()?.is_at_home {
            return Err(Error::Scan(format!(
                "{where_}: carriage is not at home - refuse to avoid grind. Park with SilverFast or power-cycle the scanner."
            )));
        }
        Ok(())
    }

    /// Read a multi-line 16-bit strip for ASIC shading.
    #[allow(clippy::too_many_arguments)]
    pub fn acquire_shading_strip(
        &mut self,
        resolution: i64,
        pixels: Option<i64>,
        lines: usize,
        timeout_s: f64,
        strpixel: Option<i64>,
        endpixel: Option<i64>,
        dpiset: Option<i64>,
        dvdset: bool,
    ) -> Result<Vec<u8>> {
        if !self.initialized {
            self.init(false)?;
        }
        let n = pixels.unwrap_or_else(|| shading_width_for_resolution(resolution));
        let size = n as usize * lines * 6;
        if !dvdset {
            self.reset_stationary_scan_engine("Shading strip")?;
        }
        self.setup_shading_strip_regs(
            n,
            lines as i64,
            resolution,
            strpixel,
            endpixel,
            dpiset,
            dvdset,
        )?;
        // Re-assert motor policy immediately before START.
        if dvdset {
            self.write(r::REG_0X02, r::MTRPWR | r::AGOHOME)?;
        } else {
            self.write(r::REG_0X02, 0x00)?;
            self.wait_idle_at_home_for_stationary(STATIONARY_IDLE_WAIT_S, "Shading strip")?;
        }
        self.write(r::REG_CLRCNT, r::CLRCNT_ALL)?;
        let mut reg01 = self.cached(r::REG_0X01, 0x22) | r::SHDAREA | r::SCAN;
        if dvdset {
            reg01 |= r::DVDSET;
        } else {
            reg01 &= !r::DVDSET;
        }
        self.write(r::REG_0X01, reg01)?;
        if dvdset {
            let rb01 = self.protocol.read_register(r::REG_0X01)?;
            let rb02 = self.protocol.read_register(r::REG_0X02)?;
            info!(
                "GL128 shading strip START 0x01={rb01:#04x} 0x02={rb02:#04x} LPERIOD={} DEPTH={:#04x}/{:#04x} (want 0x01={reg01:#04x} 0x02={:#04x})",
                self.strip_lperiod,
                self.cached(r::REG_DEPTH_A, 0),
                self.cached(r::REG_DEPTH_B, 0),
                r::MTRPWR | r::AGOHOME
            );
        }
        self.write(r::REG_START, r::START_GO)?;
        if let Err(e) = self.wait_shading_data_ready(timeout_s, "Shading strip", dvdset) {
            if e.is_scan_error() {
                self.update_bits(r::REG_0X01, 0, r::SCAN)?;
                if dvdset {
                    self.write(r::REG_0X02, 0x00)?;
                }
            }
            return Err(e);
        }
        self.protocol
            .bulk_read_begin(size, r::BULK_INDEX_RAM, r::AHB_RAM_BASE)?;
        let mut buf = Vec::with_capacity(size);
        while buf.len() < size {
            let chunk = self.protocol.bulk_read_chunk(65536.min(size - buf.len()))?;
            if chunk.is_empty() {
                break;
            }
            buf.extend_from_slice(&chunk);
        }
        // Clear SCAN so AGOHOME (armed during white) parks, then drop motor.
        self.update_bits(r::REG_0X01, 0, r::SCAN)?;
        if dvdset {
            match self.wait_until_at_home(30.0) {
                Ok(()) => info!("GL128 shading white strip parked at home after AGOHOME"),
                Err(Error::MotorTimeout(_)) => {
                    self.write(r::REG_0X02, 0x00)?;
                    return Err(Error::Scan(
                        "Shading white strip: AGOHOME did not return the carriage home. Park with SilverFast or power-cycle, then retry.".into(),
                    ));
                }
                Err(e) => return Err(e),
            }
        }
        self.write(r::REG_0X02, 0x00)?;
        if buf.len() < size {
            return Err(Error::Scan(format!(
                "Shading strip short read: {} of {size} bytes",
                buf.len()
            )));
        }
        buf.truncate(size);
        Ok(buf)
    }

    /// Dark → unity upload → white → measured upload (at home; feeds disarmed).
    ///
    /// Returns the final measured shading blob and sets `asic_shading_ready`
    /// (never for infrared, which stores a host white profile instead).
    pub fn run_asic_shading(
        &mut self,
        resolution: i64,
        method: Option<ScanMethod>,
        strpixel: Option<i64>,
        endpixel: Option<i64>,
        dpiset: Option<i64>,
    ) -> Result<Vec<u8>> {
        if !self.initialized {
            self.init(false)?;
        }
        if self.motor_moves_enabled {
            return Err(Error::Scan("ASIC shading requires motor disarmed — call disarm_bringup_motor before run_asic_shading".into()));
        }
        self.reinit_before_stationary_strips()?;
        if let Some(m) = method {
            self.set_scan_method(m);
        }
        let infrared = self.scan_method == ScanMethod::Infrared;
        self.require_carriage_at_home("ASIC shading start")?;
        let (start, end, used_dpiset) = self.shading_window(
            shading_width_for_resolution(resolution),
            resolution,
            strpixel,
            endpixel,
            dpiset,
        );
        let n = shading_acquire_width(start, end, used_dpiset, self.model.optical_resolution)?;
        let nu = n as usize;
        let declared = declared_shading_size(nu);
        self.asic_shading_ready = false;
        if infrared {
            self.ir_host_flatten_ready = false;
            self.last_ir_host_white = None;
        }
        if let Some(afe) = self.last_afe {
            self.apply_frontend(afe)?;
        }
        let settle_s = if infrared {
            IR_SHADING_DARK_SETTLE_S
        } else {
            COLOR_SHADING_DARK_SETTLE_S
        };
        self.lamp_off()?;
        self.reassert_lamp_off()?;
        sleep(settle_s);
        self.log_lamp_state("shading pre-dark")?;
        if !infrared {
            self.await_colour_optical_dark()?;
        }
        let w = (Some(start), Some(end), Some(used_dpiset));
        let dark_raw = self.acquire_shading_strip(
            resolution,
            Some(n),
            SHADING_LINES,
            30.0,
            w.0,
            w.1,
            w.2,
            false,
        )?;
        // Colour film USB is chunky (session 11); do not trust lag on a flat dark field.
        let strip_planar =
            infrared && choose_usb_planar(&dark_raw[..(nu * 6).min(dark_raw.len())], nu);
        let dark_measured = average_rgb16_columns(&dark_raw, nu, SHADING_LINES, strip_planar)?;
        let dark: Vec<Rgb> = if infrared {
            info!(
                "GL128 IR shading: zero dark terms (session 05); measured dark0={:?} (diagnostic only)",
                dark_measured[0]
            );
            vec![[0; 3]; dark_measured.len()]
        } else {
            info!(
                "GL128 colour dark strip mean={:.0} dark0={:?}",
                shading_columns_mean(&dark_measured[..nu.min(dark_measured.len())]),
                dark_measured[0]
            );
            dark_measured.clone()
        };
        let unity = make_unity_white_table(&dark, Some(declared), true)?;
        self.upload_shading_table(&unity)?;
        if !infrared {
            self.lamp_on()?;
            self.log_lamp_state("shading pre-white")?;
        }
        // SilverFast's white-strip motor table, not the image pass's.
        let period = self.model.shading_white_motor_period(resolution);
        self.upload_tables(resolution, false, false, Some(period))?;
        let reg01 = (self.cached(r::REG_0X01, 0x22) | r::SHDAREA | r::DVDSET) & !r::SCAN;
        self.write(r::REG_0X01, reg01)?;
        if infrared {
            self.lamp_on()?;
            self.write(r::REG_0X03, r::XPASEL)?;
            self.apply_infrared(true)?;
            let reg03 = self.protocol.read_register(r::REG_0X03)?;
            let reg37 = self.protocol.read_register(r::REG_IR)?;
            info!(
                "GL128 IR white strip illum 0x03={reg03:#04x} (LAMPPWR={}) 0x37={reg37:#04x} (IR_LED={})",
                reg03 & r::LAMPPWR != 0,
                reg37 & r::IR_LED != 0
            );
            if reg03 & r::LAMPPWR != 0 || reg37 & r::IR_LED == 0 {
                self.write(r::REG_0X03, r::XPASEL)?;
                self.apply_infrared(true)?;
            }
        }
        sleep(0.5);
        let white_raw = self.acquire_shading_strip(
            resolution,
            Some(n),
            SHADING_LINES,
            30.0,
            w.0,
            w.1,
            w.2,
            true,
        )?;
        let mut white = average_rgb16_columns(&white_raw, nu, SHADING_LINES, strip_planar)?;
        let mut raw_white: Option<Vec<Rgb>> = None;
        self.last_host_calib_dark = None;
        self.last_host_calib_white = None;
        self.last_color_shading_host_ok = false;
        if infrared {
            let raw0 = white[0];
            let spread = raw0.iter().max().unwrap() - raw0.iter().min().unwrap();
            raw_white = Some(white.clone());
            white = equalize_ir_white_columns(&white);
            info!(
                "GL128 IR shading: equalized white columns (session 05 shape); raw white0={raw0:?} spread={spread} → equalized white0={:?}",
                white[0]
            );
        }
        let gains = shading_gains_from_white(&white, SHADING_GAIN_TARGET);
        let measured =
            build_measured_shading_table(&dark, &white, true, Some(declared), SHADING_GAIN_TARGET)?;
        self.upload_shading_table(&measured)?;
        self.write(r::REG_0X02, 0x00)?;
        self.require_carriage_at_home("ASIC shading end")?;
        let k = nu.min(white.len());
        if infrared {
            let (ok, reason) = validate_ir_shading_table(
                &dark[..nu.min(dark.len())],
                &white[..k],
                Some(n),
                raw_white.as_deref().map(|rw| &rw[..nu.min(rw.len())]),
            );
            let white_prefix = &white[..k];
            let white_mean = white_prefix.iter().flatten().sum::<i64>() as f64
                / (white_prefix.len() * 3).max(1) as f64;
            self.asic_shading_ready = false;
            if ok {
                self.last_ir_host_white = Some(white_prefix.iter().map(|row| row[0]).collect());
                self.ir_host_flatten_ready = true;
                info!(
                    "GL128 IR host flatten ready dpi={resolution} acquire={n} window={start}..{end} dpiset={used_dpiset} \
                     white0={:?} white_mean={white_mean:.0} measured_dark0={:?} (DVDSET off)",
                    white[0], dark_measured[0]
                );
            } else {
                self.last_ir_host_white = None;
                self.ir_host_flatten_ready = false;
                warn!(
                    "GL128 IR host flatten rejected ({reason}); DVDSET off dpi={resolution} acquire={n} white0={:?} \
                     white_mean={white_mean:.0} measured_dark0={:?}",
                    white[0], dark_measured[0]
                );
            }
        } else {
            let (strip_ok, mut reason) =
                validate_color_shading_strip(&dark[..nu.min(dark.len())], &white[..k], Some(n));
            let mut ok = strip_ok;
            if ok {
                (ok, reason) = validate_color_shading_gains(&gains[..nu.min(gains.len())]);
            }
            let mut flat: Vec<i64> = gains[..nu.min(gains.len())]
                .iter()
                .flatten()
                .copied()
                .collect();
            flat.sort_unstable();
            let median_gain = flat[nu * 3 / 2];
            if ok {
                self.asic_shading_ready = true;
                self.last_color_shading_reject_reason = None;
                info!(
                    "GL128 ASIC shading ready dpi={resolution} method={} acquire={n} window={start}..{end} dpiset={used_dpiset} \
                     dark0={:?} white0={:?} gain0={:?} median_gain={:.3}x",
                    self.scan_method.as_str(),
                    dark[0],
                    white[0],
                    gains[0],
                    median_gain as f64 / SHADING_GAIN_UNITY as f64
                );
            } else {
                self.asic_shading_ready = false;
                self.last_color_shading_reject_reason = Some(reason.clone());
                self.upload_shading_table(&unity)?;
                if strip_ok {
                    self.last_host_calib_dark = Some(dark[..nu.min(dark.len())].to_vec());
                    self.last_host_calib_white = Some(white[..k].to_vec());
                    self.last_color_shading_host_ok = true;
                }
                warn!(
                    "GL128 colour ASIC shading rejected ({reason}); DVDSET left off, unity table restored, host_stretch={strip_ok}. \
                     dpi={resolution} acquire={n} dark_mean={:.0} white_mean={:.0} median_gain={:.3}x",
                    shading_columns_mean(&dark[..nu.min(dark.len())]),
                    shading_columns_mean(&white[..k]),
                    median_gain as f64 / SHADING_GAIN_UNITY as f64
                );
            }
        }
        Ok(measured)
    }

    /// Re-initialise the ASIC before stationary strips that follow an image pass.
    pub fn reinit_before_stationary_strips(&mut self) -> Result<()> {
        if !self.image_pass_since_init {
            return Ok(());
        }
        info!("GL128 re-initialising the ASIC before stationary strips (after an image pass)");
        self.asic_boot()?;
        let max_res = *self.model.resolutions_dpi.iter().max().unwrap();
        self.upload_tables(max_res, false, true, None)?;
        self.asic_shading_ready = false;
        self.image_pass_since_init = false;
        self.enter_scan_clock()
    }

    /// Upload motor slope and/or scan motor tables to scanner RAM.
    pub fn upload_tables(
        &mut self,
        resolution: i64,
        shading: bool,
        slope: bool,
        channel_exposure: Option<i64>,
    ) -> Result<()> {
        let use_slow = shading || self.image_slope_slow;
        if slope {
            let table = if use_slow {
                self.model.slope_table_slow()
            } else {
                self.model.slope_table_fast()
            };
            let slope_bytes = u16_table_bytes(table);
            self.protocol.write_ahb(r::AHB_SLOPE_SCAN, &slope_bytes)?;
            self.protocol.write_ahb(r::AHB_SLOPE_FAST, &slope_bytes)?;
        }
        let ce =
            channel_exposure.unwrap_or_else(|| self.model.channel_exposure_for(resolution, None));
        let exposure = u16_table_bytes(&self.model.scan_motor_table(ce)?);
        for addr in [
            r::AHB_SCAN_MOTOR_0,
            r::AHB_SCAN_MOTOR_1,
            r::AHB_SCAN_MOTOR_2,
        ] {
            self.protocol.write_ahb(addr, &exposure)?;
        }
        debug!(
            "GL128 uploaded {} for {resolution} dpi",
            if !slope {
                "exposure only"
            } else if use_slow {
                "slow slope + exposure"
            } else {
                "fast slope + exposure"
            }
        );
        Ok(())
    }

    /// Replay the captured cold-boot sequence (no soft reset, never `0x0E`-`0x10`).
    pub fn asic_boot(&mut self) -> Result<()> {
        self.protocol.write_ahb(BOOT_BLOB_ADDR_A, &[0u8; 34])?;
        self.protocol.write_ahb(BOOT_BLOB_ADDR_B, &boot_blob_b())?;
        let init = self.model.init_regs.clone();
        self.write_many(&init)?;
        let g = self.model.g();
        let (clk, dram) = (g.asic_clock_boot_0x8c as u8, g.asic_boot_0x0b as u16);
        self.write_clock(clk)?;
        self.write(REG_CLK_DRAM, (dram & !DRAM_ENABLE) as u8)?;
        self.write(REG_0X13, DRAM_INIT_0X13 as u8)?;
        self.write(REG_CLK_DRAM, (dram | DRAM_ENABLE) as u8)?;
        let ml = self.model.memory_layout_regs.clone();
        self.write_many(&ml)?;
        self.set_frontend_init()?;
        let gpo = self.model.gpo_regs.clone();
        self.write_many(&gpo)?;
        info!(
            "GL128 boot: {} init + {} layout + {} gpo registers",
            self.model.init_regs.len(),
            self.model.memory_layout_regs.len(),
            self.model.gpo_regs.len()
        );
        Ok(())
    }

    fn write_clock(&mut self, value: u8) -> Result<()> {
        self.protocol.write_0x8c(0x10, value)?;
        self.protocol.write_0x8c(0x13, value)
    }

    /// Switch to the scan clock the way the vendor opens every scan pass.
    pub fn enter_scan_clock(&mut self) -> Result<()> {
        if !self.initialized {
            self.init(false)?;
        }
        let g = self.model.g();
        let (boot, scan) = (g.asic_clock_boot_0x8c as u8, g.asic_clock_scan_0x8c as u8);
        self.write_clock(boot)?;
        self.write_clock(scan)?;
        let v0b = self
            .model
            .sensor_custom_regs
            .get(&REG_CLK_DRAM)
            .copied()
            .expect("sensor 0x0B") as u8;
        let v13 = self
            .model
            .init_regs
            .get(&REG_0X13)
            .copied()
            .expect("init 0x13") as u8;
        self.write(REG_CLK_DRAM, v0b)?;
        self.write(REG_0X13, v13)?;
        debug!("GL128 scan clock 0x8C: {boot:#04x} -> {scan:#04x}");
        Ok(())
    }

    pub fn init(&mut self, force: bool) -> Result<()> {
        if self.initialized && !force {
            return Ok(());
        }
        self.asic_boot()?;
        let max_res = *self.model.resolutions_dpi.iter().max().unwrap();
        self.upload_tables(max_res, false, true, None)?;
        self.last_afe = None;
        self.last_afe_dpi = None;
        self.image_pass_since_init = false;
        self.asic_shading_ready = false;
        self.last_ir_host_white = None;
        self.ir_host_flatten_ready = false;
        self.usb_planar_rgb = false;
        self.initialized = true;
        info!("GL128 initialised ({})", self.model.model);
        Ok(())
    }

    // --- buttons ---

    pub fn button_state(&mut self, timeout_ms: u32) -> Result<HashSet<Button>> {
        let mut buttons = HashSet::new();
        let data = self.protocol.read_intr(Some(timeout_ms))?;
        let Some(&b) = data.first() else {
            return Ok(buttons);
        };
        for (id, button) in BUTTON_IDS {
            if b & id != 0 {
                buttons.insert(button);
            }
        }
        Ok(buttons)
    }

    // --- lamp / infrared ---

    pub fn set_scan_method(&mut self, method: ScanMethod) {
        self.scan_method = method;
        debug!("GL128 scan_method={}", method.as_str());
    }

    pub fn apply_infrared(&mut self, enabled: bool) -> Result<()> {
        if enabled {
            self.update_bits(r::REG_IR, r::IR_LED, 0)?;
        } else if self.scan_method == ScanMethod::Infrared {
            self.update_bits(r::REG_IR, 0, r::IR_LED)?;
        } else {
            // The vendor holds 0xC0 for all colour work rather than rewriting the readback.
            let v = self.model.init_regs.get_or(r::REG_IR, 0xC0) as u8;
            self.write(r::REG_IR, v)?;
        }
        Ok(())
    }

    /// Power the lamp for the selected method.
    pub fn lamp_on(&mut self) -> Result<()> {
        if self.scan_method == ScanMethod::Infrared {
            self.write(r::REG_0X03, r::XPASEL)?;
            self.apply_infrared(true)?;
        } else {
            self.write(r::REG_0X03, r::XPASEL | r::LAMPPWR)?;
            self.apply_infrared(false)?;
        }
        info!("GL128 lamp on ({})", self.scan_method.as_str());
        Ok(())
    }

    pub fn lamp_off(&mut self) -> Result<()> {
        if !self.initialized {
            debug!("GL128 lamp_off before init — nothing to do");
            return Ok(());
        }
        self.reassert_lamp_off()?;
        info!("GL128 lamp off");
        Ok(())
    }

    /// Force white lamp + IR LED off (`0x03 = 0x20`).
    fn reassert_lamp_off(&mut self) -> Result<()> {
        self.write(r::REG_0X03, r::XPASEL)?;
        self.apply_infrared(false)
    }

    /// Replay capture lamp-stabilise toggles; end with white lamp on.
    pub fn strike_lamp_on(&mut self) -> Result<()> {
        if self.scan_method == ScanMethod::Infrared {
            self.write(r::REG_0X03, r::XPASEL)?;
            return self.apply_infrared(true);
        }
        let (off, on) = (r::XPASEL, r::XPASEL | r::LAMPPWR);
        for v in [on, off, on, off, on] {
            self.write(r::REG_0X03, v)?;
            sleep(0.02);
        }
        self.apply_infrared(false)?;
        info!("GL128 lamp strike (transparency)");
        Ok(())
    }

    fn probe_dark_mean(&mut self) -> Result<f64> {
        if let Some(afe) = self.last_afe {
            self.apply_frontend(afe)?;
        }
        let strip = self.acquire_afe_strip(AFE_STRIP_BYTES, 5.0, None)?;
        let m = channel_means_u16(&strip, AFE_STRIP_PIXELS, self.usb_planar_rgb)?;
        Ok((m[0] + m[1] + m[2]) / 3.0)
    }

    /// Cheap AFE probe after lamp-off; one extra 0.5s settle then fail-fast.
    fn await_colour_optical_dark(&mut self) -> Result<()> {
        let max = COLOR_SHADING_DARK_MEAN_MAX;
        let mean = self.probe_dark_mean()?;
        info!(
            "GL128 colour pre-dark AFE probe mean={mean:.0} (need <= {})",
            max as i64
        );
        if mean <= max {
            return Ok(());
        }
        self.reassert_lamp_off()?;
        sleep(COLOR_SHADING_DARK_SETTLE_S);
        self.log_lamp_state("shading pre-dark retry")?;
        let mean = self.probe_dark_mean()?;
        info!(
            "GL128 colour pre-dark AFE probe retry mean={mean:.0} (need <= {})",
            max as i64
        );
        if mean <= max {
            return Ok(());
        }
        Err(Error::Scan(format!(
            "shading pre-dark: lamp-off strip still bright (mean={mean:.0} > {}). Lamp did not go dark or head is \
             not on the clear home field — power-cycle / park and retry.",
            max as i64
        )))
    }

    fn log_lamp_state(&mut self, where_: &str) -> Result<()> {
        let reg03 = self.protocol.read_register(r::REG_0X03)?;
        let status = self.read_status()?;
        info!(
            "GL128 {where_} lamp state 0x03={reg03:#04x} LAMPPWR={} LAMPSTS={}",
            reg03 & r::LAMPPWR != 0,
            status.is_lamp_on
        );
        Ok(())
    }

    /// No-op: the SE captures show no GPIO poke around scan start.
    pub fn update_home_sensor_gpio(&mut self) -> Result<()> {
        Ok(())
    }

    // --- motion ---

    /// Convert millimetres to steps (experimental; prefer capture constants).
    pub fn feed_steps_for_mm(&self, distance_mm: f64) -> i64 {
        let g = self.model.g();
        let limit = distance_mm.min(g.max_feed_mm).max(0.0);
        if limit != distance_mm {
            warn!("Clamped feed {distance_mm:.2} mm to {limit:.2} mm (model max_feed_mm)");
        }
        crate::num::py_int(limit * g.feed_steps_per_inch as f64 / MM_PER_INCH)
    }

    /// Upload the motor ramp to both AHB slope windows.
    fn upload_fast_slopes(&mut self, use_slow: bool) -> Result<()> {
        let table = if use_slow {
            self.model.slope_table_slow()
        } else {
            self.model.slope_table_fast()
        };
        let slope = u16_table_bytes(table);
        self.protocol.write_ahb(r::AHB_SLOPE_SCAN, &slope)?;
        self.protocol.write_ahb(r::AHB_SLOPE_FAST, &slope)
    }

    /// Poll `0x101` until the carriage is home and the motor is idle.
    pub fn wait_until_at_home(&mut self, timeout_s: f64) -> Result<()> {
        let deadline = monotonic() + timeout_s;
        while monotonic() < deadline {
            let st = self.read_status_reliable()?;
            if st.is_at_home && !st.is_motor_enabled {
                self.park_ok = true;
                return Ok(());
            }
            sleep(HOME_POLL_S);
        }
        Err(Error::MotorTimeout(format!(
            "Carriage did not reach home within {timeout_s:.0}s (AGOHOME park)"
        )))
    }

    /// Session-10 style warm re-init while already at home (`0x02=0x78`).
    pub fn warm_prepare(&mut self) -> Result<()> {
        self.write(r::REG_0X01, 0x22)?;
        self.write(r::REG_0X02, 0x78)
    }

    /// Replay the captured fast-feed recipe.
    #[allow(clippy::too_many_arguments)]
    pub fn feed_capture(
        &mut self,
        steps: i64,
        timeout_s: f64,
        require_motion: bool,
        use_slow_slope: bool,
        final_positioning: bool,
        ignore_probe: bool,
        scan_regs: Option<&RegMap>,
    ) -> Result<()> {
        let steps = steps.max(0);
        if steps == 0 {
            return Ok(());
        }
        let max_steps = self.model.g().max_feed_steps;
        if steps > max_steps {
            return Err(Error::Asic(format!(
                "Refusing FEEDL={steps}: larger than any captured feed ({max_steps}). See docs/gl128-model-comparison.md, section 4."
            )));
        }
        self.wait_motor_idle(&format!("before arming a {steps}-step feed"))?;
        self.write_many(&feed_setup_regs(scan_regs, final_positioning))?;
        self.protocol.write_u24(r::REG_FEEDL, steps as u32)?;
        self.write(r::REG_0X02, r::MTRPWR | r::FASTFED)?;
        self.upload_fast_slopes(use_slow_slope)?;
        // The probe and FEEDFSH still carry the previous feed's completion.
        let stale_done = self.feed_done_indicated()?;
        let before = self.read_status_reliable()?;
        // Capture: start feed with 0x0f only — no 0x0d counter clear here.
        self.write(r::REG_START, r::START_GO)?;
        let started = monotonic();
        self.wait_feed_probe_done(steps, timeout_s, stale_done, require_motion, ignore_probe)?;
        let done_s = monotonic() - started;
        self.write(r::REG_0X02, r::FASTFED)?;
        self.protocol.write_u24(r::REG_FEEDL, 1)?;
        let after = self.wait_motor_idle(&format!("after a {steps}-step feed"))?;
        info!(
            "GL128 feed of {steps} steps complete in {done_s:.2}s, idle at {:.2}s (status 0x{:02x} -> 0x{:02x}, home {} -> {})",
            monotonic() - started,
            before.raw,
            after.raw,
            py_bool(before.is_at_home),
            py_bool(after.is_at_home)
        );
        Ok(())
    }

    /// Poll `0x101` until `MOTORENB` clears; error if the motor stays enabled.
    pub fn wait_motor_idle(&mut self, context: &str) -> Result<ScannerStatus> {
        let deadline = monotonic() + MOTOR_IDLE_TIMEOUT_S;
        let status = loop {
            let st = self.read_status_reliable()?;
            if !st.is_motor_enabled {
                return Ok(st);
            }
            if monotonic() >= deadline {
                break st;
            }
            sleep(MOTOR_IDLE_POLL_S);
        };
        if let Err(e) = self.update_bits(r::REG_0X02, 0, r::MTRPWR) {
            warn!("GL128 motor idle timeout motor clear: {e}");
        }
        Err(Error::MotorTimeout(format!(
            "GL128 motor stayed enabled for {MOTOR_IDLE_TIMEOUT_S:.0}s {context} (status=0x{:02x}). Check that the film holder moves freely.",
            status.raw
        )))
    }

    fn read_feed_probe(&mut self) -> i32 {
        self.protocol
            .read_request_register(FEED_PROBE_INDEX)
            .map_or(-1, |v| v as i32)
    }

    /// True when probe/status currently claim a feed has finished.
    fn feed_done_indicated(&mut self) -> Result<bool> {
        if self.read_feed_probe() == FEED_PROBE_DONE {
            return Ok(true);
        }
        let st = self.read_status_reliable()?;
        Ok(st.is_feeding_finished && !st.is_motor_enabled)
    }

    /// Wait for a feed's step count to run out (probe `0x21=0x04` or `FEEDFSH`).
    fn wait_feed_probe_done(
        &mut self,
        steps: i64,
        timeout_s: f64,
        stale_done: bool,
        require_motion: bool,
        ignore_probe: bool,
    ) -> Result<()> {
        let deadline = monotonic() + timeout_s;
        let mut motion_seen = false;
        let mut next_status = 0.0;
        while monotonic() < deadline {
            if motion_seen {
                if !ignore_probe && self.read_feed_probe() == FEED_PROBE_DONE {
                    return Ok(());
                }
                if monotonic() >= next_status {
                    let st = self.read_status_reliable()?;
                    if st.is_feeding_finished && !st.is_motor_enabled {
                        return Ok(());
                    }
                    next_status = monotonic() + HOME_POLL_S;
                }
                sleep(FEED_PROBE_POLL_S);
                continue;
            }
            let probe = if ignore_probe {
                -1
            } else {
                self.read_feed_probe()
            };
            let st = self.read_status_reliable()?;
            if !require_motion {
                if probe == FEED_PROBE_DONE || (st.is_feeding_finished && !st.is_motor_enabled) {
                    return Ok(());
                }
            } else {
                motion_seen = st.is_motor_enabled;
                if motion_seen {
                    next_status = monotonic() + HOME_POLL_S;
                    continue;
                }
            }
            sleep(HOME_POLL_S);
        }
        // Feed timed out — clear SCAN and drop motor power (not an AGOHOME park).
        self.stop_motor();
        if let Err(e) = self.update_bits(r::REG_0X02, 0, r::MTRPWR) {
            warn!("GL128 feed timeout motor clear: {e}");
        }
        if require_motion && stale_done && !motion_seen {
            return Err(Error::MotorTimeout(format!(
                "Feed of {steps} steps timed out without observing motor motion, while stale completion was already \
                 visible on the vendor probe (wIndex=0x21 -> 0x04)."
            )));
        }
        Err(Error::MotorTimeout(format!(
            "Feed of {steps} steps did not finish within {timeout_s:.0}s"
        )))
    }

    /// Move the carriage `steps` using the capture-faithful feed recipe.
    pub fn feed(&mut self, steps: i64, timeout_s: f64) -> Result<()> {
        self.require_motor_enabled()?;
        self.feed_capture(steps, timeout_s, true, false, false, false, None)
    }

    /// From home: feed to reference, then to the scan-start line.
    pub fn position_for_full_frame_scan(
        &mut self,
        scan_steps: Option<i64>,
        timeout_s: f64,
        scan_regs: Option<&RegMap>,
    ) -> Result<()> {
        self.require_motor_enabled()?;
        if !self.park_ok {
            return Err(Error::Asic(
                "GL128 park failed in the previous scan (AGOHOME timed out). Power-cycle the scanner or park with \
                 SilverFast before running another SE scan."
                    .into(),
            ));
        }
        let start = self.read_status_reliable()?;
        if !start.is_at_home {
            return Err(Error::Asic(
                "SE carriage is not at home. There is no capture-proven standalone reverse-home; park with SilverFast \
                 or power-cycle, then retry from home."
                    .into(),
            ));
        }
        self.warm_prepare()?;
        let g = self.model.g();
        let second = scan_steps.unwrap_or(g.feed_to_scan_steps);
        let first = g.feed_to_reference_steps;
        info!(
            "GL128 positioning from home (status 0x{:02x}): {first} then {second} steps",
            start.raw
        );
        self.feed_capture(
            first,
            (timeout_s / 2.0).min(REFERENCE_FEED_TIMEOUT_S),
            true,
            true,
            false,
            false,
            scan_regs,
        )?;
        self.feed_capture(second, timeout_s / 2.0, true, false, true, true, scan_regs)?;
        let end = self.read_status_reliable()?;
        info!("GL128 positioned for scan (status 0x{:02x})", end.raw);
        if end.is_at_home {
            return Err(Error::Asic(format!(
                "GL128 still reads at-home after feeding the positioning pair ({first}+{second} steps). The carriage \
                 likely did not move, so the scan would not cover the film."
            )));
        }
        Ok(())
    }

    /// Abort / end an image pass the way SilverFast does. Never fails: errors
    /// leave `park_ok` false (carriage position unknown).
    pub fn stop_motor(&mut self) {
        if !self.initialized {
            debug!("GL128 stop_motor before init — nothing to do");
            return;
        }
        if let Err(e) = self.stop_motor_inner() {
            self.park_ok = false;
            error!("GL128 stop_motor: {e} (carriage position unknown)");
        }
    }

    fn stop_motor_inner(&mut self) -> Result<()> {
        let reg02 = match self.reg_cache.get(&r::REG_0X02) {
            Some(&v) => v as u8,
            None => self.protocol.read_register(r::REG_0X02).unwrap_or(0),
        };
        if reg02 & r::AGOHOME != 0 {
            for v in CANCEL_LAMP_PRE {
                self.write(r::REG_0X03, v)?;
            }
            self.write(r::REG_0X01, CANCEL_REG01)?;
            for v in CANCEL_LAMP_POST {
                self.write(r::REG_0X03, v)?;
            }
            match self.wait_until_at_home(60.0) {
                Ok(()) => {
                    self.park_ok = true;
                    // Drop AGOHOME from the cache so a second stop does not re-strobe.
                    self.reg_cache
                        .insert(r::REG_0X02, (reg02 & !r::AGOHOME) as u16);
                    info!("GL128 parked at home after the scan");
                }
                Err(Error::MotorTimeout(m)) => {
                    self.park_ok = false;
                    error!(
                        "GL128 did not park at home: {m}. Home is the origin for the next scan's feeds, so power-cycle \
                         or park with SilverFast before scanning again."
                    );
                }
                Err(e) => return Err(e),
            }
        } else {
            self.update_bits(r::REG_0X01, 0, r::SCAN)?;
        }
        Ok(())
    }

    /// No-op when already home; otherwise refuse (no capture-proven home seek).
    pub fn home(&mut self) -> Result<()> {
        self.require_motor_enabled()?;
        if self.read_status_reliable()?.is_at_home {
            debug!("GL128 already at home");
            return Ok(());
        }
        Err(Error::Asic(
            "GL128 has no capture-proven standalone home seek. Park with SilverFast or power-cycle so the carriage is \
             at home, then continue. See docs/gl128-model-comparison.md, section 4."
                .into(),
        ))
    }

    pub fn park(&mut self) -> Result<()> {
        self.require_motor_enabled()?;
        if !self.read_status_reliable()?.is_at_home {
            return Err(Error::Asic(
                "GL128 park needs the carriage already at home (no reverse-home recipe in captures). Use SilverFast or \
                 power-cycle first."
                    .into(),
            ));
        }
        self.lamp_off()
    }
}

fn py_bool(b: bool) -> &'static str {
    if b { "True" } else { "False" }
}
