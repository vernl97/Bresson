// SPDX-License-Identifier: GPL-3.0-or-later
//! Genesys register addresses and bit masks.
//!
//! [`gl845`] mirrors genesys `gl846_registers.h`. [`gl128`] covers GL128
//! OpticFilm (map proven on the 8200i SE; also used by 8100 V2), whose map is
//! GL124-family.

/// GL845/846 registers (SANE `gl846_registers.h`).
pub mod gl845 {
    pub const REG_0X01: u16 = 0x01;
    pub const REG_0X02: u16 = 0x02;
    pub const REG_0X03: u16 = 0x03;
    pub const REG_0X04: u16 = 0x04;
    pub const REG_0X05: u16 = 0x05;
    pub const REG_0X06: u16 = 0x06;
    pub const REG_0X0E: u16 = 0x0E;
    pub const REG_0X0F: u16 = 0x0F;
    pub const REG_0X40: u16 = 0x40;
    /// Status.
    pub const REG_0X41: u16 = 0x41;
    pub const REG_0X6C: u16 = 0x6C;
    pub const REG_0XA8: u16 = 0xA8;

    // 0x01
    pub const SCAN: u8 = 0x01;
    pub const SHDAREA: u8 = 0x02;
    pub const DVDSET: u8 = 0x20;

    // 0x02 motor
    pub const NOTHOME: u8 = 0x80;
    pub const ACDCDIS: u8 = 0x40;
    pub const AGOHOME: u8 = 0x20;
    pub const MTRPWR: u8 = 0x10;
    pub const FASTFED: u8 = 0x08;
    pub const MTRREV: u8 = 0x04;
    pub const HOMENEG: u8 = 0x02;

    // 0x03 lamp
    pub const LAMPPWR: u8 = 0x10;
    pub const XPASEL: u8 = 0x20;

    // 0x04 frontend select
    pub const FESET: u8 = 0x03;
    pub const FESET_ADI: u8 = 0x02;

    // 0x06
    pub const PWRBIT: u8 = 0x10;

    // 0x40
    pub const CHKVER: u8 = 0x10;
    pub const MOTMFLG: u8 = 0x02;
    pub const DATAENB: u8 = 0x01;

    // 0x41 status bits (scanner_read_status)
    pub const STATUS_PWRBIT: u8 = 0x80;
    pub const STATUS_BUFEMPTY: u8 = 0x40;
    pub const STATUS_FEEDFSH: u8 = 0x20;
    pub const STATUS_SCANFSH: u8 = 0x10;
    pub const STATUS_HOMESNR: u8 = 0x08;
    pub const STATUS_LAMPSTS: u8 = 0x04;
    pub const STATUS_FEBUSY: u8 = 0x02;
    pub const STATUS_MOTORENB: u8 = 0x01;

    /// IR lamp GPIO bit for 8200i (command_set_common.cpp).
    pub const IR_LAMP_A8_MASK: u8 = 0x04;
}

/// GL128 (GL124-family) addresses confirmed from SE USB captures.
///
/// Status uses the same bit layout as GL845 register `0x41` but lives at the
/// high address `0x101`.
pub mod gl128 {
    pub const REG_0X01: u16 = 0x01;
    pub const REG_0X02: u16 = 0x02;
    pub const REG_0X03: u16 = 0x03;
    /// Write 0x07 to clear line/motor/feed counters.
    pub const REG_CLRCNT: u16 = 0x0D;
    /// Write 0x01 to launch the configured operation.
    pub const REG_START: u16 = 0x0F;
    /// 24-bit BE, lines x image_lincnt_per_line (not 7200 dpi units).
    pub const REG_LINCNT: u16 = 0x25;
    /// 24-bit BE line exposure period.
    pub const REG_LPERIOD: u16 = 0x28;
    /// 16-bit BE, equals dpi / 6.
    pub const REG_DPISET: u16 = 0x2C;
    /// 0x04 = 16-bit output, 0x1F = 8-bit.
    pub const REG_DEPTH_A: u16 = 0x33;
    /// Bit 2 enables the infrared LED (read-modify-write).
    pub const REG_IR: u16 = 0x37;
    /// 24-bit BE feed distance for move-only operations.
    pub const REG_FEEDL: u16 = 0x3D;
    pub const REG_FE_INDEX: u16 = 0x51;
    pub const REG_FE_HIGH: u16 = 0x5D;
    pub const REG_FE_LOW: u16 = 0x5E;
    /// 24-bit BE base exposure (14000).
    pub const REG_EXPOSURE: u16 = 0x7D;
    /// 24-bit BE, native 7200 dpi units.
    pub const REG_STRPIXEL: u16 = 0x82;
    /// 24-bit BE, native 7200 dpi units.
    pub const REG_ENDPIXEL: u16 = 0x85;
    /// 0x46 = 16-bit output, 0xFF = 8-bit.
    pub const REG_DEPTH_B: u16 = 0xAF;
    /// High-address read; GL845 0x41 bit layout.
    pub const REG_STATUS: u16 = 0x101;

    // 0x01 scan control
    pub const SCAN: u8 = 0x01;
    pub const SHDAREA: u8 = 0x02;
    pub const STAGGER: u8 = 0x10;
    pub const DVDSET: u8 = 0x20;

    // 0x02 motor
    pub const AGOHOME: u8 = 0x20;
    pub const MTRPWR: u8 = 0x10;
    pub const FASTFED: u8 = 0x08;
    pub const MTRREV: u8 = 0x04;

    // 0x03 lamp: XPASEL is held for every transparency operation, LAMPPWR gates
    // the white lamp. IR passes clear LAMPPWR and keep XPASEL.
    pub const LAMPPWR: u8 = 0x10;
    pub const XPASEL: u8 = 0x20;
    pub const AVEENB: u8 = 0x40;

    /// 0x37 infrared LED enable.
    pub const IR_LED: u8 = 0x04;

    pub const CLRCNT_ALL: u8 = 0x07;
    pub const START_GO: u8 = 0x01;

    pub const DEPTH16_A: u8 = 0x04;
    pub const DEPTH16_B: u8 = 0x46;
    pub const DEPTH8_A: u8 = 0x1F;
    pub const DEPTH8_B: u8 = 0xFF;

    pub const BULK_INDEX_RAM: u8 = 0x00;
    pub const BULK_INDEX_IMAGE: u8 = 0x08;

    pub const AHB_SCAN_MOTOR_0: u32 = 0x1000_0000;
    pub const AHB_SCAN_MOTOR_1: u32 = 0x1000_4000;
    pub const AHB_SCAN_MOTOR_2: u32 = 0x1000_8000;
    /// RAM read address for AFE and shading strip data.
    pub const AHB_RAM_BASE: u32 = 0x1000_0000;
    pub const AHB_SLOPE_SCAN: u32 = 0x1000_C000;
    pub const AHB_SLOPE_FAST: u32 = 0x1001_0000;
    pub const AHB_SHADING: u32 = 0x1001_4000;
}
