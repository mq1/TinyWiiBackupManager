// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::GameID;
use slint::ToSharedString;

impl From<[u8; 6]> for GameID {
    fn from(id: [u8; 6]) -> Self {
        Self {
            inner: String::from_utf8_lossy(&id).to_shared_string(),
        }
    }
}

impl TryFrom<&str> for GameID {
    type Error = usize;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let len = s.len();

        if !matches!(len, 6 | 4) {
            return Err(len);
        }

        Ok(Self {
            inner: s.to_shared_string(),
        })
    }
}

impl GameID {
    #[must_use]
    pub fn as_raw(&self) -> [u8; 6] {
        let bytes = self.inner.as_bytes();
        let len = bytes.len().min(6);
        let mut buf = [0; 6];
        buf[..len].copy_from_slice(&bytes[..len]);
        buf
    }

    #[must_use]
    pub fn as_partial_str(&self) -> &str {
        &self.inner[0..3]
    }

    #[must_use]
    pub fn as_region_str(&self) -> &'static str {
        let region_char = self.inner.chars().nth(3);

        match region_char {
            Some('A') => "System Wii Channels (i.e. Mii Channel)",
            Some('B') => "Ufouria: The Saga (NA)",
            Some('D') => "Germany",
            Some('E') => "USA",
            Some('F') => "France",
            Some('H') => "Netherlands / Europe alternate languages",
            Some('I') => "Italy",
            Some('J') => "Japan",
            Some('K') => "Korea",
            Some('L') => "Japanese import to Europe, Australia and other PAL regions",
            Some('M') => "American import to Europe, Australia and other PAL regions",
            Some('N') => "Japanese import to USA and other NTSC regions",
            Some('P') => "Europe and other PAL regions such as Australia",
            Some('Q') => "Japanese Virtual Console import to Korea",
            Some('R') => "Russia",
            Some('S') => "Spain",
            Some('T') => "American Virtual Console import to Korea",
            Some('U') => "Australia / Europe alternate languages",
            Some('V') => "Scandinavia",
            Some('W') => "Republic of China (Taiwan) / Hong Kong / Macau",
            Some('X' | 'Y' | 'Z') => "Europe alternate languages / US special releases",
            _ => "Unknown",
        }
    }

    #[must_use]
    pub fn as_lang_str(&self) -> &'static str {
        let region_char = self.inner.chars().nth(3);

        match region_char {
            Some('E' | 'N') => "US",
            Some('J') => "JA",
            Some('K' | 'Q' | 'T') => "KO",
            Some('R') => "RU",
            Some('W') => "ZH",
            _ => "EN",
        }
    }
}
