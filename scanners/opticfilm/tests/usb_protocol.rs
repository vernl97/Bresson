// SPDX-License-Identifier: GPL-3.0-or-later
//! Port of `test_usb.py`: USB identity and Genesys protocol framing (no hardware).
//! The pyusb descriptor-reading tests have no nusb equivalent and are not ported.

mod common;

use opticfilm::error::Error;
use opticfilm::usb::device::{
    PID_OPTICFILM_8200I, PID_OPTICFILM_8200I_SE, UsbDeviceInfo, VID_PLUSTEK,
};
use opticfilm::usb::fake::{ControlLogData, FakeUsbTransport, NoDevice};
use opticfilm::usb::protocol::*;
use opticfilm::usb::transport::find_transport;

fn proto() -> GenesysUsbProtocol {
    GenesysUsbProtocol::new(Box::new(FakeUsbTransport::<NoDevice>::new()))
}

fn fake(p: &mut GenesysUsbProtocol) -> &mut FakeUsbTransport {
    find_transport::<FakeUsbTransport>(p.transport()).unwrap()
}

fn written(d: &ControlLogData) -> &[u8] {
    match d {
        ControlLogData::Data(d) => d,
        ControlLogData::Len(_) => panic!("not a write"),
    }
}

#[test]
fn device_id_format() {
    let info = UsbDeviceInfo {
        vendor_id: VID_PLUSTEK,
        product_id: PID_OPTICFILM_8200I,
        bus: Some(1),
        address: Some(8),
        ..Default::default()
    };
    assert_eq!(info.device_id(), "plustek:usb:07b3:130d:001:008");
    assert!(info.is_supported());
}

#[test]
fn se_variant_identified_as_gl128() {
    let info = UsbDeviceInfo {
        vendor_id: VID_PLUSTEK,
        product_id: PID_OPTICFILM_8200I_SE,
        bus: Some(1),
        address: Some(2),
        ..Default::default()
    };
    assert!(info.is_supported());
    assert!(info.is_8200i_se());
    assert_eq!(info.asic_hint(), "GL128");
    let other = UsbDeviceInfo {
        vendor_id: 0x1234,
        product_id: PID_OPTICFILM_8200I_SE,
        ..Default::default()
    };
    assert!(!other.is_supported() && !other.is_8200i_se());
}

#[test]
fn genesys_constants_match_sane_low_h() {
    assert_eq!(REQUEST_TYPE_IN, 0xC0);
    assert_eq!(REQUEST_TYPE_OUT, 0x40);
    assert_eq!(REQUEST_BUFFER, 0x04);
    assert_eq!(VALUE_BUFFER, 0x82);
    assert_eq!(VALUE_SET_REGISTER, 0x83);
    assert_eq!(VALUE_READ_REGISTER, 0x84);
    assert_eq!(VALUE_WRITE_REGISTER, 0x85);
    assert_eq!(VALUE_BUF_ENDACCESS, 0x8C);
    assert_eq!(VALUE_GET_REGISTER, 0x8E);
    assert_eq!(REGISTER_LINK_OK, 0x55);
}

#[test]
fn link_status_check() {
    GenesysUsbProtocol::check_link_status(0x55).unwrap();
    let err = GenesysUsbProtocol::check_link_status(0x00).unwrap_err();
    assert!(
        matches!(err, Error::Usb(_)) && err.to_string().contains("0x00"),
        "{err}"
    );
}

#[test]
fn abort_bulk_stream_calls_transport() {
    let mut p = proto();
    fake(&mut p).core.abort_drain_bytes = 42;
    assert_eq!(p.abort_bulk_stream(), 42);
    assert_eq!(fake(&mut p).core.aborted, 1);
}

#[test]
fn read_register_gl845_framing() {
    let mut p = proto();
    fake(&mut p).core.registers.insert(0x6B, 0x30);
    assert_eq!(p.read_register(0x6B).unwrap(), 0x30);
    let (rtype, req, value, index, data) = fake(&mut p).core.control_log()[0].clone();
    assert_eq!(
        (rtype, req, value),
        (REQUEST_TYPE_IN, REQUEST_BUFFER, VALUE_GET_REGISTER)
    );
    assert_eq!(index, (0x22 + (0x6Bu32 << 8)) as u16);
    assert_eq!(data, ControlLogData::Len(2));
}

#[test]
fn write_register_gl845_framing() {
    let mut p = proto();
    p.write_register(0x6C, 0x4C).unwrap();
    let (rtype, req, value, index, data) = fake(&mut p).core.control_log()[0].clone();
    assert_eq!(
        (rtype, req, value, index),
        (REQUEST_TYPE_OUT, REQUEST_BUFFER, VALUE_SET_REGISTER, INDEX)
    );
    assert_eq!(written(&data), [0x6C, 0x4C]);
    assert_eq!(fake(&mut p).core.reg(0x6C), 0x4C);
}

#[test]
fn write_register_high_address_sets_flag() {
    let mut p = proto();
    p.write_register(0x1A6, 0x07).unwrap();
    let (_, _, value, _, data) = fake(&mut p).core.control_log()[0].clone();
    assert_eq!(value, VALUE_SET_REGISTER | 0x100);
    assert_eq!(written(&data), [0xA6, 0x07]);
}

#[test]
fn read_write_roundtrip() {
    let mut p = proto();
    p.write_register(0x6D, 0x80).unwrap();
    assert_eq!(p.read_register(0x6D).unwrap(), 0x80);
}

#[test]
fn bulk_read_data_sends_gl845_header_per_chunk() {
    let mut p = proto();
    fake(&mut p).core.bulk_in_queue.push_back(vec![1, 2, 3, 4]);
    assert_eq!(p.bulk_read_data(4).unwrap(), [1, 2, 3, 4]);
    // First control is the bulk header.
    let (rtype, req, value, index, data) = fake(&mut p).core.control_log()[0].clone();
    assert_eq!(
        (rtype, req, value, index),
        (REQUEST_TYPE_OUT, REQUEST_BUFFER, VALUE_BUFFER, 0x00)
    );
    let payload = written(&data);
    assert_eq!(payload[3], 0x10); // 0x10000000 address marker
    assert_eq!(payload[4..8], [4, 0, 0, 0]);
}

#[test]
fn write_ahb_header_and_bulk() {
    let mut p = proto();
    p.write_ahb(0x1000_0000, b"abcd").unwrap();
    let (_, _, value, index, data) = fake(&mut p).core.control_log()[0].clone();
    assert_eq!((value, index), (VALUE_BUFFER, 0x01));
    let payload = written(&data);
    assert_eq!(payload[0..4], [0, 0, 0, 0x10]);
    assert_eq!(payload[4..8], [4, 0, 0, 0]);
    assert_eq!(fake(&mut p).core.bulk_out_log(), [b"abcd".to_vec()]);
}

#[test]
fn queued_error_surfaces_on_the_next_transfer() {
    let mut p = proto();
    fake(&mut p).queue_error(Error::Usb("pipe".into()));
    assert!(matches!(p.read_register(0x01), Err(Error::Usb(_))));
    p.read_register(0x01).unwrap();
}
