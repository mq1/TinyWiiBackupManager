// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use serde::Deserialize;
use std::{fmt, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct GameID {
    inner: [u8; 6],
}

impl GameID {
    pub fn as_str(&self) -> &str {
        let fifth = unsafe { *self.inner.get_unchecked(4) };
        let end = if fifth == 0 { 4 } else { 6 };
        unsafe { std::str::from_utf8_unchecked(&self.inner[..end]) }
    }

    pub fn as_partial_str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(&self.inner[..3]) }
    }
}

impl FromStr for GameID {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let len = s.len();

        let has_right_len = len == 4 || len == 6;
        let has_valid_chars = s
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit());

        if !has_right_len || !has_valid_chars {
            return Err(());
        }

        let mut inner = [0; 6];
        inner[..len].copy_from_slice(s.as_bytes());

        Ok(GameID { inner })
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
    fn from(inner: [u8; 6]) -> Self {
        Self { inner }
    }
}
