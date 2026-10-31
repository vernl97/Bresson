# SPDX-License-Identifier: GPL-3.0-or-later
"""GL128 mock with a simulated AD9826 analog front end, for calibrated parity runs.

``MockScannerTransport`` moves the carriage on every START and returns an XY
pattern for every read, so GL128 calibration (stationary AFE and shading
strips) never completes on it. This subclass behaves like the hardware where
calibration looks:

* START with motor power off (``0x02`` without MTRPWR) is a stationary strip:
  the carriage stays at home and data is ready at once.
* Strip reads (bulk preamble index ``0x00``, scanner RAM) return a modelled
  sensor line through the AD9826: ``G(code) * (signal + black) + 24 * offset``
  with ``G = 378 / (378 - 5 * code)`` (pyopticfilm's gain law ``6 / (6 - 5c/63)``)
  and 9-bit sign-magnitude offsets, clamped to 16 bits.
* The light is the 8200i SE's measured gain-0 wide-strip means with the lamp
  on, the IR LED's under ``0x37`` bit 2, nothing otherwise, with a mild falloff
  across the sensor; native pixels below 124 are shielded optical black.

Image reads (index ``0x08``) keep the parent's pattern; DVDSET shading is not
applied. All arithmetic is integer so the Rust port (``fake_gl128_sim.rs``)
reproduces it bit for bit.
"""

from __future__ import annotations

from dataclasses import dataclass, field

from pyopticfilm.usb.fake import MockScannerTransport
from pyopticfilm.usb.protocol import REQUEST_BUFFER, REQUEST_TYPE_OUT, VALUE_BUFFER

LAMP_SIGNAL = (26773, 36702, 32143)
IR_SIGNAL = (48151, 41677, 44226)
BLACK = (600, 640, 560)
OFFSET_STEP = 24
SHIELDED_BELOW = 124
FALLOFF_CENTRE = 5300
FALLOFF_PERMILLE = 80


def _u24(regs: dict[int, int], addr: int) -> int:
    return (regs.get(addr, 0) << 16) | (regs.get(addr + 1, 0) << 8) | regs.get(addr + 2, 0)


@dataclass
class Gl128SimTransport(MockScannerTransport):
    afe: dict[int, int] = field(default_factory=dict)
    _strip: bool = False
    _strip_pixels: int = 0
    _strip_offset: int = 0

    def _after_register_write(self, addr: int, value: int) -> None:
        value &= 0xFF
        if addr == 0x5E:
            index = self.registers.get(0x51, 0)
            self.afe[index] = (self.registers.get(0x5D, 0) << 8) | value
            return
        if addr == 0x0F and value == 0x01 and not self.registers.get(0x02, 0) & 0x10:
            # Stationary strip: no carriage move, data at once.
            self.buffer_empty = False
            self._sync_status()
            return
        super()._after_register_write(addr, value)

    def _handle_control(self, txn):
        out = super()._handle_control(txn)
        if (txn.request_type, txn.request, txn.value) == (REQUEST_TYPE_OUT, REQUEST_BUFFER, VALUE_BUFFER):
            data = txn.data or b""
            if len(data) == 8 and txn.index in (0x00, 0x08):
                size = int.from_bytes(data[4:8], "little")
                lines = max(1, _u24(self.registers, 0x25))
                self._strip = txn.index == 0x00
                self._strip_pixels = max(1, size // (lines * 6))
                self._strip_offset = 0
        return out

    def _gain_offset(self, ch: int) -> tuple[int, int]:
        code = self.afe.get(0x02 + ch, 0) & 0x3F
        raw = self.afe.get(0x05 + ch, 0)
        off = -(raw & 0xFF) if raw & 0x100 else raw & 0xFF
        return code, off

    def _signal(self, ch: int, x: int) -> int:
        if x < SHIELDED_BELOW:
            return 0
        if self.registers.get(0x03, 0) & 0x10:
            s = LAMP_SIGNAL[ch]
        elif self.registers.get(0x37, 0) & 0x04:
            s = IR_SIGNAL[ch]
        else:
            return 0
        d = x - FALLOFF_CENTRE
        f = 1000 - (d * d * FALLOFF_PERMILLE) // (FALLOFF_CENTRE * FALLOFF_CENTRE)
        ripple = (x * 37 + ch * 11) % 9 - 4
        return max(0, s * max(0, f) // 1000 + ripple)

    def _bulk_in_bytes(self, size: int) -> bytes:
        if not self._strip:
            return super()._bulk_in_bytes(size)
        start = _u24(self.registers, 0x82)
        span = max(1, _u24(self.registers, 0x85) - start)
        n = self._strip_pixels
        params = [self._gain_offset(ch) for ch in range(3)]
        out = bytearray(size)
        for i in range(0, size - 1, 2):
            sample = (self._strip_offset + i) // 2
            ch = sample % 3
            col = (sample // 3) % n
            x = start + col * span // n
            code, off = params[ch]
            v = (self._signal(ch, x) + BLACK[ch]) * 378 // (378 - 5 * code) + OFFSET_STEP * off
            v = max(0, min(0xFFFF, v))
            out[i] = v & 0xFF
            out[i + 1] = v >> 8
        self._strip_offset += size
        return bytes(out)
