// SPDX-FileCopyrightText: 2026 Yanik Fuchs <yanik.fuchs@me.com>
// SPDX-License-Identifier: GPL-3.0-only

//! The consoles running a companion app: found in the network and remembered

use crate::{
    link::{
        client::Link,
        discovery::{self, Found},
        protocol::{Console, Info, PORT, Title, n3ds},
    },
    n3ds::smdh,
};
use anyhow::Result;
use iced::widget::image;
use serde::{Deserialize, Serialize};
use smol::fs;
use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr},
    path::{Path, PathBuf},
    time::Duration,
};
use which_fs::FsKind;

const FILENAME: &str = "consoles.json";
const DISCOVERY_TIMEOUT: Duration = Duration::from_secs(2);
// icons per request: one round trip for many, but not too much for the 3DS's memory
const METADATA_BATCH: usize = 20;

/// The vcinject Link version this app expects; older ones get an update offer
pub const VCINJECT_LINK_VERSION: u32 = 13;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleEntry {
    pub addr: SocketAddr,
    pub info: Info,
    #[serde(skip)]
    pub online: bool,
    #[serde(skip)]
    pub label: String,
}

impl ConsoleEntry {
    fn new(addr: SocketAddr, info: Info, online: bool) -> Self {
        let mut entry = Self {
            addr,
            info,
            online,
            label: String::new(),
        };
        entry.update_label();
        entry
    }

    /// Name and model, e.g. "Petter (3DS XL)": names alone are often alike
    fn update_label(&mut self) {
        let model = match self.info.console() {
            Some(Console::N3ds) => n3ds::model_name(self.info.get("model").unwrap_or_default()),
            Some(Console::Wii) => "Wii",
            Some(Console::WiiU) => "Wii U",
            None => "Console",
        };

        self.label = match self.info.name() {
            Some(name) => format!("{name} ({model})"),
            None => format!("{model} at {}", self.addr.ip()),
        };
    }

    fn is(&self, found: &Found) -> bool {
        match (self.info.device_id(), found.info.device_id()) {
            (Some(a), Some(b)) => a == b,
            _ => self.addr.ip() == found.addr.ip(),
        }
    }

    pub fn outdated(&self) -> bool {
        self.info.app() == "vcinject-link" && self.info.version() < VCINJECT_LINK_VERSION
    }
}

#[derive(Debug, Clone)]
pub struct InstalledTitle {
    pub title: Title,
    pub name: String,
    pub publisher: String,
    pub icon: Option<image::Handle>,
}

#[derive(Debug, Clone, Default)]
pub enum TitlesState {
    #[default]
    NotLoaded,
    Loading,
    Loaded(Vec<InstalledTitle>),
    Failed(String),
}

#[derive(Debug, Clone, Default)]
pub struct ConsolesState {
    pub searching: bool,
    pub searched_once: bool,
    pub entries: Vec<ConsoleEntry>,
    /// The console whose titles are shown
    pub selected: Option<SocketAddr>,
    pub titles: HashMap<SocketAddr, TitlesState>,
    pub drives: Vec<Drive>,
}

/// A remembered drive for a Wii or GameCube (USB disk or SD card)
#[derive(Debug, Clone)]
pub struct Drive {
    pub path: PathBuf,
    pub label: String,
    /// None while it isn't plugged in
    pub space: Option<DriveSpace>,
}

#[derive(Debug, Clone, Copy)]
pub struct DriveSpace {
    pub free: u64,
    pub total: u64,
    pub fs_kind: FsKind,
}

impl Drive {
    fn load(path: PathBuf) -> Self {
        let label = path
            .file_name()
            .unwrap_or(path.as_os_str())
            .to_string_lossy()
            .into_owned();

        let space = path
            .is_dir()
            .then(|| fs4::statvfs(&path).ok())
            .flatten()
            .map(|stat| DriveSpace {
                free: stat.available_space(),
                total: stat.total_space(),
                fs_kind: FsKind::try_from_path(&path).unwrap_or(FsKind::Unknown),
            });

        Self { path, label, space }
    }

    /// The same judgement as the toolbox's drive info
    pub fn fs_warning(&self) -> Option<&'static str> {
        match self.space?.fs_kind {
            FsKind::Fat32 => None,
            FsKind::Ntfs | FsKind::ExFat => {
                Some("limited support for game backups and homebrew apps")
            }
            FsKind::Unknown => Some("unknown support"),
            _ => Some("won't work for game backups and homebrew apps"),
        }
    }
}

pub async fn load_drives(paths: Vec<PathBuf>) -> Vec<Drive> {
    smol::unblock(move || paths.into_iter().map(Drive::load).collect()).await
}

impl ConsolesState {
    /// Online consoles whose titles haven't been read yet
    pub fn without_titles(&self) -> Vec<SocketAddr> {
        self.entries
            .iter()
            .filter(|entry| entry.online && entry.info.console() == Some(Console::N3ds))
            .map(|entry| entry.addr)
            .filter(|addr| !self.titles.contains_key(addr))
            .collect()
    }

    /// The online consoles that have a title installed
    pub fn having_title(&self, title_id: u64) -> impl Iterator<Item = &ConsoleEntry> {
        self.entries.iter().filter(move |entry| {
            entry.online
                && matches!(self.titles.get(&entry.addr), Some(TitlesState::Loaded(titles))
                    if titles.iter().any(|t| t.title.id == title_id))
        })
    }

    pub fn known_addresses(&self) -> Vec<IpAddr> {
        self.entries.iter().map(|entry| entry.addr.ip()).collect()
    }

    pub fn set_known(&mut self, entries: Vec<ConsoleEntry>) {
        self.entries = entries
            .into_iter()
            .map(|entry| ConsoleEntry::new(entry.addr, entry.info, false))
            .collect();
    }

    /// Takes the result of a search: answering consoles are online, the others offline
    pub fn merge(&mut self, found: Vec<Found>) {
        for entry in &mut self.entries {
            entry.online = false;
        }

        for found in found {
            match self.entries.iter_mut().find(|entry| entry.is(&found)) {
                Some(entry) => *entry = ConsoleEntry::new(found.addr, found.info, true),
                None => self
                    .entries
                    .push(ConsoleEntry::new(found.addr, found.info, true)),
            }
        }

        self.entries
            .sort_by(|a, b| b.online.cmp(&a.online).then(a.label.cmp(&b.label)));
    }
}

fn path(data_dir: &Path) -> PathBuf {
    data_dir.join(FILENAME)
}

pub async fn load(data_dir: &Path) -> Vec<ConsoleEntry> {
    let Ok(contents) = fs::read_to_string(path(data_dir)).await else {
        return Vec::new();
    };

    serde_json::from_str(&contents).unwrap_or_default()
}

pub async fn save(data_dir: &Path, entries: Vec<ConsoleEntry>) -> Result<()> {
    let contents = serde_json::to_string_pretty(&entries)?;
    fs::write(path(data_dir), contents).await?;
    Ok(())
}

/// Asks the network; when nobody answers, tries every address of the /24
pub async fn search(known: Vec<IpAddr>) -> Result<Vec<Found>, String> {
    smol::unblock(move || {
        let found = discovery::discover(DISCOVERY_TIMEOUT, &known, PORT)
            .map_err(|e| format!("Searching for consoles failed: {e}"))?;

        Ok(if found.is_empty() {
            discovery::scan(PORT)
        } else {
            found
        })
    })
    .await
}

fn smdh_cache_path(data_dir: &Path, title: &Title) -> PathBuf {
    data_dir
        .join("smdh-cache")
        .join(format!("{:016X}-v{}.smdh", title.id, title.version))
}

/// The games and apps on a 3DS (SD card and game card), with names and icons;
/// icons are cached, as reading them all takes long
pub async fn load_titles(
    data_dir: &'static Path,
    addr: SocketAddr,
) -> Result<Vec<InstalledTitle>, String> {
    smol::unblock(move || {
        let mut link = Link::connect(addr)?;
        link.note("Reading the titles")?;

        let mut titles = link.titles(n3ds::LOCATION_SD, true)?;
        // no game card is an error too; that's fine
        titles.extend(
            link.titles(n3ds::LOCATION_GAME_CARD, true)
                .unwrap_or_default(),
        );
        titles.retain(|title| crate::n3ds::is_app(title.id));

        let mut installed = Vec::with_capacity(titles.len());
        let (cached, missing): (Vec<_>, Vec<_>) = titles
            .into_iter()
            .map(|title| {
                let smdh = std::fs::read(smdh_cache_path(data_dir, &title)).ok();
                (title, smdh)
            })
            .partition(|(_, smdh)| smdh.as_deref().is_some_and(smdh::is_smdh));

        for (title, smdh) in cached {
            installed.push(installed_title(title, smdh.as_deref()));
        }

        let missing: Vec<Title> = missing.into_iter().map(|(title, _)| title).collect();
        if !missing.is_empty() {
            let _ = std::fs::create_dir_all(data_dir.join("smdh-cache"));
            link.note(&format!("Reading {} icons", missing.len()))?;
        }
        for batch in missing.chunks(METADATA_BATCH) {
            let metadata = link.metadata_batch(batch)?;
            for (title, smdh) in batch.iter().zip(metadata) {
                if let Some(smdh) = &smdh {
                    let _ = std::fs::write(smdh_cache_path(data_dir, title), smdh);
                }
                installed.push(installed_title(title.clone(), smdh.as_deref()));
            }
        }

        installed.sort_by_key(|title| title.name.to_lowercase());
        Ok(installed)
    })
    .await
    .map_err(|e: anyhow::Error| e.to_string())
}

fn installed_title(title: Title, smdh: Option<&[u8]>) -> InstalledTitle {
    let names = smdh.and_then(|smdh| smdh::names(smdh, "en"));
    let icon = smdh
        .and_then(smdh::large_icon_rgba)
        .map(|rgba| image::Handle::from_rgba(smdh::LARGE_ICON_SIZE, smdh::LARGE_ICON_SIZE, rgba));

    let (name, publisher) = match names {
        // some names are split over two lines
        Some(names) => (names.short.replace('\n', " "), names.publisher),
        None => (format!("{:016X}", title.id), String::new()),
    };

    InstalledTitle {
        title,
        name,
        publisher,
        icon,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn found(ip: &str, keys: &str) -> Found {
        Found {
            addr: SocketAddr::new(ip.parse().unwrap(), PORT),
            info: Info::parse(keys),
        }
    }

    #[test]
    fn merge_recognises_a_console_under_a_new_address() {
        let mut state = ConsolesState::default();
        state.merge(vec![
            found("10.0.0.2", "id=AB\nname=Petter\nmodel=1"),
            found("10.0.0.3", "console=wii"),
        ]);
        assert_eq!(state.entries[0].label, "Petter (3DS XL)");

        state.merge(vec![found("10.0.0.9", "id=AB\nname=Petter\nmodel=1")]);

        assert_eq!(state.entries.len(), 2);
        assert!(state.entries[0].online);
        assert_eq!(state.entries[0].addr.ip().to_string(), "10.0.0.9");
        assert!(!state.entries[1].online);
        assert_eq!(state.entries[1].label, "Wii at 10.0.0.3");
    }

    #[test]
    fn outdated_vcinject_link() {
        let old = ConsoleEntry::new(
            found("10.0.0.2", "").addr,
            Info::parse("app=vcinject-link\nversion=12"),
            true,
        );
        assert!(old.outdated());
    }
}
