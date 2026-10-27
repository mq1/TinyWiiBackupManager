// SPDX-FileCopyrightText: 2026 Yanik Fuchs <yanik.fuchs@me.com>
// SPDX-License-Identifier: GPL-3.0-only

//! A 3DS title's SMDH: its names and icons (ported from vcinject's smdh.py, MIT)

pub const SMDH_SIZE: usize = 0x36C0;
const TITLES_START: usize = 0x8;
const TITLE_ENTRY_SIZE: usize = 0x200;
const LARGE_ICON_START: usize = 0x24C0;
pub const LARGE_ICON_SIZE: u32 = 48;

/// The order of the title entries
pub const LANGUAGES: [&str; 12] = [
    "ja", "en", "fr", "de", "it", "es", "zh-hans", "ko", "nl", "pt", "ru", "zh-hant",
];
const ENGLISH: usize = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Names {
    pub short: String,
    pub long: String,
    pub publisher: String,
}

pub fn is_smdh(data: &[u8]) -> bool {
    data.len() >= SMDH_SIZE && data.starts_with(b"SMDH")
}

fn utf16(bytes: &[u8]) -> String {
    let units: Vec<u16> = bytes
        .as_chunks::<2>()
        .0
        .iter()
        .map(|&unit| u16::from_le_bytes(unit))
        .take_while(|&unit| unit != 0)
        .collect();

    String::from_utf16_lossy(&units).trim().to_string()
}

fn names_at(smdh: &[u8], index: usize) -> Names {
    let base = TITLES_START + index * TITLE_ENTRY_SIZE;

    Names {
        short: utf16(&smdh[base..base + 0x80]),
        long: utf16(&smdh[base + 0x80..base + 0x180]),
        publisher: utf16(&smdh[base + 0x180..base + 0x200]),
    }
}

/// The names in `language` (see [`LANGUAGES`]), English when it has none
pub fn names(smdh: &[u8], language: &str) -> Option<Names> {
    if !is_smdh(smdh) {
        return None;
    }

    let index = LANGUAGES
        .iter()
        .position(|&l| l == language)
        .unwrap_or(ENGLISH);
    let names = names_at(smdh, index);

    Some(if names.short.is_empty() {
        names_at(smdh, ENGLISH)
    } else {
        names
    })
}

/// The 48×48 icon as RGBA, decoded from tiled, Morton-ordered RGB565
pub fn large_icon_rgba(smdh: &[u8]) -> Option<Vec<u8>> {
    if !is_smdh(smdh) {
        return None;
    }

    let size = LARGE_ICON_SIZE as usize;
    let mut rgba = vec![0; size * size * 4];
    let mut pos = LARGE_ICON_START;

    for tile_y in (0..size).step_by(8) {
        for tile_x in (0..size).step_by(8) {
            for i in 0..64 {
                let x = (i & 1) | ((i >> 1) & 2) | ((i >> 2) & 4);
                let y = ((i >> 1) & 1) | ((i >> 2) & 2) | ((i >> 3) & 4);
                let value = u16::from_le_bytes([smdh[pos], smdh[pos + 1]]);
                pos += 2;

                let (r, g, b) = (value >> 11, (value >> 5) & 0x3F, value & 0x1F);
                let offset = ((tile_y + y) * size + tile_x + x) * 4;
                rgba[offset..offset + 4].copy_from_slice(&[
                    ((r << 3) | (r >> 2)) as u8,
                    ((g << 2) | (g >> 4)) as u8,
                    ((b << 3) | (b >> 2)) as u8,
                    0xFF,
                ]);
            }
        }
    }

    Some(rgba)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn put_utf16(smdh: &mut [u8], at: usize, text: &str) {
        for (i, unit) in text.encode_utf16().enumerate() {
            smdh[at + i * 2..at + i * 2 + 2].copy_from_slice(&unit.to_le_bytes());
        }
    }

    #[test]
    fn names_fall_back_to_english() {
        let mut smdh = vec![0; SMDH_SIZE];
        smdh[..4].copy_from_slice(b"SMDH");
        let english = TITLES_START + TITLE_ENTRY_SIZE;
        put_utf16(&mut smdh, english, "Zelda");
        put_utf16(&mut smdh, english + 0x180, "Nintendo");

        let names = names(&smdh, "de").unwrap();
        assert_eq!(names.short, "Zelda");
        assert_eq!(names.publisher, "Nintendo");
        assert_eq!(super::names(b"nope", "en"), None);
    }

    #[test]
    fn icon_pixels_are_untiled() {
        let mut smdh = vec![0; SMDH_SIZE];
        smdh[..4].copy_from_slice(b"SMDH");
        // the 3rd pixel of the first tile is at (0, 1); pure red
        let pos = LARGE_ICON_START + 2 * 2;
        smdh[pos..pos + 2].copy_from_slice(&0xF800_u16.to_le_bytes());

        let rgba = large_icon_rgba(&smdh).unwrap();
        let offset = 48 * 4;
        assert_eq!(&rgba[offset..offset + 4], &[0xFF, 0, 0, 0xFF]);
        assert_eq!(&rgba[..4], &[0, 0, 0, 0xFF]);
    }
}
