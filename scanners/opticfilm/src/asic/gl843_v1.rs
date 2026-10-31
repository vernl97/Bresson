// SPDX-License-Identifier: GPL-3.0-or-later
//! GL843 chip operations for OpticFilm 7600i v1 (bcdDevice 4.00), from SilverFast captures.
//!
//! Register reads set the address (`0x83`) then read `0x84`; every write is
//! acknowledged by polling `0x8E`/`0x20`.

use std::collections::HashMap;

use base64::Engine;
use serde_json::Value;

use super::status::ScannerStatus;
use crate::clock::{monotonic, sleep};
use crate::device::model::{FilmModel, ScanMethod};
use crate::device::replay_data::{replay_data, replay_profile};
use crate::error::{Error, Result};
use crate::usb::transport::{ControlData, UsbTransport};

pub const REG_STATUS: u8 = 0x41;
pub const STATUS_HOME: u8 = 0x08;
pub const STATUS_MOTORENB: u8 = 0x01;
pub const STATUS_FEEDFSH: u8 = 0x20;
/// 0x03: white LED.
pub const LAMP_BIT: u8 = 0x10;
/// 0xA8: GPIO27 drives the infrared LED.
pub const IR_GPIO_ON: u8 = 0x27;
pub const IR_GPIO_OFF: u8 = 0x20;
/// 0x02: reverse.
pub const MTRREV: u8 = 0x04;
pub const BULK_PACKET: usize = 512;

/// GL843 register, table and raw transfer helpers over a [`UsbTransport`].
pub struct Gl843V1Usb {
    pub transport: Box<dyn UsbTransport>,
    pub shadow: HashMap<u8, u8>,
}

impl Gl843V1Usb {
    pub fn new(transport: Box<dyn UsbTransport>) -> Self {
        Self {
            transport,
            shadow: HashMap::new(),
        }
    }

    pub fn ctl_out(&mut self, value: u16, index: u16, data: &[u8]) -> Result<()> {
        let request = if data.len() > 1 { 0x04 } else { 0x0C };
        let mut error: Option<Error> = None;
        for _attempt in 0..10 {
            match self.transport.control_msg(
                0x40,
                request,
                value,
                index,
                ControlData::Out(data),
                None,
            ) {
                Ok(_) => {}
                // A stalled write is sent again.
                Err(e @ Error::Usb(_)) => {
                    error = Some(e);
                    continue;
                }
                Err(e) => return Err(e),
            }
            for _poll in 0..10 {
                if self.ctl_in(0x8E, 0x20, 1)?[0] & 0x01 != 0 {
                    if value == 0x83 && data.len() > 1 {
                        for p in data.chunks_exact(2) {
                            self.shadow.insert(p[0], p[1]);
                        }
                    }
                    return Ok(());
                }
            }
        }
        Err(Error::Usb(format!(
            "control write 0x{value:02x} not acknowledged: {}",
            error.map_or("None".to_string(), |e| e.to_string())
        )))
    }

    pub fn ctl_in(&mut self, value: u16, index: u16, length: usize) -> Result<Vec<u8>> {
        let req = if length > 1 { 0x04 } else { 0x0C };
        let data =
            self.transport
                .control_msg(0xC0, req, value, index, ControlData::In(length), None)?;
        if data.len() != length {
            return Err(Error::Usb(format!(
                "control read 0x{value:02x}/0x{index:02x} returned {} bytes",
                data.len()
            )));
        }
        Ok(data)
    }

    pub fn write_register(&mut self, address: u8, value: u8) -> Result<()> {
        self.ctl_out(0x83, 0, &[address, value])
    }

    pub fn write_registers(&mut self, pairs: &[(u8, u8)]) -> Result<()> {
        for chunk in pairs.chunks(32) {
            let d: Vec<u8> = chunk.iter().flat_map(|&(a, v)| [a, v]).collect();
            self.ctl_out(0x83, 0, &d)?;
        }
        Ok(())
    }

    pub fn read_register(&mut self, address: u8) -> Result<u8> {
        self.ctl_out(0x83, 0, &[address])?;
        Ok(self.ctl_in(0x84, 0, 1)?[0])
    }

    pub fn status(&mut self) -> Result<u8> {
        self.read_register(REG_STATUS)
    }

    pub fn write_afe(&mut self, addr: u8, value: u16) -> Result<()> {
        self.ctl_out(
            0x83,
            0,
            &[
                0x51,
                addr,
                0x3A,
                (value >> 8) as u8,
                0x3B,
                (value & 0xFF) as u8,
            ],
        )
    }

    /// Motor table into RAM slot `slot`.
    pub fn upload_table(&mut self, slot: u32, table: &[i64]) -> Result<()> {
        let addr = 0x40000 + 0x8000 * slot;
        let data: Vec<u8> = table
            .iter()
            .flat_map(|&v| [(v & 0xFF) as u8, (v >> 8) as u8])
            .collect();
        self.write_register(0x5B, ((addr >> 12) & 0xFF) as u8)?;
        self.write_register(0x5C, ((addr >> 4) & 0xFF) as u8)?;
        self.ctl_out(0x83, 0, &[0x28])?;
        let n = data.len();
        self.ctl_out(
            0x82,
            0,
            &[
                1,
                0,
                0,
                0,
                (n & 0xFF) as u8,
                ((n >> 8) & 0xFF) as u8,
                ((n >> 16) & 0xFF) as u8,
                0,
            ],
        )?;
        let mut pad = data;
        pad.resize(n + (BULK_PACKET - n % BULK_PACKET) % BULK_PACKET, 0);
        if self.transport.bulk_write(&pad, None)? != pad.len() {
            return Err(Error::Usb("short motor table upload".into()));
        }
        let mut done = false;
        for _ in 0..256 {
            if self.ctl_in(0x8E, 0x18, 1)?[0] & 0x0C == 0 {
                done = true;
                break;
            }
        }
        if !done {
            return Err(Error::Usb("motor table upload did not complete".into()));
        }
        self.write_register(0x5B, (((addr >> 12) & 0xFF) as u8) & !0x40)
    }
}

/// Fast/slow slot-3 tables and the motor registers of the vendor's first fast move.
#[derive(Debug, Clone)]
pub struct RecordedMotion {
    pub fast: Vec<i64>,
    pub slow: Vec<i64>,
    pub regs: HashMap<u8, u8>,
}

pub fn recorded_motion(profile: &Value) -> Result<RecordedMotion> {
    let mut reg: HashMap<u8, u8> = HashMap::new();
    let mut slot: Option<u8> = None;
    let mut tables: Vec<Vec<i64>> = Vec::new();
    let mut ctx: Option<HashMap<u8, u8>> = None;
    for op in profile["ops"].as_array().into_iter().flatten() {
        let kind = op["kind"].as_str().unwrap_or("");
        if kind == "write" && slot == Some(3) {
            let b = base64::engine::general_purpose::STANDARD
                .decode(op["data"].as_str().unwrap_or(""))
                .map_err(|e| Error::Value(format!("bad base64 in replay profile: {e}")))?;
            tables.push(
                b.chunks_exact(2)
                    .map(|p| p[0] as i64 | ((p[1] as i64) << 8))
                    .collect(),
            );
            continue;
        }
        if kind != "control"
            || op["rt"].as_u64() != Some(0x40)
            || op["value"].as_u64() != Some(0x83)
        {
            continue;
        }
        let d: Vec<u8> = op["data"]
            .as_array()
            .map(|a| a.iter().map(|v| v.as_u64().unwrap_or(0) as u8).collect())
            .unwrap_or_default();
        if d.len() < 2 {
            continue;
        }
        for p in d.chunks_exact(2) {
            let (r, v) = (p[0], p[1]);
            if r == 0x5B {
                slot = if v & 0x40 != 0 {
                    Some((v >> 3) & 7)
                } else {
                    None
                };
            }
            if r == 0x0F && v == 1 && ctx.is_none() {
                let g = |a: u8| *reg.get(&a).unwrap_or(&0) as u32;
                let feedl = (g(0x3D) << 16) | (g(0x3E) << 8) | g(0x3F);
                if feedl > 1 && tables.len() >= 3 {
                    ctx = Some(reg.clone());
                }
            }
            reg.insert(r, v);
        }
    }
    let Some(ctx) = ctx.filter(|_| tables.len() >= 2) else {
        return Err(Error::Asic("profile has no recorded fast move".into()));
    };
    // Python `list.sort(key=t[-1])` is stable.
    tables.sort_by_key(|t| *t.last().unwrap_or(&0));
    Ok(RecordedMotion {
        fast: tables[0].clone(),
        slow: tables[tables.len() - 1].clone(),
        regs: ctx,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Table {
    Fast,
    Slow,
}

/// Homing from the vendor's motor primitives: fast reverse to the home sensor,
/// 600 steps forward, then a slow reverse approach stopped when the sensor trips.
pub struct Gl843V1Motion {
    rec: RecordedMotion,
    loaded: Option<Table>,
    base01: u8,
}

const MOTOR_REGS: [u8; 19] = [
    0x1C, 0x1F, 0x21, 0x22, 0x23, 0x24, 0x38, 0x39, 0x5E, 0x5F, 0x67, 0x68, 0x69, 0x6B, 0x6C, 0x6D,
    0x6E, 0x6F, 0x80,
];

fn feedl(n: i64) -> [(u8, u8); 3] {
    [
        (0x3D, ((n >> 16) & 0x0F) as u8),
        (0x3E, ((n >> 8) & 0xFF) as u8),
        (0x3F, (n & 0xFF) as u8),
    ]
}

impl Gl843V1Motion {
    pub fn new(rec: RecordedMotion) -> Self {
        let base01 = rec.regs.get(&0x01).copied().unwrap_or(0x22) & !0x01;
        Self {
            rec,
            loaded: None,
            base01,
        }
    }

    pub fn wait_idle(&self, usb: &mut Gl843V1Usb, timeout_s: f64) -> Result<u8> {
        let t0 = monotonic();
        let mut s = usb.status()?;
        while s & STATUS_MOTORENB != 0 {
            if monotonic() - t0 > timeout_s {
                return Err(Error::MotorTimeout(format!(
                    "motor still running after {timeout_s:.0} s"
                )));
            }
            sleep(0.01);
            s = usb.status()?;
        }
        Ok(s)
    }

    /// Vendor stop: `0x02=0x08` then FEEDL=1 while the move runs.
    pub fn stop(&self, usb: &mut Gl843V1Usb) -> Result<u8> {
        usb.write_registers(&[(0x01, self.base01)])?;
        usb.write_registers(&[(0x02, 0x08)])?;
        usb.write_registers(&feedl(1))?;
        self.wait_idle(usb, 5.0)
    }

    #[allow(clippy::too_many_arguments)]
    fn do_move(
        &mut self,
        usb: &mut Gl843V1Usb,
        steps: i64,
        reverse: bool,
        table: Table,
        fmovno: u8,
        timeout_s: f64,
        until_home: bool,
    ) -> Result<(u8, bool)> {
        self.wait_idle(usb, 5.0)?;
        if self.loaded != Some(table) {
            let t = if table == Table::Fast {
                &self.rec.fast
            } else {
                &self.rec.slow
            };
            usb.upload_table(3, t)?;
            self.loaded = Some(table);
        }
        let regs = &self.rec.regs;
        let mut pairs: Vec<(u8, u8)> = vec![(0x01, self.base01)];
        pairs.extend(
            MOTOR_REGS
                .iter()
                .map(|&r| (r, regs.get(&r).copied().unwrap_or(0))),
        );
        pairs.extend((0x10..0x16).map(|r| (r, 0)));
        pairs.push((0x6A, fmovno));
        pairs.push((0x02, 0x18 | if reverse { MTRREV } else { 0 }));
        pairs.extend(feedl(steps));
        usb.write_registers(&pairs)?;
        usb.write_registers(&[(0x0F, 1)])?;
        let t0 = monotonic();
        loop {
            let s = usb.status()?;
            if until_home && s & STATUS_HOME != 0 {
                self.stop(usb)?;
                return Ok((usb.status()?, true));
            }
            if s & STATUS_MOTORENB == 0 && s & STATUS_FEEDFSH != 0 {
                return Ok((s, false));
            }
            if monotonic() - t0 > timeout_s {
                self.stop(usb)?;
                return Err(Error::MotorTimeout(format!(
                    "move did not finish in {timeout_s:.0} s"
                )));
            }
            sleep(0.002);
        }
    }

    pub fn home(&mut self, usb: &mut Gl843V1Usb, timeout_s: f64) -> Result<u8> {
        let mut s = usb.status()?;
        if s & STATUS_MOTORENB != 0 {
            s = self.stop(usb)?;
        }
        if s & STATUS_HOME == 0 {
            let (_, reached) =
                self.do_move(usb, 80000, true, Table::Fast, 0xFF, timeout_s, true)?;
            if !reached {
                return Err(Error::MotorTimeout("home sensor not found".into()));
            }
        }
        let mut left = false;
        for _ in 0..5 {
            let (st, _) = self.do_move(usb, 600, false, Table::Fast, 0x46, 5.0, false)?;
            if st & STATUS_HOME == 0 {
                left = true;
                break;
            }
        }
        if !left {
            return Err(Error::MotorTimeout(
                "home sensor stays on after moving forward".into(),
            ));
        }
        let (st, reached) = self.do_move(usb, 3000, true, Table::Slow, 0xFF, 10.0, true)?;
        if !reached {
            return Err(Error::MotorTimeout(
                "home sensor not found on the final approach".into(),
            ));
        }
        Ok(st)
    }
}

/// High-level ASIC ops for the 7600i v1: boot, lamp, home/park, status.
pub struct Gl843V1 {
    pub usb: Gl843V1Usb,
    pub model: FilmModel,
    pub initialized: bool,
    pub scan_method: ScanMethod,
    /// Set by homing and by a complete scan (the chip parks the carriage itself).
    pub position_known: bool,
    pub lamp_on_at: Option<f64>,
    motion: Option<Gl843V1Motion>,
}

impl Gl843V1 {
    pub fn new(transport: Box<dyn UsbTransport>, model: FilmModel) -> Self {
        Self {
            usb: Gl843V1Usb::new(transport),
            model,
            initialized: false,
            scan_method: ScanMethod::Transparency,
            position_known: false,
            lamp_on_at: None,
            motion: None,
        }
    }

    pub fn read_status(&mut self) -> Result<ScannerStatus> {
        Ok(ScannerStatus::from_reg41(self.usb.status()?))
    }

    pub fn read_status_reliable(&mut self) -> Result<ScannerStatus> {
        self.read_status()
    }

    pub fn is_at_home(&mut self) -> Result<bool> {
        Ok(self.read_status()?.is_at_home)
    }

    /// Vendor boot: register program, GPIO, AFE.
    pub fn init(&mut self, force: bool) -> Result<()> {
        if self.initialized && !force {
            return Ok(());
        }
        let data = replay_data();
        self.usb.ctl_in(0x8E, 0, 1)?;
        let num = |v: &Value| -> u64 {
            v.as_u64()
                .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
                .unwrap_or(0)
        };
        for group in data["boot"].as_array().into_iter().flatten() {
            let g = group.as_array().expect("boot group");
            if g[0].as_str() == Some("8c") {
                self.usb
                    .ctl_out(0x8C, num(&g[1]) as u16, &[num(&g[2]) as u8])?;
            } else {
                let bytes: Vec<u8> = g.iter().map(|v| num(v) as u8).collect();
                self.usb.ctl_out(0x83, 0, &bytes)?;
            }
        }
        let gpio: Vec<(u8, u8)> = data["gpio"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|p| (num(&p[0]) as u8, num(&p[1]) as u8))
            .collect();
        self.usb.write_registers(&gpio)?;
        sleep(0.1);
        let afe = &data["afe"];
        let mut values: Vec<(u8, u16)> = Vec::new();
        let mut put = |a: u8, v: u16| match values.iter_mut().find(|e| e.0 == a) {
            Some(e) => e.1 = v,
            None => values.push((a, v)),
        };
        for (k, v) in afe["config"].as_object().into_iter().flatten() {
            put(k.parse().expect("afe config key"), num(v) as u16);
        }
        for c in 0..3 {
            put(2 + c as u8, num(&afe["gain"][c]) as u16);
            put(5 + c as u8, num(&afe["offset"][c]) as u16);
        }
        values.sort_by_key(|e| e.0);
        for (a, v) in values {
            self.usb.write_afe(a, v)?;
        }
        self.initialized = true;
        self.position_known = false;
        self.forget_motor_tables();
        Ok(())
    }

    pub fn set_scan_method(&mut self, method: ScanMethod) {
        self.scan_method = method;
    }

    fn reg03(&self) -> u8 {
        self.usb.shadow.get(&0x03).copied().unwrap_or(0x9F)
    }

    pub fn lamp_on(&mut self) -> Result<()> {
        let reg03 = self.reg03();
        if self.scan_method == ScanMethod::Infrared {
            self.usb
                .write_registers(&[(0x03, reg03 & !LAMP_BIT), (0xA8, IR_GPIO_ON)])?;
        } else {
            self.usb
                .write_registers(&[(0xA8, IR_GPIO_OFF), (0x03, reg03 | LAMP_BIT)])?;
        }
        self.lamp_on_at = Some(monotonic());
        Ok(())
    }

    pub fn lamp_off(&mut self) -> Result<()> {
        let reg03 = self.reg03();
        self.usb
            .write_registers(&[(0x03, reg03 & !LAMP_BIT), (0xA8, IR_GPIO_OFF)])?;
        self.lamp_on_at = None;
        Ok(())
    }

    pub fn white_led(&mut self, on: bool) -> Result<()> {
        let reg03 = self.reg03();
        self.usb.write_register(
            0x03,
            if on {
                reg03 | LAMP_BIT
            } else {
                reg03 & !LAMP_BIT
            },
        )?;
        self.lamp_on_at = if on { Some(monotonic()) } else { None };
        Ok(())
    }

    /// Boot and scan jobs overwrite the table slots homing uses.
    pub fn forget_motor_tables(&mut self) {
        if let Some(m) = self.motion.as_mut() {
            m.loaded = None;
        }
    }

    fn motion(&mut self) -> Result<&mut Gl843V1Motion> {
        if self.motion.is_none() {
            let rec = recorded_motion(&replay_profile(self.model.model, "color", 3600)?)?;
            self.motion = Some(Gl843V1Motion::new(rec));
        }
        Ok(self.motion.as_mut().unwrap())
    }

    pub fn stop_motor(&mut self) -> Result<()> {
        self.position_known = false;
        self.motion()?;
        let m = self.motion.as_ref().unwrap();
        m.stop(&mut self.usb)?;
        Ok(())
    }

    pub fn home(&mut self, timeout_s: f64) -> Result<()> {
        self.position_known = false;
        self.motion()?;
        let mut m = self.motion.take().unwrap();
        let r = m.home(&mut self.usb, timeout_s.max(5.0));
        self.motion = Some(m);
        r?;
        self.position_known = true;
        Ok(())
    }

    pub fn park(&mut self, timeout_s: f64) -> Result<()> {
        self.home(timeout_s)
    }
}
