// SPDX-License-Identifier: GPL-3.0-or-later
//! Front-panel buttons reported on the GL128 interrupt endpoint.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Button {
    IntelliScan,
    QuickScan,
}

impl Button {
    pub fn as_str(self) -> &'static str {
        match self {
            Button::IntelliScan => "intelli_scan",
            Button::QuickScan => "quick_scan",
        }
    }
}

pub const BUTTON_IDS: [(u8, Button); 2] = [(0x08, Button::QuickScan), (0x40, Button::IntelliScan)];
