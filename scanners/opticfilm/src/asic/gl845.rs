// SPDX-License-Identifier: GPL-3.0-or-later
//! GL845 chip operations for OpticFilm 8200i (and the GL843 / GL842 boots).
//!
//! Sequences ported from SANE genesys `gl846.cpp` / `low.cpp` /
//! `command_set_common.cpp` (GL845 shares the GL846 path); the GL843 and
//! GL842 variants differ only in `asic_boot` and the home path.

use log::{debug, info, warn};

use super::registers::gl845 as r;
use super::status::ScannerStatus;
use crate::clock::{monotonic, sleep};
use crate::device::model::{FilmModel, RegMap, ScanMethod};
use crate::device::sensor_lookup::frontend_regs_for;
use crate::error::{Error, Result};
use crate::scan::calib_gl128::{AfeFrontend, AfeSearchConfig, run_afe_dichotomy};
use crate::usb::protocol::GenesysUsbProtocol;

/// Home/park must never run unbounded.
pub const DEFAULT_HOME_TIMEOUT_S: f64 = 30.0;
pub const HOME_POLL_S: f64 = 0.1;
pub const MOTOR_STOP_TIMEOUT_S: f64 = 1.0;
pub const FE_BUSY_TIMEOUT_S: f64 = 5.0;
/// SANE GL846 reverse-home scan starts at this motor position.
pub const HOME_FEED_STEPS: i64 = 40000;

const ENBDRAM: u16 = 0x08;
const DRAMSEL: u16 = 0x07;

/// Which SANE command set boots the chip.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gl84xVariant {
    Gl845,
    Gl843,
    Gl842,
}

/// High-level ASIC ops: boot, lamp, home/park, status.
pub struct Gl845 {
    pub protocol: GenesysUsbProtocol,
    pub model: FilmModel,
    pub variant: Gl84xVariant,
    pub initialized: bool,
    pub reg_cache: RegMap,
    pub scan_method: ScanMethod,
    /// Last applied AFE gains/offsets (host calib may refresh these).
    pub last_afe_gains: Option<[i64; 3]>,
    pub last_afe_offsets: Option<[i64; 3]>,
}

impl Gl845 {
    pub fn new(protocol: GenesysUsbProtocol, model: FilmModel, variant: Gl84xVariant) -> Self {
        Self {
            protocol,
            model,
            variant,
            initialized: false,
            reg_cache: RegMap::default(),
            scan_method: ScanMethod::Transparency,
            last_afe_gains: None,
            last_afe_offsets: None,
        }
    }

    /// SANE-style reverse mini-scan home and verified stop (GL845 only;
    /// GL843/GL842 keep the AGOHOME/FEEDL path).
    pub fn reverse_scan_home(&self) -> bool {
        self.variant == Gl84xVariant::Gl845
    }

    fn cache(&mut self, addr: u16, v: u8) {
        self.reg_cache.insert(addr, v as u16);
    }

    // --- status ---

    pub fn read_status(&mut self) -> Result<ScannerStatus> {
        Ok(ScannerStatus::from_reg41(
            self.protocol.read_register(r::REG_0X41)?,
        ))
    }

    pub fn read_status_reliable(&mut self) -> Result<ScannerStatus> {
        self.read_status()?;
        sleep(0.1);
        self.read_status()
    }

    pub fn is_at_home(&mut self) -> Result<bool> {
        Ok(self.read_status_reliable()?.is_at_home)
    }

    /// Match SANE's GL845 stop check across status and data/motor flags.
    pub fn is_motor_stopped(&mut self) -> Result<bool> {
        let status = self.read_status()?;
        let flags = self.protocol.read_register(r::REG_0X40)?;
        Ok(!status.is_motor_enabled && flags & (r::MOTMFLG | r::DATAENB) == 0)
    }

    /// True when PWRBIT in 0x06 is clear (fresh power-up).
    pub fn is_cold_boot(&mut self) -> Result<bool> {
        Ok(self.protocol.read_register(r::REG_0X06)? & r::PWRBIT == 0)
    }

    // --- boot / init ---

    pub fn soft_reset(&mut self) -> Result<()> {
        self.protocol.write_register(r::REG_0X0E, 0x01)?;
        self.protocol.write_register(r::REG_0X0E, 0x00)
    }

    fn write_sorted(&mut self, regs: &RegMap) -> Result<()> {
        self.protocol.write_registers(&regs.sorted_bytes())?;
        self.reg_cache = regs.clone();
        Ok(())
    }

    /// Power-on ASIC bring-up.
    pub fn asic_boot(&mut self, cold: Option<bool>) -> Result<()> {
        let cold = match cold {
            Some(c) => c,
            None => self.is_cold_boot()?,
        };
        match self.variant {
            Gl84xVariant::Gl845 => self.asic_boot_gl845(cold),
            Gl84xVariant::Gl843 => self.asic_boot_gl843(cold),
            Gl84xVariant::Gl842 => self.asic_boot_gl842(cold),
        }
    }

    /// `CommandSetGl846::asic_boot` for 8200i.
    fn asic_boot_gl845(&mut self, cold: bool) -> Result<()> {
        info!("asic_boot cold={cold}");
        if cold {
            self.soft_reset()?;
        }
        // Optional chip version probe.
        match self.protocol.read_register(r::REG_0X40) {
            Ok(v40) if v40 & r::CHKVER != 0 => match self.protocol.read_register(0x00) {
                Ok(ver) => info!("genesys chip version 0x{ver:02x}"),
                Err(e) => debug!("CHKVER probe skipped: {e}"),
            },
            Ok(_) => {}
            Err(e) => debug!("CHKVER probe skipped: {e}"),
        }
        let regs = self.model.boot_register_map();
        self.write_sorted(&regs)?;
        // Clocks (8200i / 7400 path)
        self.protocol.write_0x8c(0x10, 0x0C)?;
        self.protocol.write_0x8c(0x13, 0x0C)?;
        self.init_gpio()?;
        self.init_memory_layout()?;
        self.protocol.write_register(0xF8, 0x05)?;
        self.cache(0xF8, 0x05);
        Ok(())
    }

    /// SANE `gl843.cpp` film-scanner boot.
    fn asic_boot_gl843(&mut self, cold: bool) -> Result<()> {
        info!("gl843 asic_boot cold={cold} model={}", self.model.name);
        if cold {
            self.soft_reset()?;
        }
        let usb_mode = match self.protocol.read_usb_speed_byte() {
            Ok(speed) if speed & 0x08 != 0 => 1,
            _ => 2,
        };
        self.protocol
            .write_0x8c(0x0F, if usb_mode == 1 { 0x14 } else { 0x11 })?;
        // Pre-bulk 0x6B poke then restore from model tables
        self.protocol.write_register(0x6B, 0x02)?;
        let mut regs = self.model.boot_register_map();
        // DRAM enable: keep DRAMSEL, set ENBDRAM; then apply CLKSET from model
        let clkset = self.model.gl843_clkset.unwrap_or(0x40) as u16;
        let reg0b = (regs.get_or(0x0B, 0x4A) & DRAMSEL) | ENBDRAM | clkset;
        regs.insert(0x0B, reg0b);
        self.write_sorted(&regs)?;
        let clock = self.model.gl843_clock_0x8c10.unwrap_or(0xD4) as u8;
        self.protocol.write_0x8c(0x10, clock)?;
        // Clear RAM address high bytes
        for addr in [0x29, 0x2A, 0x2B] {
            match self.protocol.write_register(addr, 0x00) {
                Ok(()) => self.cache(addr, 0),
                Err(e) => debug!("RAM addr clear 0x{addr:02x} skipped: {e}"),
            }
        }
        self.init_gpio()?;
        sleep(0.1);
        info!("gl843 asic_boot complete");
        Ok(())
    }

    /// SANE `gl842.cpp` (7200 path).
    fn asic_boot_gl842(&mut self, cold: bool) -> Result<()> {
        info!("gl842 asic_boot cold={cold} model={}", self.model.name);
        if cold {
            self.soft_reset()?;
        }
        let regs = self.model.boot_register_map();
        self.write_sorted(&regs)?;
        self.protocol.write_0x8c(0x10, 0x94)?;
        for addr in [0x2A, 0x2B] {
            self.protocol.write_register(addr, 0x00)?;
            self.cache(addr, 0);
        }
        self.init_gpio()?;
        sleep(0.1);
        info!("gl842 asic_boot complete");
        Ok(())
    }

    /// Write GPO table with 0x6e/0x6f first (genesys apply_registers_ordered).
    fn init_gpio(&mut self) -> Result<()> {
        let gpo = self.model.gpo_regs.clone();
        for addr in [0x6E, 0x6F] {
            if let Some(&v) = gpo.get(&addr) {
                self.protocol.write_register(addr, v as u8)?;
                self.cache(addr, v as u8);
            }
        }
        for (&addr, &v) in gpo.iter() {
            if addr == 0x6E || addr == 0x6F {
                continue;
            }
            self.protocol.write_register(addr, v as u8)?;
            self.cache(addr, v as u8);
        }
        Ok(())
    }

    fn init_memory_layout(&mut self) -> Result<()> {
        for (addr, v) in self.model.memory_layout_regs.sorted_bytes() {
            self.protocol.write_register(addr, v)?;
            self.cache(addr, v);
        }
        Ok(())
    }

    fn wait_frontend_ready(&mut self, timeout_s: f64) -> Result<()> {
        let deadline = monotonic() + timeout_s;
        loop {
            if !self.read_status()?.is_front_end_busy {
                return Ok(());
            }
            if monotonic() >= deadline {
                return Err(Error::Asic("Analog frontend stayed busy".into()));
            }
            sleep(0.01);
        }
    }

    /// ADI frontend init (`gl846_set_adi_fe` / AFE_INIT).
    pub fn set_frontend_init(&mut self) -> Result<()> {
        let m = self.scan_method;
        self.apply_frontend_for_scan(None, Some(m))
    }

    fn frontend_feset(&mut self) -> Result<u8> {
        // Python evaluates the read even when the cache has 0x04.
        let read = self.protocol.read_register(0x04)?;
        let v = self.reg_cache.get(&0x04).map_or(read, |&v| v as u8);
        Ok(v & r::FESET)
    }

    /// True when SANE `scanner_offset_calibration` / gain search run (non-ADI FESET).
    pub fn afe_dichotomy_applicable(&mut self) -> Result<bool> {
        Ok(self.frontend_feset()? != r::FESET_ADI)
    }

    /// Write AFE regs from model tables (+ optional dpi overlay / last search).
    pub fn apply_frontend_for_scan(
        &mut self,
        resolution: Option<i64>,
        method: Option<ScanMethod>,
    ) -> Result<()> {
        let feset = self.frontend_feset()?;
        if feset != r::FESET_ADI {
            return Err(Error::Asic(format!(
                "Unsupported frontend type FESET={feset:#x} (need ADI=0x2)"
            )));
        }
        let scan_method = method.unwrap_or(self.scan_method);
        let mut fe = match resolution {
            None => self.model.frontend_regs.clone(),
            Some(res) => frontend_regs_for(&self.model, res, scan_method),
        };
        if let Some(g) = self.last_afe_gains {
            for (i, v) in g.iter().enumerate() {
                fe.insert(0x02 + i as u16, (*v & 0xFF) as u16);
            }
        }
        if let Some(o) = self.last_afe_offsets {
            for (i, v) in o.iter().enumerate() {
                fe.insert(0x05 + i as u16, (*v & 0xFF) as u16);
            }
        }
        self.wait_frontend_ready(FE_BUSY_TIMEOUT_S)?;
        self.protocol
            .write_fe_register(0x00, fe.get_or(0x00, 0xF8) & 0xFF)?;
        self.protocol
            .write_fe_register(0x01, fe.get_or(0x01, 0x80) & 0xFF)?;
        for i in 0..3u8 {
            self.protocol
                .write_fe_register(0x02 + i, fe.get_or(0x02 + i as u16, 0) & 0xFF)?;
        }
        for i in 0..3u8 {
            self.protocol
                .write_fe_register(0x05 + i, fe.get_or(0x05 + i as u16, 0) & 0xFF)?;
        }
        self.last_afe_gains = Some([0x02, 0x03, 0x04].map(|a| fe.get_or(a, 0) as i64));
        self.last_afe_offsets = Some([0x05, 0x06, 0x07].map(|a| fe.get_or(a, 0) as i64));
        debug!(
            "GL845 AFE applied gains={:?} offsets={:?} method={} dpi={resolution:?}",
            self.last_afe_gains,
            self.last_afe_offsets,
            scan_method.as_str()
        );
        Ok(())
    }

    /// Program FE gain/offset codes and remember them for later configure.
    pub fn apply_afe_frontend(
        &mut self,
        fe: AfeFrontend,
        method: Option<ScanMethod>,
        resolution: Option<i64>,
    ) -> Result<()> {
        self.last_afe_gains = Some(fe.gains.map(|v| v & 0xFF));
        self.last_afe_offsets = Some(fe.offsets.map(|v| v & 0xFF));
        self.apply_frontend_for_scan(resolution, method)
    }

    /// Offset then gain search (SANE `scanner_*_calibration`) or table seed.
    pub fn search_afe(
        &mut self,
        method: ScanMethod,
        resolution: Option<i64>,
        measure: Option<&mut dyn FnMut(&mut Self, AfeFrontend) -> Result<[f64; 3]>>,
        config: Option<AfeSearchConfig>,
        force_dichotomy: bool,
    ) -> Result<AfeFrontend> {
        self.last_afe_gains = None;
        self.last_afe_offsets = None;
        self.apply_frontend_for_scan(resolution, Some(method))?;
        let seed = AfeFrontend {
            offsets: self.last_afe_offsets.unwrap(),
            gains: self.last_afe_gains.unwrap(),
        };
        let run_search = force_dichotomy || self.afe_dichotomy_applicable()?;
        let Some(measure) = measure.filter(|_| run_search) else {
            info!(
                "GL845 AFE search (table defaults) method={} dpi={resolution:?} gains={:?} offsets={:?}",
                method.as_str(),
                self.last_afe_gains,
                self.last_afe_offsets
            );
            return Ok(seed);
        };
        let cfg = config.unwrap_or_default();
        let result = {
            let mut m = |fe: AfeFrontend| -> Result<[f64; 3]> {
                self.apply_afe_frontend(fe, Some(method), resolution)?;
                measure(self, fe)
            };
            run_afe_dichotomy(&mut m, Some(cfg), Some(seed))?
        };
        self.apply_afe_frontend(result, Some(method), resolution)?;
        info!(
            "GL845 AFE dichotomy method={} dpi={resolution:?} gains={:?} offsets={:?}",
            method.as_str(),
            self.last_afe_gains,
            self.last_afe_offsets
        );
        Ok(result)
    }

    /// Full init: USB speed probe, asic_boot, frontend (SANE `asic_init` core).
    pub fn init(&mut self, force: bool) -> Result<()> {
        if self.initialized && !force && !self.is_cold_boot()? {
            info!("ASIC already initialized (warm); skipping boot");
            return Ok(());
        }
        match self.protocol.read_usb_speed_byte() {
            Ok(speed) => info!(
                "USB mode probe=0x{speed:02x} → USB {}.0",
                if speed & 0x08 != 0 { 1 } else { 2 }
            ),
            Err(e) => debug!("USB speed probe failed (continuing): {e}"),
        }
        let mut cold = self.is_cold_boot()?;
        // Only a warm (already powered) ASIC can hold a scan left by a dead process.
        if !cold && self.reverse_scan_home() && self.stop_abandoned_scan()? {
            cold = true;
        }
        self.asic_boot(Some(cold))?;
        self.set_frontend_init()?;
        self.initialized = true;
        info!("ASIC init complete");
        Ok(())
    }

    /// Stop a scan a previous process left running; true when one was found.
    fn stop_abandoned_scan(&mut self) -> Result<bool> {
        let reg01 = self.protocol.read_register(r::REG_0X01)?;
        let status = self.read_status()?;
        if reg01 & r::SCAN == 0 && status.is_buffer_empty && !status.is_motor_enabled {
            return Ok(false);
        }
        warn!(
            "Stopping a scan left running by an earlier session (reg01=0x{reg01:02x} status=0x{:02x})",
            status.raw
        );
        self.protocol
            .write_register(r::REG_0X01, reg01 & !r::SCAN)?;
        self.protocol.write_register(r::REG_0X0F, 0x00)?;
        let drained = self.protocol.abort_bulk_stream();
        info!("abandoned scan: drained {drained} bytes; soft-resetting");
        Ok(true)
    }

    // --- lamp ---

    pub fn set_scan_method(&mut self, method: ScanMethod) {
        self.scan_method = method;
    }

    /// Enable lamp (reg 0x03 LAMPPWR). IR uses XPA GPIO instead of white lamp.
    pub fn lamp_on(&mut self) -> Result<()> {
        let ir = self.scan_method == ScanMethod::Infrared;
        if ir && !self.model.supports_infrared {
            return Err(Error::Asic(format!(
                "{} does not support infrared",
                self.model.model
            )));
        }
        let mut reg = self.protocol.read_register(r::REG_0X03)?;
        if ir {
            // genesys: IR scans clear LAMPPWR and set IR GPIO
            reg &= !r::LAMPPWR;
            self.protocol.write_register(r::REG_0X03, reg)?;
            self.set_ir_lamp(true)?;
        } else {
            self.set_ir_lamp(false)?;
            reg |= r::LAMPPWR;
            self.protocol.write_register(r::REG_0X03, reg)?;
        }
        self.cache(0x03, reg);
        info!(
            "lamp_on method={} reg03=0x{reg:02x}",
            self.scan_method.as_str()
        );
        Ok(())
    }

    pub fn lamp_off(&mut self) -> Result<()> {
        let reg = self.protocol.read_register(r::REG_0X03)? & !r::LAMPPWR;
        self.protocol.write_register(r::REG_0X03, reg)?;
        self.cache(0x03, reg);
        self.set_ir_lamp(false)?;
        info!("lamp_off reg03=0x{reg:02x}");
        Ok(())
    }

    /// 8200i IR XPA bit on GPIO 0xa8 (mask 0x04).
    fn set_ir_lamp(&mut self, enabled: bool) -> Result<()> {
        let mut a8 = self.protocol.read_register(r::REG_0XA8)?;
        if enabled {
            a8 = (a8 & !r::IR_LAMP_A8_MASK) | r::IR_LAMP_A8_MASK;
        } else {
            a8 &= !r::IR_LAMP_A8_MASK;
        }
        self.protocol.write_register(r::REG_0XA8, a8)?;
        self.cache(0xA8, a8);
        Ok(())
    }

    // --- motor / home / park ---

    /// `CommandSetGl846::update_home_sensor_gpio`.
    pub fn update_home_sensor_gpio(&mut self) -> Result<()> {
        let val = self.protocol.read_register(r::REG_0X6C)? | 0x41;
        self.protocol.write_register(r::REG_0X6C, val)?;
        self.cache(0x6C, val);
        Ok(())
    }

    /// Stop scan/motor action and verify the ASIC reports the motor idle.
    pub fn stop_motor(&mut self, timeout_s: f64) -> Result<()> {
        if !self.reverse_scan_home() {
            return self.stop_motor_legacy();
        }
        if timeout_s < 0.0 {
            return Err(Error::Value("timeout_s must be non-negative".into()));
        }
        self.update_home_sensor_gpio()?;
        if self.is_motor_stopped()? {
            return Ok(());
        }
        let reg01 = self.protocol.read_register(r::REG_0X01)? & !r::SCAN;
        self.protocol.write_register(r::REG_0X01, reg01)?;
        self.cache(0x01, reg01);
        let deadline = monotonic() + timeout_s;
        loop {
            if self.is_motor_stopped()? {
                return Ok(());
            }
            let remaining = deadline - monotonic();
            if remaining <= 0.0 {
                break;
            }
            sleep(HOME_POLL_S.min(remaining));
        }
        Err(Error::MotorTimeout(format!(
            "Could not stop GL845 motor within {timeout_s:.1}s"
        )))
    }

    /// Unverified stop kept for GL843/GL842 until their home path is re-validated.
    fn stop_motor_legacy(&mut self) -> Result<()> {
        self.update_home_sensor_gpio()?;
        let reg01 = self.protocol.read_register(r::REG_0X01)? & !r::SCAN;
        self.protocol.write_register(r::REG_0X01, reg01)?;
        self.cache(0x01, reg01);
        let reg02 = self.protocol.read_register(r::REG_0X02)? & !r::MTRPWR;
        self.protocol.write_register(r::REG_0X02, reg02)?;
        self.cache(0x02, reg02);
        self.protocol.write_register(r::REG_0X0F, 0x00)?;
        sleep(0.1);
        Ok(())
    }

    fn wait_until_home(&mut self, timeout_s: f64) -> Result<()> {
        let deadline = monotonic() + timeout_s;
        while monotonic() < deadline {
            if self.read_status()?.is_at_home {
                return Ok(());
            }
            sleep(HOME_POLL_S);
        }
        Err(Error::MotorTimeout(format!(
            "Head did not reach home within {timeout_s:.0}s"
        )))
    }

    /// Move carriage to home with a hard timeout. Always stops the motor.
    pub fn home(&mut self, timeout_s: f64, wait: bool) -> Result<()> {
        if !self.initialized {
            return Err(Error::Asic("Call init()/warmup() before home()".into()));
        }
        self.update_home_sensor_gpio()?;
        if self.is_at_home()? {
            info!("already at home");
            return Ok(());
        }
        if !wait {
            return self.start_home_seek();
        }
        let r = self
            .start_home_seek()
            .and_then(|()| self.wait_until_home(timeout_s));
        // Python `finally`: a failing stop replaces the original error.
        self.stop_motor(MOTOR_STOP_TIMEOUT_S)?;
        r?;
        info!("reached home");
        if !self.is_at_home()? {
            return Err(Error::MotorTimeout(format!(
                "Head did not reach home within {timeout_s:.0}s — motor stopped for safety."
            )));
        }
        Ok(())
    }

    fn start_home_seek(&mut self) -> Result<()> {
        if !self.reverse_scan_home() {
            return self.start_home_seek_legacy();
        }
        // The mini-scan register block lives with the GL845 scan configure.
        crate::scan::session::start_home_seek(self, HOME_FEED_STEPS)
    }

    fn start_home_seek_legacy(&mut self) -> Result<()> {
        // Motor: power on, auto-go-home, reverse, not-home flag; clear fastfed
        let reg02 = r::MTRPWR | r::AGOHOME | r::NOTHOME | r::MTRREV;
        self.protocol.write_register(r::REG_0X02, reg02)?;
        self.cache(0x02, reg02);
        // FEEDL ≈ genesys reverse-home starty
        self.protocol.write_u24(0x3D, HOME_FEED_STEPS as u32)?;
        // Ensure SCAN cleared before feed
        let reg01 = self.protocol.read_register(r::REG_0X01)? & !r::SCAN;
        self.protocol.write_register(r::REG_0X01, reg01)?;
        self.cache(0x01, reg01);
        self.protocol.write_register(r::REG_0X0F, 0x01)?; // start motor
        self.update_home_sensor_gpio()?;
        info!("home seek started (feed={HOME_FEED_STEPS})");
        Ok(())
    }

    /// Park GL845 OpticFilm carriages at home.
    pub fn park(&mut self, timeout_s: f64) -> Result<()> {
        self.home(timeout_s, true)
    }
}
