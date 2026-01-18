// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use std::fmt;

#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Default)]
pub struct GameID([u8; 6]);

impl From<[u8; 6]> for GameID {
    fn from(id: [u8; 6]) -> Self {
        GameID(id)
    }
}

impl From<&str> for GameID {
    fn from(s: &str) -> Self {
        let mut id = [0u8; 6];
        let bytes = s.as_bytes();
        let len = bytes.len().min(6);
        id[..len].copy_from_slice(&bytes[..len]);
        GameID(id)
    }
}

impl fmt::Display for GameID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for c in self.0 {
            write!(f, "{}", c as char)?;
        }

        Ok(())
    }
}

impl GameID {
    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.0).unwrap_or("invalid")
    }

    pub fn as_partial_str(&self) -> &str {
        std::str::from_utf8(&self.0[..3]).unwrap_or("invalid")
    }

    pub fn as_region_str(&self) -> &'static str {
        match self.0[3] {
            b'A' => "System Wii Channels (i.e. Mii Channel)",
            b'B' => "Ufouria: The Saga (NA)",
            b'D' => "Germany",
            b'E' => "USA",
            b'F' => "France",
            b'H' => "Netherlands / Europe alternate languages",
            b'I' => "Italy",
            b'J' => "Japan",
            b'K' => "Korea",
            b'L' => "Japanese import to Europe, Australia and other PAL regions",
            b'M' => "American import to Europe, Australia and other PAL regions",
            b'N' => "Japanese import to USA and other NTSC regions",
            b'P' => "Europe and other PAL regions such as Australia",
            b'Q' => "Japanese Virtual Console import to Korea",
            b'R' => "Russia",
            b'S' => "Spain",
            b'T' => "American Virtual Console import to Korea",
            b'U' => "Australia / Europe alternate languages",
            b'V' => "Scandinavia",
            b'W' => "Republic of China (Taiwan) / Hong Kong / Macau",
            b'X' => "Europe alternate languages / US special releases",
            b'Y' => "Europe alternate languages / US special releases",
            b'Z' => "Europe alternate languages / US special releases",
            _ => "Unknown",
        }
    }

    pub fn as_lang_str(&self) -> &'static str {
        match self.0[3] {
            b'E' | b'N' => "US",
            b'J' => "JA",
            b'K' | b'Q' | b'T' => "KO",
            b'R' => "RU",
            b'W' => "ZH",
            _ => "EN",
        }
    }
}
