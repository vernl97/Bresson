// SPDX-License-Identifier: GPL-3.0-or-later
//! Raw [`UsbTransaction`] → semantic event, confidence-tagged.
//!
//! Register read/write and feed-probe framing are PROVEN against real
//! captures; what a given address *means* physically is not answered here.

use serde_json::{Map, Value, json};

use super::protocol::{
    REQUEST_BUFFER, REQUEST_REGISTER, REQUEST_TYPE_IN, REQUEST_TYPE_OUT, VALUE_BUFFER,
    VALUE_GET_REGISTER, VALUE_SET_REGISTER,
};
use super::trace::{UsbOp, UsbTransaction};

pub const DECODER_NAME: &str = "gl128_control_decoder";
pub const DECODER_VERSION: &str = "1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Confidence {
    Proven,
    Strong,
    Likely,
    Speculative,
    Unknown,
}

impl Confidence {
    pub fn as_str(self) -> &'static str {
        match self {
            Confidence::Proven => "PROVEN",
            Confidence::Strong => "STRONG",
            Confidence::Likely => "LIKELY",
            Confidence::Speculative => "SPECULATIVE",
            Confidence::Unknown => "UNKNOWN",
        }
    }
}

#[derive(Debug, Clone)]
pub struct DecodedEvent<'a> {
    /// reg_write | reg_read | probe_read | buffer_preamble | bulk_in | bulk_out | unclassified_*
    pub kind: &'static str,
    pub fields: Map<String, Value>,
    pub confidence: Confidence,
    pub raw: &'a UsbTransaction,
}

fn hexs(v: u32) -> String {
    format!("{v:#x}")
}

impl DecodedEvent<'_> {
    pub fn to_json(&self) -> Value {
        let mut out = Map::new();
        out.insert("kind".into(), json!(self.kind));
        out.insert("fields".into(), Value::Object(self.fields.clone()));
        out.insert("confidence".into(), json!(self.confidence.as_str()));
        out.insert("decoder_name".into(), json!(DECODER_NAME));
        out.insert("decoder_version".into(), json!(DECODER_VERSION));
        if let Some(t0) = self.raw.t0 {
            out.insert("raw_t0".into(), json!(t0));
        }
        if let Some(t1) = self.raw.t1 {
            out.insert("raw_t1".into(), json!(t1));
        }
        if let Some(e) = &self.raw.error {
            out.insert("error".into(), json!(e));
        }
        for (k, v) in [
            ("investigation_id", &self.raw.investigation_id),
            ("experiment_id", &self.raw.experiment_id),
            ("run_id", &self.raw.run_id),
            ("scan_id", &self.raw.scan_id),
            ("operation_id", &self.raw.operation_id),
        ] {
            if let Some(v) = v {
                out.insert(k.into(), json!(v));
            }
        }
        Value::Object(out)
    }
}

/// Decode one transaction. Bulk payloads are not decoded byte by byte.
pub fn decode_transaction(txn: &UsbTransaction) -> Option<DecodedEvent<'_>> {
    let ev = |kind, fields: Map<String, Value>, confidence| DecodedEvent {
        kind,
        fields,
        confidence,
        raw: txn,
    };
    match txn.operation {
        UsbOp::ControlWrite => Some(decode_control_write(txn)),
        UsbOp::ControlRead => Some(decode_control_read(txn)),
        UsbOp::BulkRead => {
            let mut f = Map::new();
            f.insert(
                "length".into(),
                json!(txn.length.or(txn.data.as_ref().map(Vec::len))),
            );
            Some(ev("bulk_in", f, Confidence::Proven))
        }
        UsbOp::BulkWrite => {
            let mut f = Map::new();
            f.insert(
                "length".into(),
                json!(txn.data.as_ref().map_or(0, Vec::len)),
            );
            Some(ev("bulk_out", f, Confidence::Proven))
        }
    }
}

fn unclassified<'a>(txn: &'a UsbTransaction, kind: &'static str) -> DecodedEvent<'a> {
    let mut f = Map::new();
    f.insert("request_type".into(), json!(txn.request_type));
    f.insert("request".into(), json!(txn.request));
    f.insert("value".into(), json!(txn.value));
    f.insert("index".into(), json!(txn.index));
    DecodedEvent {
        kind,
        fields: f,
        confidence: Confidence::Unknown,
        raw: txn,
    }
}

fn decode_control_write(txn: &UsbTransaction) -> DecodedEvent<'_> {
    let data = txn.data.as_deref().unwrap_or(&[]);
    let value = txn.value.unwrap_or(0);
    let cmd = value & 0xFF;
    let high = value & 0x100 != 0;
    let (rt, req) = (txn.request_type, txn.request);
    if rt == Some(REQUEST_TYPE_OUT) && req == Some(REQUEST_BUFFER) && cmd == VALUE_SET_REGISTER {
        let pairs: Vec<Value> = data
            .chunks_exact(2)
            .map(|p| {
                let addr = p[0] as u32 | if high { 0x100 } else { 0 };
                json!({"addr": hexs(addr), "value": p[1]})
            })
            .collect();
        let mut f = Map::new();
        f.insert("pairs".into(), Value::Array(pairs));
        return DecodedEvent {
            kind: "reg_write",
            fields: f,
            confidence: Confidence::Proven,
            raw: txn,
        };
    }
    if rt == Some(REQUEST_TYPE_OUT)
        && req == Some(REQUEST_BUFFER)
        && cmd == VALUE_BUFFER
        && data.len() >= 8
    {
        let bulk_addr = u32::from_le_bytes(data[0..4].try_into().unwrap());
        let bulk_size = u32::from_le_bytes(data[4..8].try_into().unwrap());
        let mut f = Map::new();
        f.insert("w_index".into(), json!(hexs(txn.index.unwrap_or(0) as u32)));
        f.insert("bulk_addr".into(), json!(hexs(bulk_addr)));
        f.insert("bulk_size".into(), json!(bulk_size));
        return DecodedEvent {
            kind: "buffer_preamble",
            fields: f,
            confidence: Confidence::Proven,
            raw: txn,
        };
    }
    unclassified(txn, "unclassified_control_write")
}

fn decode_control_read(txn: &UsbTransaction) -> DecodedEvent<'_> {
    let value = txn.value.unwrap_or(0);
    let index = txn.index.unwrap_or(0) as i64;
    let cmd = value & 0xFF;
    let response = txn.response.as_deref().unwrap_or(&[]);
    let (rt, req) = (txn.request_type, txn.request);
    if rt == Some(REQUEST_TYPE_IN) && req == Some(REQUEST_BUFFER) && cmd == VALUE_GET_REGISTER {
        // SANE: address16 = 0x22 + (address << 8); invert to recover address.
        let mut addr = (index - 0x22) >> 8;
        if value & 0x100 != 0 {
            addr |= 0x100;
        }
        let mut f = Map::new();
        f.insert("addr".into(), json!(format!("{addr:#x}")));
        if response.len() >= 2 {
            f.insert("value".into(), json!(response[0]));
            f.insert("link_status".into(), json!(response[1]));
        }
        return DecodedEvent {
            kind: "reg_read",
            fields: f,
            confidence: Confidence::Proven,
            raw: txn,
        };
    }
    if rt == Some(REQUEST_TYPE_IN) && req == Some(REQUEST_REGISTER) && cmd == VALUE_GET_REGISTER {
        let mut f = Map::new();
        f.insert("w_index".into(), json!(hexs(index as u32)));
        if let Some(&v) = response.first() {
            f.insert("value".into(), json!(v));
        }
        return DecodedEvent {
            kind: "probe_read",
            fields: f,
            confidence: Confidence::Proven,
            raw: txn,
        };
    }
    unclassified(txn, "unclassified_control_read")
}

/// One-line human-readable rendering, raw bytes not included.
pub fn format_decoded_line(ev: &DecodedEvent<'_>) -> String {
    let f = &ev.fields;
    let byte = |k: &str| {
        f.get(k)
            .and_then(Value::as_u64)
            .map_or("?".to_string(), |v| format!("{v:#04x}"))
    };
    let s = |k: &str| {
        f.get(k).map_or("?".to_string(), |v| {
            v.as_str().map_or(v.to_string(), str::to_string)
        })
    };
    match ev.kind {
        "reg_write" => {
            let pairs: Vec<String> = f
                .get("pairs")
                .and_then(Value::as_array)
                .map(|a| {
                    a.iter()
                        .map(|p| {
                            format!(
                                "{}={:#04x}",
                                p["addr"].as_str().unwrap_or("?"),
                                p["value"].as_u64().unwrap_or(0)
                            )
                        })
                        .collect()
                })
                .unwrap_or_default();
            format!("REG_WRITE  {}", pairs.join(", "))
        }
        "reg_read" => {
            let link = f
                .get("link_status")
                .and_then(Value::as_u64)
                .map_or(String::new(), |l| format!(" link={l:#04x}"));
            format!("REG_READ   {}={}{link}", s("addr"), byte("value"))
        }
        "probe_read" => format!("PROBE_READ {}={}", s("w_index"), byte("value")),
        "buffer_preamble" => format!(
            "BULK_PREAMBLE addr={} size={}",
            s("bulk_addr"),
            s("bulk_size")
        ),
        "bulk_in" => format!("BULK_IN    {}b", s("length")),
        "bulk_out" => format!("BULK_OUT   {}b", s("length")),
        _ => format!(
            "UNCLASSIFIED {} {} [confidence={}]",
            ev.kind,
            py_fields_repr(f),
            ev.confidence.as_str()
        ),
    }
}

/// Python `repr` of an unclassified event's fields dict, in its insertion order.
fn py_fields_repr(f: &Map<String, Value>) -> String {
    const ORDER: [&str; 4] = ["request_type", "request", "value", "index"];
    let keys = ORDER
        .iter()
        .copied()
        .filter(|k| f.contains_key(*k))
        .chain(f.keys().map(String::as_str).filter(|k| !ORDER.contains(k)));
    let repr = |v: &Value| match v {
        Value::Null => "None".to_string(),
        Value::String(s) => format!("'{s}'"),
        Value::Bool(b) => if *b { "True" } else { "False" }.to_string(),
        other => other.to_string(),
    };
    let items: Vec<String> = keys.map(|k| format!("'{k}': {}", repr(&f[k]))).collect();
    format!("{{{}}}", items.join(", "))
}
