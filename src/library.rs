// SPDX-FileCopyrightText: 2026 Yanik Fuchs <yanik.fuchs@me.com>
// SPDX-License-Identifier: GPL-3.0-only

//! The game collection on the computer: everything found in the ROM sources

use crate::n3ds::{cia, smdh};
use iced::widget::image;
use std::{
    ffi::OsStr,
    fs::{self, File},
    io::BufReader,
    path::{Path, PathBuf},
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
        // archives outside of a platform's folder are rarely games (films, e-books …)
        "rar" | "zip" | "7z" => match platform_from_folders(path) {
            Platform::Other => return None,
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
        Some((first, rest)) if platform_of_folder(first).is_some() => rest.trim().to_string(),
        _ => name.to_string(),
    }
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

fn read_item(path: PathBuf, platform: Platform, kind: ItemKind) -> Option<LibraryItem> {
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
        return Some(LibraryItem {
            id,
            name,
            size,
            platform,
            kind,
            icon: None,
            in_cloud: false,
            copies: Vec::new(),
            path,
        });
    }

    if kind == ItemKind::SdBackup {
        return Some(LibraryItem {
            name: display_name(&path),
            size: smol::block_on(crate::util::fs::get_dir_size(&path)),
            platform,
            kind,
            id: None,
            icon: None,
            in_cloud: false,
            copies: Vec::new(),
            path,
        });
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
        path,
    };

    if kind == ItemKind::WiiUTitle {
        // the title folder is the game: name and size are the folder's
        let folder = item.path.parent()?.to_path_buf();
        item.name = display_name(&folder);
        item.size = smol::block_on(crate::util::fs::get_dir_size(&folder));
        item.path = folder;
        return Some(item);
    }

    if in_cloud {
        return Some(item);
    }

    match kind {
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
                item.id = Some(ItemId::Game(disc.game_id().as_str().to_string()));
                if !disc.game_title().is_empty() {
                    item.name = disc.game_title().to_string();
                }
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
        }
        _ => {}
    }

    Some(item)
}

fn walk(dir: &Path, depth: usize, found: &mut Vec<(PathBuf, Platform, ItemKind)>) {
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
                    walk(&path, depth + 1, found);
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
                && (search.is_empty() || item.name.to_lowercase().contains(&search))
        })
    }
}

/// Scans the sources; blocking, run it with smol::unblock
pub fn scan(sources: &[PathBuf]) -> Vec<LibraryItem> {
    let mut found = Vec::new();
    for source in sources {
        walk(source, 0, &mut found);
    }

    let mut items: Vec<LibraryItem> = found
        .into_iter()
        .filter_map(|(path, platform, kind)| read_item(path, platform, kind))
        .collect();

    items.sort_by(|a, b| {
        a.platform
            .cmp(&b.platform)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    merge_copies(items)
}

/// One entry per file: the same file name and size in several places is one item with copies.
/// The copy that is on this computer (not only in the cloud) leads.
fn merge_copies(items: Vec<LibraryItem>) -> Vec<LibraryItem> {
    let mut merged: Vec<LibraryItem> = Vec::with_capacity(items.len());
    let mut index: std::collections::HashMap<(String, u64, bool), usize> =
        std::collections::HashMap::new();

    for item in items {
        let key = (
            item.path
                .file_name()
                .map(|n| n.to_string_lossy().to_lowercase())
                .unwrap_or_default(),
            item.size,
            item.path.is_dir(),
        );

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

pub async fn scan_async(sources: Vec<PathBuf>) -> Vec<LibraryItem> {
    smol::unblock(move || scan(&sources)).await
}

const CACHE_FILENAME: &str = "library.json";

/// The last scan, to show the library at once while the sources are scanned again
pub async fn load_cache(data_dir: &'static Path) -> Vec<LibraryItem> {
    smol::unblock(move || {
        let Ok(contents) = fs::read_to_string(data_dir.join(CACHE_FILENAME)) else {
            return Vec::new();
        };
        let mut items: Vec<LibraryItem> = serde_json::from_str(&contents).unwrap_or_default();

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
}
