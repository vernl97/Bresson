// SPDX-License-Identifier: GPL-3.0-or-later
//! Port of the timing tests in `test_gl128_acquire_read.py`. One test only:
//! it sets `PYOPTICFILM_ACQUIRE_TIMING`, which is process-wide.

mod common;

use std::collections::VecDeque;

use common::caplog;
use opticfilm::asic::Asic;
use opticfilm::asic::registers::{gl128 as se, gl845 as r};
use opticfilm::clock;
use opticfilm::device::model_by_name;
use opticfilm::scan::geometry::{ScanGeometry, compute_geometry};
use opticfilm::scan::request::NoProgress;
use opticfilm::scan::session_gl128::{ACQUIRE_TIMING_ENV, Gl128ScanSession};
use opticfilm::usb::fake::{FakeCore, FakeDevice, FakeUsbTransport};

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

fn run(reads: Vec<Vec<u8>>, total: i64, line: i64) -> opticfilm::Result<Vec<u8>> {
    clock::use_virtual(1000.0);
    // One line per announce, as with the Python mock returning one line per `bulk_read_exact`.
    let mut m = model_by_name("plustek-opticfilm-8200i-se").unwrap();
    m.gl128.as_mut().unwrap().image_read_chunk_bytes = line;
    let g = compute_geometry(7200, &m, None).unwrap();
    let g = ScanGeometry {
        line_bytes: line,
        optical_line_count: total / line,
        ..g
    };
    let usb = FakeUsbTransport::with_device(
        FakeCore::new(),
        Bulk {
            reads: reads.into(),
        },
    );
    let Asic::Gl128(mut a) = Asic::create(Box::new(usb), m.clone()).unwrap() else {
        panic!()
    };
    Gl128ScanSession::new(m).acquire(&mut a, &g, &mut NoProgress)
}

#[test]
fn timing_summary_logged_and_csv_appended_even_when_the_read_fails() {
    let dir = common::tmp_path("acquire-timing");
    let out = dir.join("timing.csv");
    // SAFETY: the only test in this binary, so no other thread reads the environment.
    unsafe { std::env::set_var(ACQUIRE_TIMING_ENV, &out) };

    // Two 8-byte reads of two 4-byte lines each: rows are appended, header once.
    caplog::start();
    for _ in 0..2 {
        run(vec![vec![0; 4], vec![0; 4]], 8, 4).unwrap();
    }
    assert!(caplog::contains(
        log::Level::Info,
        "GL128 image read: 7200 dpi"
    ));
    assert!(caplog::contains(log::Level::Info, "chunks=2"));
    assert!(!caplog::contains(log::Level::Trace, "quiet_drain"));
    let text = std::fs::read_to_string(&out).unwrap();
    let rows: Vec<&str> = text.lines().collect();
    assert_eq!(rows[0], "dpi,offset,bytes,read_s,gap_s");
    assert_eq!(rows.len(), 1 + 4);
    let cols = |row: &str| {
        row.split(',')
            .skip(1)
            .take(2)
            .map(str::to_owned)
            .collect::<Vec<_>>()
    };
    assert_eq!(cols(rows[1]), ["0", "4"]);
    assert_eq!(cols(rows[2]), ["4", "4"]);

    // A read that fails mid-image still writes the chunks it got.
    std::fs::remove_file(&out).unwrap();
    assert!(run(vec![vec![0; 4], vec![]], 12, 4).is_err());
    let text = std::fs::read_to_string(&out).unwrap();
    let rows: Vec<&str> = text.lines().collect();
    assert_eq!(rows.len(), 2, "{rows:?}");
    assert_eq!(rows[0], "dpi,offset,bytes,read_s,gap_s");
    assert_eq!(cols(rows[1]), ["0", "4"]);

    unsafe { std::env::remove_var(ACQUIRE_TIMING_ENV) };
}
