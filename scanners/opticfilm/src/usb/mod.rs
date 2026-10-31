// SPDX-License-Identifier: GPL-3.0-or-later
//! USB device discovery, framing, tracing and fakes.

pub mod correlate;
pub mod decode;
pub mod device;
pub mod fake;
pub mod fake_gl128_sim;
pub mod fake_gl843_v1;
pub mod protocol;
pub mod trace;
pub mod transport;

pub use device::{UsbDeviceHandle, UsbDeviceInfo, find_devices, list_devices};
pub use fake::{FakeUsbTransport, MockScannerTransport};
pub use protocol::GenesysUsbProtocol;
pub use trace::{RecordingTransport, UsbTransaction};
pub use transport::{ControlData, UsbTransport};
