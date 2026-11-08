// SPDX-License-Identifier: GPL-3.0-or-later
//! Fake USB transports for hardwareless scans and protocol tests.

use std::any::Any;
use std::collections::{HashMap, VecDeque};

use super::protocol::{
    REGISTER_LINK_OK, REQUEST_BUFFER, REQUEST_REGISTER, REQUEST_TYPE_IN, REQUEST_TYPE_OUT,
    VALUE_BUF_ENDACCESS, VALUE_BUFFER, VALUE_GET_REGISTER, VALUE_SET_REGISTER,
};
use super::trace::{UsbOp, UsbTransaction};
use super::transport::{ControlData, UsbTransport};
use crate::asic::registers::{gl128 as se, gl845 as r};
use crate::error::{Error, Result};

/// Idle: at home, buffer empty, frontend ready, powered (not replugged).
pub const IDLE_STATUS_REG41: u8 = r::STATUS_BUFEMPTY | r::STATUS_HOMESNR | r::STATUS_PWRBIT;

/// Recover the ASIC address from a GL845 GET_REGISTER control transfer.
pub fn gl845_read_address(value: u16, index: u16) -> u16 {
    let mut addr = (index >> 8) & 0xFF;
    if value & 0x100 != 0 {
        addr |= 0x100;
    }
    addr
}

/// State shared by every fake: register file, transaction log, queues.
#[derive(Debug, Default)]
pub struct FakeCore {
    pub registers: HashMap<u16, u8>,
    pub transactions: Vec<UsbTransaction>,
    pub bulk_in_queue: VecDeque<Vec<u8>>,
    pub request_register: HashMap<u16, u8>,
    pub errors: VecDeque<Error>,
    pub aborted: usize,
    pub abort_drain_bytes: usize,
}

impl FakeCore {
    pub fn new() -> Self {
        let mut c = Self {
            abort_drain_bytes: 42,
            ..Default::default()
        };
        c.registers.entry(r::REG_0X41).or_insert(IDLE_STATUS_REG41);
        c.registers
            .entry(se::REG_STATUS)
            .or_insert(IDLE_STATUS_REG41);
        c
    }

    pub fn reg(&self, addr: u16) -> u8 {
        self.registers.get(&addr).copied().unwrap_or(0)
    }

    /// Legacy 5-tuples: `(type, request, value, index, data_or_length)`.
    pub fn control_log(&self) -> Vec<(u8, u8, u16, u16, ControlLogData)> {
        self.transactions
            .iter()
            .filter_map(|t| {
                let d = match t.operation {
                    UsbOp::ControlRead => ControlLogData::Len(t.length.unwrap_or(0)),
                    UsbOp::ControlWrite => ControlLogData::Data(t.data.clone().unwrap_or_default()),
                    _ => return None,
                };
                Some((
                    t.request_type.unwrap_or(0),
                    t.request.unwrap_or(0),
                    t.value.unwrap_or(0),
                    t.index.unwrap_or(0),
                    d,
                ))
            })
            .collect()
    }

    pub fn bulk_out_log(&self) -> Vec<Vec<u8>> {
        self.transactions
            .iter()
            .filter(|t| t.operation == UsbOp::BulkWrite)
            .map(|t| t.data.clone().unwrap_or_default())
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ControlLogData {
    Len(usize),
    Data(Vec<u8>),
}

/// Device behaviour behind a [`FakeUsbTransport`] (the Python subclass hooks).
pub trait FakeDevice: Send + 'static {
    fn before_register_read(&mut self, core: &mut FakeCore, addr: u16) {
        let _ = (core, addr);
    }
    fn before_probe_read(&mut self, core: &mut FakeCore, index: u16) {
        let _ = (core, index);
    }
    fn after_register_write(&mut self, core: &mut FakeCore, addr: u16, value: u8) {
        let _ = (core, addr, value);
    }
    fn bulk_in_bytes(&mut self, core: &mut FakeCore, size: usize) -> Vec<u8> {
        let _ = core;
        vec![0; size]
    }
    /// A `VALUE_BUFFER` control write (bulk preamble or AHB upload header).
    fn after_buffer_write(&mut self, core: &mut FakeCore, index: u16, data: &[u8]) {
        let _ = (core, index, data);
    }
}

/// Plain register file: no device behaviour.
#[derive(Debug, Default)]
pub struct NoDevice;
impl FakeDevice for NoDevice {}

/// Scripted USB device that records every transfer in order.
///
/// Register reads use the GL845 vendor-request framing. Status register
/// `0x41` defaults to an idle scanner so ASIC poll loops exit on first read.
pub struct FakeUsbTransport<D: FakeDevice = NoDevice> {
    pub core: FakeCore,
    pub dev: D,
}

impl FakeUsbTransport<NoDevice> {
    pub fn new() -> Self {
        Self {
            core: FakeCore::new(),
            dev: NoDevice,
        }
    }
}

impl Default for FakeUsbTransport<NoDevice> {
    fn default() -> Self {
        Self::new()
    }
}

impl<D: FakeDevice> FakeUsbTransport<D> {
    pub fn with_device(core: FakeCore, dev: D) -> Self {
        Self { core, dev }
    }

    /// Raise `err` on the next control or bulk transfer.
    pub fn queue_error(&mut self, err: Error) {
        self.core.errors.push_back(err);
    }

    fn raise_queued_error(&mut self) -> Result<()> {
        match self.core.errors.pop_front() {
            Some(e) => Err(e),
            None => Ok(()),
        }
    }

    fn handle_control(&mut self, txn: &UsbTransaction) -> Result<Vec<u8>> {
        let rtype = txn.request_type.unwrap_or(0);
        let request = txn.request.unwrap_or(0);
        let value = txn.value.unwrap_or(0);
        let index = txn.index.unwrap_or(0);

        if rtype == REQUEST_TYPE_IN && request == REQUEST_BUFFER {
            if value & 0xFF != VALUE_GET_REGISTER {
                return Err(Error::Usb(format!(
                    "unexpected GET_REGISTER value=0x{value:04x} index=0x{index:04x}"
                )));
            }
            let addr = gl845_read_address(value, index);
            self.dev.before_register_read(&mut self.core, addr);
            return Ok(vec![self.core.reg(addr), REGISTER_LINK_OK]);
        }
        if rtype == REQUEST_TYPE_IN && request == REQUEST_REGISTER {
            self.dev.before_probe_read(&mut self.core, index);
            return Ok(vec![
                self.core.request_register.get(&index).copied().unwrap_or(0),
            ]);
        }
        if rtype == REQUEST_TYPE_OUT && request == REQUEST_BUFFER {
            if value & 0xFF == VALUE_SET_REGISTER {
                let payload = txn.data.as_deref().unwrap_or(&[]);
                let high = if value & 0x100 != 0 { 0x100 } else { 0 };
                if payload.len() < 2 || !payload.len().is_multiple_of(2) {
                    return Err(Error::Usb(format!(
                        "SET_REGISTER payload length {}",
                        payload.len()
                    )));
                }
                for p in payload.chunks_exact(2) {
                    let addr = high | p[0] as u16;
                    self.core.registers.insert(addr, p[1]);
                    self.dev.after_register_write(&mut self.core, addr, p[1]);
                }
                return Ok(Vec::new());
            }
            if value == VALUE_BUFFER {
                self.dev.after_buffer_write(
                    &mut self.core,
                    index,
                    txn.data.as_deref().unwrap_or(&[]),
                );
                return Ok(Vec::new());
            }
            return Err(Error::Usb(format!(
                "unexpected BUFFER out value=0x{value:04x}"
            )));
        }
        if rtype == REQUEST_TYPE_OUT && request == REQUEST_REGISTER && value == VALUE_BUF_ENDACCESS
        {
            return Ok(Vec::new());
        }
        Err(Error::Usb(format!(
            "unexpected control_msg type=0x{rtype:02x} req=0x{request:02x} val=0x{value:04x} idx=0x{index:04x}"
        )))
    }
}

impl<D: FakeDevice> UsbTransport for FakeUsbTransport<D> {
    fn control_msg(
        &mut self,
        rt: u8,
        req: u8,
        value: u16,
        index: u16,
        data: ControlData<'_>,
        timeout_ms: Option<u32>,
    ) -> Result<Vec<u8>> {
        self.raise_queued_error()?;
        let txn = UsbTransaction::control(rt, req, value, index, data, timeout_ms);
        let r = self.handle_control(&txn);
        self.core.transactions.push(txn);
        r
    }

    fn bulk_read(&mut self, size: usize, timeout_ms: Option<u32>) -> Result<Vec<u8>> {
        self.raise_queued_error()?;
        self.core
            .transactions
            .push(UsbTransaction::bulk_read(size, timeout_ms));
        if let Some(mut chunk) = self.core.bulk_in_queue.pop_front() {
            chunk.truncate(size);
            return Ok(chunk);
        }
        Ok(self.dev.bulk_in_bytes(&mut self.core, size))
    }

    fn bulk_write(&mut self, data: &[u8], timeout_ms: Option<u32>) -> Result<usize> {
        self.raise_queued_error()?;
        self.core
            .transactions
            .push(UsbTransaction::bulk_write(data, timeout_ms));
        Ok(data.len())
    }

    fn abort_bulk_in(&mut self) -> Option<usize> {
        self.core.aborted += 1;
        Some(self.core.abort_drain_bytes)
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Fake scanner that can complete a full scan session.
///
/// Status / feed / valid-word registers are updated so GL845 and GL128 poll
/// loops exit. `bulk_read` returns a deterministic XY/channel pattern.
#[derive(Debug)]
pub struct MockScanner {
    pub at_home: bool,
    pub buffer_empty: bool,
    pub motor_enabled: bool,
    pub feed_finished: bool,
    pub motion_reads: u32,
    pub bulk_offset: usize,
}

pub type MockScannerTransport = FakeUsbTransport<MockScanner>;

impl MockScannerTransport {
    pub fn new() -> Self {
        Self::from_core(FakeCore::new())
    }

    /// Build on a core with pre-set registers (Python field overrides).
    pub fn from_core(mut core: FakeCore) -> Self {
        let mut dev = MockScanner {
            at_home: true,
            buffer_empty: true,
            motor_enabled: false,
            feed_finished: false,
            motion_reads: 0,
            bulk_offset: 0,
        };
        dev.sync_status(&mut core);
        for (a, v) in [
            (0x48, 0x1F),
            (0x49, 0xFF),
            (0x4A, 0xFF),
            (0x42, 0x02),
            (0x43, 0x00),
            (0x44, 0x00),
            (0x45, 0x01),
            (0x06, r::PWRBIT),
        ] {
            core.registers.entry(a).or_insert(v);
        }
        Self { core, dev }
    }
}

impl Default for MockScannerTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl MockScanner {
    fn status_byte(&self, core: &FakeCore) -> u8 {
        let mut v = r::STATUS_PWRBIT;
        if core.reg(0x03) & r::LAMPPWR != 0 {
            v |= r::STATUS_LAMPSTS;
        }
        if self.buffer_empty {
            v |= r::STATUS_BUFEMPTY;
        }
        if self.at_home {
            v |= r::STATUS_HOMESNR;
        }
        if self.feed_finished {
            v |= r::STATUS_FEEDFSH;
        }
        if self.motor_enabled {
            v |= r::STATUS_MOTORENB;
        }
        v
    }

    pub fn sync_status(&mut self, core: &mut FakeCore) {
        let raw = self.status_byte(core);
        core.registers.insert(r::REG_0X41, raw);
        core.registers.insert(se::REG_STATUS, raw);
        let mut motor_flags = if !self.buffer_empty { r::DATAENB } else { 0 };
        if self.motor_enabled {
            motor_flags |= r::MOTMFLG;
        }
        core.registers.insert(0x40, motor_flags);
    }

    fn tick_motion(&mut self, core: &mut FakeCore) {
        if !self.motor_enabled {
            return;
        }
        self.motion_reads += 1;
        // First status sample shows motor on (GL128 require_motion); then finish.
        if self.motion_reads >= 4 {
            self.motor_enabled = false;
            self.feed_finished = true;
            if core.reg(0x02) & r::AGOHOME != 0 {
                self.at_home = true;
            }
            core.request_register.insert(0x21, 0x04);
            self.sync_status(core);
        }
    }

    fn line_width(core: &FakeCore) -> usize {
        let g = |a: u16| core.reg(a) as usize;
        let str_px = (g(0x82) << 16) | (g(0x83) << 8) | g(0x84);
        let end_px = (g(0x85) << 16) | (g(0x86) << 8) | g(0x87);
        if end_px > str_px {
            return (end_px - str_px).max(1);
        }
        let start = (g(0x30) << 8) | g(0x31);
        let end = (g(0x32) << 8) | g(0x33);
        if end > start {
            return (end - start).max(1);
        }
        256
    }
}

impl FakeDevice for MockScanner {
    fn before_register_read(&mut self, core: &mut FakeCore, addr: u16) {
        if addr == r::REG_0X41 || addr == se::REG_STATUS {
            self.tick_motion(core);
            self.sync_status(core);
        }
    }

    fn before_probe_read(&mut self, core: &mut FakeCore, index: u16) {
        if index == 0x21 {
            self.tick_motion(core);
        }
    }

    fn after_register_write(&mut self, core: &mut FakeCore, addr: u16, value: u8) {
        if addr == 0x03 {
            self.sync_status(core);
            return;
        }
        if addr == 0x01 {
            let scanning = value & r::SCAN != 0;
            self.buffer_empty = !scanning;
            if !scanning && core.reg(0x02) & r::AGOHOME != 0 {
                self.at_home = true;
                self.motor_enabled = false;
            }
            self.sync_status(core);
            return;
        }
        if addr == 0x0F && value == 0x01 {
            self.at_home = false;
            self.motor_enabled = true;
            self.feed_finished = false;
            self.motion_reads = 0;
            core.request_register.insert(0x21, 0x00);
            self.sync_status(core);
            return;
        }
        if addr == 0x02 && value & r::MTRPWR == 0 {
            self.motor_enabled = false;
            self.sync_status(core);
        }
    }

    /// Chunky RGB16 LE: R=x, G=y, B=x XOR y. Values scale with LPERIOD (0x38)
    /// relative to 14000, and drop to near-black with no light source on.
    fn bulk_in_bytes(&mut self, core: &mut FakeCore, size: usize) -> Vec<u8> {
        let width = Self::line_width(core) as u64;
        let lperiod = ((core.reg(0x38) as u64) << 8) | core.reg(0x39) as u64;
        let (gain_num, mut gain_den) = if lperiod != 0 {
            (lperiod, 14000u64)
        } else {
            (1, 1)
        };
        let lit = core.reg(0x03) & r::LAMPPWR != 0 || core.reg(0xA8) & r::IR_LAMP_A8_MASK != 0;
        if !lit {
            gain_den *= 64;
        }
        let mut out = vec![0u8; size];
        let off = self.bulk_offset as u64;
        let mut i = 0usize;
        while i + 1 < size {
            let sample = (off + i as u64) / 2;
            let pixel = sample / 3;
            let channel = sample % 3;
            let x = pixel % width;
            let y = pixel / width;
            let val = match channel {
                0 => x & 0xFFFF,
                1 => y & 0xFFFF,
                _ => (x ^ y) & 0xFFFF,
            };
            let val = (val * gain_num / gain_den).min(0xFFFF);
            out[i] = (val & 0xFF) as u8;
            out[i + 1] = (val >> 8) as u8;
            i += 2;
        }
        self.bulk_offset += size;
        out
    }
}
