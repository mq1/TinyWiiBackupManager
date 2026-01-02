// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

pub trait GameID {
    fn from_str(s: &str) -> Self;
    fn as_str(&self) -> &str;
    fn as_partial_str(&self) -> &str;
    fn as_region_str(&self) -> &str;
    fn as_lang_str(&self) -> &str;
}

impl GameID for [u8; 6] {
    fn from_str(s: &str) -> Self {
        let mut id = [0u8; 6];
        let bytes = s.as_bytes();
        let len = bytes.len().min(6);
        id[..len].copy_from_slice(&bytes[..len]);
        id
    }

    #[inline]
    fn as_str(&self) -> &str {
        std::str::from_utf8(self).unwrap_or("invalid")
    }

    #[inline]
    fn as_partial_str(&self) -> &str {
        std::str::from_utf8(&self[..3]).unwrap_or("invalid")
    }

    fn as_region_str(&self) -> &'static str {
        match self[3] {
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

    fn as_lang_str(&self) -> &'static str {
        match self[3] {
            b'E' | b'N' => "US",
            b'J' => "JA",
            b'K' | b'Q' | b'T' => "KO",
            b'R' => "RU",
            b'W' => "ZH",
            _ => "EN",
        }
    }
}
