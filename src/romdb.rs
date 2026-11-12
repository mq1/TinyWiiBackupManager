// SPDX-FileCopyrightText: 2026 Yanik Fuchs <yanik.fuchs@me.com>
// SPDX-License-Identifier: GPL-3.0-only

//! Game databases for the ROMs of the older consoles: a ROM is identified by its checksum against
//! the No-Intro lists of libretro-database, which the app ships (`assets/databases/`, refreshed
//! with `scripts/update-databases.py`).

use crate::library::Platform;
use sha1::{Digest, Sha1};
use std::{collections::HashMap, sync::LazyLock};

/// One No-Intro list, as compacted by `scripts/update-databases.py`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum List {
    GameBoy,
    GameBoyColor,
    GameBoyAdvance,
    Nes,
    Snes,
    MegaDrive,
}

impl List {
    fn text(self) -> &'static str {
        match self {
            Self::GameBoy => include_str!("../assets/databases/no-intro/gb.tsv"),
            Self::GameBoyColor => include_str!("../assets/databases/no-intro/gbc.tsv"),
            Self::GameBoyAdvance => include_str!("../assets/databases/no-intro/gba.tsv"),
            Self::Nes => include_str!("../assets/databases/no-intro/nes.tsv"),
            Self::Snes => include_str!("../assets/databases/no-intro/snes.tsv"),
            Self::MegaDrive => include_str!("../assets/databases/no-intro/md.tsv"),
        }
    }

    fn platform(self) -> Platform {
        match self {
            Self::GameBoy => Platform::GameBoy,
            Self::GameBoyColor => Platform::GameBoyColor,
            Self::GameBoyAdvance => Platform::GameBoyAdvance,
            Self::Nes => Platform::Nes,
            Self::Snes => Platform::Snes,
            Self::MegaDrive => Platform::MegaDrive,
        }
    }

    /// The lists a ROM of `platform` is looked up in; a Game Boy Color game is often a `.gb` file
    fn for_platform(platform: Platform) -> &'static [Self] {
        match platform {
            Platform::GameBoy | Platform::GameBoyColor => &[Self::GameBoy, Self::GameBoyColor],
            Platform::GameBoyAdvance => &[Self::GameBoyAdvance],
            Platform::Nes => &[Self::Nes],
            Platform::Snes => &[Self::Snes],
            Platform::MegaDrive => &[Self::MegaDrive],
            _ => &[],
        }
    }

    fn database(self) -> &'static Database {
        static DATABASES: LazyLock<[Database; 6]> = LazyLock::new(|| {
            [
                List::GameBoy,
                List::GameBoyColor,
                List::GameBoyAdvance,
                List::Nes,
                List::Snes,
                List::MegaDrive,
            ]
            .map(|list| Database::parse(list.text()))
        });
        &DATABASES[self as usize]
    }
}

#[derive(Debug, Default)]
struct Database {
    /// the release name of each dump, e.g. "Metroid Fusion (USA)"
    dumps: Vec<&'static str>,
    by_sha1: HashMap<[u8; 20], usize>,
    by_crc32: HashMap<u32, usize>,
    by_serial: HashMap<&'static str, Vec<usize>>,
}

impl Database {
    /// Lines of `CRC32 \t SHA-1 \t serial \t name`; `#` starts a comment. A line that doesn't
    /// fit is skipped, so one bad line doesn't cost the whole list.
    fn parse(text: &'static str) -> Self {
        let mut db = Self::default();
        for line in text
            .lines()
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
        {
            let mut fields = line.splitn(4, '\t');
            let (Some(crc32), Some(sha1), Some(serial), Some(name)) =
                (fields.next(), fields.next(), fields.next(), fields.next())
            else {
                continue;
            };
            let Ok(crc32) = u32::from_str_radix(crc32, 16) else {
                continue;
            };
            let Some(sha1) = sha1_from_hex(sha1) else {
                continue;
            };

            let index = db.dumps.len();
            db.by_sha1.entry(sha1).or_insert(index);
            db.by_crc32.entry(crc32).or_insert(index);
            if !serial.is_empty() {
                db.by_serial.entry(serial).or_default().push(index);
            }
            db.dumps.push(name);
        }
        db
    }

    /// The title all dumps with `serial` share, without region and other tags; none when the
    /// serial is a placeholder that several games use (Mega Drive "00000000-00" and the like)
    fn title_of_serial(&self, serial: &str) -> Option<&'static str> {
        let mut titles = self.by_serial.get(serial)?.iter().map(|&i| {
            let name = self.dumps[i];
            name.split(" (").next().unwrap_or(name)
        });
        let first = titles.next()?;
        titles.all(|t| t == first).then_some(first)
    }
}

pub fn sha1_from_hex(hex: &str) -> Option<[u8; 20]> {
    let mut sha1 = [0; 20];
    hex::decode_to_slice(hex, &mut sha1).ok()?;
    Some(sha1)
}

/// How a ROM was found in a database
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Match {
    /// its SHA-1 is a known good dump
    Verified,
    /// its CRC32 is a known dump's, its SHA-1 isn't (very unlikely: a damaged list or a collision)
    Crc32,
    /// only its cartridge serial is known: a bad dump, a hack or a translation of that game
    Serial,
}

/// The release a ROM was identified as
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Identified {
    /// the No-Intro name, e.g. "Metroid Fusion (USA)"; for a [`Match::Serial`] only the title,
    /// e.g. "Metroid Fusion", since which release the data comes from isn't known
    pub release: String,
    /// the platform of the list it is in (a `.gb` file can be a Game Boy Color game)
    pub platform: Platform,
    pub how: Match,
}

/// Looks a ROM up in the lists of its platform: by SHA-1, then CRC32, then serial
pub fn identify(platform: Platform, rom: &RomHashes) -> Option<Identified> {
    let lists = List::for_platform(platform);
    let found = |list: List, index: usize, how| Identified {
        release: list.database().dumps[index].to_string(),
        platform: list.platform(),
        how,
    };

    for &list in lists {
        if let Some(&i) = list.database().by_sha1.get(&rom.sha1) {
            return Some(found(list, i, Match::Verified));
        }
    }
    for &list in lists {
        if let Some(&i) = list.database().by_crc32.get(&rom.crc32) {
            return Some(found(list, i, Match::Crc32));
        }
    }
    let serial = rom.serial.as_deref()?;
    lists.iter().find_map(|&list| {
        Some(Identified {
            release: list.database().title_of_serial(serial)?.to_string(),
            platform: list.platform(),
            how: Match::Serial,
        })
    })
}

/// Number of dumps the app knows per platform, for tests and the About page later
#[cfg(test)]
fn known_dumps(platform: Platform) -> usize {
    List::for_platform(platform)
        .iter()
        .map(|l| l.database().dumps.len())
        .sum()
}

/// Checksums and serial of a ROM, as the lists have them
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RomHashes {
    pub crc32: u32,
    pub sha1: [u8; 20],
    pub serial: Option<String>,
}

/// The ROM's data as No-Intro hashes it: without the iNES header (NES) or a copier's 512-byte
/// header (SNES)
fn strip_header(platform: Platform, data: &[u8]) -> &[u8] {
    match platform {
        Platform::Nes if data.starts_with(b"NES\x1a") && data.len() > 16 => &data[16..],
        Platform::Snes if data.len() % 1024 == 512 => &data[512..],
        _ => data,
    }
}

/// The cartridge serial in the ROM's header, in the form the lists use
fn serial(platform: Platform, data: &[u8]) -> Option<String> {
    let code = |range: std::ops::Range<usize>| {
        let bytes = data.get(range)?;
        bytes
            .iter()
            .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit())
            .then(|| String::from_utf8_lossy(bytes).into_owned())
    };

    match platform {
        // the game code, e.g. "BPEE" for Pokémon Emerald (USA)
        Platform::GameBoyAdvance => code(0xAC..0xB0),
        // newer Game Boy Color cartridges carry a game code too, e.g. "BO7E"
        Platform::GameBoy | Platform::GameBoyColor
            if matches!(data.get(0x143), Some(0x80 | 0xC0)) =>
        {
            code(0x13F..0x143)
        }
        // "GM 00001009-00" at 0x180: the lists have "00001009-00" (and no spaces in it)
        Platform::MegaDrive => {
            let field = data.get(0x182..0x18E)?;
            let serial: String = String::from_utf8_lossy(field)
                .chars()
                .filter(|c| !c.is_whitespace())
                .collect();
            (serial.len() >= 4 && serial.chars().all(|c| c.is_ascii_graphic())).then_some(serial)
        }
        _ => None,
    }
}

/// Reads checksums and serial from a ROM's whole data
pub fn hash_rom(platform: Platform, data: &[u8]) -> RomHashes {
    let dump = strip_header(platform, data);
    RomHashes {
        crc32: crc32fast::hash(dump),
        sha1: Sha1::digest(dump).into(),
        serial: serial(platform, dump),
    }
}

/// Whether ROMs of `platform` are hashed and looked up (the lists the app has)
pub fn has_database(platform: Platform) -> bool {
    !List::for_platform(platform).is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_list_is_read() {
        for platform in [
            Platform::GameBoy,
            Platform::GameBoyAdvance,
            Platform::Nes,
            Platform::Snes,
            Platform::MegaDrive,
        ] {
            assert!(known_dumps(platform) > 1000, "{platform:?}");
        }
        for list in [
            List::GameBoy,
            List::GameBoyColor,
            List::GameBoyAdvance,
            List::Nes,
            List::Snes,
            List::MegaDrive,
        ] {
            let db = list.database();
            assert!(db.dumps.iter().all(|name| !name.is_empty()), "{list:?}");
            // every line was read
            let lines = list.text().lines().filter(|l| !l.starts_with('#')).count();
            assert_eq!(db.dumps.len(), lines, "{list:?}");
        }
    }

    #[test]
    fn a_known_dump_is_found_by_its_checksums() {
        // the list's own entry, as if a file with these checksums had been read
        let line = List::GameBoyAdvance
            .text()
            .lines()
            .find(|l| l.ends_with("\tMetroid Fusion (USA)"))
            .expect("Metroid Fusion (USA) in the GBA list");
        let mut fields = line.split('\t');
        let crc32 = u32::from_str_radix(fields.next().unwrap(), 16).unwrap();
        let sha1 = sha1_from_hex(fields.next().unwrap()).unwrap();
        let serial = fields.next().unwrap().to_string();
        assert_eq!(serial, "AMTE");

        let rom = RomHashes {
            crc32,
            sha1,
            serial: Some(serial.clone()),
        };
        let found = identify(Platform::GameBoyAdvance, &rom).unwrap();
        assert_eq!(found.release, "Metroid Fusion (USA)");
        assert_eq!(found.how, Match::Verified);

        // a hack or bad dump keeps the cartridge's serial: only the title is known
        let other = RomHashes {
            crc32: crc32 ^ 1,
            sha1: [7; 20],
            serial: Some(serial),
        };
        let found = identify(Platform::GameBoyAdvance, &other).unwrap();
        assert_eq!(
            (found.release.as_str(), found.how),
            ("Metroid Fusion", Match::Serial)
        );

        // the GBA list isn't asked for a SNES ROM
        assert_eq!(identify(Platform::Snes, &rom), None);
    }

    #[test]
    fn a_placeholder_serial_identifies_nothing() {
        let rom = RomHashes {
            crc32: 0,
            sha1: [0; 20],
            serial: Some("00000000-00".into()),
        };
        assert_eq!(identify(Platform::MegaDrive, &rom), None);
    }

    #[test]
    fn a_game_boy_color_game_in_a_gb_file_is_found_in_the_color_list() {
        let line = List::GameBoyColor
            .text()
            .lines()
            .find(|l| !l.starts_with('#'))
            .unwrap();
        let mut fields = line.split('\t');
        let rom = RomHashes {
            crc32: u32::from_str_radix(fields.next().unwrap(), 16).unwrap(),
            sha1: sha1_from_hex(fields.next().unwrap()).unwrap(),
            serial: None,
        };
        let found = identify(Platform::GameBoy, &rom).unwrap();
        assert_eq!(found.platform, Platform::GameBoyColor);
    }

    #[test]
    fn headers_are_left_out_of_the_checksums() {
        let rom = vec![0x5a; 0x8000];
        let mut nes = b"NES\x1a".to_vec();
        nes.resize(16, 0);
        nes.extend_from_slice(&rom);
        assert_eq!(hash_rom(Platform::Nes, &nes), hash_rom(Platform::Nes, &rom));

        let mut snes = vec![0; 512];
        snes.extend_from_slice(&rom);
        assert_eq!(
            hash_rom(Platform::Snes, &snes),
            hash_rom(Platform::Snes, &rom)
        );
        assert_eq!(hash_rom(Platform::Nes, &rom).crc32, crc32fast::hash(&rom));
    }

    #[test]
    fn serials_are_read_from_the_header() {
        let mut gba = vec![0; 0x200];
        gba[0xAC..0xB0].copy_from_slice(b"BPEE");
        assert_eq!(
            serial(Platform::GameBoyAdvance, &gba).as_deref(),
            Some("BPEE")
        );
        gba[0xAC] = 0;
        assert_eq!(serial(Platform::GameBoyAdvance, &gba), None);

        let mut md = vec![b' '; 0x200];
        md[0x180..0x18E].copy_from_slice(b"GM T-48073 -00");
        assert_eq!(
            serial(Platform::MegaDrive, &md).as_deref(),
            Some("T-48073-00")
        );

        let mut gbc = vec![0; 0x150];
        gbc[0x13F..0x143].copy_from_slice(b"BO7E");
        assert_eq!(serial(Platform::GameBoyColor, &gbc), None); // not flagged as a Color game
        gbc[0x143] = 0xC0;
        assert_eq!(serial(Platform::GameBoy, &gbc).as_deref(), Some("BO7E"));
        assert_eq!(serial(Platform::Nes, &gbc), None);
    }
}
