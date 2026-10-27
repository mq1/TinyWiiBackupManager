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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
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
}

impl Platform {
    pub const ALL: [Self; 10] = [
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
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
}

#[derive(Debug, Clone)]
pub struct LibraryItem {
    pub path: PathBuf,
    pub platform: Platform,
    pub kind: ItemKind,
    pub name: String,
    pub size: u64,
    /// 3DS title ID or Wii/GameCube game ID, when the file was read
    pub id: Option<ItemId>,
    pub icon: Option<image::Handle>,
    /// only in the cloud: not read, so that scanning doesn't download it
    pub in_cloud: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemId {
    Title(u64),
    Game(String),
}

fn classify(path: &Path) -> Option<(Platform, ItemKind)> {
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    let name = path.file_name()?.to_str()?.to_ascii_lowercase();

    Some(match ext.as_str() {
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

    let is_sd = has("nintendo 3ds")
        || has("wiiu")
        || (has("apps") && SD_MARKERS.iter().any(|marker| has(marker)));
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
    if kind == ItemKind::SdBackup {
        return Some(LibraryItem {
            name: display_name(&path),
            size: smol::block_on(crate::util::fs::get_dir_size(&path)),
            platform,
            kind,
            id: None,
            icon: None,
            in_cloud: false,
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
                if let Some(platform) = sd_backup_platform(&path) {
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
    items
}

pub async fn scan_async(sources: Vec<PathBuf>) -> Vec<LibraryItem> {
    smol::unblock(move || scan(&sources)).await
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
        fs::write(root.join("GBA/Metroid_Fusion.gba"), b"rom").unwrap();
        fs::write(root.join("GBA/.hidden/x.gba"), b"rom").unwrap();
        fs::write(root.join("GBA/notes.txt"), b"").unwrap();
        fs::write(root.join("wiiU/Mario Kart 8/title.tmd"), b"tmd").unwrap();
        fs::write(root.join("broken.cia"), b"not a cia").unwrap();
        fs::create_dir_all(root.join("wii/2024-04-12/apps/nintendont")).unwrap();
        fs::create_dir_all(root.join("wii/2024-04-12/bootmii")).unwrap();
        fs::write(root.join("wii/2024-04-12/IOS36-64-v3351.wad"), b"wad").unwrap();

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
                (Platform::WiiU, "Mario Kart 8"),
                (Platform::GameBoyAdvance, "Metroid Fusion"),
            ]
        );
        assert_eq!(items[0].id, None);
    }
}
