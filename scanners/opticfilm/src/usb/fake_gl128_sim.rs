// SPDX-License-Identifier: GPL-3.0-or-later
//! GL128 mock with a simulated AD9826 analog front end, for calibrated runs.
//!
//! Port of `tools/gl128_sim.py` (the parity fixtures' reference). The plain
//! [`MockScanner`] moves the carriage on every START and returns an XY pattern
//! for every read, so GL128 calibration never completes on it. [`Gl128Sim`]
//! behaves like the hardware where calibration looks:
//!
//! * START with motor power off (`0x02` without MTRPWR) is a stationary strip:
//!   the carriage stays at home and data is ready at once.
//! * Strip reads (bulk preamble index `0x00`, scanner RAM) return a modelled
//!   sensor line through the AD9826: `G(code) * (signal + black) + 24 * offset`
//!   with `G = 378 / (378 - 5 * code)` and 9-bit sign-magnitude offsets,
//!   clamped to 16 bits.
//! * The light is the 8200i SE's measured gain-0 wide-strip means with the
//!   lamp on, the IR LED's under `0x37` bit 2, nothing otherwise, with a mild
//!   falloff across the sensor; native pixels below 124 are shielded.
//!
//! Image reads (index `0x08`) keep the plain mock's pattern; DVDSET shading is
//! not applied. All arithmetic is integer, as in the Python reference.

use std::collections::HashMap;

use super::fake::{FakeCore, FakeDevice, FakeUsbTransport, MockScanner, MockScannerTransport};

const LAMP_SIGNAL: [i64; 3] = [26773, 36702, 32143];
const IR_SIGNAL: [i64; 3] = [48151, 41677, 44226];
const BLACK: [i64; 3] = [600, 640, 560];
const OFFSET_STEP: i64 = 24;
const SHIELDED_BELOW: i64 = 124;
const FALLOFF_CENTRE: i64 = 5300;
const FALLOFF_PERMILLE: i64 = 80;

/// Simulated GL128 scanner: [`MockScanner`] plus the AD9826 strip model.
#[derive(Debug)]
pub struct Gl128Sim {
    pub inner: MockScanner,
    /// AFE registers by index, as written through `0x51` / `0x5D` / `0x5E`.
    pub afe: HashMap<u8, u16>,
    strip: bool,
    strip_pixels: i64,
    strip_offset: usize,
}

pub type Gl128SimTransport = FakeUsbTransport<Gl128Sim>;

impl Gl128Sim {
    /// A simulated scanner at home, idle, lamp off.
    pub fn transport() -> Gl128SimTransport {
        let MockScannerTransport { core, dev } = MockScannerTransport::new();
        FakeUsbTransport::with_device(core, Gl128Sim { inner: dev, afe: HashMap::new(), strip: false, strip_pixels: 1, strip_offset: 0 })
    }

    fn u24(core: &FakeCore, addr: u16) -> i64 {
        ((core.reg(addr) as i64) << 16) | ((core.reg(addr + 1) as i64) << 8) | core.reg(addr + 2) as i64
    }

    fn gain_offset(&self, ch: usize) -> (i64, i64) {
        let code = (self.afe.get(&(0x02 + ch as u8)).copied().unwrap_or(0) & 0x3F) as i64;
        let raw = self.afe.get(&(0x05 + ch as u8)).copied().unwrap_or(0);
        let off = if raw & 0x100 != 0 { -((raw & 0xFF) as i64) } else { (raw & 0xFF) as i64 };
        (code, off)
    }

    fn signal(core: &FakeCore, ch: usize, x: i64) -> i64 {
        if x < SHIELDED_BELOW {
            return 0;
        }
        let s = if core.reg(0x03) & 0x10 != 0 {
            LAMP_SIGNAL[ch]
        } else if core.reg(0x37) & 0x04 != 0 {
            IR_SIGNAL[ch]
        } else {
            return 0;
        };
        let d = x - FALLOFF_CENTRE;
        let f = 1000 - (d * d * FALLOFF_PERMILLE) / (FALLOFF_CENTRE * FALLOFF_CENTRE);
        let ripple = (x * 37 + ch as i64 * 11) % 9 - 4;
        (s * f.max(0) / 1000 + ripple).max(0)
    }
}

impl FakeDevice for Gl128Sim {
    fn before_register_read(&mut self, core: &mut FakeCore, addr: u16) {
        self.inner.before_register_read(core, addr);
    }

    fn before_probe_read(&mut self, core: &mut FakeCore, index: u16) {
        self.inner.before_probe_read(core, index);
    }

    fn after_register_write(&mut self, core: &mut FakeCore, addr: u16, value: u8) {
        if addr == 0x5E {
            self.afe.insert(core.reg(0x51), ((core.reg(0x5D) as u16) << 8) | value as u16);
            return;
        }
        if addr == 0x0F && value == 0x01 && core.reg(0x02) & 0x10 == 0 {
            // Stationary strip: no carriage move, data at once.
            self.inner.buffer_empty = false;
            self.inner.sync_status(core);
            return;
        }
        self.inner.after_register_write(core, addr, value);
    }

    fn after_buffer_write(&mut self, core: &mut FakeCore, index: u16, data: &[u8]) {
        if data.len() == 8 && (index == 0x00 || index == 0x08) {
            let size = u32::from_le_bytes([data[4], data[5], data[6], data[7]]) as i64;
            let lines = Self::u24(core, 0x25).max(1);
            self.strip = index == 0x00;
            self.strip_pixels = (size / (lines * 6)).max(1);
            self.strip_offset = 0;
        }
    }

    fn bulk_in_bytes(&mut self, core: &mut FakeCore, size: usize) -> Vec<u8> {
        if !self.strip {
            return self.inner.bulk_in_bytes(core, size);
        }
        let start = Self::u24(core, 0x82);
        let span = (Self::u24(core, 0x85) - start).max(1);
        let n = self.strip_pixels;
        let params = [0, 1, 2].map(|ch| self.gain_offset(ch));
        let mut out = vec![0u8; size];
        let mut i = 0;
        while i + 1 < size {
            let sample = ((self.strip_offset + i) / 2) as i64;
            let ch = (sample % 3) as usize;
            let col = (sample / 3) % n;
            let x = start + col * span / n;
            let (code, off) = params[ch];
            let v = ((Self::signal(core, ch, x) + BLACK[ch]) * 378 / (378 - 5 * code) + OFFSET_STEP * off).clamp(0, 0xFFFF);
            out[i] = (v & 0xFF) as u8;
            out[i + 1] = (v >> 8) as u8;
            i += 2;
        }
        self.strip_offset += size;
        out
    }
}
