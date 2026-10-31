// SPDX-License-Identifier: GPL-3.0-or-later
//! USB fakes for the OpticFilm 7600i v1: strict capture playback, and a simulated GL843.

use std::any::Any;
use std::collections::HashMap;

use base64::Engine;
use serde_json::Value;

use super::trace::{UsbOp, UsbTransaction};
use super::transport::{ControlData, UsbTransport};
use crate::error::{Error, Result};

const STATUS: u8 = 0x41;
const PWRBIT: u8 = 0x80;
const BUFEMPTY: u8 = 0x40;
const FEEDFSH: u8 = 0x20;
const HOME: u8 = 0x08;
const MOTORENB: u8 = 0x01;

fn out_txn(rt: u8, req: u8, value: u16, index: u16, data: &[u8]) -> UsbTransaction {
    UsbTransaction::control(rt, req, value, index, ControlData::Out(data), None)
}

fn in_txn(rt: u8, req: u8, value: u16, index: u16, reply: &[u8]) -> UsbTransaction {
    let mut t = UsbTransaction::control(rt, req, value, index, ControlData::In(reply.len()), None);
    t.response = Some(reply.to_vec());
    t
}

/// A playback mismatch (Python `PlaybackMismatch(AssertionError)`), reported as `Error::Usb`.
fn mismatch(msg: String) -> Error {
    Error::Usb(format!("PlaybackMismatch: {msg}"))
}

fn u(v: &Value) -> i64 {
    v.as_i64().unwrap_or(0)
}

/// Every transfer must match the next vendor op; reads return what the scanner returned.
pub struct CapturePlaybackTransport {
    pub ops: Vec<Value>,
    pub pos: usize,
    pub read_left: Option<usize>,
    pub transactions: Vec<UsbTransaction>,
}

impl CapturePlaybackTransport {
    pub fn new(ops: &[Value]) -> Self {
        Self {
            ops: ops
                .iter()
                .filter(|o| o["kind"] != "delay")
                .cloned()
                .collect(),
            pos: 0,
            read_left: None,
            transactions: Vec::new(),
        }
    }

    fn next(&self, what: &str) -> Result<&Value> {
        self.ops
            .get(self.pos)
            .ok_or_else(|| mismatch(format!("{what} after the end of the vendor sequence")))
    }

    pub fn finished(&self) -> bool {
        self.pos == self.ops.len() && self.read_left.is_none()
    }
}

impl UsbTransport for CapturePlaybackTransport {
    fn control_msg(
        &mut self,
        rt: u8,
        req: u8,
        value: u16,
        index: u16,
        data: ControlData<'_>,
        _t: Option<u32>,
    ) -> Result<Vec<u8>> {
        let op = self.next(&format!("control 0x{value:02x}"))?.clone();
        let where_ = format!("vendor op {}", self.pos);
        if op["kind"] != "control" {
            return Err(mismatch(format!(
                "{where_}: expected {}, got control 0x{value:02x}",
                op["kind"].as_str().unwrap_or("?")
            )));
        }
        let want = (
            u(&op["rt"]),
            u(&op["request"]),
            u(&op["value"]),
            u(&op["index"]),
        );
        if want != (rt as i64, req as i64, value as i64, index as i64) {
            return Err(mismatch(format!(
                "{where_}: expected rt=0x{:02x} req=0x{:02x} value=0x{:02x} index=0x{:02x}, got rt=0x{rt:02x} \
                 req=0x{req:02x} value=0x{value:02x} index=0x{index:02x}",
                want.0, want.1, want.2, want.3
            )));
        }
        self.pos += 1;
        match data {
            ControlData::Out(d) => {
                let vendor: Vec<i64> = op["data"]
                    .as_array()
                    .map(|a| a.iter().map(u).collect())
                    .unwrap_or_default();
                let got: Vec<i64> = d.iter().map(|&b| b as i64).collect();
                if got != vendor {
                    return Err(mismatch(format!(
                        "{where_}: payload {:?} != vendor {:?}",
                        &got[..got.len().min(16)],
                        &vendor[..vendor.len().min(16)]
                    )));
                }
                self.transactions.push(out_txn(rt, req, value, index, d));
                Ok(Vec::new())
            }
            ControlData::In(n) => {
                if n as i64 != u(&op["length"]) {
                    return Err(mismatch(format!(
                        "{where_}: read length {n} != vendor {}",
                        u(&op["length"])
                    )));
                }
                let reply: Vec<u8> = op["expected"]
                    .as_array()
                    .map(|a| a.iter().map(|v| u(v) as u8).collect())
                    .unwrap_or_default();
                self.transactions
                    .push(in_txn(rt, req, value, index, &reply));
                Ok(reply)
            }
        }
    }

    fn bulk_write(&mut self, data: &[u8], _t: Option<u32>) -> Result<usize> {
        let op = self.next("bulk write")?;
        let ok = op["kind"] == "write"
            && base64::engine::general_purpose::STANDARD
                .decode(op["data"].as_str().unwrap_or(""))
                .is_ok_and(|d| d == data);
        if !ok {
            return Err(mismatch(format!(
                "vendor op {}: unexpected bulk write of {} bytes",
                self.pos,
                data.len()
            )));
        }
        self.pos += 1;
        let mut t = UsbTransaction::new(UsbOp::BulkWrite);
        t.data = Some(data.to_vec());
        t.length = Some(data.len());
        self.transactions.push(t);
        Ok(data.len())
    }

    fn bulk_read(&mut self, size: usize, _t: Option<u32>) -> Result<Vec<u8>> {
        let op = self.next("bulk read")?;
        if op["kind"] != "read" {
            return Err(mismatch(format!(
                "vendor op {}: expected {}, got bulk read",
                self.pos,
                op["kind"].as_str().unwrap_or("?")
            )));
        }
        let len = u(&op["length"]) as usize;
        let left = self.read_left.get_or_insert(len);
        let n = size.min(*left);
        *left -= n;
        if *left == 0 {
            self.pos += 1;
            self.read_left = None;
        }
        let mut t = UsbTransaction::new(UsbOp::BulkRead);
        t.length = Some(n);
        self.transactions.push(t);
        Ok(vec![0; n])
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Behavioural GL843 v1: enough for boot, homing and replayed scans without hardware.
pub struct SimulatedGl843V1Transport {
    /// Carriage position in motor steps from the home sensor (sensor on at <= 0).
    pub position: i64,
    /// Motor steps per status read while moving.
    pub steps_per_poll: i64,
    /// RGB16 sample values returned with a light source on and off.
    pub lit_rgb: [i64; 3],
    pub dark_rgb: [i64; 3],
    pub regs: HashMap<u8, u8>,
    pub afe: HashMap<u8, i64>,
    pub transactions: Vec<UsbTransaction>,
    pub address: u8,
    pub moving: bool,
    pub reverse: bool,
    pub steps_left: i64,
    pub feed_finished: bool,
    pub scan_move: bool,
    pub read_phase: usize,
    pub aborts: usize,
    pub is_open: bool,
}

impl Default for SimulatedGl843V1Transport {
    fn default() -> Self {
        Self {
            position: 6000,
            steps_per_poll: 2500,
            lit_rgb: [30000, 40000, 34000],
            dark_rgb: [1000, 1070, 1230],
            regs: HashMap::new(),
            afe: HashMap::new(),
            transactions: Vec::new(),
            address: 0,
            moving: false,
            reverse: false,
            steps_left: 0,
            feed_finished: false,
            scan_move: false,
            read_phase: 0,
            aborts: 0,
            is_open: true,
        }
    }
}

impl SimulatedGl843V1Transport {
    fn reg(&self, r: u8) -> i64 {
        self.regs.get(&r).copied().unwrap_or(0) as i64
    }

    fn status(&self) -> u8 {
        let mut s = PWRBIT;
        if self.reg(0x01) & 1 == 0 {
            s |= BUFEMPTY;
        }
        if self.feed_finished {
            s |= FEEDFSH;
        }
        if self.position <= 0 {
            s |= HOME;
        }
        if self.moving {
            s |= MOTORENB;
        }
        s
    }

    fn tick(&mut self) {
        if !self.moving {
            return;
        }
        let step = self.steps_per_poll.min(self.steps_left);
        self.position += if self.reverse { -step } else { step };
        self.position = self.position.max(-200); // mechanical end stop
        self.steps_left -= step;
        if self.steps_left <= 0 {
            self.moving = false;
            self.feed_finished = true;
            if self.scan_move && self.reg(0x02) & 0x20 != 0 && self.position > 0 {
                // AGOHOME: after a scan the GL843 drives the carriage back to the home sensor
                self.scan_move = false;
                self.moving = true;
                self.reverse = true;
                self.steps_left = self.position;
            }
        }
    }

    fn write(&mut self, reg: u8, value: u8) {
        self.regs.insert(reg, value);
        if reg == 0x3B {
            let idx = self.reg(0x51) as u8;
            self.afe.insert(idx, self.reg(0x3A) * 256 + value as i64);
        }
        let feedl = (self.reg(0x3D) << 16) | (self.reg(0x3E) << 8) | self.reg(0x3F);
        if reg == 0x0F && value == 1 {
            self.moving = true;
            self.feed_finished = false;
            self.reverse = self.reg(0x02) & 0x04 != 0;
            self.steps_left = feedl.max(1);
            self.scan_move = self.reg(0x01) & 1 != 0;
        } else if matches!(reg, 0x3D..=0x3F) && self.moving && feedl == 1 {
            // The vendor's stop: 0x02=0x08 then FEEDL=1 while moving.
            self.moving = false;
            self.feed_finished = true;
        }
    }
}

impl UsbTransport for SimulatedGl843V1Transport {
    fn control_msg(
        &mut self,
        rt: u8,
        req: u8,
        value: u16,
        index: u16,
        data: ControlData<'_>,
        _t: Option<u32>,
    ) -> Result<Vec<u8>> {
        if rt == 0x40 {
            let payload = match data {
                ControlData::Out(d) => d,
                ControlData::In(_) => &[][..],
            };
            self.transactions
                .push(out_txn(rt, req, value, index, payload));
            if value == 0x83 {
                if payload.len() == 1 {
                    self.address = payload[0];
                } else {
                    for p in payload.chunks_exact(2) {
                        self.write(p[0], p[1]);
                    }
                }
            }
            return Ok(Vec::new());
        }
        let n = match data {
            ControlData::In(n) => n,
            ControlData::Out(d) => d.len(),
        };
        let reply = if value == 0x8E {
            vec![if index == 0x20 { 1 } else { 0 }; n]
        } else if value == 0x84 {
            let r = if self.address == STATUS {
                self.tick();
                vec![self.status()]
            } else {
                vec![self.reg(self.address) as u8]
            };
            self.address = self.address.wrapping_add(1);
            r
        } else {
            vec![0; n]
        };
        self.transactions
            .push(in_txn(rt, req, value, index, &reply));
        Ok(reply)
    }

    fn bulk_write(&mut self, data: &[u8], _t: Option<u32>) -> Result<usize> {
        let mut t = UsbTransaction::new(UsbOp::BulkWrite);
        t.data = Some(data.to_vec());
        t.length = Some(data.len());
        self.transactions.push(t);
        Ok(data.len())
    }

    fn bulk_read(&mut self, size: usize, _t: Option<u32>) -> Result<Vec<u8>> {
        let mut t = UsbTransaction::new(UsbOp::BulkRead);
        t.length = Some(size);
        self.transactions.push(t);
        let lit = self.reg(0x03) & 0x10 != 0 || self.regs.get(&0xA8) == Some(&0x27);
        let [mut r, mut g, mut b] = if lit { self.lit_rgb } else { self.dark_rgb };
        let start_pixel = self.reg(0x30) * 256 + self.reg(0x31);
        let end_pixel = self.reg(0x32) * 256 + self.reg(0x33);
        let dpi = self.reg(0x2C) * 256 + self.reg(0x2D);
        if dpi != 0 && crate::num::floor_div((end_pixel - start_pixel) * dpi, 1200) == 512 {
            let mut values = [0i64; 3];
            for (c, v) in values.iter_mut().enumerate() {
                let mut offset = self.afe.get(&(5 + c as u8)).copied().unwrap_or(128);
                offset = if offset & 256 != 0 {
                    -(offset & 255)
                } else {
                    offset
                };
                let gain = 6.0
                    / (6.0
                        - 5.0 * self.afe.get(&(2 + c as u8)).copied().unwrap_or(0) as f64 / 63.0);
                *v = (((offset + 70) as f64 * 20.0 * gain) as i64).clamp(0, 65535);
            }
            [r, g, b] = values;
        }
        let pixel = [
            (r & 0xFF) as u8,
            (r >> 8) as u8,
            (g & 0xFF) as u8,
            (g >> 8) as u8,
            (b & 0xFF) as u8,
            (b >> 8) as u8,
        ];
        let start = self.read_phase % 6;
        self.read_phase += size;
        Ok((0..size).map(|i| pixel[(start + i) % 6]).collect())
    }

    fn abort_bulk_in(&mut self) -> Option<usize> {
        self.aborts += 1;
        Some(0)
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
