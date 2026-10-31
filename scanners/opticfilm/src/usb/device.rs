// SPDX-License-Identifier: GPL-3.0-or-later
//! USB enumeration, claim, and transfer helpers (nusb).

use std::any::Any;
use std::collections::VecDeque;
use std::num::NonZeroU8;
use std::time::Duration;

use log::{debug, info, warn};
use nusb::transfer::{
    Buffer, Bulk, ControlIn, ControlOut, ControlType, In, Interrupt, Out, Recipient, TransferError,
};
use nusb::{Endpoint, Interface, MaybeFuture};

use super::transport::{ControlData, UsbTransport};
use crate::error::{Error, Result};

/// OpticFilm 8200i (GL845) — probe-only in this release.
pub const VID_PLUSTEK: u16 = 0x07B3;
pub const PID_OPTICFILM_8200I: u16 = 0x130D;
/// OpticFilm 8200i SE (GL128) — scan-ready.
pub const PID_OPTICFILM_8200I_SE: u16 = 0x1825;
/// OpticFilm 8100 V2 (GL128) — scan-ready (no IR).
pub const PID_OPTICFILM_8100_V2: u16 = 0x1824;
/// OpticFilm 8300i SE (GL128) — scan-ready (IR capable).
pub const PID_OPTICFILM_8300I_SE: u16 = 0x181F;
pub const PID_OPTICFILM_7200: u16 = 0x0807;
pub const PID_OPTICFILM_7200I: u16 = 0x0C04;
pub const PID_OPTICFILM_7200_V2: u16 = 0x0C07;
pub const PID_OPTICFILM_7300: u16 = 0x0C12;
/// bcd 0x0400 → 7300 tables; bcd 0x0605 → 7400-v2.
pub const PID_OPTICFILM_7400: u16 = 0x0C3A;
pub const PID_OPTICFILM_7500I: u16 = 0x0C13;
/// bcd 0x0400 → 7600i v1; bcd 0x0605 → 7600i v2 (8200i tables).
pub const PID_OPTICFILM_7600I: u16 = 0x0C3B;
pub const PID_OPTICFILM_8100: u16 = 0x130C;

pub const SUPPORTED_PIDS: [u16; 12] = [
    PID_OPTICFILM_8200I,
    PID_OPTICFILM_8200I_SE,
    PID_OPTICFILM_8100_V2,
    PID_OPTICFILM_8300I_SE,
    PID_OPTICFILM_7200,
    PID_OPTICFILM_7200I,
    PID_OPTICFILM_7200_V2,
    PID_OPTICFILM_7300,
    PID_OPTICFILM_7400,
    PID_OPTICFILM_7500I,
    PID_OPTICFILM_7600I,
    PID_OPTICFILM_8100,
];

pub const DEFAULT_TIMEOUT_MS: u32 = 5000;
/// SANE genesys bulk ceiling (low.h BULKOUT_MAXSIZE).
pub const BULK_MAX_SIZE: usize = 0xF000;

/// Bus-visible device identity (no open handle).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UsbDeviceInfo {
    pub vendor_id: u16,
    pub product_id: u16,
    pub bus: Option<u32>,
    pub address: Option<u32>,
    pub manufacturer: Option<String>,
    pub product: Option<String>,
    pub bcd_device: u16,
    /// `iSerialNumber` string, read on open. GL128 firmware reports its build date here.
    pub serial_number: Option<String>,
}

impl UsbDeviceInfo {
    pub fn device_id(&self) -> String {
        format!(
            "plustek:usb:{:04x}:{:04x}:{:03}:{:03}",
            self.vendor_id,
            self.product_id,
            self.bus.unwrap_or(0),
            self.address.unwrap_or(0)
        )
    }

    pub fn is_supported(&self) -> bool {
        self.vendor_id == VID_PLUSTEK && SUPPORTED_PIDS.contains(&self.product_id)
    }

    pub fn is_8200i_se(&self) -> bool {
        self.vendor_id == VID_PLUSTEK && self.product_id == PID_OPTICFILM_8200I_SE
    }

    pub fn asic_hint(&self) -> &'static str {
        crate::device::select::model_for_device(self.product_id, self.bcd_device)
            .map_or("unknown", |m| m.asic)
    }
}

fn map_err(message: String) -> Error {
    let lowered = message.to_lowercase();
    const HINTS: [&str; 9] = [
        "access denied",
        "permission",
        "busy",
        "entity not found",
        "not supported",
        "winerror",
        "libusb0",
        "could not claim",
        "device not opened",
    ];
    if HINTS.iter().any(|h| lowered.contains(h)) {
        return Error::DriverBinding(format!(
            "{message}. On Windows, bind WinUSB with Zadig — see docs/windows-setup.md."
        ));
    }
    Error::Usb(message)
}

fn map_nusb(e: nusb::Error) -> Error {
    match e.kind() {
        nusb::ErrorKind::Busy
        | nusb::ErrorKind::PermissionDenied
        | nusb::ErrorKind::Unsupported => Error::DriverBinding(format!(
            "{e}. On Windows, bind WinUSB with Zadig — see docs/windows-setup.md."
        )),
        _ => map_err(e.to_string()),
    }
}

fn map_transfer(e: TransferError) -> Error {
    Error::Usb(e.to_string())
}

/// Like [`map_transfer`], but a blocking transfer cut off at its timeout (nusb
/// reports it as `Cancelled`) says so, as libusb's "Operation timed out" does.
fn map_timed(e: TransferError, what: &str, timeout: Duration) -> Error {
    match e {
        TransferError::Cancelled => Error::Usb(format!(
            "USB {what} timed out after {} ms",
            timeout.as_millis()
        )),
        e => map_transfer(e),
    }
}

fn bus_number(info: &nusb::DeviceInfo) -> u32 {
    info.bus_id().parse::<u32>().unwrap_or_else(|_| {
        u32::from_str_radix(info.bus_id().trim_start_matches("0x"), 16).unwrap_or(0)
    })
}

fn info_from_nusb(d: &nusb::DeviceInfo) -> UsbDeviceInfo {
    UsbDeviceInfo {
        vendor_id: d.vendor_id(),
        product_id: d.product_id(),
        bus: Some(bus_number(d)),
        address: Some(d.device_address() as u32),
        manufacturer: d.manufacturer_string().map(str::to_string),
        product: d.product_string().map(str::to_string),
        bcd_device: d.device_version(),
        serial_number: None,
    }
}

fn plustek_devices() -> Result<Vec<nusb::DeviceInfo>> {
    let all = nusb::list_devices().wait().map_err(map_nusb)?;
    Ok(all.filter(|d| d.vendor_id() == VID_PLUSTEK).collect())
}

/// Plustek film scanners visible on the bus (supported and known-bad).
///
/// Does not open devices: product strings come from the OS's cached copy.
pub fn list_devices() -> Result<Vec<UsbDeviceInfo>> {
    let found: Vec<UsbDeviceInfo> = plustek_devices()?.iter().map(info_from_nusb).collect();
    for i in &found {
        debug!("Found {} supported={}", i.device_id(), i.is_supported());
    }
    Ok(found)
}

/// List devices, optionally filtering to supported VID/PID pairs.
pub fn find_devices(supported_only: bool) -> Result<Vec<UsbDeviceInfo>> {
    let devices = list_devices()?;
    Ok(if supported_only {
        devices
            .into_iter()
            .filter(UsbDeviceInfo::is_supported)
            .collect()
    } else {
        devices
    })
}

/// Prefer a scan-ready model when several OpticFilms are present.
fn pick_preferred_device(devices: &[UsbDeviceInfo]) -> UsbDeviceInfo {
    use crate::device::select::{model_for_device, model_is_scan_ready};
    devices
        .iter()
        .find(|i| {
            model_for_device(i.product_id, i.bcd_device).is_ok_and(|m| model_is_scan_ready(&m))
        })
        .unwrap_or(&devices[0])
        .clone()
}

const NOT_FOUND_HELP: &str = "This release scans OpticFilm 8200i SE (07b3:1825), OpticFilm 8100 (V2) (07b3:1824), \
OpticFilm 8300i SE (07b3:181f), and OpticFilm 7600i v1 and v2 (07b3:0c3b)";

/// Opened and claimed USB device with control + bulk transfers.
pub struct UsbDeviceHandle {
    pub info: UsbDeviceInfo,
    pub timeout_ms: u32,
    device: Option<nusb::Device>,
    interface: Option<Interface>,
    ep_in: Option<Endpoint<Bulk, In>>,
    ep_out: Option<Endpoint<Bulk, Out>>,
    ep_intr_in: Option<Endpoint<Interrupt, In>>,
    ep_in_addr: u8,
    ep_out_addr: u8,
    /// Bytes received beyond what a `bulk_read` asked for (IN transfers are
    /// rounded up to whole packets).
    pending_in: VecDeque<u8>,
    needs_usb_reset: bool,
}

impl UsbDeviceHandle {
    fn new(info: UsbDeviceInfo) -> Result<Self> {
        if !info.is_supported() {
            return Err(Error::UnsupportedDevice(format!(
                "Device {} is not a known OpticFilm. {NOT_FOUND_HELP}; other PIDs may open for probe/status only.",
                info.device_id()
            )));
        }
        Ok(Self {
            info,
            timeout_ms: DEFAULT_TIMEOUT_MS,
            device: None,
            interface: None,
            ep_in: None,
            ep_out: None,
            ep_intr_in: None,
            ep_in_addr: 0,
            ep_out_addr: 0,
            pending_in: VecDeque::new(),
            needs_usb_reset: false,
        })
    }

    /// Open by `device_id`, or prefer a scan-ready model among supported devices.
    pub fn open(device_id: Option<&str>) -> Result<Self> {
        let devices = find_devices(true)?;
        let mut handle = match device_id {
            Some(id) => match devices.iter().find(|d| d.device_id() == id) {
                Some(m) => Self::new(m.clone())?,
                None => match list_devices()?.into_iter().find(|d| d.device_id() == id) {
                    Some(bad) => Self::new(bad)?,
                    None => return Err(Error::DeviceNotFound(format!("No device with id {id:?}"))),
                },
            },
            None => {
                if devices.is_empty() {
                    return Err(Error::DeviceNotFound(format!(
                        "No OpticFilm found. {NOT_FOUND_HELP}. Check cabling and WinUSB binding — see docs/windows-setup.md."
                    )));
                }
                Self::new(pick_preferred_device(&devices))?
            }
        };
        handle.open_device()?;
        Ok(handle)
    }

    fn find_nusb_device(&self) -> Result<nusb::DeviceInfo> {
        let matches: Vec<_> = plustek_devices()?
            .into_iter()
            .filter(|d| d.product_id() == self.info.product_id)
            .collect();
        if matches.is_empty() {
            return Err(Error::DeviceNotFound(format!(
                "Device {} disappeared before open.",
                self.info.device_id()
            )));
        }
        if let (Some(bus), Some(addr)) = (self.info.bus, self.info.address)
            && let Some(d) = matches
                .iter()
                .find(|d| bus_number(d) == bus && d.device_address() as u32 == addr)
        {
            return Ok(d.clone());
        }
        if matches.len() == 1 {
            return Ok(matches[0].clone());
        }
        Err(Error::DeviceNotFound(format!(
            "Ambiguous match for {}; found {} devices with same VID/PID.",
            self.info.device_id(),
            matches.len()
        )))
    }

    fn open_device(&mut self) -> Result<()> {
        let r = self.open_inner();
        if r.is_err() {
            self.close();
        }
        r
    }

    fn open_inner(&mut self) -> Result<()> {
        let di = self.find_nusb_device()?;
        let device = di.open().wait().map_err(map_nusb)?;
        if let Err(e) = device.detach_kernel_driver(0) {
            debug!("Kernel detach skipped: {e}");
        }
        // Configure only an unconfigured device: setting it again resets the host's bulk
        // data toggles but not the 7600i's GL843, so the next session lost its first packet.
        if device.active_configuration().is_err() {
            if let Some(v) = device
                .configurations()
                .next()
                .map(|c| c.configuration_value())
            {
                if let Err(e) = device.set_configuration(v).wait() {
                    debug!("set_configuration: {e}");
                }
            }
        }
        let (iface_num, eps) = {
            let cfg = device
                .active_configuration()
                .map_err(|e| Error::Usb(e.to_string()))?;
            let intf = cfg
                .interface_alt_settings()
                .find(|i| i.alternate_setting() == 0)
                .ok_or_else(|| Error::Usb("device has no interface 0".into()))?;
            // (address, attributes & 3: 2 = bulk, 3 = interrupt)
            let eps: Vec<(u8, u8)> = intf
                .endpoints()
                .map(|e| (e.address(), e.attributes() & 0x03))
                .collect();
            (intf.interface_number(), eps)
        };
        let interface = device
            .detach_and_claim_interface(iface_num)
            .wait()
            .map_err(map_nusb)?;
        for (addr, tt) in eps {
            let is_in = addr & 0x80 != 0;
            match (tt, is_in) {
                (2, true) if self.ep_in.is_none() => {
                    self.ep_in = Some(interface.endpoint::<Bulk, In>(addr).map_err(map_nusb)?);
                    self.ep_in_addr = addr;
                }
                (2, false) if self.ep_out.is_none() => {
                    self.ep_out = Some(interface.endpoint::<Bulk, Out>(addr).map_err(map_nusb)?);
                    self.ep_out_addr = addr;
                }
                (3, true) if self.ep_intr_in.is_none() => {
                    self.ep_intr_in = Some(
                        interface
                            .endpoint::<Interrupt, In>(addr)
                            .map_err(map_nusb)?,
                    );
                }
                _ => {}
            }
        }
        self.info.serial_number = read_serial_number(&device);
        info!(
            "Opened {} (iface={iface_num} ep_in={:#x} ep_out={:#x} intr={} serial={:?})",
            self.info.device_id(),
            self.ep_in_addr,
            self.ep_out_addr,
            self.ep_intr_in.is_some(),
            self.info.serial_number
        );
        self.device = Some(device);
        self.interface = Some(interface);
        self.needs_usb_reset = false;
        Ok(())
    }

    pub fn is_open(&self) -> bool {
        self.interface.is_some()
    }

    pub fn close(&mut self) {
        self.cancel_queued();
        if self.needs_usb_reset
            && let Some(d) = &self.device
        {
            // Reset while still claimed — more reliable on WinUSB than after release.
            match d.reset().wait() {
                Ok(()) => info!("USB device reset after aborted bulk stream"),
                Err(e) => warn!("USB reset after aborted bulk failed: {e}"),
            }
        }
        self.ep_in = None;
        self.ep_out = None;
        self.ep_intr_in = None;
        self.interface = None;
        self.device = None;
        self.needs_usb_reset = false;
        self.pending_in.clear();
        debug!("Closed {}", self.info.device_id());
    }

    fn interface(&self) -> Result<&Interface> {
        self.interface
            .as_ref()
            .ok_or_else(|| Error::Usb("USB device is not open.".into()))
    }

    fn timeout(&self, t: Option<u32>) -> Duration {
        Duration::from_millis(t.unwrap_or(self.timeout_ms) as u64)
    }

    /// Clear a stall on the bulk IN (or OUT) endpoint.
    pub fn clear_halt(&mut self, out: bool) {
        let r = if out {
            self.ep_out.as_mut().map(|e| e.clear_halt().wait())
        } else {
            self.ep_in.as_mut().map(|e| e.clear_halt().wait())
        };
        let ep = if out {
            self.ep_out_addr
        } else {
            self.ep_in_addr
        };
        match r {
            Some(Ok(())) => info!("clear_halt endpoint=0x{ep:02x}"),
            Some(Err(e)) => warn!("clear_halt endpoint=0x{ep:02x} failed: {e}"),
            None => {}
        }
    }

    fn read_ep(
        ep: &mut Endpoint<Bulk, In>,
        size: usize,
        timeout: Duration,
    ) -> std::result::Result<Vec<u8>, TransferError> {
        let mps = ep.max_packet_size().max(1);
        let want = size.div_ceil(mps).max(1) * mps;
        let mut buf = Buffer::new(want);
        buf.set_requested_len(want);
        let c = ep.transfer_blocking(buf, timeout);
        c.into_result().map(Buffer::into_vec)
    }

    /// Cancel queued bulk IN reads and wait until each has returned.
    fn cancel_queued(&mut self) {
        let Some(ep) = self.ep_in.as_mut() else {
            return;
        };
        if ep.pending() == 0 {
            return;
        }
        ep.cancel_all();
        while ep.pending() > 0 {
            if ep.wait_next_complete(Duration::from_secs(1)).is_none() {
                warn!("cancelled bulk read has not returned yet");
            }
        }
    }
}

impl Drop for UsbDeviceHandle {
    fn drop(&mut self) {
        self.close();
    }
}

/// `iSerialNumber` of an opened device, or `None` if absent or unreadable.
fn read_serial_number(dev: &nusb::Device) -> Option<String> {
    let index: NonZeroU8 = dev.device_descriptor().serial_number_string_index()?;
    let t = Duration::from_millis(1000);
    // The vendor driver asks with LANGID 0; try the normal LANGID lookup first.
    let mut langs: Vec<u16> = dev
        .get_string_descriptor_supported_languages(t)
        .wait()
        .map(|l| l.take(1).collect())
        .unwrap_or_default();
    langs.extend([0x0409, 0]);
    for lang in langs {
        if let Ok(s) = dev.get_string_descriptor(index, lang, t).wait() {
            let s = s.trim().to_string();
            if !s.is_empty() {
                return Some(s);
            }
        }
    }
    None
}

fn control_type(rt: u8) -> ControlType {
    match (rt >> 5) & 0x03 {
        0 => ControlType::Standard,
        1 => ControlType::Class,
        _ => ControlType::Vendor,
    }
}

fn recipient(rt: u8) -> Recipient {
    match rt & 0x1F {
        1 => Recipient::Interface,
        2 => Recipient::Endpoint,
        3 => Recipient::Other,
        _ => Recipient::Device,
    }
}

impl UsbTransport for UsbDeviceHandle {
    fn control_msg(
        &mut self,
        rt: u8,
        req: u8,
        value: u16,
        index: u16,
        data: ControlData<'_>,
        timeout_ms: Option<u32>,
    ) -> Result<Vec<u8>> {
        let timeout = self.timeout(timeout_ms);
        let intf = self.interface()?;
        let (control_type, recipient) = (control_type(rt), recipient(rt));
        let r = match data {
            ControlData::In(n) => intf
                .control_in(
                    ControlIn {
                        control_type,
                        recipient,
                        request: req,
                        value,
                        index,
                        length: n as u16,
                    },
                    timeout,
                )
                .wait(),
            ControlData::Out(d) => intf
                .control_out(
                    ControlOut {
                        control_type,
                        recipient,
                        request: req,
                        value,
                        index,
                        data: d,
                    },
                    timeout,
                )
                .wait()
                .map(|()| Vec::new()),
        };
        r.map_err(|e| {
            let mapped = map_timed(e, "control transfer", timeout);
            log::error!("control_msg failed type=0x{rt:02x} req=0x{req:02x} val=0x{value:04x} idx=0x{index:04x}: {mapped}");
            mapped
        })
    }

    fn bulk_read(&mut self, size: usize, timeout_ms: Option<u32>) -> Result<Vec<u8>> {
        let timeout = self.timeout(timeout_ms);
        if self.interface.is_none() {
            return Err(Error::Usb("USB device is not open.".into()));
        }
        if self.pending_in.len() < size {
            let ep = self
                .ep_in
                .as_mut()
                .ok_or_else(|| Error::Usb("Device has no bulk IN endpoint.".into()))?;
            if ep.pending() > 0 {
                return Err(Error::Usb(
                    "bulk read while queued reads are pending".into(),
                ));
            }
            let need = size - self.pending_in.len();
            let got =
                Self::read_ep(ep, need, timeout).map_err(|e| map_timed(e, "bulk read", timeout))?;
            self.pending_in.extend(got);
        }
        let n = size.min(self.pending_in.len());
        Ok(self.pending_in.drain(..n).collect())
    }

    fn bulk_read_submit(&mut self, size: usize) -> Result<bool> {
        self.interface()?;
        if !self.pending_in.is_empty() {
            return Err(Error::Usb(format!(
                "{} unread bytes before a queued bulk read",
                self.pending_in.len()
            )));
        }
        let ep = self
            .ep_in
            .as_mut()
            .ok_or_else(|| Error::Usb("Device has no bulk IN endpoint.".into()))?;
        let mps = ep.max_packet_size().max(1);
        let want = size.div_ceil(mps).max(1) * mps;
        let mut buf = Buffer::new(want);
        buf.set_requested_len(want);
        ep.submit(buf);
        Ok(true)
    }

    fn bulk_read_wait(&mut self, timeout_ms: Option<u32>) -> Result<Vec<u8>> {
        let timeout = self.timeout(timeout_ms);
        let ep = self
            .ep_in
            .as_mut()
            .ok_or_else(|| Error::Usb("Device has no bulk IN endpoint.".into()))?;
        if ep.pending() == 0 {
            return Err(Error::Usb("no queued bulk read".into()));
        }
        let r = match ep.wait_next_complete(timeout) {
            Some(c) => c
                .into_result()
                .map(Buffer::into_vec)
                .map_err(|e| map_timed(e, "bulk read", timeout)),
            None => Err(map_timed(TransferError::Cancelled, "bulk read", timeout)),
        };
        if r.is_err() {
            // Later reads would land out of order after a failed one.
            self.cancel_queued();
        }
        r
    }

    fn bulk_write(&mut self, data: &[u8], timeout_ms: Option<u32>) -> Result<usize> {
        let timeout = self.timeout(timeout_ms);
        self.interface()?;
        let ep = self
            .ep_out
            .as_mut()
            .ok_or_else(|| Error::Usb("Device has no bulk OUT endpoint.".into()))?;
        let mut buf = Buffer::new(data.len());
        buf.extend_from_slice(data);
        let c = ep.transfer_blocking(buf, timeout);
        let n = c.actual_len;
        c.status.map_err(|e| map_timed(e, "bulk write", timeout))?;
        Ok(n)
    }

    fn read_intr(&mut self, timeout_ms: Option<u32>) -> Result<Vec<u8>> {
        let timeout = self.timeout(timeout_ms);
        self.interface()?;
        let ep = self
            .ep_intr_in
            .as_mut()
            .ok_or_else(|| Error::Usb("Device has no interrupt IN endpoint.".into()))?;
        let mps = ep.max_packet_size().max(1);
        let mut buf = Buffer::new(mps);
        buf.set_requested_len(mps);
        match ep.transfer_blocking(buf, timeout).into_result() {
            Ok(b) => Ok(b.into_vec()),
            Err(TransferError::Cancelled) => Ok(Vec::new()),
            Err(e) => Err(map_transfer(e)),
        }
    }

    /// Drain residual bulk IN then clear halt. Call after mid-scan cancel once
    /// SCAN is cleared so the ASIC stops producing.
    fn abort_bulk_in(&mut self) -> Option<usize> {
        self.cancel_queued();
        let ep = self.ep_in.as_mut()?;
        let max_drain = BULK_MAX_SIZE * 4;
        let mut drained = self.pending_in.len();
        self.pending_in.clear();
        while drained < max_drain {
            match Self::read_ep(
                ep,
                BULK_MAX_SIZE.min(max_drain - drained),
                Duration::from_millis(50),
            ) {
                Ok(d) if !d.is_empty() => drained += d.len(),
                Ok(_) => break,
                Err(e) => {
                    debug!("abort_bulk_in drain stop after {drained} bytes: {e}");
                    break;
                }
            }
        }
        self.clear_halt(false);
        self.clear_halt(true);
        self.needs_usb_reset = true;
        info!("abort_bulk_in drained={drained} bytes (usb reset on close)");
        Some(drained)
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timeouts_read_as_timeouts() {
        let t = Duration::from_millis(5000);
        assert_eq!(
            map_timed(TransferError::Cancelled, "bulk read", t).to_string(),
            "USB bulk read timed out after 5000 ms"
        );
        assert!(
            matches!(map_timed(TransferError::Stall, "bulk read", t), Error::Usb(m) if !m.contains("timed out"))
        );
    }

    #[test]
    fn closed_handle_refuses_transfers() {
        let info = UsbDeviceInfo {
            vendor_id: VID_PLUSTEK,
            product_id: PID_OPTICFILM_8200I_SE,
            ..Default::default()
        };
        let mut h = UsbDeviceHandle::new(info).unwrap();
        for r in [
            h.bulk_read(4, None).map(|_| ()),
            h.bulk_write(&[1], None).map(|_| ()),
            h.read_intr(None).map(|_| ()),
            h.bulk_read_submit(4).map(|_| ()),
        ] {
            assert!(matches!(r, Err(Error::Usb(m)) if m.contains("not open")));
        }
        assert!(h.abort_bulk_in().is_none());
    }

    #[test]
    fn unsupported_device_is_refused() {
        let info = UsbDeviceInfo {
            vendor_id: VID_PLUSTEK,
            product_id: 0x0001,
            ..Default::default()
        };
        assert!(matches!(
            UsbDeviceHandle::new(info),
            Err(Error::UnsupportedDevice(_))
        ));
    }
}
