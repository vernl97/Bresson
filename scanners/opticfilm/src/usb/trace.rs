// SPDX-License-Identifier: GPL-3.0-or-later
//! Structured USB transactions and a recording transport wrapper.

use std::any::Any;
use std::sync::OnceLock;
use std::time::Instant;

use serde_json::{Map, Value, json};

use super::correlate;
use super::transport::{ControlData, UsbTransport};
use crate::error::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UsbOp {
    ControlRead,
    ControlWrite,
    BulkRead,
    BulkWrite,
}

impl UsbOp {
    pub fn as_str(self) -> &'static str {
        match self {
            UsbOp::ControlRead => "control_read",
            UsbOp::ControlWrite => "control_write",
            UsbOp::BulkRead => "bulk_read",
            UsbOp::BulkWrite => "bulk_write",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "control_read" => UsbOp::ControlRead,
            "control_write" => UsbOp::ControlWrite,
            "bulk_read" | "bulk_in" => UsbOp::BulkRead,
            "bulk_write" => UsbOp::BulkWrite,
            _ => return None,
        })
    }
}

/// One USB transfer as seen at the [`UsbTransport`] boundary.
#[derive(Debug, Clone, PartialEq)]
pub struct UsbTransaction {
    pub operation: UsbOp,
    pub request_type: Option<u8>,
    pub request: Option<u8>,
    pub value: Option<u16>,
    pub index: Option<u16>,
    pub endpoint: Option<u8>,
    pub data: Option<Vec<u8>>,
    pub length: Option<usize>,
    pub timeout_ms: Option<u32>,
    /// Host-side monotonic timestamps in seconds.
    pub t0: Option<f64>,
    pub t1: Option<f64>,
    pub response: Option<Vec<u8>>,
    pub error: Option<String>,
    pub investigation_id: Option<String>,
    pub experiment_id: Option<String>,
    pub run_id: Option<String>,
    pub scan_id: Option<String>,
    pub operation_id: Option<String>,
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

pub fn from_hex(s: &str) -> Option<Vec<u8>> {
    if s.len() % 2 != 0 {
        return None;
    }
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok()).collect()
}

impl UsbTransaction {
    pub fn new(operation: UsbOp) -> Self {
        Self {
            operation,
            request_type: None,
            request: None,
            value: None,
            index: None,
            endpoint: None,
            data: None,
            length: None,
            timeout_ms: None,
            t0: None,
            t1: None,
            response: None,
            error: None,
            investigation_id: None,
            experiment_id: None,
            run_id: None,
            scan_id: None,
            operation_id: None,
        }
    }

    /// Transaction for a control transfer about to be issued.
    pub fn control(rt: u8, req: u8, value: u16, index: u16, data: ControlData<'_>, timeout_ms: Option<u32>) -> Self {
        let mut t = match data {
            ControlData::In(n) => {
                let mut t = Self::new(UsbOp::ControlRead);
                t.length = Some(n);
                t
            }
            ControlData::Out(d) => {
                let mut t = Self::new(UsbOp::ControlWrite);
                t.data = Some(d.to_vec());
                t
            }
        };
        t.request_type = Some(rt);
        t.request = Some(req);
        t.value = Some(value);
        t.index = Some(index);
        t.timeout_ms = timeout_ms;
        t
    }

    pub fn bulk_read(size: usize, timeout_ms: Option<u32>) -> Self {
        let mut t = Self::new(UsbOp::BulkRead);
        t.endpoint = Some(0x81);
        t.length = Some(size);
        t.timeout_ms = timeout_ms;
        t
    }

    pub fn bulk_write(data: &[u8], timeout_ms: Option<u32>) -> Self {
        let mut t = Self::new(UsbOp::BulkWrite);
        t.endpoint = Some(0x02);
        t.data = Some(data.to_vec());
        t.timeout_ms = timeout_ms;
        t
    }

    pub fn to_json(&self) -> Value {
        let mut out = Map::new();
        out.insert("op".into(), json!(self.operation.as_str()));
        if let Some(t0) = self.t0 {
            out.insert("t0".into(), json!(t0));
        }
        if let Some(t1) = self.t1 {
            out.insert("t1".into(), json!(t1));
            let lat = (t1 - self.t0.unwrap_or(t1)) * 1000.0;
            out.insert("latency_ms".into(), json!((lat * 1000.0).round() / 1000.0));
        }
        if let Some(r) = &self.response {
            out.insert("response".into(), json!(hex(r)));
        }
        if let Some(e) = &self.error {
            out.insert("error".into(), json!(e));
        }
        for (k, v) in [
            ("investigation_id", &self.investigation_id),
            ("experiment_id", &self.experiment_id),
            ("run_id", &self.run_id),
            ("scan_id", &self.scan_id),
            ("operation_id", &self.operation_id),
        ] {
            if let Some(v) = v {
                out.insert(k.into(), json!(v));
            }
        }
        if let Some(v) = self.request_type {
            out.insert("request_type".into(), json!(v));
        }
        if let Some(v) = self.request {
            out.insert("request".into(), json!(v));
        }
        if let Some(v) = self.value {
            out.insert("value".into(), json!(v));
        }
        if let Some(v) = self.index {
            out.insert("index".into(), json!(v));
        }
        if let Some(v) = self.endpoint {
            out.insert("endpoint".into(), json!(v));
        }
        if let Some(d) = &self.data {
            out.insert("data".into(), json!(hex(d)));
        }
        if let Some(v) = self.length {
            out.insert("length".into(), json!(v));
        }
        if let Some(v) = self.timeout_ms {
            out.insert("timeout_ms".into(), json!(v));
        }
        Value::Object(out)
    }

    pub fn from_json(obj: &Value) -> Option<Self> {
        let u = |k: &str| obj.get(k).and_then(Value::as_u64);
        let mut t = Self::new(UsbOp::parse(obj.get("op")?.as_str()?)?);
        t.request_type = u("request_type").map(|v| v as u8);
        t.request = u("request").map(|v| v as u8);
        t.value = u("value").map(|v| v as u16);
        t.index = u("index").map(|v| v as u16);
        t.endpoint = u("endpoint").map(|v| v as u8);
        t.data = obj.get("data").and_then(Value::as_str).and_then(from_hex);
        t.length = u("length").map(|v| v as usize);
        t.timeout_ms = u("timeout_ms").map(|v| v as u32);
        t.response = obj.get("response").and_then(Value::as_str).and_then(from_hex);
        Some(t)
    }
}

/// One-line USB log entry (payloads truncated).
pub fn format_usb_log_line(txn: &UsbTransaction, max_data: usize) -> String {
    let trunc = |d: &[u8]| {
        let shown = hex(&d[..d.len().min(max_data)]);
        let extra = if d.len() > max_data { format!("…(+{}b)", d.len() - max_data) } else { String::new() };
        format!("{shown}{extra}")
    };
    match txn.operation {
        UsbOp::BulkRead => format!(
            "bulk_read  ep=0x{:02x} length={}",
            txn.endpoint.unwrap_or(0),
            txn.length.map_or("None".into(), |l| l.to_string())
        ),
        UsbOp::BulkWrite => {
            let p = txn.data.as_deref().unwrap_or(&[]);
            format!("bulk_write ep=0x{:02x} {}b {}", txn.endpoint.unwrap_or(0), p.len(), trunc(p))
        }
        _ => {
            let mut bits = vec![
                txn.operation.as_str().to_string(),
                format!("type=0x{:02x}", txn.request_type.unwrap_or(0)),
                format!("req=0x{:02x}", txn.request.unwrap_or(0)),
                format!("val=0x{:04x}", txn.value.unwrap_or(0)),
                format!("idx=0x{:04x}", txn.index.unwrap_or(0)),
            ];
            if let Some(l) = txn.length {
                bits.push(format!("len={l}"));
            }
            if let Some(d) = txn.data.as_deref().filter(|d| !d.is_empty()) {
                bits.push(format!("data={}", trunc(d)));
            }
            bits.join(" ")
        }
    }
}

fn perf_counter() -> f64 {
    static START: OnceLock<Instant> = OnceLock::new();
    START.get_or_init(Instant::now).elapsed().as_secs_f64()
}

pub type TxnCallback = Box<dyn FnMut(&UsbTransaction) + Send>;

/// Wrap a [`UsbTransport`] and record every transfer in order.
///
/// `keep_in_memory = false` drops each transaction after it is handed to
/// `sink`/`listener` instead of accumulating it in `transactions`.
pub struct RecordingTransport {
    pub inner: Box<dyn UsbTransport>,
    pub listener: Option<TxnCallback>,
    pub sink: Option<TxnCallback>,
    pub keep_in_memory: bool,
    pub transactions: Vec<UsbTransaction>,
}

impl RecordingTransport {
    pub fn new(inner: Box<dyn UsbTransport>) -> Self {
        Self { inner, listener: None, sink: None, keep_in_memory: true, transactions: Vec::new() }
    }

    fn record(&mut self, mut txn: UsbTransaction) {
        let corr = correlate::current();
        txn.investigation_id = corr.investigation_id;
        txn.experiment_id = corr.experiment_id;
        txn.run_id = corr.run_id;
        txn.scan_id = corr.scan_id;
        txn.operation_id = corr.operation_id;
        if let Some(s) = self.sink.as_mut() {
            s(&txn);
        }
        if let Some(l) = self.listener.as_mut() {
            l(&txn);
        }
        if self.keep_in_memory {
            self.transactions.push(txn);
        }
    }

    fn finish<T>(&mut self, mut txn: UsbTransaction, r: Result<T>, on_ok: impl FnOnce(&mut UsbTransaction, &T)) -> Result<T> {
        txn.t1 = Some(perf_counter());
        match &r {
            Ok(v) => on_ok(&mut txn, v),
            Err(e) => txn.error = Some(format!("{e:?}")),
        }
        self.record(txn);
        r
    }
}

impl UsbTransport for RecordingTransport {
    fn control_msg(&mut self, rt: u8, req: u8, value: u16, index: u16, data: ControlData<'_>, timeout_ms: Option<u32>) -> Result<Vec<u8>> {
        let is_read = data.is_read();
        let mut txn = UsbTransaction::control(rt, req, value, index, data, timeout_ms);
        txn.t0 = Some(perf_counter());
        let r = self.inner.control_msg(rt, req, value, index, data, timeout_ms);
        self.finish(txn, r, |t, v| {
            if is_read {
                t.response = Some(v.clone());
            }
        })
    }

    fn bulk_read(&mut self, size: usize, timeout_ms: Option<u32>) -> Result<Vec<u8>> {
        let mut txn = UsbTransaction::bulk_read(size, timeout_ms);
        txn.t0 = Some(perf_counter());
        let r = self.inner.bulk_read(size, timeout_ms);
        self.finish(txn, r, |_, _| {})
    }

    fn bulk_write(&mut self, data: &[u8], timeout_ms: Option<u32>) -> Result<usize> {
        let mut txn = UsbTransaction::bulk_write(data, timeout_ms);
        txn.t0 = Some(perf_counter());
        let r = self.inner.bulk_write(data, timeout_ms);
        self.finish(txn, r, |_, _| {})
    }

    fn read_intr(&mut self, timeout_ms: Option<u32>) -> Result<Vec<u8>> {
        self.inner.read_intr(timeout_ms)
    }

    fn abort_bulk_in(&mut self) -> Option<usize> {
        Some(self.inner.abort_bulk_in().unwrap_or(0))
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn inner_mut(&mut self) -> Option<&mut dyn UsbTransport> {
        Some(self.inner.as_mut())
    }
}
