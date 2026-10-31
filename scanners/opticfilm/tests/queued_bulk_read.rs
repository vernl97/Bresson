// SPDX-License-Identifier: GPL-3.0-or-later
//! Queued bulk IN reads (`bulk_read_submit` / `bulk_read_wait`): the 7600i v1
//! replay keeps one read queued, like pyopticfilm's worker thread. The mocks
//! cannot queue, so [`Queuing`] adds it on top of the GL843 v1 simulator: the
//! queued scan must send the same traffic and return the same image as the
//! plain one, with never more than one read in flight.

mod common;

use std::any::Any;
use std::collections::VecDeque;

use opticfilm::device::model_by_name;
use opticfilm::error::Result;
use opticfilm::scan::request::{ScanControl, ScanMode, ScanRequest};
use opticfilm::usb::fake_gl843_v1::SimulatedGl843V1Transport;
use opticfilm::usb::trace::{RecordingTransport, UsbOp, UsbTransaction};
use opticfilm::usb::transport::{ControlData, UsbTransport, find_transport};
use opticfilm::{ScanImage, Scanner, clock};
use serde_json::Value;

/// Queues reads by size; each completes (as a plain read) when waited for.
struct Queuing {
    inner: Box<dyn UsbTransport>,
    queue: VecDeque<usize>,
    submits: usize,
    max_in_flight: usize,
}

impl UsbTransport for Queuing {
    fn control_msg(
        &mut self,
        rt: u8,
        req: u8,
        value: u16,
        index: u16,
        data: ControlData<'_>,
        timeout_ms: Option<u32>,
    ) -> Result<Vec<u8>> {
        assert!(
            self.queue.is_empty(),
            "control transfer while a read is queued"
        );
        self.inner
            .control_msg(rt, req, value, index, data, timeout_ms)
    }
    fn bulk_read(&mut self, size: usize, timeout_ms: Option<u32>) -> Result<Vec<u8>> {
        assert!(self.queue.is_empty(), "plain read while a read is queued");
        self.inner.bulk_read(size, timeout_ms)
    }
    fn bulk_write(&mut self, data: &[u8], timeout_ms: Option<u32>) -> Result<usize> {
        self.inner.bulk_write(data, timeout_ms)
    }
    fn bulk_read_submit(&mut self, size: usize) -> Result<bool> {
        self.queue.push_back(size);
        self.submits += 1;
        self.max_in_flight = self.max_in_flight.max(self.queue.len());
        Ok(true)
    }
    fn bulk_read_wait(&mut self, timeout_ms: Option<u32>) -> Result<Vec<u8>> {
        let size = self.queue.pop_front().expect("wait without a queued read");
        self.inner.bulk_read(size, timeout_ms)
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn inner_mut(&mut self) -> Option<&mut dyn UsbTransport> {
        Some(self.inner.as_mut())
    }
}

fn untimed(t: &UsbTransaction) -> Value {
    let mut v = t.to_json();
    for k in ["t0", "t1", "latency_ms"] {
        v.as_object_mut().unwrap().remove(k);
    }
    v
}

/// Run `req` on the 7600i v1 simulator, optionally through [`Queuing`], recording all traffic.
fn scan(
    req: &ScanRequest,
    queuing: bool,
    tag: &str,
) -> (ScanImage, Vec<Value>, Option<(usize, usize)>) {
    clock::use_virtual(1000.0);
    let sim: Box<dyn UsbTransport> = Box::new(SimulatedGl843V1Transport::default());
    let base: Box<dyn UsbTransport> = if queuing {
        Box::new(Queuing {
            inner: sim,
            queue: VecDeque::new(),
            submits: 0,
            max_in_flight: 0,
        })
    } else {
        sim
    };
    let dir = common::tmp_path(&format!("queued-{tag}-{queuing}"));
    let model = model_by_name("plustek-opticfilm-7600i-v1").unwrap();
    let mut scanner = Scanner::open_fake(
        model,
        Some(Box::new(RecordingTransport::new(base))),
        Some(dir.join("c.json")),
    )
    .unwrap();
    let image = scanner.scan(req, &mut ScanControl::default()).unwrap();
    let t = scanner.transport();
    let stats = find_transport::<Queuing>(t).map(|q| (q.submits, q.max_in_flight));
    let txns = find_transport::<RecordingTransport>(t)
        .unwrap()
        .transactions
        .iter()
        .map(untimed)
        .collect();
    (image, txns, stats)
}

#[test]
fn queued_replay_reads_match_plain_reads() {
    let cases = [
        (
            "1440",
            ScanRequest {
                resolution: 1440,
                area: Some([0.4, 0.4, 0.45, 0.45]),
                ..Default::default()
            },
        ),
        (
            "3600ir",
            ScanRequest {
                resolution: 3600,
                mode: ScanMode::Infrared,
                apply_calib: false,
                area: Some([0.4, 0.4, 0.42, 0.42]),
                ..Default::default()
            },
        ),
    ];
    for (tag, req) in cases {
        let (plain_img, plain_txns, _) = scan(&req, false, tag);
        let (q_img, q_txns, stats) = scan(&req, true, tag);
        let (submits, max_in_flight) = stats.unwrap();
        let reads = plain_txns
            .iter()
            .filter(|t| t["op"] == UsbOp::BulkRead.as_str())
            .count();
        assert!(
            reads > 1 && submits == reads,
            "{tag}: {submits} queued of {reads} reads"
        );
        assert_eq!(max_in_flight, 1, "{tag}");
        assert_eq!(q_txns.len(), plain_txns.len(), "{tag}: transaction count");
        for (k, (a, b)) in q_txns.iter().zip(&plain_txns).enumerate() {
            assert_eq!(a, b, "{tag} #{k}");
        }
        assert_eq!(q_img.rgb, plain_img.rgb, "{tag}: image");
        assert_eq!(q_img.ir, plain_img.ir, "{tag}: IR");
    }
}
