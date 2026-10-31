// SPDX-License-Identifier: GPL-3.0-or-later
//! Genesys USB vendor-request helpers (GL845 path).
//!
//! Constants and register/bulk framing mirror SANE genesys `low.h` and
//! `scanner_interface_usb.cpp`.

use log::{debug, info};

use super::device::BULK_MAX_SIZE;
use super::transport::{ControlData, UsbTransport};
use crate::error::{Error, Result};

pub const USB_TYPE_VENDOR: u8 = 0x40;
pub const USB_DIR_OUT: u8 = 0x00;
pub const USB_DIR_IN: u8 = 0x80;

pub const REQUEST_TYPE_IN: u8 = USB_TYPE_VENDOR | USB_DIR_IN; // 0xC0
pub const REQUEST_TYPE_OUT: u8 = USB_TYPE_VENDOR | USB_DIR_OUT; // 0x40

pub const REQUEST_REGISTER: u8 = 0x0C;
pub const REQUEST_BUFFER: u8 = 0x04;

pub const VALUE_BUFFER: u16 = 0x82;
pub const VALUE_SET_REGISTER: u16 = 0x83;
pub const VALUE_READ_REGISTER: u16 = 0x84;
pub const VALUE_WRITE_REGISTER: u16 = 0x85;
pub const VALUE_INIT: u16 = 0x87;
pub const GPIO_OUTPUT_ENABLE: u16 = 0x89;
pub const GPIO_READ: u16 = 0x8A;
pub const GPIO_WRITE: u16 = 0x8B;
pub const VALUE_BUF_ENDACCESS: u16 = 0x8C;
pub const VALUE_GET_REGISTER: u16 = 0x8E;

pub const INDEX: u16 = 0x00;

pub const BULK_OUT: u8 = 0x01;
pub const BULK_IN: u8 = 0x00;
pub const BULK_RAM: u8 = 0x00;
pub const BULK_REGISTER: u8 = 0x11;

/// GL845/846/847/GL124 read path expects this status byte.
pub const REGISTER_LINK_OK: u8 = 0x55;

/// Register and bulk helpers for Genesys ASICs.
pub struct GenesysUsbProtocol {
    transport: Box<dyn UsbTransport>,
}

impl GenesysUsbProtocol {
    pub fn new(transport: Box<dyn UsbTransport>) -> Self {
        Self { transport }
    }

    pub fn transport(&mut self) -> &mut dyn UsbTransport {
        self.transport.as_mut()
    }

    pub fn into_transport(self) -> Box<dyn UsbTransport> {
        self.transport
    }

    fn ctrl(&mut self, rt: u8, req: u8, value: u16, index: u16, data: ControlData<'_>) -> Result<Vec<u8>> {
        self.transport.control_msg(rt, req, value, index, data, None)
    }

    /// SANE genesys validates USB link with `0x55` on GL845-class reads.
    pub fn check_link_status(status_byte: u8) -> Result<()> {
        if status_byte != REGISTER_LINK_OK {
            return Err(Error::Usb(format!(
                "Invalid USB link status 0x{status_byte:02x} (expected 0x55); \
                 scanner unplugged or driver binding wrong?"
            )));
        }
        Ok(())
    }

    /// Read one ASIC register (GL845/846/847/GL124 control-msg path).
    pub fn read_register(&mut self, address: u16) -> Result<u8> {
        // SANE: address16 = 0x22 + (address << 8) as uint16
        let address16 = 0x22u16.wrapping_add(address.wrapping_shl(8));
        let mut usb_value = VALUE_GET_REGISTER;
        if address > 0xFF {
            usb_value |= 0x100;
        }
        let raw = self.ctrl(REQUEST_TYPE_IN, REQUEST_BUFFER, usb_value, address16, ControlData::In(2))?;
        if raw.len() != 2 {
            return Err(Error::Usb(format!("Register read returned {} bytes, expected 2", raw.len())));
        }
        Self::check_link_status(raw[1])?;
        debug!("read_register(0x{address:04x}) -> 0x{:02x}", raw[0]);
        Ok(raw[0])
    }

    /// Write one ASIC register (GL845/846/847/GL124 control-msg path).
    pub fn write_register(&mut self, address: u16, value: u8) -> Result<()> {
        let payload = [(address & 0xFF) as u8, value];
        let mut usb_value = VALUE_SET_REGISTER;
        if address > 0xFF {
            usb_value |= 0x100;
        }
        self.ctrl(REQUEST_TYPE_OUT, REQUEST_BUFFER, usb_value, INDEX, ControlData::Out(&payload))?;
        debug!("write_register(0x{address:04x}, 0x{value:02x})");
        Ok(())
    }

    /// Write many registers (GL845 uses per-register writes).
    pub fn write_registers(&mut self, pairs: &[(u16, u8)]) -> Result<()> {
        for &(a, v) in pairs {
            self.write_register(a, v)?;
        }
        Ok(())
    }

    /// Write many registers, packing `(addr, value)` pairs per transfer.
    ///
    /// The Windows driver for the 8200i SE sends up to 32 pairs in a single
    /// `VALUE_SET_REGISTER` transfer. Addresses above `0xFF` need the
    /// high-address flag in `wValue`, so they are emitted individually.
    pub fn write_registers_batched(&mut self, pairs: &[(u16, u8)], max_pairs: usize) -> Result<()> {
        if max_pairs < 1 {
            return Err(Error::Value("max_pairs must be >= 1".into()));
        }
        let mut batch: Vec<(u16, u8)> = Vec::new();
        for &(address, value) in pairs {
            if address > 0xFF {
                self.flush_batch(&mut batch)?;
                self.write_register(address, value)?;
                continue;
            }
            batch.push((address, value));
            if batch.len() >= max_pairs {
                self.flush_batch(&mut batch)?;
            }
        }
        self.flush_batch(&mut batch)
    }

    fn flush_batch(&mut self, batch: &mut Vec<(u16, u8)>) -> Result<()> {
        match batch.len() {
            0 => {}
            1 => self.write_register(batch[0].0, batch[0].1)?,
            n => {
                let payload: Vec<u8> = batch.iter().flat_map(|&(a, v)| [(a & 0xFF) as u8, v]).collect();
                self.ctrl(REQUEST_TYPE_OUT, REQUEST_BUFFER, VALUE_SET_REGISTER, INDEX, ControlData::Out(&payload))?;
                debug!("write_registers_batched: {n} pairs");
            }
        }
        batch.clear();
        Ok(())
    }

    /// SANE `write_0x8c` — VALUE_BUF_ENDACCESS vendor request.
    pub fn write_0x8c(&mut self, index: u8, value: u8) -> Result<()> {
        self.ctrl(REQUEST_TYPE_OUT, REQUEST_REGISTER, VALUE_BUF_ENDACCESS, index as u16, ControlData::Out(&[value]))?;
        Ok(())
    }

    /// Cold-init probe used by SANE `sanei_genesys_asic_init` (REQUEST_REGISTER).
    pub fn read_usb_speed_byte(&mut self) -> Result<u8> {
        self.read_request_register(0x00)
    }

    /// 1-byte `REQUEST_REGISTER` / `VALUE_GET_REGISTER` probe.
    ///
    /// Index `0x00` is the SANE USB-speed / warm-boot probe; index `0x21` is
    /// polled during fast feeds until it returns `0x04`.
    pub fn read_request_register(&mut self, index: u8) -> Result<u8> {
        let raw = self.ctrl(REQUEST_TYPE_IN, REQUEST_REGISTER, VALUE_GET_REGISTER, index as u16, ControlData::In(1))?;
        raw.first()
            .copied()
            .ok_or_else(|| Error::Usb(format!("REQUEST_REGISTER probe index=0x{index:02x} returned no data")))
    }

    /// Write analog frontend register (GL845 non-GL124 path).
    pub fn write_fe_register(&mut self, address: u8, value: u16) -> Result<()> {
        self.write_register(0x51, address)?;
        self.write_register(0x3A, (value >> 8) as u8)?;
        self.write_register(0x3B, (value & 0xFF) as u8)
    }

    /// Write analog frontend register (GL124 path, used by GL128 OpticFilm).
    pub fn write_fe_register_gl124(&mut self, address: u8, value: u16) -> Result<()> {
        self.write_register(0x51, address)?;
        self.write_register(0x5D, (value >> 8) as u8)?;
        self.write_register(0x5E, (value & 0xFF) as u8)
    }

    /// Read analog frontend register.
    pub fn read_fe_register(&mut self, address: u8) -> Result<u16> {
        self.write_register(0x50, address)?;
        let high = self.read_register(0x46)?;
        let low = self.read_register(0x47)?;
        Ok(((high as u16) << 8) | low as u16)
    }

    /// Write a 16-bit big-endian value starting at `address`.
    pub fn write_u16(&mut self, address: u16, value: u32) -> Result<()> {
        let value = value & 0xFFFF;
        self.write_register(address, (value >> 8) as u8)?;
        self.write_register(address + 1, (value & 0xFF) as u8)
    }

    /// Write a 24-bit big-endian value starting at `address`.
    pub fn write_u24(&mut self, address: u16, value: u32) -> Result<()> {
        let value = value & 0xFF_FFFF;
        self.write_register(address, (value >> 16) as u8)?;
        self.write_register(address + 1, ((value >> 8) & 0xFF) as u8)?;
        self.write_register(address + 2, (value & 0xFF) as u8)
    }

    pub fn read_intr(&mut self, timeout_ms: Option<u32>) -> Result<Vec<u8>> {
        self.transport.read_intr(timeout_ms)
    }

    /// 8-byte little-endian `(address, size)` header used by every bulk op.
    pub fn bulk_preamble(addr: u32, size: u32) -> [u8; 8] {
        let a = addr.to_le_bytes();
        let s = size.to_le_bytes();
        [a[0], a[1], a[2], a[3], s[0], s[1], s[2], s[3]]
    }

    /// GL845/846/847/GL124 bulk-read preamble (fixed 0x10000000 address).
    fn bulk_read_send_header(&mut self, size: usize) -> Result<()> {
        let pre = Self::bulk_preamble(0x1000_0000, size as u32);
        self.ctrl(REQUEST_TYPE_OUT, REQUEST_BUFFER, VALUE_BUFFER, 0x00, ControlData::Out(&pre))?;
        Ok(())
    }

    /// Announce a single bulk transfer of `size` bytes and stop.
    ///
    /// The GL128 path sends one preamble for a whole image and then streams it.
    /// `index` `0x00` reads scanner RAM and `0x08` reads the live image stream.
    pub fn bulk_read_begin(&mut self, size: usize, index: u8, addr: u32) -> Result<()> {
        let pre = Self::bulk_preamble(addr, size as u32);
        self.ctrl(REQUEST_TYPE_OUT, REQUEST_BUFFER, VALUE_BUFFER, index as u16, ControlData::Out(&pre))?;
        debug!("bulk_read_begin size={size} index=0x{index:02x} addr=0x{addr:08x}");
        Ok(())
    }

    /// Read one chunk of an in-flight transfer started by `bulk_read_begin`.
    pub fn bulk_read_chunk(&mut self, size: usize) -> Result<Vec<u8>> {
        if size == 0 {
            return Ok(Vec::new());
        }
        let data = self.transport.bulk_read(size.min(BULK_MAX_SIZE), None)?;
        if data.is_empty() {
            return Err(Error::Usb("bulk_read returned empty data mid-transfer".into()));
        }
        Ok(data)
    }

    /// Read exactly `size` bytes from an in-flight `bulk_read_begin`.
    pub fn bulk_read_exact(&mut self, size: usize, max_transfer: usize) -> Result<Vec<u8>> {
        if size == 0 {
            return Ok(Vec::new());
        }
        let step = max_transfer.max(1);
        let mut buf = Vec::with_capacity(size);
        while buf.len() < size {
            let data = self.transport.bulk_read((size - buf.len()).min(step), None)?;
            if data.is_empty() {
                return Err(Error::Usb(format!("bulk_read returned empty after {} of {size} bytes", buf.len())));
            }
            buf.extend_from_slice(&data);
        }
        Ok(buf)
    }

    /// Abandon an in-flight `bulk_read_begin` stream (cancel / error).
    pub fn abort_bulk_stream(&mut self) -> usize {
        match self.transport.abort_bulk_in() {
            None => {
                debug!("abort_bulk_stream: transport has no abort_bulk_in");
                0
            }
            Some(drained) => {
                info!("abort_bulk_stream drained={drained}");
                drained
            }
        }
    }

    /// Read `size` bytes via GL845 bulk path (header before each chunk).
    pub fn bulk_read_data(&mut self, size: usize) -> Result<Vec<u8>> {
        let mut out = Vec::with_capacity(size);
        let mut remaining = size as isize;
        while remaining > 0 {
            let block = (remaining as usize).min(BULK_MAX_SIZE);
            self.bulk_read_send_header(block)?;
            let data = self.transport.bulk_read(block, None)?;
            if data.is_empty() {
                return Err(Error::Usb("bulk_read returned empty data mid-transfer".into()));
            }
            remaining -= data.len() as isize;
            out.extend_from_slice(&data);
            debug!("bulk_read_data: got {} bytes, {remaining} remaining", data.len());
        }
        Ok(out)
    }

    /// Write an AHB memory window (GL845/846/847/GL124 table upload path).
    pub fn write_ahb(&mut self, addr: u32, data: &[u8]) -> Result<()> {
        let pre = Self::bulk_preamble(addr, data.len() as u32);
        self.ctrl(REQUEST_TYPE_OUT, REQUEST_BUFFER, VALUE_BUFFER, 0x01, ControlData::Out(&pre))?;
        let mut offset = 0;
        while offset < data.len() {
            let block = (data.len() - offset).min(BULK_MAX_SIZE);
            let written = self.transport.bulk_write(&data[offset..offset + block], None)?;
            if written == 0 {
                return Err(Error::Usb("bulk_write wrote 0 bytes mid-AHB transfer".into()));
            }
            offset += written;
        }
        Ok(())
    }
}
