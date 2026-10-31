// SPDX-License-Identifier: GPL-3.0-or-later
//! Port of `test_gl128_feed_settle.py`: GL128 feeds hand off like SilverFast
//! (power off, wait for MOTORENB to clear, then arm), and stalled feeds give up.
//!
//! Python patches `Gl128._write`, `read_status_reliable` and the feed probe;
//! here [`Motor`] plays the same motor behind the USB mock.

mod common;

use opticfilm::asic::registers::{gl128 as se, gl845 as r};
use opticfilm::asic::{Asic, Gl128};
use opticfilm::clock::{self, monotonic};
use opticfilm::device::{FilmModel, model_by_name};
use opticfilm::error::Error;
use opticfilm::usb::fake::{FakeCore, FakeDevice, FakeUsbTransport, MockScanner, MockScannerTransport};
use opticfilm::usb::transport::find_transport;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Ev {
    Reg02(u8),
    Start,
    FeedL(u32),
    /// One status register read and whether MOTORENB was reported.
    Status(bool),
}

/// Session-04 behaviour: the probe reads done while MOTORENB stays set until power drops.
struct Motor {
    /// MOTORENB clears `idle_polls` status reads after motor power drops.
    stops: bool,
    /// The step count runs out (probe `0x04`) three status reads after START.
    finishes: bool,
    idle_polls: u32,
    enabled: bool,
    running: bool,
    polls: u32,
    unpowered_polls: u32,
    feeds: u32,
    log: Vec<Ev>,
}

type MotorTransport = FakeUsbTransport<Motor>;

impl Motor {
    fn new(stops: bool, idle_polls: u32) -> Self {
        Self { stops, finishes: true, idle_polls, enabled: false, running: false, polls: 0, unpowered_polls: 0, feeds: 0, log: vec![] }
    }

    fn guard(&self, what: &str, value: u8) {
        // Dropping MTRPWR is the one write allowed on a moving motor.
        if what == "0x02" && value & se::MTRPWR == 0 {
            return;
        }
        assert!(!self.enabled, "{what}={value:#04x} written while MOTORENB is set");
    }
}

impl FakeDevice for Motor {
    fn before_register_read(&mut self, core: &mut FakeCore, addr: u16) {
        if addr != se::REG_STATUS {
            return;
        }
        if self.running {
            self.polls += 1;
        } else if self.enabled && self.stops {
            self.unpowered_polls += 1;
            if self.unpowered_polls >= self.idle_polls {
                self.enabled = false;
            }
        }
        self.log.push(Ev::Status(self.enabled));
        let mut v = r::STATUS_PWRBIT | r::STATUS_BUFEMPTY;
        if self.enabled {
            v |= r::STATUS_MOTORENB;
        }
        if !self.running {
            v |= r::STATUS_FEEDFSH;
        }
        if self.feeds == 0 {
            v |= r::STATUS_HOMESNR;
        }
        core.registers.insert(se::REG_STATUS, v);
    }

    fn before_probe_read(&mut self, core: &mut FakeCore, index: u16) {
        if index == 0x21 {
            let done = !self.running || (self.finishes && self.polls >= 3);
            core.request_register.insert(0x21, if done { 0x04 } else { 0x00 });
        }
    }

    fn after_register_write(&mut self, core: &mut FakeCore, addr: u16, value: u8) {
        if addr == se::REG_0X02 {
            self.guard("0x02", value);
            if value & se::MTRPWR == 0 {
                self.unpowered_polls = 0;
                self.running = false;
            }
            self.log.push(Ev::Reg02(value));
        } else if addr == se::REG_START {
            self.guard("start", value);
            self.log.push(Ev::Start);
            self.enabled = true;
            self.running = true;
            self.polls = 0;
            self.feeds += 1;
        } else if addr == se::REG_FEEDL + 2 {
            let g = |a: u16| core.reg(a) as u32;
            self.log.push(Ev::FeedL((g(se::REG_FEEDL) << 16) | (g(se::REG_FEEDL + 1) << 8) | g(se::REG_FEEDL + 2)));
        }
    }
}

fn se_model() -> FilmModel {
    model_by_name("plustek-opticfilm-8200i-se").unwrap()
}

fn asic_with(motor: Motor) -> Gl128 {
    clock::use_virtual(1000.0);
    let usb: MotorTransport = FakeUsbTransport::with_device(FakeCore::new(), motor);
    let Asic::Gl128(mut a) = Asic::create(Box::new(usb), se_model()).unwrap() else { panic!("not GL128") };
    a.motor_moves_enabled = true;
    a
}

fn motor(a: &mut Gl128) -> &mut Motor {
    &mut find_transport::<MotorTransport>(a.protocol.transport()).unwrap().dev
}

#[test]
fn motor_power_drops_before_the_idle_wait() {
    let mut a = asic_with(Motor::new(true, 3));
    a.feed_capture(28292, 1.0, true, true, false, false, None).unwrap();

    let m = motor(&mut a);
    let at = m.log.iter().position(|e| *e == Ev::Reg02(se::FASTFED)).unwrap();
    let tail = &m.log[at..];
    assert_eq!(tail[1], Ev::FeedL(1));
    let statuses: Vec<bool> = tail[2..].iter().filter_map(|e| if let Ev::Status(s) = e { Some(*s) } else { None }).collect();
    assert!(statuses[0]);
    assert!(!statuses.last().unwrap());
    assert!(!m.enabled);
}

/// [`Motor`] asserts that no `0x02` power-up or START lands while MOTORENB is set.
#[test]
fn the_second_feed_arms_only_after_the_motor_is_idle() {
    let mut a = asic_with(Motor::new(true, 6));
    a.feed_capture(28292, 1.0, true, true, false, false, None).unwrap();
    a.feed_capture(13704, 1.0, true, false, false, false, None).unwrap();

    let m = motor(&mut a);
    assert_eq!(m.feeds, 2);
    let second_arm = m.log.iter().position(|e| *e == Ev::FeedL(13704)).unwrap();
    let last_status = m.log[..second_arm].iter().rev().find(|e| matches!(e, Ev::Status(_))).unwrap();
    assert_eq!(*last_status, Ev::Status(false));
}

#[test]
fn a_motor_that_never_stops_raises_and_the_next_feed_is_not_armed() {
    let mut a = asic_with(Motor::new(false, 3));
    let err = a.feed_capture(28292, 1.0, true, true, false, false, None).unwrap_err();
    assert!(matches!(err, Error::MotorTimeout(_)) && err.to_string().contains("after a 28292-step feed"), "{err}");
    assert_eq!(motor(&mut a).feeds, 1);
}

#[test]
fn a_feed_refuses_to_arm_a_moving_motor() {
    let mut m = Motor::new(false, 3);
    m.enabled = true;
    let mut a = asic_with(m);
    let err = a.feed_capture(13704, 1.0, true, false, false, false, None).unwrap_err();
    assert!(matches!(err, Error::MotorTimeout(_)) && err.to_string().contains("before arming a 13704-step feed"), "{err}");
    assert!(!motor(&mut a).log.iter().any(|e| matches!(e, Ev::Start | Ev::FeedL(_))));
}

#[test]
fn the_idle_wait_returns_once_the_motor_bit_clears() {
    // Raw status reads 1-4 report MOTORENB, read 5 clears it: three reliable (double) reads.
    let mut m = Motor::new(true, 5);
    m.enabled = true;
    let mut a = asic_with(m);
    a.wait_motor_idle("before the test").unwrap();
    let reads = motor(&mut a).log.iter().filter(|e| matches!(e, Ev::Status(_))).count();
    assert_eq!(reads, 6);
}

/// The reference feed's timeout is short (Python: `_REFERENCE_FEED_TIMEOUT_S < 3`),
/// so a runaway carriage stops long before the end stop, and motor power drops.
#[test]
fn reference_feed_gives_up_long_before_a_runaway_reaches_the_end_stop() {
    let mut m = Motor::new(true, 3);
    m.finishes = false;
    let mut a = asic_with(m);
    let t0 = monotonic();
    let err = a.position_for_full_frame_scan(Some(13704), 60.0, None).unwrap_err();
    let waited = monotonic() - t0;
    assert!(matches!(err, Error::MotorTimeout(_)) && err.to_string().contains("did not finish within"), "{err}");
    assert!((2.0..3.0).contains(&waited), "gave up after {waited:.2}s");
    let m = motor(&mut a);
    assert_eq!(m.feeds, 1);
    let Some(Ev::Reg02(last)) = m.log.iter().rev().find(|e| matches!(e, Ev::Reg02(_))) else { panic!("no 0x02 write") };
    assert_eq!(last & se::MTRPWR, 0, "motor power left on");
}

/// Records the value of `0xAF` at every START on the regular mock.
struct AfAtStart {
    inner: MockScanner,
    af: Vec<u8>,
}

impl FakeDevice for AfAtStart {
    fn before_register_read(&mut self, core: &mut FakeCore, addr: u16) {
        self.inner.before_register_read(core, addr);
    }
    fn before_probe_read(&mut self, core: &mut FakeCore, index: u16) {
        self.inner.before_probe_read(core, index);
    }
    fn after_register_write(&mut self, core: &mut FakeCore, addr: u16, value: u8) {
        if addr == se::REG_START {
            self.af.push(core.reg(0xAF));
        }
        self.inner.after_register_write(core, addr, value);
    }
}

fn positioned(name: &str) -> (Gl128, Vec<u8>, bool) {
    clock::use_virtual(1000.0);
    let MockScannerTransport { core, dev } = MockScannerTransport::new();
    let usb = FakeUsbTransport::with_device(core, AfAtStart { inner: dev, af: vec![] });
    let Asic::Gl128(mut a) = Asic::create(Box::new(usb), model_by_name(name).unwrap()).unwrap() else { panic!() };
    a.motor_moves_enabled = true;
    a.position_for_full_frame_scan(Some(13128), 60.0, None).unwrap();
    let t = find_transport::<FakeUsbTransport<AfAtStart>>(a.protocol.transport()).unwrap();
    let (af, home) = (t.dev.af.clone(), t.dev.inner.at_home);
    (a, af, home)
}

/// #102: SilverFast writes 0xAF=0x7F before feed1 and 0xFF before feed2 on every sibling.
#[test]
fn the_scan_start_feed_uses_the_vendor_0xaf() {
    for name in ["plustek-opticfilm-8200i-se", "plustek-opticfilm-8300i-se", "plustek-opticfilm-8100-v2"] {
        let (_, af, _) = positioned(name);
        assert_eq!(af, [0x7F, 0xFF], "{name}");
    }
}

#[test]
fn positioning_pair_still_completes_on_the_mock() {
    let (_, _, at_home) = positioned("plustek-opticfilm-8200i-se");
    assert!(!at_home);
}
