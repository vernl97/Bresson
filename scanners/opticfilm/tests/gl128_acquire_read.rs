// SPDX-License-Identifier: GPL-3.0-or-later
//! Port of `test_gl128_acquire_read.py`: GL128 image read fills a
//! preallocated buffer, never sleeps, and announces whole lines per chunk.
//! (The timing CSV tests are in `gl128_acquire_timing.rs`: they set an env var.)

mod common;

use std::collections::VecDeque;

use opticfilm::asic::Asic;
use opticfilm::asic::registers::{gl128 as se, gl845 as r};
use opticfilm::clock::{self, monotonic};
use opticfilm::device::{FilmModel, model_by_name};
use opticfilm::scan::geometry::{ScanGeometry, compute_geometry};
use opticfilm::scan::request::NoProgress;
use opticfilm::scan::session_gl128::Gl128ScanSession;
use opticfilm::usb::fake::{FakeCore, FakeDevice, FakeUsbTransport, NoDevice};
use opticfilm::usb::protocol::GenesysUsbProtocol;
use opticfilm::usb::trace::UsbOp;
use opticfilm::usb::transport::find_transport;

/// Data always ready; bulk IN returns the scripted chunks (any length), then zeros.
struct Bulk {
    reads: VecDeque<Vec<u8>>,
}

impl FakeDevice for Bulk {
    fn before_register_read(&mut self, core: &mut FakeCore, addr: u16) {
        if addr == se::REG_STATUS {
            core.registers
                .insert(se::REG_STATUS, r::STATUS_PWRBIT | r::STATUS_HOMESNR);
        }
    }
    fn bulk_in_bytes(&mut self, _core: &mut FakeCore, size: usize) -> Vec<u8> {
        self.reads.pop_front().unwrap_or_else(|| vec![0; size])
    }
}

type BulkTransport = FakeUsbTransport<Bulk>;

fn se_model() -> FilmModel {
    model_by_name("plustek-opticfilm-8200i-se").unwrap()
}

fn geometry(model: &FilmModel, total: i64, line: i64) -> ScanGeometry {
    let g = compute_geometry(7200, model, None).unwrap();
    assert_eq!(total % line, 0);
    ScanGeometry {
        line_bytes: line,
        optical_line_count: total / line,
        disable_buffer_full_move: false,
        ..g
    }
}

/// Run one `_acquire` (Python patches `_wait_data`; here data is always ready).
fn acquire(
    model: FilmModel,
    reads: Vec<Vec<u8>>,
    g: &ScanGeometry,
) -> (opticfilm::Result<Vec<u8>>, FakeCore) {
    clock::use_virtual(1000.0);
    let usb = FakeUsbTransport::with_device(
        FakeCore::new(),
        Bulk {
            reads: reads.into(),
        },
    );
    let Asic::Gl128(mut a) = Asic::create(Box::new(usb), model.clone()).unwrap() else {
        panic!()
    };
    let raw = Gl128ScanSession::new(model).acquire(&mut a, g, &mut NoProgress);
    let core = std::mem::take(
        &mut find_transport::<BulkTransport>(a.protocol.transport())
            .unwrap()
            .core,
    );
    (raw, core)
}

/// Announced sizes (`bulk_read_begin` preambles) and bulk IN request sizes.
fn traffic(core: &FakeCore) -> (Vec<usize>, Vec<usize>) {
    let mut begins = vec![];
    let mut reads = vec![];
    for x in &core.transactions {
        match x.operation {
            UsbOp::ControlWrite if x.index == Some(se::BULK_INDEX_IMAGE as u16) => {
                let d = x.data.as_ref().unwrap();
                if d.len() == 8 {
                    begins.push(u32::from_le_bytes([d[4], d[5], d[6], d[7]]) as usize);
                }
            }
            UsbOp::BulkRead => reads.push(x.length.unwrap()),
            _ => {}
        }
    }
    (begins, reads)
}

#[test]
fn short_reads_fill_the_preallocated_buffer_in_order() {
    let g = geometry(&se_model(), 10, 2);
    let (raw, _) = acquire(se_model(), vec![vec![1; 3], vec![2; 5], vec![3; 2]], &g);
    let want: Vec<u8> = [vec![1u8; 3], vec![2; 5], vec![3; 2]].concat();
    assert_eq!(raw.unwrap(), want);
}

#[test]
fn overlong_last_read_is_truncated_to_total() {
    let g = geometry(&se_model(), 6, 6);
    let (raw, _) = acquire(se_model(), vec![vec![7; 8]], &g);
    assert_eq!(raw.unwrap(), vec![7u8; 6]);
}

#[test]
fn empty_read_raises() {
    let g = geometry(&se_model(), 10, 2);
    let (raw, _) = acquire(se_model(), vec![vec![0; 4], vec![]], &g);
    let err = raw.unwrap_err();
    assert!(err.to_string().contains("4 of 10"), "{err}");
}

#[test]
fn acquire_never_sleeps() {
    let g = geometry(&se_model(), 3 * 4096, 4096);
    clock::use_virtual(1000.0);
    let t0 = monotonic();
    let usb = FakeUsbTransport::with_device(
        FakeCore::new(),
        Bulk {
            reads: VecDeque::new(),
        },
    );
    let Asic::Gl128(mut a) = Asic::create(Box::new(usb), se_model()).unwrap() else {
        panic!()
    };
    Gl128ScanSession::new(se_model())
        .acquire(&mut a, &g, &mut NoProgress)
        .unwrap();
    assert_eq!(monotonic(), t0);
}

#[test]
fn one_announce_per_full_width_7200_line() {
    let line = 62208; // 10368 px x 3 channels x 2 bytes at 7200 dpi
    for name in ["plustek-opticfilm-8200i-se", "plustek-opticfilm-8300i-se"] {
        let m = model_by_name(name).unwrap();
        let g = geometry(&m, 3 * line, line);
        let (raw, t) = acquire(m, vec![], &g);
        raw.unwrap();
        let (begins, reads) = traffic(&t);
        assert_eq!(begins, vec![line as usize; 3], "{name}");
        // Default max transfer: 0xF000-byte requests.
        assert_eq!(reads, [0xF000, line as usize - 0xF000].repeat(3), "{name}");
    }
}

#[test]
fn large_chunk_model_passes_max_transfer() {
    let line = 62208usize;
    let mut m = model_by_name("plustek-opticfilm-8300i-se").unwrap();
    m.gl128.as_mut().unwrap().image_read_chunk_bytes = 0x80000;
    let g = geometry(&m, 20 * line as i64, line as i64);
    let (raw, t) = acquire(m, vec![], &g);
    raw.unwrap();
    let (begins, reads) = traffic(&t);
    assert_eq!(begins, [8 * line, 8 * line, 4 * line]);
    // One request per announce: each fits in max_transfer=0x80000.
    assert_eq!(reads, [8 * line, 8 * line, 4 * line]);
}

#[test]
fn bulk_read_exact_max_transfer_sets_request_size() {
    let sizes = |n: usize, max: usize| {
        let mut p = GenesysUsbProtocol::new(Box::new(FakeUsbTransport::<NoDevice>::new()));
        p.bulk_read_exact(n, max).unwrap();
        let t = find_transport::<FakeUsbTransport>(p.transport()).unwrap();
        t.core
            .transactions
            .iter()
            .map(|x| x.length.unwrap())
            .collect::<Vec<_>>()
    };
    assert_eq!(sizes(500_000, 0x80000), [500_000]);
    assert_eq!(sizes(62208, 0xF000), [0xF000, 62208 - 0xF000]);
}
