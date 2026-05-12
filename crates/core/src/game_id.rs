// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use radix_fmt::Radix;
use std::fmt::{self, Write};

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

    pub fn from_byte_string(b: [u8; 6]) -> Option<Self> {
        let s = str::from_utf8(&b).ok()?;
        Self::new(s)
    }

    pub fn partial(&self) -> String {
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

pub fn make_list_string(list: &[GameID]) -> String {
    let mut s = String::new();

    let last_i = list.len() - 1;
    for (i, id) in list.iter().enumerate() {
        write!(&mut s, "{id}").unwrap();

        if i != last_i {
            s.push_str(", ");
        }
    }

    s
}
