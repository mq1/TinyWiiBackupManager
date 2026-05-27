// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use radix_fmt::Radix;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct GameID(u32);

impl GameID {
    pub const fn new(id_str: &str) -> Option<Self> {
        if !matches!(id_str.len(), 4 | 6) {
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

    pub const fn to_u32(self) -> u32 {
        self.0
    }

    pub fn from_byte_string(b: [u8; 6]) -> Option<Self> {
        let s = str::from_utf8(&b).ok()?;
        Self::new(s)
    }

    pub fn partial(self) -> String {
        let mut s = self.to_string();
        s.truncate(3);
        s
    }
}

impl fmt::Display for GameID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#}", Radix::new(self.0, 36))
    }
}
