// SPDX-License-Identifier: GPL-3.0-or-later
//! ASIC register maps and chip operations.

pub mod buttons;
pub mod gl128;
pub mod gl843_v1;
pub mod gl845;
pub mod motor;
pub mod registers;
pub mod status;

pub use gl128::Gl128;
pub use gl843_v1::Gl843V1;
pub use gl845::{Gl84xVariant, Gl845};
pub use status::ScannerStatus;

use crate::device::model::{FilmModel, ModelKind, ScanMethod};
use crate::error::{Error, Result};
use crate::usb::protocol::GenesysUsbProtocol;
use crate::usb::transport::UsbTransport;

/// The chip driver for one model (Python `create_asic`).
pub enum Asic {
    Gl845(Gl845),
    Gl128(Gl128),
    Gl843V1(Gl843V1),
}

impl Asic {
    /// Return the ASIC driver for `model`.
    pub fn create(transport: Box<dyn UsbTransport>, model: FilmModel) -> Result<Self> {
        let proto = GenesysUsbProtocol::new;
        Ok(match model.asic {
            "GL128" => Asic::Gl128(Gl128::new(proto(transport), model)),
            "GL845" => Asic::Gl845(Gl845::new(proto(transport), model, Gl84xVariant::Gl845)),
            "GL843" if model.kind == ModelKind::Opticfilm7600iV1 => Asic::Gl843V1(Gl843V1::new(transport, model)),
            "GL843" => Asic::Gl845(Gl845::new(proto(transport), model, Gl84xVariant::Gl843)),
            "GL842" => Asic::Gl845(Gl845::new(proto(transport), model, Gl84xVariant::Gl842)),
            a => return Err(Error::UnsupportedDevice(format!("No ASIC driver for model {:?} asic={a:?}", model.name))),
        })
    }

    pub fn model(&self) -> &FilmModel {
        match self {
            Asic::Gl845(a) => &a.model,
            Asic::Gl128(a) => &a.model,
            Asic::Gl843V1(a) => &a.model,
        }
    }

    pub fn initialized(&self) -> bool {
        match self {
            Asic::Gl845(a) => a.initialized,
            Asic::Gl128(a) => a.initialized,
            Asic::Gl843V1(a) => a.initialized,
        }
    }

    pub fn init(&mut self, force: bool) -> Result<()> {
        match self {
            Asic::Gl845(a) => a.init(force),
            Asic::Gl128(a) => a.init(force),
            Asic::Gl843V1(a) => a.init(force),
        }
    }

    pub fn read_status(&mut self) -> Result<ScannerStatus> {
        match self {
            Asic::Gl845(a) => a.read_status(),
            Asic::Gl128(a) => a.read_status(),
            Asic::Gl843V1(a) => a.read_status(),
        }
    }

    pub fn is_at_home(&mut self) -> Result<bool> {
        match self {
            Asic::Gl845(a) => a.is_at_home(),
            Asic::Gl128(a) => a.is_at_home(),
            Asic::Gl843V1(a) => a.is_at_home(),
        }
    }

    pub fn set_scan_method(&mut self, m: ScanMethod) {
        match self {
            Asic::Gl845(a) => a.set_scan_method(m),
            Asic::Gl128(a) => a.set_scan_method(m),
            Asic::Gl843V1(a) => a.set_scan_method(m),
        }
    }

    pub fn lamp_on(&mut self) -> Result<()> {
        match self {
            Asic::Gl845(a) => a.lamp_on(),
            Asic::Gl128(a) => a.lamp_on(),
            Asic::Gl843V1(a) => a.lamp_on(),
        }
    }

    pub fn lamp_off(&mut self) -> Result<()> {
        match self {
            Asic::Gl845(a) => a.lamp_off(),
            Asic::Gl128(a) => a.lamp_off(),
            Asic::Gl843V1(a) => a.lamp_off(),
        }
    }

    pub fn home(&mut self, timeout_s: f64) -> Result<()> {
        match self {
            Asic::Gl845(a) => a.home(timeout_s, true),
            Asic::Gl128(a) => a.home(),
            Asic::Gl843V1(a) => a.home(timeout_s),
        }
    }

    pub fn park(&mut self, timeout_s: f64) -> Result<()> {
        match self {
            Asic::Gl845(a) => a.park(timeout_s),
            Asic::Gl128(a) => a.park(),
            Asic::Gl843V1(a) => a.park(timeout_s),
        }
    }

    pub fn stop_motor(&mut self) -> Result<()> {
        match self {
            Asic::Gl845(a) => a.stop_motor(gl845::MOTOR_STOP_TIMEOUT_S),
            Asic::Gl128(a) => {
                a.stop_motor();
                Ok(())
            }
            Asic::Gl843V1(a) => a.stop_motor(),
        }
    }

    /// Raw register read for bring-up (GL843 v1 uses its own framing).
    pub fn read_register(&mut self, address: u16) -> Result<u8> {
        match self {
            Asic::Gl845(a) => a.protocol.read_register(address),
            Asic::Gl128(a) => a.protocol.read_register(address),
            Asic::Gl843V1(a) => a.usb.read_register(address as u8),
        }
    }

    pub fn write_register(&mut self, address: u16, value: u8) -> Result<()> {
        match self {
            Asic::Gl845(a) => a.protocol.write_register(address, value),
            Asic::Gl128(a) => a.protocol.write_register(address, value),
            Asic::Gl843V1(a) => a.usb.write_register(address as u8, value),
        }
    }

    pub fn write_registers(&mut self, pairs: &[(u16, u8)]) -> Result<()> {
        match self {
            Asic::Gl845(a) => a.protocol.write_registers(pairs),
            Asic::Gl128(a) => a.protocol.write_registers(pairs),
            Asic::Gl843V1(a) => {
                let p: Vec<(u8, u8)> = pairs.iter().map(|&(a, v)| (a as u8, v)).collect();
                a.usb.write_registers(&p)
            }
        }
    }

    /// The USB transport under the protocol layer.
    pub fn transport(&mut self) -> &mut dyn UsbTransport {
        match self {
            Asic::Gl845(a) => a.protocol.transport(),
            Asic::Gl128(a) => a.protocol.transport(),
            Asic::Gl843V1(a) => a.usb.transport.as_mut(),
        }
    }
}
