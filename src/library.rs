// SPDX-FileCopyrightText: 2026 Yanik Fuchs <yanik.fuchs@me.com>
// SPDX-License-Identifier: GPL-3.0-only

//! The game collection on the computer: everything found in the ROM sources

use crate::{
    n3ds::{cia, smdh},
    romdb,
};
use iced::widget::image;
use smol::{Timer, future::FutureExt};
mod sevenzip;

use std::{
    collections::HashMap,
    ffi::OsStr,
    fs::{self, File},
    io::{BufReader, Read},
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
    time::{Duration, Instant, UNIX_EPOCH},
};

/// How deep a scan goes into a source's folders
const MAX_DEPTH: usize = 8;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum Platform {
    N3ds,
    Wii,
    GameCube,
    WiiU,
    GameBoy,
    GameBoyColor,
    GameBoyAdvance,
    Nes,
    Snes,
    MegaDrive,
    N64,
    Nds,
    Msx,
    Other,
}

impl Platform {
    pub const ALL: [Self; 14] = [
        Self::N3ds,
        Self::Wii,
        Self::GameCube,
        Self::WiiU,
        Self::GameBoy,
        Self::GameBoyColor,
        Self::GameBoyAdvance,
        Self::Nes,
        Self::Snes,
        Self::MegaDrive,
        Self::N64,
        Self::Nds,
        Self::Msx,
        Self::Other,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::N3ds => "3DS",
            Self::Wii => "Wii",
            Self::GameCube => "GameCube",
            Self::WiiU => "Wii U",
            Self::GameBoy => "Game Boy",
            Self::GameBoyColor => "Game Boy Color",
            Self::GameBoyAdvance => "GBA",
            Self::Nes => "NES",
            Self::Snes => "SNES",
            Self::MegaDrive => "Mega Drive",
            Self::N64 => "N64",
            Self::Nds => "DS",
            Self::Msx => "MSX",
            Self::Other => "Other",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ItemKind {
    /// an installable 3DS title
    Cia,
    /// a 3DS game card dump
    Cart,
    Disc,
    Wad,
    /// a Wii U title folder for installing (with title.tmd)
    WiiUTitle,
    /// a ROM for an emulator or an inject
    Rom,
    /// a copy of a console's SD card: listed as one entry, not looked into
    SdBackup,
    /// a RAR, ZIP or 7z archive, or a release folder of RAR parts; unpacked when needed
    Archive,
}

/// The platform a folder name stands for, e.g. "GBA", "Wii U inject", "gc isos"
fn platform_of_folder(name: &str) -> Option<Platform> {
    Some(match name.to_ascii_lowercase().as_str() {
        "3ds" | "3ds inject" | "n3ds" => Platform::N3ds,
        "wiiu" | "wii u" | "wii u inject" => Platform::WiiU,
        "wii" | "wbfs" => Platform::Wii,
        "gc" | "ngc" | "gamecube" | "gc isos" | "games" => Platform::GameCube,
        "n64" => Platform::N64,
        "nds" | "ds" => Platform::Nds,
        "gba" => Platform::GameBoyAdvance,
        "gbc" => Platform::GameBoyColor,
        "gb" | "gb(c)" | "gameboy" => Platform::GameBoy,
        "nes" | "famicom" => Platform::Nes,
        "snes" | "super nintendo" => Platform::Snes,
        "sega" | "md" | "genesis" | "mega drive" | "megadrive" => Platform::MegaDrive,
        "msx" => Platform::Msx,
        _ => return None,
    })
}

/// The platform of the nearest folder above `path` that names one
fn platform_from_folders(path: &Path) -> Platform {
    path.ancestors()
        .skip(1)
        .filter_map(|dir| dir.file_name()?.to_str())
        .find_map(platform_of_folder)
        .unwrap_or(Platform::Other)
}

/// A title ID (16 hex digits) or game ID (6 letters and digits) in a name, e.g. "[RSBE01]"
fn id_in_name(name: &str) -> Option<ItemId> {
    let title_id = |s: &str| {
        (s.len() == 16 && s.chars().all(|c| c.is_ascii_hexdigit()))
            .then(|| u64::from_str_radix(s, 16).ok())
            .flatten()
            .map(ItemId::Title)
    };

    let bracketed = name.split('[').skip(1).filter_map(|s| s.split(']').next());
    for s in bracketed {
        if let Some(id) = title_id(s) {
            return Some(id);
        }
        if s.len() == 6 && s.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Some(ItemId::Game(s.to_ascii_uppercase()));
        }
    }

    title_id(name.split('.').next()?)
}

/// The name of the release a RAR part belongs to: "x.part02.rar" and "x.rar" are "x"
fn rar_base(name: &str) -> Option<&str> {
    let stem = name.strip_suffix(".rar")?;
    Some(match stem.rsplit_once(".part") {
        Some((base, n)) if n.chars().all(|c| c.is_ascii_digit()) => base,
        _ => stem,
    })
}

/// A folder of one release split into RAR parts (.r00 … or .partNN.rar), maybe with .sfv/.nfo
fn is_release_folder(names: &[String]) -> bool {
    if names.iter().any(|n| n.ends_with(".r00")) {
        return true;
    }

    let mut bases = names.iter().filter_map(|n| rar_base(n));
    let Some(first) = bases.next() else {
        return false;
    };
    let one_release = bases.all(|base| base == first);
    let only_parts = names.iter().all(|n| {
        rar_base(n).is_some()
            || [".sfv", ".nfo", ".txt", ".jpg", ".png"]
                .iter()
                .any(|ext| n.ends_with(ext))
            || !n.contains('.')
    });

    one_release && only_parts
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LibraryItem {
    pub path: PathBuf,
    pub platform: Platform,
    pub kind: ItemKind,
    pub name: String,
    pub size: u64,
    /// 3DS title ID or Wii/GameCube game ID, when the file was read
    pub id: Option<ItemId>,
    #[serde(skip)]
    pub icon: Option<image::Handle>,
    /// only in the cloud: not read, so that scanning doesn't download it
    pub in_cloud: bool,
    /// the same file in other places (other sources, backups)
    pub copies: Vec<PathBuf>,
    /// checksums of a ROM of a platform the app has a game database for, and what it is
    #[serde(default)]
    pub rom: Option<RomInfo>,
    /// the Redump name of a Wii or GameCube disc, found by its game ID
    #[serde(default)]
    pub release: Option<String>,
}

/// What a ROM is, from its checksums
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RomInfo {
    /// CRC32 of the ROM's data as No-Intro hashes it: without an iNES or copier header, and
    /// unpacked when it is zipped
    pub crc32: u32,
    /// SHA-1 (hex) of the same data
    pub sha1: String,
    /// the cartridge serial in the ROM's header, e.g. "BPEE"
    pub serial: Option<String>,
    /// the release it is in the game database, if it is there
    pub identified: Option<romdb::Identified>,
}

impl RomInfo {
    fn new(platform: Platform, checked: &Checked) -> Self {
        let mut info = Self {
            crc32: checked.crc32,
            sha1: checked.sha1.clone(),
            serial: checked.serial.clone(),
            identified: None,
        };
        info.identify(platform);
        info
    }

    /// Looks the ROM up again, e.g. after the databases were updated
    fn identify(&mut self, platform: Platform) {
        self.identified = romdb::sha1_from_hex(&self.sha1).and_then(|sha1| {
            romdb::identify(
                platform,
                &romdb::RomHashes {
                    crc32: self.crc32,
                    sha1,
                    serial: self.serial.clone(),
                },
            )
        });
    }

    /// The release's name when the checksum identified it (not only the serial)
    pub fn release(&self) -> Option<&str> {
        self.identified
            .as_ref()
            .filter(|found| found.how != romdb::Match::Serial)
            .map(|found| found.release.as_str())
    }
}

impl LibraryItem {
    /// The name to show: the release's when the ROM or disc was identified, else the file's
    pub fn title(&self) -> &str {
        self.rom
            .as_ref()
            .and_then(RomInfo::release)
            .or(self.release.as_deref())
            .unwrap_or(&self.name)
    }

    /// Looks a Wii or GameCube disc up in the Redump list by its game ID
    fn identify_disc(&mut self) {
        self.release = match &self.id {
            Some(ItemId::Game(id)) => romdb::disc_name(self.platform, id),
            _ => None,
        };
    }

    /// Whether the item is a Wii or GameCube disc, which can be copied to a drive
    pub fn is_drive_game(&self) -> bool {
        self.kind == ItemKind::Disc && matches!(self.platform, Platform::Wii | Platform::GameCube)
    }

    /// Whether the item is a folder (an unpacked release, an SD card copy, a Wii U title), told
    /// from its kind and name so that a disk that doesn't answer isn't asked.
    fn is_folder(&self) -> bool {
        match self.kind {
            ItemKind::SdBackup | ItemKind::WiiUTitle => true,
            ItemKind::Archive => classify(&self.path).is_none(),
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ItemId {
    Title(u64),
    Game(String),
}

fn classify(path: &Path) -> Option<(Platform, ItemKind)> {
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    let name = path.file_name()?.to_str()?.to_ascii_lowercase();

    Some(match ext.as_str() {
        // archives outside of a platform's folder are rarely games (films, e-books …); a zip
        // with one ROM in it is read anywhere ("Other" until it was looked into)
        "rar" | "zip" | "7z" => match platform_from_folders(path) {
            Platform::Other if ext != "zip" => return None,
            platform => (platform, ItemKind::Archive),
        },
        "cia" => (Platform::N3ds, ItemKind::Cia),
        "3ds" | "cci" => (Platform::N3ds, ItemKind::Cart),
        "wbfs" | "rvz" | "wia" | "ciso" => (Platform::Wii, ItemKind::Disc),
        // GameCube or Wii: decided by the header when it is read
        "iso" | "gcz" | "gcm" => (Platform::GameCube, ItemKind::Disc),
        "wad" => (Platform::Wii, ItemKind::Wad),
        "gb" => (Platform::GameBoy, ItemKind::Rom),
        "gbc" => (Platform::GameBoyColor, ItemKind::Rom),
        "gba" => (Platform::GameBoyAdvance, ItemKind::Rom),
        "nes" => (Platform::Nes, ItemKind::Rom),
        "sfc" | "smc" => (Platform::Snes, ItemKind::Rom),
        "md" | "gen" | "smd" => (Platform::MegaDrive, ItemKind::Rom),
        _ if name == "title.tmd" => (Platform::WiiU, ItemKind::WiiUTitle),
        _ => return None,
    })
}

/// Files that iCloud & co. keep only in the cloud; reading them would download them
#[cfg(target_os = "macos")]
fn is_dataless(meta: &fs::Metadata) -> bool {
    use std::os::macos::fs::MetadataExt;
    const SF_DATALESS: u32 = 0x4000_0000;
    meta.st_flags() & SF_DATALESS != 0
}

#[cfg(not(target_os = "macos"))]
fn is_dataless(_meta: &fs::Metadata) -> bool {
    false
}

/// A release folder's name; scene names use dots and underscores for spaces and often start
/// with the platform, as in "gc_skies_of_arcadia_legends_usa"
fn display_name_of_folder(path: &Path) -> String {
    let name = path
        .file_name()
        .and_then(OsStr::to_str)
        .unwrap_or_default()
        .replace(['_', '.'], " ");
    let name = name.trim();

    match name.split_once(' ') {
        Some((first, rest))
            if platform_of_folder(first).is_some() || first.eq_ignore_ascii_case("wiiware") =>
        {
            rest.trim().to_string()
        }
        _ => name.to_string(),
    }
}

/// Whether a name is a bare title ID (16 hex digits), maybe with an extension or RAR part
fn is_title_id_name(name: &str) -> bool {
    let stem = name.split('.').next().unwrap_or_default();
    stem.len() == 16 && stem.chars().all(|c| c.is_ascii_hexdigit())
}

/// The title ID a file or folder name starts with, e.g. "0005000010104839 - WS Patched.rar"
fn leading_title_id(name: &str) -> Option<&str> {
    let start = name.get(..16)?;
    (start.chars().all(|c| c.is_ascii_hexdigit())
        && !name[16..].starts_with(|c: char| c.is_ascii_alphanumeric()))
    .then_some(start)
}

/// A name for an item that is only a title ID (a Wii U inject or forwarder), from the folders
/// above it, nearest first: one that names this title ID in brackets
/// ("SMSAdvance - Roms A-G v4 [0005000010102139]"), else one that starts with a platform
/// ("n64_paper_mario_usa", also above "_eShop" or a variant's folder), else the nearest one that
/// holds nothing but this title. None at a platform's folder ("N64") or at a pack of several
/// titles ("PocketNES Pack (USB)"), where no folder names this one.
fn name_from_folders(path: &Path, title_id: u64) -> Option<String> {
    let id = format!("{title_id:016x}");
    let pretty = |dir: &Path| {
        let name = display_name_of_folder(dir);
        let name = match name.rsplit_once(" [") {
            Some((base, rest)) if rest.ends_with(']') => base.trim_end().to_string(),
            _ => name,
        };
        (!name.is_empty()).then_some(name)
    };

    let mut fallback = None;
    for dir in path.ancestors().skip(1).take(5) {
        let name = dir.file_name()?.to_str()?;
        if platform_of_folder(name).is_some() {
            break;
        }
        if name.to_ascii_lowercase().contains(&format!("[{id}]")) {
            return pretty(dir);
        }

        let names: Vec<String> = fs::read_dir(dir)
            .ok()?
            .flatten()
            .filter_map(|entry| entry.file_name().into_string().ok())
            .filter(|name| !name.starts_with('.'))
            .collect();
        let mut ids: Vec<String> = names
            .iter()
            .filter_map(|name| leading_title_id(name))
            .map(str::to_ascii_lowercase)
            .collect();
        ids.sort_unstable();
        ids.dedup();
        if ids.len() > 1 {
            break;
        }

        if name.starts_with('_') || is_title_id_name(name) {
            continue;
        }
        if has_platform_prefix(name) {
            return pretty(dir);
        }
        let only_this_title = names.iter().all(|name| {
            leading_title_id(name).is_some()
                || name.starts_with('_')
                || !name.contains('.')
                || [".sfv", ".nfo", ".txt", ".jpg", ".png"]
                    .iter()
                    .any(|ext| name.to_ascii_lowercase().ends_with(ext))
        });
        if fallback.is_none() && only_this_title {
            fallback = pretty(dir);
        }
    }
    fallback
}

/// Whether a folder's name starts with a platform, e.g. "n64_paper_mario_usa", "wiiware_fluidity"
fn has_platform_prefix(name: &str) -> bool {
    let name = name.replace(['_', '.'], " ");
    name.trim().split_once(' ').is_some_and(|(first, _)| {
        platform_of_folder(first).is_some() || first.eq_ignore_ascii_case("wiiware")
    })
}

fn display_name(path: &Path) -> String {
    path.file_stem()
        .and_then(OsStr::to_str)
        .unwrap_or_default()
        .replace('_', " ")
}

/// Folders on the SD card of a console next to `apps/`; with them, a folder is an SD card copy
const SD_MARKERS: [&str; 8] = [
    "private",
    "bootmii",
    "usb-loader",
    "wad",
    "wiiu",
    "luma",
    "3ds",
    "nintendo 3ds",
];

/// The console of a folder that is a copy of an SD card, if it is one
fn sd_backup_platform(dir: &Path) -> Option<Platform> {
    let names: Vec<String> = fs::read_dir(dir)
        .ok()?
        .flatten()
        .filter(|entry| entry.file_type().is_ok_and(|t| t.is_dir()))
        .filter_map(|entry| entry.file_name().to_str().map(str::to_ascii_lowercase))
        .collect();
    let has = |name: &str| names.iter().any(|n| n == name);

    // a folder of games may well have a "WiiU" folder; only next to apps/ is it an SD card
    let is_sd = has("nintendo 3ds") || (has("apps") && SD_MARKERS.iter().any(|marker| has(marker)));
    if !is_sd {
        return None;
    }

    Some(if has("wiiu") {
        Platform::WiiU
    } else if has("nintendo 3ds") || has("luma") || has("3ds") {
        Platform::N3ds
    } else {
        Platform::Wii
    })
}

/// Checksums of a ROM file as the last scans found them, to recognise a file that didn't change,
/// or was only renamed or moved, without reading it again
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct Checked {
    path: PathBuf,
    size: u64,
    /// last change, in nanoseconds since 1970
    modified: u64,
    crc32: u32,
    sha1: String,
    serial: Option<String>,
}

/// The checksums of earlier scans, by path and by size and time of the last change, and what
/// the RAR and 7z archives hold
#[derive(Debug, Default)]
struct Known {
    by_path: HashMap<PathBuf, Checked>,
    by_stamp: HashMap<(u64, u64), Vec<Checked>>,
    /// archives looked into, by path; true when this scan came across it
    archives: Mutex<HashMap<PathBuf, (Peeked, bool)>>,
}

impl Known {
    fn new(checked: Vec<Checked>) -> Self {
        let mut known = Self::default();
        for c in checked {
            known
                .by_stamp
                .entry((c.size, c.modified))
                .or_default()
                .push(c.clone());
            known.by_path.insert(c.path.clone(), c);
        }
        known
    }

    /// Adds what earlier scans found in the archives
    fn with_archives(self, peeked: Vec<Peeked>) -> Self {
        let archives = peeked.into_iter().map(|p| (p.path.clone(), (p, false)));
        Self {
            archives: Mutex::new(archives.collect()),
            ..self
        }
    }

    /// The archives this scan came across, to remember for the next one
    fn archives_seen(&self) -> Vec<Peeked> {
        self.archives
            .lock()
            .map(|archives| {
                archives
                    .values()
                    .filter(|(_, seen)| *seen)
                    .map(|(peeked, _)| peeked.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// The checksums of `path` when it is unchanged; or, when the path is new, of the file with
    /// the same size and time of last change (renaming or moving a file keeps both). Of several
    /// such files, the one with the same name; none when they differ, then the file is read.
    fn lookup(&self, path: &Path, size: u64, modified: u64) -> Option<&Checked> {
        if let Some(c) = self.by_path.get(path) {
            return (c.size == size && c.modified == modified).then_some(c);
        }

        let candidates = self.by_stamp.get(&(size, modified))?;
        let same_name: Vec<&Checked> = candidates
            .iter()
            .filter(|c| c.path.file_name() == path.file_name())
            .collect();
        let candidates: Vec<&Checked> = if same_name.is_empty() {
            candidates.iter().collect()
        } else {
            same_name
        };
        let first = *candidates.first()?;
        candidates
            .iter()
            .all(|c| c.sha1 == first.sha1)
            .then_some(first)
    }
}

/// ROMs larger than this aren't read for their checksum (none of the platforms with a database
/// has one this big)
const MAX_ROM_SIZE: u64 = 64 * 1024 * 1024;

/// The ROM in a zip, when the zip holds exactly one (besides text files, pictures and the like):
/// its name in the zip and platform
fn single_rom_in_zip(path: &Path) -> Option<(String, Platform)> {
    let mut archive = zip::ZipArchive::new(BufReader::new(File::open(path).ok()?)).ok()?;
    let mut roms = (0..archive.len()).filter_map(|i| {
        let entry = archive.by_index_raw(i).ok()?;
        let name = entry.name().ok()?.into_owned();
        let file_name = Path::new(&name).file_name()?.to_str()?;
        if entry.is_dir() || name.starts_with("__MACOSX/") || file_name.starts_with('.') {
            return None;
        }
        match classify(Path::new(&name))? {
            (platform, ItemKind::Rom) => Some((name, platform)),
            _ => None,
        }
    });
    let rom = roms.next()?;
    roms.next().is_none().then_some(rom)
}

/// Whether a file is a zip, by its name
fn is_zip(path: &Path) -> bool {
    path.extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"))
}

/// Whether a RAR file is the second or a later part of a release ("x.part02.rar")
fn is_later_part(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    let stem = lower.strip_suffix(".rar").unwrap_or(&lower);
    stem.rsplit_once(".part").is_some_and(|(_, part)| {
        part.chars().all(|c| c.is_ascii_digit()) && part.trim_start_matches('0') != "1"
    })
}

/// What an archive holds, as far as the library cares
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct Contents {
    /// the one ROM in it (its path in the archive), when it holds exactly one
    rom: Option<String>,
    /// Wii U titles in it in Loadiine's form (a folder with `meta/meta.xml`): title ID and name
    titles: Vec<(u64, String)>,
}

/// What an archive held when it had this size and time of the last change
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct Peeked {
    path: PathBuf,
    size: u64,
    /// last change, in nanoseconds since 1970
    modified: u64,
    contents: Contents,
}

/// At most this many `meta.xml` are read from one archive (a pack has up to 25 titles)
const MAX_TITLES_IN_ARCHIVE: usize = 64;

/// What is in an archive (also a zip), looked into with 7-Zip once and remembered. None without
/// 7-Zip or when it can't be read; then it is tried again at the next scan.
fn peek(path: &Path, known: &Known) -> Option<Contents> {
    let meta = fs::metadata(path).ok()?;
    let (size, modified) = (meta.len(), modified_nanos(&meta).unwrap_or_default());
    if let Ok(mut archives) = known.archives.lock()
        && let Some((peeked, seen)) = archives.get_mut(path)
        && peeked.size == size
        && peeked.modified == modified
    {
        *seen = true;
        return Some(peeked.contents.clone());
    }

    let entries = sevenzip::list(path)?;
    let files: Vec<&sevenzip::Entry> = entries.iter().filter(|e| !e.is_dir).collect();

    let mut roms = files.iter().filter(|e| {
        let name = e.slash_path();
        let file_name = name.rsplit('/').next().unwrap_or_default();
        !name.starts_with("__MACOSX/")
            && !file_name.starts_with('.')
            && matches!(classify(Path::new(&name)), Some((_, ItemKind::Rom)))
    });
    let rom = match (roms.next(), roms.next()) {
        (Some(rom), None) => Some(rom.path.clone()),
        _ => None,
    };

    let metas: Vec<&str> = files
        .iter()
        .filter(|e| {
            e.size <= 64 * 1024
                && e.slash_path()
                    .to_ascii_lowercase()
                    .ends_with("meta/meta.xml")
        })
        .take(MAX_TITLES_IN_ARCHIVE)
        .map(|e| e.path.as_str())
        .collect();
    let titles = sevenzip::read(path, &metas, 4 * 1024 * 1024)
        .map(|data| titles_in_meta_xml(&data))
        .unwrap_or_default();

    let contents = Contents { rom, titles };
    if let Ok(mut archives) = known.archives.lock() {
        let peeked = Peeked {
            path: path.to_path_buf(),
            size,
            modified,
            contents: contents.clone(),
        };
        archives.insert(path.to_path_buf(), (peeked, true));
    }
    Some(contents)
}

/// The text of the first `<tag …>text</tag>` in an XML document, unescaped
fn xml_text(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}");
    let start = xml
        .match_indices(&open)
        .map(|(i, _)| i + open.len())
        .find(|&i| xml[i..].starts_with([' ', '>', '\t', '\n', '\r']))?;
    let after = &xml[start..];
    let text = &after[after.find('>')? + 1..];
    let text = &text[..text.find(&format!("</{tag}>"))?];

    let text = text
        .replace("&#x0A;", " ")
        .replace("&#x0a;", " ")
        .replace("&#10;", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&");
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    (!text.is_empty()).then_some(text)
}

/// The title IDs and English names in one or more `meta.xml` of Wii U titles, one after the other
fn titles_in_meta_xml(data: &[u8]) -> Vec<(u64, String)> {
    String::from_utf8_lossy(data)
        .split("</menu>")
        .filter_map(|xml| {
            let id = u64::from_str_radix(xml_text(xml, "title_id")?.trim(), 16).ok()?;
            let name = xml_text(xml, "longname_en").or_else(|| xml_text(xml, "shortname_en"))?;
            Some((id, name))
        })
        .collect()
}

/// Archives (RAR, 7z, zip) are looked into for a title's name only this many to a folder
const MAX_ARCHIVES_FOR_NAMES: usize = 16;

/// The name of a Wii U title that is only a title ID, from a `meta.xml` in the archive itself or
/// in an archive next to it or a few folders up (a pack's Loadiine RAR next to its folder of
/// installable titles); never past a platform's folder
fn name_from_archives(path: &Path, title_id: u64, known: &Known) -> Option<String> {
    let in_archive = |archive: &Path| {
        let contents = peek(archive, known)?;
        contents
            .titles
            .into_iter()
            .find(|(id, _)| *id == title_id)
            .map(|(_, name)| name)
    };
    let is_archive = |p: &Path| {
        let name = p.file_name().and_then(OsStr::to_str).unwrap_or_default();
        let ext = p
            .extension()
            .and_then(OsStr::to_str)
            .unwrap_or_default()
            .to_ascii_lowercase();
        !name.starts_with('.')
            && matches!(ext.as_str(), "rar" | "7z" | "zip")
            && !is_later_part(name)
    };

    if is_archive(path)
        && let Some(name) = in_archive(path)
    {
        return Some(name);
    }

    for dir in path.ancestors().skip(1).take(3) {
        let name = dir.file_name()?.to_str()?;
        if platform_of_folder(name).is_some() {
            break;
        }
        let mut archives: Vec<PathBuf> = fs::read_dir(dir)
            .ok()?
            .flatten()
            .map(|entry| entry.path())
            .filter(|p| p != path && p.is_file() && is_archive(p))
            .collect();
        archives.sort();
        if let Some(name) = archives
            .iter()
            .take(MAX_ARCHIVES_FOR_NAMES)
            .find_map(|archive| in_archive(archive))
        {
            return Some(name);
        }
    }
    None
}

/// The whole data of a ROM file, or of `member` in the archive at `path`
fn read_rom(path: &Path, member: Option<&str>) -> Option<Vec<u8>> {
    let mut data = Vec::new();
    match member {
        None => {
            File::open(path).ok()?.read_to_end(&mut data).ok()?;
        }
        Some(member) if !is_zip(path) => {
            data = sevenzip::read(path, &[member], MAX_ROM_SIZE)?;
        }
        Some(member) => {
            let mut archive = zip::ZipArchive::new(BufReader::new(File::open(path).ok()?)).ok()?;
            let mut file = archive.by_name(member).ok()?;
            if file.size() > MAX_ROM_SIZE {
                return None;
            }
            file.read_to_end(&mut data).ok()?;
        }
    }
    Some(data)
}

/// Last change of a file in nanoseconds since 1970
fn modified_nanos(meta: &fs::Metadata) -> Option<u64> {
    let since = meta.modified().ok()?.duration_since(UNIX_EPOCH).ok()?;
    u64::try_from(since.as_nanos()).ok()
}

/// Fills in a ROM's checksums and identifies it: from `known` when the file is known, else by
/// reading it. Returns the checksums to remember for the next scan.
fn check_rom(
    item: &mut LibraryItem,
    meta: &fs::Metadata,
    member: Option<&str>,
    known: &Known,
) -> Option<Checked> {
    if !romdb::has_database(item.platform) || item.size > MAX_ROM_SIZE {
        return None;
    }
    let modified = modified_nanos(meta);

    let checked = match modified.and_then(|m| known.lookup(&item.path, item.size, m)) {
        Some(c) => Checked {
            path: item.path.clone(),
            ..c.clone()
        },
        None => {
            let mut data = read_rom(&item.path, member)?;
            // a zip under a ROM's name ("game.gba"): the one ROM in it
            if member.is_none() && data.starts_with(b"PK\x03\x04") {
                let (member, _) = single_rom_in_zip(&item.path)?;
                data = read_rom(&item.path, Some(&member))?;
            }
            let hashes = romdb::hash_rom(item.platform, &data);
            Checked {
                path: item.path.clone(),
                size: item.size,
                modified: modified.unwrap_or_default(),
                crc32: hashes.crc32,
                sha1: hex::encode(hashes.sha1),
                serial: hashes.serial,
            }
        }
    };

    let info = RomInfo::new(item.platform, &checked);
    if let Some(found) = &info.identified {
        item.platform = found.platform;
    }
    item.rom = Some(info);
    // without the time of the last change, a file can't be told unchanged: read it every time
    modified.map(|_| checked)
}

fn read_item(
    path: PathBuf,
    platform: Platform,
    kind: ItemKind,
    known: &Known,
) -> Option<(LibraryItem, Option<Checked>)> {
    if kind == ItemKind::Archive && path.is_dir() {
        let size = fs::read_dir(&path)
            .ok()?
            .flatten()
            .filter_map(|entry| entry.metadata().ok())
            .filter(fs::Metadata::is_file)
            .map(|meta| meta.len())
            .sum();
        let name = display_name_of_folder(&path);
        // the title ID is often only in the parts' names, e.g. 0005000010105317.part01.rar
        let id = id_in_name(&name).or_else(|| {
            fs::read_dir(&path)
                .ok()?
                .flatten()
                .filter_map(|e| e.file_name().to_str().map(str::to_string))
                .find_map(|n| id_in_name(&n))
        });
        let mut item = LibraryItem {
            id,
            name,
            size,
            platform,
            kind,
            icon: None,
            in_cloud: false,
            copies: Vec::new(),
            rom: None,
            release: None,
            path,
        };
        item.identify_disc();
        return Some((item, None));
    }

    if kind == ItemKind::SdBackup {
        let item = LibraryItem {
            name: display_name(&path),
            size: smol::block_on(crate::util::fs::get_dir_size(&path)),
            platform,
            kind,
            id: None,
            icon: None,
            in_cloud: false,
            copies: Vec::new(),
            rom: None,
            release: None,
            path,
        };
        return Some((item, None));
    }

    let meta = fs::metadata(&path).ok()?;
    let in_cloud = is_dataless(&meta);

    let mut item = LibraryItem {
        name: display_name(&path),
        size: meta.len(),
        platform,
        kind,
        id: None,
        icon: None,
        in_cloud,
        copies: Vec::new(),
        rom: None,
        release: None,
        path,
    };

    if kind == ItemKind::WiiUTitle {
        // the title folder is the game: name and size are the folder's
        let folder = item.path.parent()?.to_path_buf();
        item.name = display_name(&folder);
        if is_title_id_name(&item.name)
            && let Ok(title_id) = u64::from_str_radix(&item.name, 16)
        {
            item.name = name_from_folders(&folder, title_id)
                .or_else(|| name_from_archives(&folder, title_id, known))
                .unwrap_or(item.name);
        }
        item.size = smol::block_on(crate::util::fs::get_dir_size(&folder));
        item.path = folder;
        return Some((item, None));
    }

    if in_cloud {
        // a zip outside a platform's folder is only a game when it holds a ROM, which can't be
        // seen without downloading it
        return (platform != Platform::Other).then_some((item, None));
    }

    // an archive with one ROM in it is that ROM
    let mut member = None;
    if kind == ItemKind::Archive && is_zip(&item.path) {
        match single_rom_in_zip(&item.path) {
            Some((name, rom_platform)) => {
                item.kind = ItemKind::Rom;
                item.platform = rom_platform;
                member = Some(name);
            }
            None if platform == Platform::Other => return None,
            None => {}
        }
    } else if kind == ItemKind::Archive
        && romdb::has_database(platform)
        && !is_later_part(&item.name)
        // named after a title ID: a Wii U inject, not a ROM; not looked into, as there are
        // thousands of them
        && !is_title_id_name(&item.name)
        && leading_title_id(&item.name).is_none()
        && let Some(rom) = peek(&item.path, known).and_then(|contents| contents.rom)
        && let Some((rom_platform, ItemKind::Rom)) = classify(Path::new(&rom))
    {
        item.kind = ItemKind::Rom;
        item.platform = rom_platform;
        member = Some(rom);
    }

    match item.kind {
        ItemKind::Rom => {
            let checked = check_rom(&mut item, &meta, member.as_deref(), known);
            return Some((item, checked));
        }
        ItemKind::Cia => {
            if let Ok(info) = File::open(&item.path).and_then(|f| cia::read(BufReader::new(f))) {
                item.id = Some(ItemId::Title(info.title_id));
                if let Some(smdh) = info.smdh.as_deref() {
                    if let Some(names) = smdh::names(smdh, "en") {
                        item.name = names.short.replace('\n', " ");
                    }
                    item.icon = smdh::large_icon_rgba(smdh).map(|rgba| {
                        image::Handle::from_rgba(smdh::LARGE_ICON_SIZE, smdh::LARGE_ICON_SIZE, rgba)
                    });
                }
            }
        }
        ItemKind::Disc => {
            if let Ok(disc) =
                File::open(&item.path).and_then(|f| wii_disc_info::Meta::read(BufReader::new(f)))
            {
                item.platform = if disc.is_wii() {
                    Platform::Wii
                } else {
                    Platform::GameCube
                };
                let id = disc.game_id().as_str().to_string();
                // GameTDB's name; the disc's own is often a short internal one
                // ("AITD5_111_080403_EU")
                if let Some(title) = twbm_idmap::get_title(&id) {
                    item.name = title.to_string();
                } else if !disc.game_title().is_empty() {
                    item.name = disc.game_title().to_string();
                }
                item.id = Some(ItemId::Game(id));
            }
        }
        ItemKind::Archive => {
            // part2 and later belong to part1
            let lower = item.name.to_ascii_lowercase();
            if let Some((_, part)) = lower.rsplit_once(".part")
                && part.trim_start_matches('0') != "1"
            {
                return None;
            }
            item.id = id_in_name(&item.name);
            item.name = item
                .name
                .rsplit_once(".part")
                .map_or(item.name.clone(), |(name, _)| name.to_string());
            match &item.id {
                Some(ItemId::Game(id)) => {
                    if let Some(title) = twbm_idmap::get_title(id) {
                        item.name = title.to_string();
                    }
                }
                Some(ItemId::Title(title_id)) if is_title_id_name(&item.name) => {
                    if let Some(name) = name_from_folders(&item.path, *title_id)
                        .or_else(|| name_from_archives(&item.path, *title_id, known))
                    {
                        item.name = name;
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }
    item.identify_disc();

    Some((item, None))
}

fn walk(
    dir: &Path,
    depth: usize,
    found: &mut Vec<(PathBuf, Platform, ItemKind)>,
    progress: &AtomicUsize,
) {
    progress.fetch_add(1, Ordering::Relaxed);
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let hidden = path
            .file_name()
            .and_then(OsStr::to_str)
            .is_some_and(|name| name.starts_with('.'));
        if hidden {
            continue;
        }

        match entry.file_type() {
            Ok(t) if t.is_dir() => {
                let names: Vec<String> = fs::read_dir(&path)
                    .map(|entries| {
                        entries
                            .flatten()
                            .filter_map(|e| e.file_name().to_str().map(str::to_ascii_lowercase))
                            .collect()
                    })
                    .unwrap_or_default();

                let platform_folder = path
                    .file_name()
                    .and_then(OsStr::to_str)
                    .and_then(platform_of_folder)
                    .is_some();

                if !platform_folder && is_release_folder(&names) {
                    let platform = platform_from_folders(&path.join("x"));
                    if platform != Platform::Other {
                        found.push((path, platform, ItemKind::Archive));
                    }
                } else if let Some(platform) = sd_backup_platform(&path) {
                    found.push((path, platform, ItemKind::SdBackup));
                } else if depth < MAX_DEPTH {
                    walk(&path, depth + 1, found, progress);
                }
            }
            Ok(t) if t.is_file() => {
                if let Some((platform, kind)) = classify(&path) {
                    found.push((path, platform, kind));
                }
            }
            _ => {}
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct LibraryState {
    pub scanning: bool,
    pub scanned_once: bool,
    pub items: Vec<LibraryItem>,
    pub platform: Option<Platform>,
    pub search: String,
    pub show_sources: bool,
}

impl LibraryState {
    pub fn platforms(&self) -> Vec<Platform> {
        Platform::ALL
            .into_iter()
            .filter(|p| self.items.iter().any(|item| item.platform == *p))
            .collect()
    }

    pub fn visible(&self) -> impl Iterator<Item = &LibraryItem> {
        let search = self.search.to_lowercase();
        self.items.iter().filter(move |item| {
            self.platform.is_none_or(|p| item.platform == p)
                && (search.is_empty()
                    || item.name.to_lowercase().contains(&search)
                    || item.title().to_lowercase().contains(&search))
        })
    }

    /// The shown Wii and GameCube discs, for "Copy all to drive"
    pub fn shown_drive_games(&self) -> impl Iterator<Item = &LibraryItem> {
        self.visible().filter(|item| item.is_drive_game())
    }
}

/// Scans the sources one after the other; blocking. The app uses [`scan_async`].
#[cfg(test)]
fn scan(sources: &[PathBuf]) -> Vec<LibraryItem> {
    let progress = AtomicUsize::new(0);
    let items = sources
        .iter()
        .flat_map(|source| scan_source(source, &progress, &Known::default()).0)
        .collect();
    sort_and_merge(items)
}

/// The items of one source, each on its own, before copies are merged, and the checksums of its
/// ROMs; blocking. Counts up `progress` for every folder and file it reads, so a caller can tell
/// a source that doesn't answer.
fn scan_source(
    source: &Path,
    progress: &AtomicUsize,
    known: &Known,
) -> (Vec<LibraryItem>, Vec<Checked>) {
    let mut found = Vec::new();
    walk(source, 0, &mut found, progress);

    let mut checked = Vec::new();
    let items = found
        .into_iter()
        .filter_map(|(path, platform, kind)| {
            progress.fetch_add(1, Ordering::Relaxed);
            let (item, c) = read_item(path, platform, kind, known)?;
            checked.extend(c);
            Some(item)
        })
        .collect();
    (items, checked)
}

fn sort_and_merge(mut items: Vec<LibraryItem>) -> Vec<LibraryItem> {
    items.sort_by(|a, b| {
        a.platform
            .cmp(&b.platform)
            .then_with(|| a.title().to_lowercase().cmp(&b.title().to_lowercase()))
    });
    merge_copies(items)
}

/// What makes two items copies of each other
#[derive(Debug, PartialEq, Eq, Hash)]
enum CopyKey {
    /// the same ROM data, under any name, zipped or not
    Rom(String),
    /// the same file name and size
    File(String, u64, bool),
}

/// One entry per file: the same ROM data, or the same file name and size, in several places is
/// one item with copies. The copy that is on this computer (not only in the cloud) leads.
fn merge_copies(items: Vec<LibraryItem>) -> Vec<LibraryItem> {
    let mut merged: Vec<LibraryItem> = Vec::with_capacity(items.len());
    let mut index: HashMap<CopyKey, usize> = HashMap::new();

    for item in items {
        let key = match &item.rom {
            Some(rom) => CopyKey::Rom(rom.sha1.clone()),
            None => CopyKey::File(
                item.path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_lowercase())
                    .unwrap_or_default(),
                item.size,
                item.is_folder(),
            ),
        };

        match index.get(&key) {
            Some(&i) => {
                let first = &mut merged[i];
                if first.in_cloud && !item.in_cloud {
                    let old = std::mem::replace(first, item);
                    first.copies.push(old.path);
                    first.copies.extend(old.copies);
                } else {
                    first.copies.push(item.path);
                }
            }
            None => {
                index.insert(key, merged.len());
                merged.push(item);
            }
        }
    }

    merged
}

/// How long a source may go without reading a single folder or file before the scan gives up
/// on it (a sleeping or hung disk, a lost network share).
const SOURCE_SILENCE: Duration = Duration::from_secs(15);

/// A scan of all sources; the ones that stop answering keep their items from the last scan.
pub struct Scan {
    pub items: Vec<LibraryItem>,
    pub unanswered: Vec<PathBuf>,
}

/// Scans every source on its own thread. A source that reads nothing for [`SOURCE_SILENCE`] is
/// given up (its thread is left to finish or hang); its items come from the last scan saved in
/// `data_dir`, so one dead disk doesn't hold up the others.
pub async fn scan_async(sources: Vec<PathBuf>, data_dir: &'static Path) -> Scan {
    let (sender, receiver) = smol::channel::unbounded();
    let mut pending: Vec<(PathBuf, Arc<AtomicUsize>, usize, Instant)> = Vec::new();
    let (previous_checksums, previous_archives) =
        smol::unblock(move || (read_checksums(data_dir), read_archives(data_dir))).await;
    let all_known =
        Arc::new(Known::new(previous_checksums.clone()).with_archives(previous_archives.clone()));
    let known = all_known.clone();

    for (index, source) in sources.iter().enumerate() {
        let progress = Arc::new(AtomicUsize::new(0));
        pending.push((source.clone(), progress.clone(), 0, Instant::now()));
        let sender = sender.clone();
        let source = source.clone();
        let known = known.clone();
        thread::spawn(move || {
            let (items, checked) = scan_source(&source, &progress, &known);
            let _ = sender.send_blocking((index, items, checked));
        });
    }
    drop(sender);
    drop(known);

    let mut done: Vec<Option<Vec<LibraryItem>>> = vec![None; sources.len()];
    // the checksums each source's scan found; none for a source that was given up
    let mut checksums: Vec<Option<Vec<Checked>>> = vec![None; sources.len()];
    let mut unanswered = Vec::new();
    // read only when a source is given up; the app may not have loaded it yet at the start
    let mut previous: Option<Vec<LibraryItem>> = None;

    // every thread that is still running has ended without a result (it panicked)
    let mut closed = false;

    while done.iter().any(Option::is_none) {
        let next = async { Some(receiver.recv().await) }
            .or(async {
                Timer::after(Duration::from_secs(1)).await;
                None
            })
            .await;
        match next {
            Some(Ok((index, items, checked))) => {
                done[index] = Some(items);
                checksums[index] = Some(checked);
                // a source that was given up may still answer while others are scanned
                unanswered.retain(|source| source != &sources[index]);
            }
            Some(Err(_)) => closed = true,
            None => {}
        }

        let now = Instant::now();
        for (index, (source, progress, last_seen, last_change)) in pending.iter_mut().enumerate() {
            if done[index].is_some() {
                continue;
            }
            let seen = progress.load(Ordering::Relaxed);
            if seen != *last_seen {
                *last_seen = seen;
                *last_change = now;
            } else if closed || now.duration_since(*last_change) >= SOURCE_SILENCE {
                if previous.is_none() {
                    previous = Some(smol::unblock(move || read_cache(data_dir)).await);
                }
                let previous = previous.as_deref().unwrap_or_default();
                done[index] = Some(previous_items_in(previous, source));
                unanswered.push(source.clone());
            }
        }
    }

    // the checksums of a source that didn't answer are kept for the next scan
    let checksums: Vec<Checked> = checksums
        .into_iter()
        .zip(&sources)
        .flat_map(|(checked, source)| {
            checked.unwrap_or_else(|| {
                previous_checksums
                    .iter()
                    .filter(|c| c.path.starts_with(source))
                    .cloned()
                    .collect()
            })
        })
        .collect();
    smol::unblock(move || write_checksums(data_dir, &checksums)).await;

    // likewise what is in the archives
    // (a source given up may still be scanning: its fresh entries count)
    let mut archives = all_known.archives_seen();
    let fresh: std::collections::HashSet<PathBuf> =
        archives.iter().map(|p| p.path.clone()).collect();
    archives.extend(previous_archives.into_iter().filter(|p| {
        !fresh.contains(&p.path) && unanswered.iter().any(|source| p.path.starts_with(source))
    }));
    smol::unblock(move || write_archives(data_dir, &archives)).await;

    let items = done.into_iter().flatten().flatten().collect();
    Scan {
        items: smol::unblock(move || sort_and_merge(items)).await,
        unanswered,
    }
}

/// The items of the last scan that were in `source`, one per file again (copies split off), so
/// they can be merged with the other sources' fresh items. Doesn't touch the disk.
fn previous_items_in(previous: &[LibraryItem], source: &Path) -> Vec<LibraryItem> {
    previous
        .iter()
        .flat_map(|item| {
            std::iter::once(&item.path)
                .chain(&item.copies)
                .filter(|path| path.starts_with(source))
                .map(|path| LibraryItem {
                    path: path.clone(),
                    copies: Vec::new(),
                    // a split-off copy never takes the lead from a fresh item on a disk that answers
                    in_cloud: item.in_cloud || *path != item.path,
                    ..item.clone()
                })
        })
        .collect()
}

const CACHE_FILENAME: &str = "library.json";

/// The checksums of every ROM file the last scan found, copies included; the library cache
/// (`library.json`) has one entry per game, this one per file
const CHECKSUMS_FILENAME: &str = "library-checksums.json";

fn read_checksums(data_dir: &Path) -> Vec<Checked> {
    fs::read_to_string(data_dir.join(CHECKSUMS_FILENAME))
        .ok()
        .and_then(|contents| serde_json::from_str(&contents).ok())
        .unwrap_or_default()
}

/// Writes the checksums; a failure only costs reading the files again at the next scan
fn write_checksums(data_dir: &Path, checksums: &[Checked]) {
    if !data_dir.is_dir() {
        return;
    }
    // a path that isn't UTF-8 can't be in JSON: that file is read again at the next scan
    let checksums: Vec<&Checked> = checksums
        .iter()
        .filter(|c| c.path.to_str().is_some())
        .collect();
    if let Ok(contents) = serde_json::to_string(&checksums) {
        let _ = fs::write(data_dir.join(CHECKSUMS_FILENAME), contents);
    }
}

/// What the RAR and 7z archives hold, as the last scan found it, so they aren't listed again
const ARCHIVES_FILENAME: &str = "library-archives.json";

fn read_archives(data_dir: &Path) -> Vec<Peeked> {
    fs::read_to_string(data_dir.join(ARCHIVES_FILENAME))
        .ok()
        .and_then(|contents| serde_json::from_str(&contents).ok())
        .unwrap_or_default()
}

/// Writes what is in the archives; a failure only costs listing them again at the next scan
fn write_archives(data_dir: &Path, archives: &[Peeked]) {
    if !data_dir.is_dir() {
        return;
    }
    let archives: Vec<&Peeked> = archives
        .iter()
        .filter(|p| p.path.to_str().is_some())
        .collect();
    if let Ok(contents) = serde_json::to_string(&archives) {
        let _ = fs::write(data_dir.join(ARCHIVES_FILENAME), contents);
    }
}

/// The last scan as saved, without icons, so no source is read; blocking.
fn read_cache(data_dir: &Path) -> Vec<LibraryItem> {
    fs::read_to_string(data_dir.join(CACHE_FILENAME))
        .ok()
        .and_then(|contents| serde_json::from_str(&contents).ok())
        .unwrap_or_default()
}

/// Looks the ROMs and discs up again, after the game databases changed
pub fn reidentify(items: &mut [LibraryItem]) {
    for item in items {
        item.identify_disc();
        if let Some(rom) = &mut item.rom {
            rom.identify(item.platform);
            if let Some(found) = &rom.identified {
                item.platform = found.platform;
            }
        }
    }
}

/// The last scan, to show the library at once while the sources are scanned again
pub async fn load_cache(data_dir: &'static Path) -> Vec<LibraryItem> {
    smol::unblock(move || {
        let mut items = read_cache(data_dir);
        // the databases may be newer than the cache (and get newer while it loads: the app
        // looks the ROMs up again when it gets the items)
        reidentify(&mut items);

        // icons aren't cached; a CIA's are read from a few KB of it
        for item in items
            .iter_mut()
            .filter(|item| item.kind == ItemKind::Cia && !item.in_cloud)
        {
            item.icon = File::open(&item.path)
                .and_then(|f| cia::read(BufReader::new(f)))
                .ok()
                .and_then(|info| info.smdh)
                .and_then(|smdh| smdh::large_icon_rgba(&smdh))
                .map(|rgba| {
                    image::Handle::from_rgba(smdh::LARGE_ICON_SIZE, smdh::LARGE_ICON_SIZE, rgba)
                });
        }

        items
    })
    .await
}

pub async fn save_cache(data_dir: &'static Path, items: Vec<LibraryItem>) -> anyhow::Result<()> {
    let contents = serde_json::to_string(&items)?;
    smol::fs::write(data_dir.join(CACHE_FILENAME), contents).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_item_that_is_only_a_title_id_is_named_after_its_folder() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("Wii U inject");
        let write = |path: &str| {
            let path = root.join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, path.to_string_lossy().as_bytes()).unwrap();
        };
        // the game's folder, with a patched copy next to it and the eShop's inject in "_eShop"
        write("N64/n64/n64_paper_mario_usa/0005000010104839.rar");
        write("N64/n64/n64_paper_mario_usa/0005000010104839 - WS Patched.rar");
        write("N64/n64/n64_paper_mario_usa/_eShop/0005000010199700.rar");
        write("N64/n64/n64_paper_mario_usa/_eShop/0005000010199700 - WS Patched.rar");
        write("Wii/forwarders/wiiware_cave_story/0005000010104755.rar");
        write("Wii/forwarders/uae_wii_forwarder/0005000010105279.rar");
        // a title folder in a folder that names it and its ID, and two in a pack
        write("packs/SMSAdvance - Roms A-G v4 [0005000010102139]/0005000010102139/title.tmd");
        write("packs/PocketNES Pack (USB)/0005000010106006/title.tmd");
        write("packs/PocketNES Pack (USB)/0005000010106007/title.tmd");
        // straight in a platform's folder: nothing better than the ID
        write("NDS/0005000010102044.rar");
        // a folder of several games doesn't name one of them
        write("N64/Injects/0005000010105317.rar");
        write("N64/Injects/Mario.zip");

        let items = scan(std::slice::from_ref(&root));
        let mut names: Vec<_> = items.iter().map(|item| item.name.as_str()).collect();
        names.sort_unstable();
        assert_eq!(
            names,
            vec![
                "0005000010102044",
                "0005000010104839 - WS Patched",
                "0005000010105317",
                "0005000010106006",
                "0005000010106007",
                "0005000010199700 - WS Patched",
                "Mario",
                "SMSAdvance - Roms A-G v4",
                "cave story",
                "paper mario usa",
                "paper mario usa",
                "uae wii forwarder",
            ]
        );
    }

    #[test]
    fn names_are_read_from_meta_xml() {
        let xml = "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<menu type=\"complex\">\n\
                   <title_id type=\"hexBinary\" length=\"8\">0005000010106006</title_id>\n\
                   <longname_ja type=\"string\" length=\"512\"></longname_ja>\n\
                   <longname_en type=\"string\" length=\"512\">PocketNES:&#x0A;Roms S &amp; T</longname_en>\n\
                   </menu>\n\
                   <?xml version=\"1.0\"?>\n<menu>\n\
                   <title_id>00050000101060FF</title_id>\n\
                   <longname_en></longname_en>\n<shortname_en>Short</shortname_en>\n</menu>\n\
                   <?xml version=\"1.0\"?>\n<menu><title_id>nonsense</title_id></menu>";
        assert_eq!(
            titles_in_meta_xml(xml.as_bytes()),
            vec![
                (0x0005_0000_1010_6006, "PocketNES: Roms S & T".to_string()),
                (0x0005_0000_1010_60FF, "Short".to_string()),
            ]
        );
    }

    #[test]
    fn later_rar_parts_are_told_apart() {
        assert!(is_later_part("x.part02.rar"));
        assert!(is_later_part("x.part2"));
        assert!(!is_later_part("x.part01.rar"));
        assert!(!is_later_part("x.part1"));
        assert!(!is_later_part("x.rar"));
        assert!(!is_later_part("Counterpart.rar"));
    }

    /// An archive that didn't change isn't listed again: what it held comes from the last scan
    #[test]
    fn an_unchanged_archive_is_remembered() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("game.7z");
        fs::write(&path, b"not an archive at all").unwrap();
        let meta = fs::metadata(&path).unwrap();
        let contents = Contents {
            rom: Some("game.gba".to_string()),
            titles: Vec::new(),
        };
        let peeked = Peeked {
            path: path.clone(),
            size: meta.len(),
            modified: modified_nanos(&meta).unwrap(),
            contents: contents.clone(),
        };
        let known = Known::default().with_archives(vec![
            peeked.clone(),
            Peeked {
                path: dir.path().join("gone.7z"),
                ..peeked.clone()
            },
        ]);
        assert_eq!(peek(&path, &known), Some(contents));
        // only what this scan came across is kept
        assert_eq!(known.archives_seen(), vec![peeked]);

        // changed: listed again, and that fails
        fs::write(&path, b"not an archive, and longer").unwrap();
        assert_eq!(peek(&path, &known), None);
    }

    /// Packs 7z archives with the 7-Zip on this computer; skipped without one
    #[test]
    fn roms_and_names_are_found_in_7z_archives() {
        let Some(program) = sevenzip::program() else {
            eprintln!("no 7-Zip here: skipped");
            return;
        };
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("Games");
        let pack = |archive: &Path, files: &[(&str, &[u8])]| {
            let src = dir.path().join("src");
            let _ = fs::remove_dir_all(&src);
            for (name, data) in files {
                let path = src.join(name);
                fs::create_dir_all(path.parent().unwrap()).unwrap();
                fs::write(path, data).unwrap();
            }
            fs::create_dir_all(archive.parent().unwrap()).unwrap();
            let packed = std::process::Command::new(program)
                .current_dir(&src)
                .args(["a", "-bd"])
                .arg(archive)
                .arg(".")
                .stdout(std::process::Stdio::null())
                .status()
                .unwrap();
            assert!(packed.success());
        };

        // a hack, one ROM in a 7z, and the same ROM unpacked elsewhere
        let rom: Vec<u8> = (0..4096u32).map(|i| (i % 251) as u8).collect();
        pack(
            &root.join("GBA/Pokemon GS Chronicles.7z"),
            &[
                ("Pokemon GS Chronicles 2.7.1.gba", &rom),
                ("readme.txt", b"hi"),
            ],
        );
        fs::write(root.join("GBA/Chronicles.gba"), &rom).unwrap();
        // two ROMs: an archive, not a ROM
        pack(
            &root.join("GBA/Two.7z"),
            &[("a.gba", b"a"), ("b.gba", b"b")],
        );
        // a pack: installable titles named by their ID, their names in the Loadiine archive
        let title = "Wii U inject/gba_pocketnes_pack/PocketNES Pack (USB)/0005000010106006";
        fs::create_dir_all(root.join(title)).unwrap();
        fs::write(root.join(title).join("title.tmd"), b"tmd").unwrap();
        let other = title.replace("6006", "6007");
        fs::create_dir_all(root.join(&other)).unwrap();
        fs::write(root.join(other).join("title.tmd"), b"tmd").unwrap();
        let meta = b"<?xml version=\"1.0\"?><menu>\
                     <title_id type=\"hexBinary\" length=\"8\">0005000010106006</title_id>\
                     <longname_en type=\"string\">PocketNES: Roms S Pack 1</longname_en></menu>";
        pack(
            &root.join("Wii U inject/gba_pocketnes_pack/PocketNES Pack (Loadiine).7z"),
            &[("PocketNES - Roms S 1 [PNES06]/meta/meta.xml", meta)],
        );

        let progress = AtomicUsize::new(0);
        let known = Known::default();
        let (items, checked) = scan_source(&root, &progress, &known);
        let items = sort_and_merge(items);
        let summary: Vec<_> = items
            .iter()
            .map(|item| (item.kind, item.name.as_str(), item.copies.len()))
            .collect();
        assert!(
            summary.contains(&(ItemKind::Rom, "Chronicles", 1)),
            "{summary:?}"
        );
        assert!(
            summary.contains(&(ItemKind::Archive, "Two", 0)),
            "{summary:?}"
        );
        assert!(
            summary.contains(&(ItemKind::WiiUTitle, "PocketNES: Roms S Pack 1", 0)),
            "{summary:?}"
        );
        // not in the archive: no name
        assert!(
            summary.contains(&(ItemKind::WiiUTitle, "0005000010106007", 0)),
            "{summary:?}"
        );
        // the ROM in the 7z has the same checksums as the unpacked one
        assert_eq!(checked.len(), 2);
        assert_eq!(checked[0].sha1, checked[1].sha1);
        // three archives looked into, remembered for the next scan
        assert_eq!(known.archives_seen().len(), 3);
    }

    #[test]
    fn scans_by_extension() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fs::create_dir_all(root.join("GBA/.hidden")).unwrap();
        fs::create_dir_all(root.join("wiiU/Mario Kart 8/meta")).unwrap();
        // a games folder with a WiiU folder is not an SD card
        fs::create_dir_all(root.join("WiiU")).unwrap();
        fs::write(root.join("GBA/Metroid_Fusion.gba"), b"rom").unwrap();
        fs::write(root.join("GBA/.hidden/x.gba"), b"rom").unwrap();
        fs::write(root.join("GBA/notes.txt"), b"").unwrap();
        fs::write(root.join("wiiU/Mario Kart 8/title.tmd"), b"tmd").unwrap();
        fs::write(root.join("broken.cia"), b"not a cia").unwrap();
        fs::create_dir_all(root.join("wii/2024-04-12/apps/nintendont")).unwrap();
        fs::create_dir_all(root.join("wii/2024-04-12/bootmii")).unwrap();
        fs::write(root.join("wii/2024-04-12/IOS36-64-v3351.wad"), b"wad").unwrap();
        let release = root.join("WiiU/Cars_3_EUR_WiiU-CAT");
        fs::create_dir_all(&release).unwrap();
        for part in ["cat-ba4p.rar", "cat-ba4p.r00", "cat-ba4p.sfv"] {
            fs::write(release.join(part), b"12").unwrap();
        }
        let injects = root.join("N64/Injects");
        fs::create_dir_all(&injects).unwrap();
        fs::write(injects.join("0005000010105317.part01.rar"), b"1").unwrap();
        fs::write(injects.join("0005000010105317.part02.rar"), b"1").unwrap();
        fs::write(injects.join("Mario [NSME00].zip"), b"1").unwrap();

        let items = scan(&[root.to_path_buf()]);
        let summary: Vec<_> = items
            .iter()
            .map(|item| (item.platform, item.name.as_str()))
            .collect();

        assert_eq!(
            summary,
            vec![
                (Platform::N3ds, "broken"),
                (Platform::Wii, "2024-04-12"),
                (Platform::WiiU, "Cars 3 EUR WiiU-CAT"),
                (Platform::WiiU, "Mario Kart 8"),
                (Platform::GameBoyAdvance, "Metroid Fusion"),
                (Platform::N64, "0005000010105317"),
                (Platform::N64, "Mario [NSME00]"),
            ]
        );
        let mut copy = root.join("backup");
        fs::create_dir_all(&copy).unwrap();
        copy.push("Metroid_Fusion.gba");
        fs::write(&copy, b"rom").unwrap();
        let fusion = scan(&[root.to_path_buf()]);
        let fusion = fusion.iter().find(|i| i.name == "Metroid Fusion").unwrap();
        assert_eq!(fusion.copies.len(), 1);

        let release = items
            .iter()
            .find(|i| i.name == "Cars 3 EUR WiiU-CAT")
            .unwrap();
        assert_eq!((release.platform, release.size), (Platform::WiiU, 6));
        assert_eq!(
            display_name_of_folder(Path::new("gc_skies_of_arcadia_usa")),
            "skies of arcadia usa"
        );
        let inject = items.iter().find(|i| i.name == "0005000010105317").unwrap();
        assert_eq!(inject.id, Some(ItemId::Title(0x0005_0000_1010_5317)));
        let zip = items.iter().find(|i| i.name == "Mario [NSME00]").unwrap();
        assert_eq!(zip.id, Some(ItemId::Game("NSME00".into())));
        assert_eq!(items[0].id, None);
    }

    #[test]
    fn a_scan_of_every_source_matches_the_blocking_one() {
        let a = tempfile::tempdir().unwrap();
        let b = tempfile::tempdir().unwrap();
        fs::create_dir_all(a.path().join("GBA")).unwrap();
        fs::create_dir_all(b.path().join("GBA")).unwrap();
        fs::write(a.path().join("GBA/Metroid_Fusion.gba"), b"rom").unwrap();
        fs::write(b.path().join("GBA/Metroid_Fusion.gba"), b"rom").unwrap();
        fs::write(b.path().join("GBA/Golden_Sun.gba"), b"other rom").unwrap();
        let sources = vec![a.path().to_path_buf(), b.path().to_path_buf()];

        // every source answers, so the last scan isn't read
        let data_dir = Path::new("no-cache-needed");
        let result = smol::block_on(scan_async(sources.clone(), data_dir));
        assert!(result.unanswered.is_empty());
        let names = |items: &[LibraryItem]| -> Vec<(String, usize)> {
            items
                .iter()
                .map(|i| (i.name.clone(), i.copies.len()))
                .collect()
        };
        assert_eq!(names(&result.items), names(&scan(&sources)));
        assert_eq!(
            names(&result.items),
            vec![("Golden Sun".into(), 0), ("Metroid Fusion".into(), 1)]
        );
    }

    #[test]
    fn only_shown_discs_go_to_the_drive() {
        let item = |name: &str, platform, kind| LibraryItem {
            path: PathBuf::from(format!("/Volumes/a/{name}")),
            platform,
            kind,
            name: name.into(),
            size: 1,
            id: None,
            icon: None,
            in_cloud: false,
            copies: Vec::new(),
            rom: None,
            release: None,
        };
        let mut library = LibraryState {
            items: vec![
                item("Mario Kart Wii", Platform::Wii, ItemKind::Disc),
                item("Mario Kart DD", Platform::GameCube, ItemKind::Disc),
                item("Mario Kart 7", Platform::N3ds, ItemKind::Cia),
                item("Mario Kart 8", Platform::WiiU, ItemKind::WiiUTitle),
                item("Mario Kart Wii channel", Platform::Wii, ItemKind::Wad),
                item("Zelda", Platform::Wii, ItemKind::Disc),
            ],
            search: "mario".into(),
            ..LibraryState::default()
        };
        let names = |library: &LibraryState| {
            library
                .shown_drive_games()
                .map(|item| item.name.clone())
                .collect::<Vec<_>>()
        };

        assert_eq!(names(&library), ["Mario Kart Wii", "Mario Kart DD"]);
        library.platform = Some(Platform::Wii);
        assert_eq!(names(&library), ["Mario Kart Wii"]);
    }

    #[test]
    fn the_last_scan_of_a_source_is_split_into_its_files() {
        let item = LibraryItem {
            path: PathBuf::from("/Volumes/a/GBA/Metroid_Fusion.gba"),
            platform: Platform::GameBoyAdvance,
            kind: ItemKind::Rom,
            name: "Metroid Fusion".into(),
            size: 3,
            id: None,
            icon: None,
            in_cloud: false,
            copies: vec![
                PathBuf::from("/Volumes/b/GBA/Metroid_Fusion.gba"),
                PathBuf::from("/Volumes/b/backup/Metroid_Fusion.gba"),
            ],
            rom: None,
            release: None,
        };

        let from_b = previous_items_in(std::slice::from_ref(&item), Path::new("/Volumes/b"));
        assert_eq!(from_b.len(), 2);
        assert!(
            from_b
                .iter()
                .all(|i| i.copies.is_empty() && i.name == "Metroid Fusion")
        );
        // only copies here, so neither may take the lead from a fresh item
        assert!(from_b.iter().all(|i| i.in_cloud));
        assert!(previous_items_in(&[item], Path::new("/Volumes/c")).is_empty());
    }

    fn write_zip(path: &Path, files: &[(&str, &[u8])]) {
        use std::io::Write;
        let mut zip = zip::ZipWriter::new(File::create(path).unwrap());
        for (name, data) in files {
            zip.start_file(*name, zip::write::SimpleFileOptions::default())
                .unwrap();
            zip.write_all(data).unwrap();
        }
        zip.finish().unwrap();
    }

    /// A GBA ROM with `serial` as its game code
    fn gba_rom(serial: &[u8; 4], fill: u8) -> Vec<u8> {
        let mut rom = vec![fill; 0x4000];
        rom[0xAC..0xB0].copy_from_slice(serial);
        rom
    }

    #[test]
    fn a_zipped_rom_is_read_like_an_unpacked_one() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let rom = gba_rom(b"AMTE", 1);
        fs::create_dir_all(root.join("GBA")).unwrap();
        fs::create_dir_all(root.join("Downloads")).unwrap();
        fs::write(root.join("GBA/Metroid_Fusion.gba"), &rom).unwrap();
        // outside a platform's folder too, with a text file next to the ROM
        write_zip(
            &root.join("Downloads/Metroid Fusion (USA).zip"),
            &[("readme.txt", b"hi"), ("Metroid Fusion (USA).gba", &rom)],
        );
        // two ROMs: not one game, and not in a platform's folder either
        write_zip(
            &root.join("Downloads/two.zip"),
            &[("a.gba", &rom), ("b.gba", &gba_rom(b"BPEE", 2))],
        );
        // a zip under a ROM's name
        write_zip(
            &root.join("GBA/Metroid Fusion copy.gba"),
            &[("Metroid Fusion (USA).gba", &rom)],
        );
        // no ROM at all: an archive in a platform's folder, nothing elsewhere
        write_zip(&root.join("GBA/saves.zip"), &[("a.sav", b"save")]);
        write_zip(&root.join("Downloads/photos.zip"), &[("a.jpg", b"jpg")]);

        let items = scan(&[root.to_path_buf()]);
        let summary: Vec<_> = items.iter().map(|i| (i.kind, i.name.as_str())).collect();
        assert_eq!(
            summary,
            vec![
                (ItemKind::Rom, "Metroid Fusion"),
                (ItemKind::Archive, "saves")
            ]
        );

        // the same data zipped, unpacked and zipped under a ROM's name: one game, two copies
        let fusion = &items[0];
        assert_eq!(fusion.copies.len(), 2);
        let info = fusion.rom.as_ref().unwrap();
        assert_eq!(info.crc32, crc32fast::hash(&rom));
        assert_eq!(info.serial.as_deref(), Some("AMTE"));
        // only the serial is known (the data isn't a real dump): the title, not a release
        let found = info.identified.as_ref().unwrap();
        assert_eq!(
            (found.how, found.release.as_str()),
            (romdb::Match::Serial, "Metroid Fusion")
        );
        assert_eq!(info.release(), None);
        assert_eq!(fusion.title(), "Metroid Fusion");
    }

    #[test]
    fn a_renamed_or_moved_rom_isnt_read_again() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fs::create_dir_all(root.join("GBA/old")).unwrap();
        let path = root.join("GBA/old/fusion.gba");
        fs::write(&path, gba_rom(b"AMTE", 1)).unwrap();

        let progress = AtomicUsize::new(0);
        let (items, checked) = scan_source(root, &progress, &Known::default());
        assert_eq!(checked.len(), 1);
        let first = items[0].rom.clone().unwrap();

        // moved and renamed; the data is changed behind the scan's back, keeping size and time,
        // so a result from the old checksums shows the file wasn't read
        let modified = fs::metadata(&path).unwrap().modified().unwrap();
        fs::create_dir_all(root.join("GBA/new")).unwrap();
        let moved = root.join("GBA/new/Metroid Fusion.gba");
        fs::rename(&path, &moved).unwrap();
        fs::write(&moved, gba_rom(b"BPEE", 9)).unwrap();
        File::options()
            .write(true)
            .open(&moved)
            .unwrap()
            .set_modified(modified)
            .unwrap();

        let known = Known::new(checked);
        let (items, checked) = scan_source(root, &progress, &known);
        assert_eq!(items[0].rom.as_ref(), Some(&first));
        assert_eq!(checked[0].path, moved);

        // without what was known, it is read
        let (items, _) = scan_source(root, &progress, &Known::default());
        assert_eq!(
            items[0].rom.as_ref().unwrap().serial.as_deref(),
            Some("BPEE")
        );

        // a changed file is read again
        fs::write(&moved, gba_rom(b"BPEE", 9)).unwrap();
        File::options()
            .write(true)
            .open(&moved)
            .unwrap()
            .set_modified(modified + Duration::from_secs(2))
            .unwrap();
        let (items, _) = scan_source(root, &progress, &known);
        assert_eq!(
            items[0].rom.as_ref().unwrap().serial.as_deref(),
            Some("BPEE")
        );
    }

    #[test]
    fn of_several_files_with_the_same_size_and_time_none_is_guessed() {
        let checked = |path: &str, sha1: &str| Checked {
            path: PathBuf::from(path),
            size: 4,
            modified: 7,
            crc32: 0,
            sha1: sha1.into(),
            serial: None,
        };
        let known = Known::new(vec![
            checked("/a/x.gb", "11"),
            checked("/a/y.gb", "22"),
            checked("/b/z.gb", "22"),
        ]);
        // same name
        assert_eq!(
            known
                .lookup(Path::new("/c/x.gb"), 4, 7)
                .map(|c| c.sha1.as_str()),
            Some("11")
        );
        // renamed: two candidates with different data
        assert_eq!(known.lookup(Path::new("/c/w.gb"), 4, 7), None);
        // a known path that changed isn't taken from another file
        assert_eq!(known.lookup(Path::new("/a/x.gb"), 4, 8), None);
        assert_eq!(known.lookup(Path::new("/c/x.gb"), 5, 7), None);
    }

    #[test]
    fn folders_are_told_without_the_disk() {
        let item = |path: &str, kind| LibraryItem {
            path: PathBuf::from(path),
            platform: Platform::WiiU,
            kind,
            name: String::new(),
            size: 0,
            id: None,
            icon: None,
            in_cloud: false,
            copies: Vec::new(),
            rom: None,
            release: None,
        };
        assert!(item("/x/WiiU/Cars_3_EUR_WiiU-CAT", ItemKind::Archive).is_folder());
        assert!(!item("/x/WiiU/Cars 3.rar", ItemKind::Archive).is_folder());
        assert!(item("/x/WiiU/Mario Kart 8", ItemKind::WiiUTitle).is_folder());
        assert!(!item("/x/Wii/Glacier.wbfs", ItemKind::Disc).is_folder());
    }
}
