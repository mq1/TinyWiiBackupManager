// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use std::ffi::OsStr;

#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Default)]
pub struct GameID([u8; 6]);

impl From<[u8; 6]> for GameID {
    fn from(id: [u8; 6]) -> Self {
        Self(id)
    }
}

impl From<GameID> for [u8; 6] {
    fn from(val: GameID) -> Self {
        val.0
    }
}

impl TryFrom<&str> for GameID {
    type Error = usize;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let bytes = s.as_bytes();

        match s.len() {
            4 => Ok(Self([bytes[0], bytes[1], bytes[2], bytes[3], 0, 0])),
            6 => Ok(Self([
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5],
            ])),
            n => Err(n),
        }
    }
}

impl GameID {
    #[inline]
    pub const fn is_wii(self) -> bool {
        matches!(self.0[0], b'H' | b'R' | b'S' | b'W' | b'X')
    }

    #[inline]
    pub const fn is_gc(self) -> bool {
        matches!(self.0[0], b'D' | b'G')
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        if self.0[4] == 0 {
            std::str::from_utf8(&self.0[..4]).unwrap_or("invalid")
        } else {
            std::str::from_utf8(&self.0).unwrap_or("invalid")
        }
    }

    #[inline]
    pub fn as_partial_str(&self) -> &str {
        std::str::from_utf8(&self.0[..3]).unwrap_or("invalid")
    }

    #[inline]
    pub fn as_os_str(&self) -> &OsStr {
        if self.0[4] == 0 {
            unsafe { OsStr::from_encoded_bytes_unchecked(&self.0[..4]) }
        } else {
            unsafe { OsStr::from_encoded_bytes_unchecked(&self.0) }
        }
    }

    pub const fn as_region_str(self) -> &'static str {
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
            b'X' | b'Y' | b'Z' => "Europe alternate languages / US special releases",
            _ => "Unknown",
        }
    }

    pub const fn as_lang_str(self) -> &'static str {
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
