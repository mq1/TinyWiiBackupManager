// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use phf::phf_map;

/// A static map to convert the region character from a game's ID to a language code
/// used by the GameTDB API for fetching cover art.
pub static REGION_TO_LANG: phf::Map<char, &'static str> = phf_map! {
    'A' => "EN", // System Wii Channels (i.e. Mii Channel)
    'B' => "EN", // Ufouria: The Saga (NA)
    'D' => "DE", // Germany
    'E' => "US", // USA
    'F' => "FR", // France
    'H' => "NL", // Netherlands
    'I' => "IT", // Italy
    'J' => "JA", // Japan
    'K' => "KO", // Korea
    'L' => "EN", // Japanese import to Europe, Australia and other PAL regions
    'M' => "EN", // American import to Europe, Australia and other PAL regions
    'N' => "US", // Japanese import to USA and other NTSC regions
    'P' => "EN", // Europe and other PAL regions such as Australia
    'Q' => "KO", // Japanese Virtual Console import to Korea
    'R' => "RU", // Russia
    'S' => "ES", // Spain
    'T' => "KO", // American Virtual Console import to Korea
    'U' => "EN", // Australia / Europe alternate languages
    'V' => "EN", // Scandinavia
    'W' => "ZH", // Republic of China (Taiwan) / Hong Kong / Macau
    'X' => "EN", // Europe alternate languages / US special releases
    'Y' => "EN", // Europe alternate languages / US special releases
    'Z' => "EN", // Europe alternate languages / US special releases
};
