// SPDX-FileCopyrightText: 2026 Yanik Fuchs <yanik.fuchs@me.com>
// SPDX-License-Identifier: GPL-3.0-only

//! Game databases for the ROMs of the older consoles: a ROM is identified by its checksum against
//! the No-Intro lists of libretro-database. The app ships them (`assets/databases/`, refreshed
//! with `scripts/update-databases.py`) and asks libretro-database for newer ones once a day
//! ([`check_for_updates`]); a newer list is kept in the data folder and used in place of the
//! shipped one.

mod update;

pub use update::{Updated, check_for_updates};

use crate::library::Platform;
use sha1::{Digest, Sha1};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, LazyLock, OnceLock, PoisonError, RwLock},
};

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
    const ALL: [Self; 6] = [
        Self::GameBoy,
        Self::GameBoyColor,
        Self::GameBoyAdvance,
        Self::Nes,
        Self::Snes,
        Self::MegaDrive,
    ];

    /// The list's file name, in `assets/databases/no-intro/` and in the data folder
    fn file_name(self) -> &'static str {
        match self {
            Self::GameBoy => "gb.tsv",
            Self::GameBoyColor => "gbc.tsv",
            Self::GameBoyAdvance => "gba.tsv",
            Self::Nes => "nes.tsv",
            Self::Snes => "snes.tsv",
            Self::MegaDrive => "md.tsv",
        }
    }

    /// The DAT's name in libretro-database (`metadat/no-intro/<name>.dat`)
    fn dat_name(self) -> &'static str {
        match self {
            Self::GameBoy => "Nintendo - Game Boy",
            Self::GameBoyColor => "Nintendo - Game Boy Color",
            Self::GameBoyAdvance => "Nintendo - Game Boy Advance",
            Self::Nes => "Nintendo - Nintendo Entertainment System",
            Self::Snes => "Nintendo - Super Nintendo Entertainment System",
            Self::MegaDrive => "Sega - Mega Drive - Genesis",
        }
    }

    /// The list the app was built with
    fn shipped(self) -> &'static str {
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
}

/// The version in a list's first line, e.g. "2026.10.07" from
/// `# Nintendo - Game Boy (No-Intro via libretro-database, 2026.10.07); CC BY-SA 4.0`
fn header_version(text: &str) -> Option<&str> {
    let header = text.lines().next()?.strip_prefix('#')?;
    let rest = header.split_once("libretro-database, ")?.1;
    rest.split_once(')').map(|(version, _)| version)
}

/// Whether list version `a` is newer than `b`. libretro-database dates its lists
/// ("2026.10.07"); a version that isn't a date is never newer.
fn is_newer(a: &str, b: &str) -> bool {
    let is_date = |v: &str| v.len() == 10 && v.bytes().all(|c| c.is_ascii_digit() || c == b'.');
    is_date(a) && (!is_date(b) || a > b)
}

/// Where the app keeps the lists it downloaded
fn downloaded_dir(data_dir: &Path) -> PathBuf {
    data_dir.join("databases").join("no-intro")
}

/// The folder with downloaded lists, set once at start
static DOWNLOADED: OnceLock<PathBuf> = OnceLock::new();

/// The lists in use; replaced when newer ones were downloaded
static ACTIVE: LazyLock<RwLock<Arc<Databases>>> =
    LazyLock::new(|| RwLock::new(Arc::new(Databases::load(DOWNLOADED.get()))));

/// Tells the databases where the data folder is, so the lists downloaded at an earlier start are
/// used where they are newer than the shipped ones. Called once, before the first lookup.
pub fn use_data_dir(data_dir: &Path) {
    let _ = DOWNLOADED.set(downloaded_dir(data_dir));
}

fn active() -> Arc<Databases> {
    ACTIVE
        .read()
        .unwrap_or_else(PoisonError::into_inner)
        .clone()
}

/// Reads the lists again, after newer ones were downloaded
fn reload() {
    let databases = Arc::new(Databases::load(DOWNLOADED.get()));
    *ACTIVE.write().unwrap_or_else(PoisonError::into_inner) = databases;
}

/// One list for every [`List`]
struct Databases([Database; List::ALL.len()]);

impl Databases {
    /// The shipped lists, each replaced by the one in `downloaded` where that one is newer
    fn load(downloaded: Option<&PathBuf>) -> Self {
        Self(List::ALL.map(|list| {
            let shipped = Database::parse(list.shipped());
            downloaded
                .and_then(|dir| fs::read_to_string(dir.join(list.file_name())).ok())
                .map(|text| Database::parse(&text))
                .filter(|db| !db.dumps.is_empty() && is_newer(&db.version, &shipped.version))
                .unwrap_or(shipped)
        }))
    }

    fn get(&self, list: List) -> &Database {
        &self.0[list as usize]
    }

    /// Looks a ROM up in the lists of its platform: by SHA-1, then CRC32, then serial
    fn identify(&self, platform: Platform, rom: &RomHashes) -> Option<Identified> {
        let lists = List::for_platform(platform);
        let found = |list: List, index: usize, how| Identified {
            release: self.get(list).dumps[index].to_string(),
            platform: list.platform(),
            how,
        };

        for &list in lists {
            if let Some(&i) = self.get(list).by_sha1.get(&rom.sha1) {
                return Some(found(list, i, Match::Verified));
            }
        }
        for &list in lists {
            if let Some(&i) = self.get(list).by_crc32.get(&rom.crc32) {
                return Some(found(list, i, Match::Crc32));
            }
        }
        let serial = rom.serial.as_deref()?;
        lists.iter().find_map(|&list| {
            Some(Identified {
                release: self.get(list).title_of_serial(serial)?.to_string(),
                platform: list.platform(),
                how: Match::Serial,
            })
        })
    }
}

#[derive(Debug, Default)]
struct Database {
    /// the list's version, e.g. "2026.10.07"; empty when its first line doesn't name one
    version: String,
    /// the release name of each dump, e.g. "Metroid Fusion (USA)"
    dumps: Vec<Box<str>>,
    by_sha1: HashMap<[u8; 20], usize>,
    by_crc32: HashMap<u32, usize>,
    by_serial: HashMap<Box<str>, Vec<usize>>,
}

impl Database {
    /// Lines of `CRC32 \t SHA-1 \t serial \t name`; `#` starts a comment, the first line names
    /// the version. A line that doesn't fit is skipped, so one bad line doesn't cost the whole
    /// list.
    fn parse(text: &str) -> Self {
        let mut db = Self {
            version: header_version(text).unwrap_or_default().to_string(),
            ..Self::default()
        };
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
                db.by_serial.entry(serial.into()).or_default().push(index);
            }
            db.dumps.push(name.into());
        }
        db
    }

    /// The title all dumps with `serial` share, without region and other tags; none when the
    /// serial is a placeholder that several games use (Mega Drive "00000000-00" and the like)
    fn title_of_serial(&self, serial: &str) -> Option<&str> {
        let mut titles = self.by_serial.get(serial)?.iter().map(|&i| {
            let name = &*self.dumps[i];
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
    active().identify(platform, rom)
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
        let databases = Databases::load(None);
        for list in List::ALL {
            let db = databases.get(list);
            assert!(db.dumps.len() > 1000, "{list:?}");
            assert!(db.dumps.iter().all(|name| !name.is_empty()), "{list:?}");
            assert!(
                is_newer(&db.version, "2026.01.01"),
                "{list:?}: {}",
                db.version
            );
            // every line was read
            let lines = list
                .shipped()
                .lines()
                .filter(|l| !l.starts_with('#'))
                .count();
            assert_eq!(db.dumps.len(), lines, "{list:?}");
        }
    }

    #[test]
    fn a_known_dump_is_found_by_its_checksums() {
        // the list's own entry, as if a file with these checksums had been read
        let line = List::GameBoyAdvance
            .shipped()
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
            .shipped()
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
