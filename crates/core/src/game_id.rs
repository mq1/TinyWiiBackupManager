// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use arrayvec::ArrayString;
use radix_fmt::Radix;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct GameID(u32);

impl GameID {
    pub const fn new(id_str: &str) -> Option<Self> {
        let len = id_str.len();

        if len != 4 && len != 6 {
            return None;
        }

        match u32::from_str_radix(id_str, 36) {
            Ok(id) => Some(Self(id)),
            Err(_) => None,
        }
    }

    pub const fn from_u32(id: u32) -> Self {
        Self(id)
    }

    pub fn to_i32(self) -> i32 {
        let bytes = self.0.to_ne_bytes();
        i32::from_ne_bytes(bytes)
    }

    pub fn from_i32(id: i32) -> Self {
        let bytes = id.to_ne_bytes();
        Self(u32::from_ne_bytes(bytes))
    }

    pub fn from_byte_string(b: [u8; 6]) -> Option<Self> {
        let s = ArrayString::from_byte_string(&b).ok()?;
        Self::new(&s)
    }
}

impl fmt::Display for GameID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#}", Radix::new(self.0, 36))
    }
}
