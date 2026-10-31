// SPDX-License-Identifier: GPL-3.0-or-later
//! High-level scanner façade.

use std::collections::HashSet;
use std::path::PathBuf;

use log::{debug, info};

use crate::asic::buttons::Button;
use crate::asic::{Asic, ScannerStatus};
use crate::device::model::{FilmModel, ModelKind, ScanMethod, gl128_firmware_variant};
use crate::device::select::{model_for_device, model_is_scan_ready};
use crate::error::{Error, Result};
use crate::image::ScanImage;
use crate::scan::calibrate::{CalibEntry, Calibrator};
use crate::scan::exposure_override::{validate_manual_exposure, validate_n_passes};
use crate::scan::geometry::{Area, ScanGeometry};
use crate::scan::me_debug::{MeScanDebug, MultiPassDebug};
use crate::scan::request::{ScanControl, ScanMode, ScanRequest};
use crate::scan::session::ScanSession;
use crate::scan::session_7600i_v1::Gl843V1ScanSession;
use crate::scan::session_gl128::{Gl128ScanSession, check_gl128_multi_pass};
use crate::usb::device::{UsbDeviceHandle, UsbDeviceInfo, VID_PLUSTEK};
use crate::usb::fake::MockScannerTransport;
use crate::usb::fake_gl843_v1::SimulatedGl843V1Transport;
use crate::usb::transport::UsbTransport;

/// User-facing entry point for OpticFilm scanners.
pub struct Scanner {
    info: UsbDeviceInfo,
    model: FilmModel,
    asic: Asic,
    calibrator: Calibrator,
    closed: bool,
    last_me_debug: Option<MeScanDebug>,
    last_multi_pass_debug: Option<MultiPassDebug>,
    last_align_shift_ir: Option<(f64, f64)>,
    /// Lab / session may disarm GL128 briefly for stationary shading.
    bringup_motor_armed: bool,
    /// When true, scan/home/park are allowed even if `scan_ready` is false
    /// (set only by [`Scanner::open_fake`]; real hardware stays gated).
    allow_unvalidated_scan: bool,
}

impl Scanner {
    /// Wrap an opened transport. `model` defaults to the USB id mapping.
    pub fn new(transport: Box<dyn UsbTransport>, info: UsbDeviceInfo, model: Option<FilmModel>, calib_cache: Option<PathBuf>) -> Result<Self> {
        let base = match model {
            Some(m) => m,
            None => model_for_device(info.product_id, info.bcd_device)?,
        };
        let model = gl128_firmware_variant(&base, info.serial_number.as_deref());
        if model != base {
            info!(
                "{} firmware {:?} is not {}: using the vendor's plain register set (0x36=0x{:02X})",
                model.name,
                info.serial_number,
                crate::device::model::GL128_FIRMWARE_CURRENT,
                model.init_regs.reg(0x36)
            );
        }
        let asic = Asic::create(transport, model.clone())?;
        let calibrator = Calibrator::new(model.clone(), calib_cache, false);
        let armed = model_is_scan_ready(&model);
        Ok(Self {
            info,
            model,
            asic,
            calibrator,
            closed: false,
            last_me_debug: None,
            last_multi_pass_debug: None,
            last_align_shift_ir: None,
            bringup_motor_armed: armed,
            allow_unvalidated_scan: false,
        })
    }

    /// Open a scan-ready device when present, else the first matching OpticFilm.
    pub fn open(device_id: Option<&str>, calib_cache: Option<PathBuf>) -> Result<Self> {
        let handle = UsbDeviceHandle::open(device_id)?;
        let info = handle.info.clone();
        let s = Self::new(Box::new(handle), info, None, calib_cache)?;
        info!("Scanner open: {} model={} asic={} scan_ready={}", s.info.device_id(), s.model.name, s.model.asic, model_is_scan_ready(&s.model));
        Ok(s)
    }

    /// Open `model` against a mock USB device (no hardware). Does not change
    /// `model.scan_ready`; real [`Scanner::open`] stays gated.
    pub fn open_fake(model: FilmModel, transport: Option<Box<dyn UsbTransport>>, calib_cache: Option<PathBuf>) -> Result<Self> {
        let inner: Box<dyn UsbTransport> = match transport {
            Some(t) => t,
            None if model.kind == ModelKind::Opticfilm7600iV1 => Box::new(SimulatedGl843V1Transport::default()),
            None => Box::new(MockScannerTransport::new()),
        };
        let info = UsbDeviceInfo {
            vendor_id: if model.usb_vendor_id != 0 { model.usb_vendor_id as u16 } else { VID_PLUSTEK },
            product_id: model.usb_product_id as u16,
            bus: Some(0),
            address: Some(0),
            product: Some(model.model.to_string()),
            manufacturer: Some(model.vendor.to_string()),
            ..Default::default()
        };
        let mut s = Self::new(inner, info, Some(model), calib_cache)?;
        s.allow_unvalidated_scan = true;
        s.bringup_motor_armed = true;
        if let Asic::Gl128(a) = &mut s.asic {
            a.motor_moves_enabled = true;
        }
        info!("Scanner open_fake: model={} asic={} (mock USB)", s.model.name, s.model.asic);
        Ok(s)
    }

    pub fn close(&mut self) {
        if self.closed {
            return;
        }
        if self.asic.initialized() {
            if let Err(e) = self.asic.lamp_off().and_then(|()| self.asic.stop_motor()) {
                debug!("close cleanup: {e}");
            }
        }
        self.closed = true;
    }

    pub fn device_id(&self) -> String {
        self.info.device_id()
    }

    pub fn info(&self) -> &UsbDeviceInfo {
        &self.info
    }

    pub fn model(&self) -> &FilmModel {
        &self.model
    }

    pub fn asic(&mut self) -> Result<&mut Asic> {
        self.ensure_open()?;
        Ok(&mut self.asic)
    }

    pub fn calibrator(&mut self) -> Result<&mut Calibrator> {
        self.ensure_open()?;
        Ok(&mut self.calibrator)
    }

    /// The USB transport (tests read fake transports through this).
    pub fn transport(&mut self) -> &mut dyn UsbTransport {
        self.asic.transport()
    }

    // --- advanced registers (debug / bring-up only) ---

    pub fn read_register(&mut self, address: u16) -> Result<u8> {
        self.ensure_open()?;
        self.asic.read_register(address)
    }

    pub fn write_register(&mut self, address: u16, value: u8) -> Result<()> {
        self.ensure_open()?;
        self.asic.write_register(address, value)
    }

    pub fn write_registers(&mut self, pairs: &[(u16, u8)]) -> Result<()> {
        self.ensure_open()?;
        self.asic.write_registers(pairs)
    }

    pub fn read_registers(&mut self, addresses: &[u16]) -> Result<Vec<(u16, u8)>> {
        addresses.iter().map(|&a| Ok((a, self.read_register(a)?))).collect()
    }

    // --- device control ---

    pub fn status(&mut self) -> Result<ScannerStatus> {
        self.ensure_open()?;
        match &mut self.asic {
            Asic::Gl845(a) => a.read_status_reliable(),
            Asic::Gl128(a) => a.read_status_reliable(),
            Asic::Gl843V1(a) => a.read_status_reliable(),
        }
    }

    /// QuickScan / IntelliScan buttons (GL128 only).
    pub fn button_state(&mut self, timeout_ms: Option<u32>) -> Result<HashSet<Button>> {
        match &mut self.asic {
            Asic::Gl128(a) => a.button_state(timeout_ms.unwrap_or(5000)),
            _ => Err(Error::NotImplemented(format!("{} has no button endpoint support", self.model.model))),
        }
    }

    /// ASIC boot, frontend init, optional home + lamp on.
    pub fn warmup(&mut self, home: bool, lamp: bool) -> Result<()> {
        self.ensure_scan_ready()?;
        self.ensure_open()?;
        self.asic.init(false)?;
        if home {
            self.asic.home(30.0)?;
        }
        if lamp {
            self.asic.set_scan_method(ScanMethod::Transparency);
            self.asic.lamp_on()?;
        }
        Ok(())
    }

    /// Turn the lamp on. Allowed on probe-only models that implement lamp.
    pub fn lamp_on(&mut self, method: ScanMethod) -> Result<()> {
        self.ensure_open()?;
        self.asic.set_scan_method(method);
        self.asic.lamp_on()
    }

    pub fn lamp_off(&mut self) -> Result<()> {
        self.ensure_open()?;
        self.asic.lamp_off()
    }

    pub fn home(&mut self, timeout_s: f64) -> Result<()> {
        self.ensure_scan_ready()?;
        self.ensure_open()?;
        self.asic.home(timeout_s)
    }

    pub fn park(&mut self, timeout_s: f64) -> Result<()> {
        self.ensure_scan_ready()?;
        self.ensure_open()?;
        self.asic.park(timeout_s)
    }

    /// Run dark/white shading (or GL128 IR white-only) and update the cache.
    pub fn calibrate(&mut self, resolution: i64, mode: ScanMode, force: bool, area: Option<Area>, geometry: Option<&ScanGeometry>) -> Result<CalibEntry> {
        self.ensure_scan_ready()?;
        self.ensure_open()?;
        if self.model.kind == ModelKind::Opticfilm7600iV1 {
            return Err(Error::NotImplemented(format!("{} calibrates within each scan", self.model.model)));
        }
        if !self.asic.initialized() {
            self.asic.init(false)?;
            if !self.asic.is_at_home()? {
                self.asic.home(30.0)?;
            }
        }
        let was_armed = self.bringup_motor_armed;
        let r = (|| {
            if let Asic::Gl128(a) = &mut self.asic {
                self.bringup_motor_armed = false;
                a.motor_moves_enabled = false;
                a.enter_scan_clock()?;
            }
            self.calibrator.run(&mut self.asic, resolution, mode, force, area, geometry)
        })();
        if was_armed {
            self.arm_bringup_motor();
        }
        r
    }

    /// Bracket planes / IVW stats from the last ME scan, or `None`.
    pub fn last_me_debug(&self) -> Option<&MeScanDebug> {
        self.last_me_debug.as_ref()
    }

    /// Per-slot Multi-Pass stacking stats from the last scan with `n_passes > 1`.
    pub fn last_multi_pass_debug(&self) -> Option<&MultiPassDebug> {
        self.last_multi_pass_debug.as_ref()
    }

    /// IR→colour-short alignment shift from the last multi-pass scan.
    pub fn last_align_shift_ir(&self) -> Option<(f64, f64)> {
        self.last_align_shift_ir
    }

    /// Scan one frame. See [`ScanRequest`] for the arguments.
    pub fn scan(&mut self, req: &ScanRequest, ctl: &mut ScanControl) -> Result<ScanImage> {
        // Fail fast on bad manual-exposure input before touching the ASIC.
        validate_manual_exposure(req.single_pass_exposure, "single_pass_exposure")?;
        validate_manual_exposure(req.me_short_exposure, "me_short_exposure")?;
        validate_manual_exposure(req.me_long_exposure, "me_long_exposure")?;
        validate_n_passes(req.n_passes)?;
        if self.model.asic == "GL128" {
            check_gl128_multi_pass(&self.model, req.resolution(), req.multi_exposure, req.n_passes)?;
        }
        self.ensure_scan_ready()?;
        self.ensure_open()?;
        if !self.asic.initialized() {
            self.asic.init(false)?;
            if !self.asic.is_at_home()? {
                self.asic.home(30.0)?;
            }
        }
        if let Some(f) = ctl.on_status.as_mut() {
            f("scanning");
        }
        let model = self.model.clone();
        let (image, me, mp, ir) = match &mut self.asic {
            Asic::Gl128(a) => {
                let mut s = Gl128ScanSession::new(model);
                let img = s.run(a, Some(&mut self.calibrator), req, ctl)?;
                (img, s.last_me_debug, s.last_multi_pass_debug, s.last_align_shift_ir)
            }
            Asic::Gl843V1(a) => {
                let mut s = Gl843V1ScanSession::new(model);
                (s.run(a, req, ctl, None)?, None, None, None)
            }
            Asic::Gl845(a) => {
                let mut s = ScanSession::new(model, a.variant);
                let img = s.run(a, Some(&mut self.calibrator), req, ctl)?;
                (img, s.last_me_debug, None, s.last_align_shift_ir)
            }
        };
        self.last_me_debug = me;
        self.last_multi_pass_debug = mp;
        self.last_align_shift_ir = ir;
        Ok(image)
    }

    /// Enable GL128 motor moves (default on for scan-ready GL128).
    pub fn arm_bringup_motor(&mut self) {
        self.bringup_motor_armed = true;
        if let Asic::Gl128(a) = &mut self.asic {
            a.motor_moves_enabled = true;
        }
        debug!("Motor armed for {}", self.model.model);
    }

    /// Temporarily disable GL128 motor moves (stationary shading safety).
    pub fn disarm_bringup_motor(&mut self) {
        self.bringup_motor_armed = false;
        if let Asic::Gl128(a) = &mut self.asic {
            a.motor_moves_enabled = false;
        }
    }

    fn ensure_scan_ready(&self) -> Result<()> {
        if self.allow_unvalidated_scan {
            return Ok(());
        }
        if self.bringup_motor_armed && self.model.asic == "GL128" {
            return Ok(());
        }
        if !model_is_scan_ready(&self.model) {
            return Err(Error::Asic(format!(
                "{} ({}) is locked out in this release: only OpticFilm 8200i SE (07b3:1825), OpticFilm 8100 (V2) \
                 (07b3:1824), OpticFilm 8300i SE (07b3:181f), and OpticFilm 7600i v1 and v2 (07b3:0c3b) are validated \
                 for scanning. Open, status, lamp and register dumps still work.",
                self.model.model, self.model.asic
            )));
        }
        Ok(())
    }

    fn ensure_open(&self) -> Result<()> {
        if self.closed {
            return Err(Error::Usb("Scanner is closed.".into()));
        }
        Ok(())
    }
}

impl Drop for Scanner {
    fn drop(&mut self) {
        self.close();
    }
}
