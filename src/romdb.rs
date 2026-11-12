// SPDX-FileCopyrightText: 2026 Yanik Fuchs <yanik.fuchs@me.com>
// SPDX-License-Identifier: GPL-3.0-only

//! Game databases: a ROM of the older consoles is identified by its checksum against the
//! No-Intro lists of libretro-database, a Wii or GameCube disc by its game ID (or its checksum)
//! against the Redump lists. The app ships them (`assets/databases/`, refreshed with
//! `scripts/update-databases.py`) and asks libretro-database for newer ones once a day
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

/// One No-Intro or Redump list, as compacted by `scripts/update-databases.py`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum List {
    GameBoy,
    GameBoyColor,
    GameBoyAdvance,
    Nes,
    Snes,
    MegaDrive,
    Wii,
    GameCube,
}

impl List {
    const ALL: [Self; 8] = [
        Self::GameBoy,
        Self::GameBoyColor,
        Self::GameBoyAdvance,
        Self::Nes,
        Self::Snes,
        Self::MegaDrive,
        Self::Wii,
        Self::GameCube,
    ];

    /// The list's folder in libretro-database's `metadat/`, in `assets/databases/` and in the
    /// data folder
    fn source(self) -> &'static str {
        match self {
            Self::Wii | Self::GameCube => "redump",
            _ => "no-intro",
        }
    }

    /// How the list's first line names its source
    fn source_name(self) -> &'static str {
        match self.source() {
            "redump" => "Redump",
            _ => "No-Intro",
        }
    }

    /// The list's file name in its source's folder
    fn file_name(self) -> &'static str {
        match self {
            Self::GameBoy => "gb.tsv",
            Self::GameBoyColor => "gbc.tsv",
            Self::GameBoyAdvance => "gba.tsv",
            Self::Nes => "nes.tsv",
            Self::Snes => "snes.tsv",
            Self::MegaDrive => "md.tsv",
            Self::Wii => "wii.tsv",
            Self::GameCube => "gc.tsv",
        }
    }

    /// The list's path below the folder of the databases, e.g. "redump/wii.tsv"
    fn path(self) -> String {
        format!("{}/{}", self.source(), self.file_name())
    }

    /// The DAT's name in libretro-database (`metadat/<source>/<name>.dat`)
    fn dat_name(self) -> &'static str {
        match self {
            Self::GameBoy => "Nintendo - Game Boy",
            Self::GameBoyColor => "Nintendo - Game Boy Color",
            Self::GameBoyAdvance => "Nintendo - Game Boy Advance",
            Self::Nes => "Nintendo - Nintendo Entertainment System",
            Self::Snes => "Nintendo - Super Nintendo Entertainment System",
            Self::MegaDrive => "Sega - Mega Drive - Genesis",
            Self::Wii => "Nintendo - Wii",
            Self::GameCube => "Nintendo - GameCube",
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
            Self::Wii => include_str!("../assets/databases/redump/wii.tsv"),
            Self::GameCube => include_str!("../assets/databases/redump/gc.tsv"),
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
            Self::Wii => Platform::Wii,
            Self::GameCube => Platform::GameCube,
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

    /// The list of a disc of `platform`
    fn for_disc(platform: Platform) -> Option<Self> {
        match platform {
            Platform::Wii => Some(Self::Wii),
            Platform::GameCube => Some(Self::GameCube),
            _ => None,
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

/// Where the app keeps the lists it downloaded, each in its source's folder
fn downloaded_dir(data_dir: &Path) -> PathBuf {
    data_dir.join("databases")
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
                .and_then(|dir| fs::read_to_string(dir.join(list.path())).ok())
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

    /// The release name of a disc of `platform` with the SHA-1 of the whole disc
    fn disc_by_sha1(&self, platform: Platform, sha1: &[u8; 20]) -> Option<&str> {
        let db = self.get(List::for_disc(platform)?);
        db.by_sha1.get(sha1).map(|&i| &*db.dumps[i])
    }

    /// The release name of a disc of `platform` by its game ID, e.g. "RMGP01"
    fn disc_by_game_id(&self, platform: Platform, game_id: &str) -> Option<String> {
        let code = game_id.get(..4)?;
        let db = self.get(List::for_disc(platform)?);
        db.name_of_game_code(code)
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
    /// the dumps of each Wii or GameCube game code, e.g. "RMGP" (the game ID without the maker)
    by_game_code: HashMap<[u8; 4], Vec<usize>>,
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
            if let Some(code) = game_code_of_serial(serial) {
                db.by_game_code.entry(code).or_default().push(index);
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

    /// The name of the release with a Wii or GameCube game code. A game's discs and revisions
    /// share the code, and Redump lists some discs twice (with and without the disc's revision
    /// in the serial): without "(Disc 1)", "(Rev 2)" and the like they are one name. When they
    /// still differ (a demo on the same code, say), only the title they share.
    fn name_of_game_code(&self, code: &str) -> Option<String> {
        let code: [u8; 4] = code.as_bytes().try_into().ok()?;
        let mut names = self.by_game_code.get(&code)?.iter().map(|&i| {
            let name = &*self.dumps[i];
            without_disc_and_revision(name)
        });
        let first = names.next()?;
        let rest = names.collect::<Vec<_>>();
        if rest.iter().all(|n| *n == first) {
            return Some(first);
        }
        let title = |n: &str| n.split(" (").next().unwrap_or(n).to_string();
        let first_title = title(&first);
        rest.iter()
            .all(|n| title(n) == first_title)
            .then_some(first_title)
    }
}

/// The game code in a Redump serial: "RMGE" in "RVL-RMGE-USA-B0" (Wii) and "GW7P" in
/// "DL-DOL-GW7P-EUR" (GameCube); none in the serials of other lists
fn game_code_of_serial(serial: &str) -> Option<[u8; 4]> {
    let rest = serial.strip_prefix("DL-").unwrap_or(serial);
    let (prefix, rest) = rest.split_once('-')?;
    if !matches!(prefix, "RVL" | "RVLE" | "DOL") {
        return None;
    }
    let code = rest.split('-').next()?;
    let code: [u8; 4] = code.as_bytes().try_into().ok()?;
    code.iter()
        .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit())
        .then_some(code)
}

/// A release name without its "(Disc N)" and "(Rev N)" tags
fn without_disc_and_revision(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut rest = name;
    while let Some(start) = rest.find(" (") {
        let Some(len) = rest[start..].find(')') else {
            break;
        };
        let tag = &rest[start + 2..start + len];
        out.push_str(&rest[..start]);
        if !(tag.starts_with("Disc ") || tag.starts_with("Rev ")) {
            out.push_str(&rest[start..=start + len]);
        }
        rest = &rest[start + len + 1..];
    }
    out.push_str(rest);
    out
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

/// The Redump name of a Wii or GameCube disc by its game ID, e.g. "Super Mario Galaxy (Europe)
/// (En,Fr,De,Es,It)" for "RMGP01"; without the disc's number and revision, which the game ID
/// doesn't tell
///
/// Hacks and custom discs often keep another game's code under their own maker code
/// ("RMCE13" is a Wiimms Mario Kart Wii mod, "RQQE52" a Guitar Hero III custom on Backyard
/// Football's code), which the Redump serials don't have. So when GameTDB knows the whole game
/// ID under a different name, that disc isn't the Redump release and gets no name here.
pub fn disc_name(platform: Platform, game_id: &str) -> Option<String> {
    let release = active().disc_by_game_id(platform, game_id)?;
    match twbm_idmap::get_title(game_id) {
        Some(title) if !names_the_same_game(title, &release) => None,
        _ => Some(release),
    }
}

/// The letter without its accent ("é" is "e"), for comparing names; other characters as they are
fn without_accent(c: char) -> char {
    const ACCENTED: &str = "ÀÁÂÃÄÅàáâãäåÇçÈÉÊËèéêëÌÍÎÏìíîïÑñÒÓÔÕÖØòóôõöøÙÚÛÜùúûüÝýÿ";
    const PLAIN: &str = "AAAAAAaaaaaaCcEEEEeeeeIIIIiiiiNnOOOOOOooooooUUUUuuuuYyy";
    ACCENTED
        .chars()
        .position(|a| a == c)
        .and_then(|i| PLAIN.chars().nth(i))
        .unwrap_or(c)
}

/// Whether GameTDB's title of a game ID and a Redump name are the same game: every word of the
/// title (of three letters or more, or a number) is in the Redump name, regardless of case,
/// accents and punctuation. A title without such words (Japanese) and GameTDB's placeholder
/// "Scene release" say nothing.
fn names_the_same_game(gametdb: &str, redump: &str) -> bool {
    fn words(name: &str) -> Vec<String> {
        let folded: String = name.chars().map(without_accent).collect();
        folded
            .split(|c: char| !c.is_ascii_alphanumeric())
            .filter(|w| w.len() >= 3 || (!w.is_empty() && w.chars().all(|c| c.is_ascii_digit())))
            .map(str::to_ascii_lowercase)
            .collect()
    }

    if gametdb.to_ascii_lowercase().starts_with("scene release") {
        return true;
    }
    // the Redump name without its regions and languages: "(Europe) (En,Fr,De,Es,It)"
    let redump = redump.split(" (").next().unwrap_or(redump);
    let redump = words(redump);
    words(gametdb).iter().all(|word| redump.contains(word))
}

/// The Redump name of the Wii or GameCube disc with this SHA-1 (of the whole disc), when it is a
/// known good dump
pub fn disc_by_sha1(platform: Platform, sha1: &[u8; 20]) -> Option<String> {
    active().disc_by_sha1(platform, sha1).map(str::to_string)
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

    #[test]
    fn a_hack_on_another_games_code_gets_no_redump_name() {
        for (gametdb, redump) in [
            (
                "Mario Kart: Double Dash!!",
                "Mario Kart - Double Dash!! (Europe, Australia)",
            ),
            (
                "Pokemon Battle Revolution",
                "Pokemon Battle Revolution (Europe, Australia)",
            ),
            (
                "Die drei ???: Das verfluchte Schloss",
                "drei Fragezeichen, Die - Das verfluchte Schloss (Germany)",
            ),
            (
                "Scene release [SE8P41]",
                "Just Dance 2018 (Europe) (En,Fr,De,Es,It,Nl)",
            ),
        ] {
            assert!(names_the_same_game(gametdb, redump), "{gametdb}");
        }
        for (gametdb, redump) in [
            (
                "Wiimms MKW Fun 2012-01.ntsc",
                "Mario Kart Wii (USA) (En,Fr,Es)",
            ),
            (
                "Guitar Hero III Custom : Queen",
                "Backyard Football '09 (USA)",
            ),
            (
                "Super Mario Skyland",
                "New Super Mario Bros. Wii (USA) (En,Fr,Es)",
            ),
            (
                "Mario Kart Wii CTGP Revolution",
                "Mario Kart Wii (USA) (En,Fr,Es)",
            ),
        ] {
            assert!(!names_the_same_game(gametdb, redump), "{gametdb}");
        }
    }
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
    fn a_disc_is_named_by_its_game_id() {
        assert_eq!(
            disc_name(Platform::Wii, "RMGP01").as_deref(),
            Some("Super Mario Galaxy (Europe, Australia) (En,Fr,De,Es,It)")
        );
        // both discs of a GameCube game share the ID
        assert_eq!(
            disc_name(Platform::GameCube, "GQSE01").as_deref(),
            Some("Tales of Symphonia (USA)")
        );
        // the Wii list isn't asked for a GameCube game, nor for a ROM
        assert_eq!(disc_name(Platform::GameCube, "RMGP01"), None);
        assert_eq!(disc_name(Platform::Wii, "ZZZZ01"), None);
        assert_eq!(disc_name(Platform::Wii, "RM"), None);
        assert_eq!(disc_name(Platform::GameBoyAdvance, "AMTE01"), None);
    }

    #[test]
    fn game_codes_are_read_from_redump_serials() {
        let code =
            |serial| game_code_of_serial(serial).map(|c| String::from_utf8_lossy(&c).into_owned());
        assert_eq!(code("RVL-RMGE-USA-B0").as_deref(), Some("RMGE"));
        assert_eq!(code("RVLE-SBSE-USA-B0").as_deref(), Some("SBSE"));
        assert_eq!(code("DL-DOL-GQSE-1-USA-1").as_deref(), Some("GQSE"));
        assert_eq!(code("DOL-GALE-USA").as_deref(), Some("GALE"));
        // other lists' serials, and Redump's that aren't game codes
        assert_eq!(code("AMTE"), None);
        assert_eq!(code("T-48073-00"), None);
        assert_eq!(code("DS000102"), None);
    }

    #[test]
    fn disc_and_revision_are_left_out_of_a_name() {
        assert_eq!(
            without_disc_and_revision("Tales of Symphonia (USA) (Disc 2)"),
            "Tales of Symphonia (USA)"
        );
        assert_eq!(
            without_disc_and_revision("Mario Strikers Charged Football (Europe) (En,Fr) (Rev 2)"),
            "Mario Strikers Charged Football (Europe) (En,Fr)"
        );
        assert_eq!(without_disc_and_revision("Odd (Name"), "Odd (Name");
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
