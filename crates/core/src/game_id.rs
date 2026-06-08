// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct GameID([u8; 6]);

impl GameID {
    pub fn new(id_str: &str) -> Option<Self> {
        let len = id_str.len();

        if (len != 4 && len != 6)
            || !id_str
                .chars()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
        {
            return None;
        }

        let mut id = [0; 6];
        id[..len].copy_from_slice(id_str.as_bytes());

        Some(Self(id))
    }

    pub fn as_partial(&self) -> &str {
        &self.as_ref()[..3]
    }
}

impl fmt::Display for GameID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl AsRef<str> for GameID {
    fn as_ref(&self) -> &str {
        let end = if self.0[4] == 0 { 4 } else { 6 };
        unsafe { std::str::from_utf8_unchecked(&self.0[..end]) }
    }
}

impl From<[u8; 6]> for GameID {
    fn from(value: [u8; 6]) -> Self {
        Self(value)
    }
}
