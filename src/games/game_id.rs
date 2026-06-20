// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct GameID([u8; 6]);

impl GameID {
    pub fn new(id_str: impl AsRef<str>) -> Option<Self> {
        let id_str = id_str.as_ref();

        let has_right_len = id_str.len() == 4 || id_str.len() == 6;
        let has_valid_chars = id_str
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit());

        if !has_right_len || !has_valid_chars {
            return None;
        }

        let mut id = [0; 6];
        id[..id_str.len()].copy_from_slice(id_str.as_bytes());

        Some(Self(id))
    }

    pub fn as_str(&self) -> &str {
        let fifth = unsafe { *self.0.get_unchecked(4) };
        let end = if fifth == 0 { 4 } else { 6 };
        unsafe { std::str::from_utf8_unchecked(&self.0[..end]) }
    }

    pub fn as_partial_str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(&self.0[..3]) }
    }
}

impl fmt::Display for GameID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl AsRef<str> for GameID {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl From<[u8; 6]> for GameID {
    fn from(value: [u8; 6]) -> Self {
        Self(value)
    }
}
