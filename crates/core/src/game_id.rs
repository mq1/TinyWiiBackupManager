// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use radix_fmt::Radix;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct GameID(pub u32);

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
}

impl fmt::Display for GameID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#}", Radix::new(self.0, 36))
    }
}
