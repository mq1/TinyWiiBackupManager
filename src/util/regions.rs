// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use phf::phf_map;

static REGION_MAP: phf::Map<char, &'static str> = phf_map! {
    'A' => "System Wii Channels (i.e. Mii Channel)",
    'B' => "Ufouria: The Saga (NA)",
    'D' => "Germany",
    'E' => "USA",
    'F' => "France",
    'H' => "Netherlands",
    'I' => "Italy",
    'J' => "Japan",
    'K' => "Korea",
    'L' => "Japanese import to Europe, Australia and other PAL regions",
    'M' => "American import to Europe, Australia and other PAL regions",
    'N' => "Japanese import to USA and other NTSC regions",
    'P' => "Europe and other PAL regions such as Australia",
    'Q' => "Japanese Virtual Console import to Korea",
    'R' => "Russia",
    'S' => "Spain",
    'T' => "American Virtual Console import to Korea",
    'U' => "Australia / Europe alternate languages",
    'V' => "Scandinavia",
    'W' => "Republic of China (Taiwan) / Hong Kong / Macau",
    'X' => "Europe alternate languages / US special releases",
    'Y' => "Europe alternate languages / US special releases",
    'Z' => "Europe alternate languages / US special releases",
};

/// A static map to convert the region character from a game's ID to a language code
/// used by the GameTDB API for fetching cover art.
static REGION_TO_LANG: phf::Map<char, &'static str> = phf_map! {
    'A' => "EN",
    'B' => "EN",
    'D' => "DE",
    'E' => "US",
    'F' => "FR",
    'H' => "NL",
    'I' => "IT",
    'J' => "JA",
    'K' => "KO",
    'L' => "EN",
    'M' => "EN",
    'N' => "US",
    'P' => "EN",
    'Q' => "KO",
    'R' => "RU",
    'S' => "ES",
    'T' => "KO",
    'U' => "EN",
    'V' => "EN",
    'W' => "ZH",
    'X' => "EN",
    'Y' => "EN",
    'Z' => "EN",
};

// The 4th character in a Wii/GameCube ID represents the region.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Region(char);

impl Region {
    pub fn from_char(c: char) -> Self {
        Self(c)
    }

    pub fn from_id(id: &str) -> Self {
        Self(id.chars().nth(3).unwrap_or('E'))
    }

    pub fn to_lang(self) -> &'static str {
        REGION_TO_LANG.get(&self.0).unwrap_or(&"EN")
    }

    pub fn to_name(self) -> &'static str {
        REGION_MAP.get(&self.0).unwrap_or(&"Unknown")
    }
}
