// SPDX-License-Identifier: GPL-3.0-or-later
//! The minimal transport every protocol layer talks to.

use std::any::Any;

use crate::error::{Error, Result};

/// Data stage of a control transfer: a read of `n` bytes, or bytes to write.
#[derive(Debug, Clone, Copy)]
pub enum ControlData<'a> {
    In(usize),
    Out(&'a [u8]),
}

impl ControlData<'_> {
    pub fn is_read(&self) -> bool {
        matches!(self, ControlData::In(_))
    }
}

/// USB transfers as seen by [`GenesysUsbProtocol`](super::protocol::GenesysUsbProtocol).
///
/// Implemented by the real device ([`UsbDeviceHandle`](super::device::UsbDeviceHandle)),
/// the fakes in [`fake`](super::fake) and the [`RecordingTransport`](super::trace::RecordingTransport).
pub trait UsbTransport: Send + Any {
    /// Vendor control transfer. Reads return the response; writes return an empty vec.
    fn control_msg(
        &mut self,
        request_type: u8,
        request: u8,
        value: u16,
        index: u16,
        data: ControlData<'_>,
        timeout_ms: Option<u32>,
    ) -> Result<Vec<u8>>;

    /// Read up to `size` bytes from the bulk IN endpoint.
    fn bulk_read(&mut self, size: usize, timeout_ms: Option<u32>) -> Result<Vec<u8>>;

    /// Write to the bulk OUT endpoint; returns bytes written.
    fn bulk_write(&mut self, data: &[u8], timeout_ms: Option<u32>) -> Result<usize>;

    /// Read the interrupt IN endpoint (empty on timeout).
    fn read_intr(&mut self, timeout_ms: Option<u32>) -> Result<Vec<u8>> {
        let _ = timeout_ms;
        Err(Error::Usb("transport has no interrupt endpoint".into()))
    }

    /// Queue a bulk IN read of `size` bytes without waiting for it, so the
    /// device always has a request pending. `Ok(false)` when the transport
    /// cannot queue reads; use [`UsbTransport::bulk_read`] then.
    fn bulk_read_submit(&mut self, size: usize) -> Result<bool> {
        let _ = size;
        Ok(false)
    }

    /// Wait for the oldest read queued with [`UsbTransport::bulk_read_submit`].
    fn bulk_read_wait(&mut self, timeout_ms: Option<u32>) -> Result<Vec<u8>> {
        let _ = timeout_ms;
        Err(Error::Usb("no queued bulk read".into()))
    }

    /// Drain and clear an abandoned bulk IN stream. `None` when unsupported.
    fn abort_bulk_in(&mut self) -> Option<usize> {
        None
    }

    /// For tests and the GL843 v1 path, which reach through to a concrete transport.
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// The transport this one wraps, if any (recording wrappers).
    fn inner_mut(&mut self) -> Option<&mut dyn UsbTransport> {
        None
    }
}

/// Find a concrete transport `T` in a chain of wrappers.
pub fn find_transport<T: UsbTransport>(t: &mut dyn UsbTransport) -> Option<&mut T> {
    let mut cur: &mut dyn UsbTransport = t;
    loop {
        if cur.as_any_mut().is::<T>() {
            return cur.as_any_mut().downcast_mut::<T>();
        }
        cur = cur.inner_mut()?;
    }
}
