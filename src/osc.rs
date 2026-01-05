// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{http_util, message::Message, state::State};
use anyhow::{Result, bail};
use iced::Task;
use serde::{Deserialize, Deserializer};
use size::Size;
use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};
use time::OffsetDateTime;

const CONTENTS_URL: &str = "https://hbb1.oscwii.org/api/v4/contents";

pub fn get_load_osc_apps_task(state: &State) -> Task<Message> {
    let data_dir = state.data_dir.clone();

    Task::perform(
        async move { load_osc_apps(data_dir).map_err(|e| e.to_string()) },
        Message::GotOscApps,
    )
}

fn load_osc_apps(data_dir: PathBuf) -> Result<Box<[OscApp]>> {
    let cache_path = data_dir.join("osc-cache.json");
    let icons_dir = data_dir.join("osc-icons");

    fs::create_dir_all(&icons_dir)?;

    let cache = match load_cache(&cache_path) {
        Some(cache) => cache,
        None => {
            let bytes = http_util::get(CONTENTS_URL)?;
            fs::write(&cache_path, &bytes)?;
            serde_json::from_slice(&bytes)?
        }
    };

    let apps = cache.into_iter().filter_map(OscApp::from_meta).collect();

    Ok(apps)
}

fn load_cache(path: &Path) -> Option<Vec<OscAppMeta>> {
    // get file time
    let file_time = fs::metadata(path).ok()?.modified().ok()?;

    // get difference
    let elapsed = file_time.elapsed().ok()?;

    if elapsed > Duration::from_secs(60 * 60 * 24) {
        return None;
    }

    let bytes = fs::read(path).ok()?;
    let apps = serde_json::from_slice(&bytes).ok()?;

    Some(apps)
}

pub fn download_icon(meta: &OscAppMeta, icons_dir: &Path) -> Result<()> {
    let icon_path = icons_dir.join(&meta.slug).with_extension("png");

    if icon_path.exists() {
        bail!("{} already exists", icon_path.display());
    }

    http_util::download_file(&meta.assets.icon.url, &icon_path)?;

    Ok(())
}

impl OscApp {
    fn from_meta(meta: OscAppMeta) -> Option<Self> {
        let search_term = format!("{}{}", meta.name, meta.slug).to_lowercase();

        Some(Self { meta, search_term })
    }

    pub fn matches_search(&self, search: &str) -> bool {
        self.search_term.contains(search)
    }

    pub fn get_trimmed_version_str(&self) -> &str {
        let len = self.meta.version.len().min(8);
        &self.meta.version[..len]
    }
}

#[derive(Debug, Clone)]
pub struct OscApp {
    pub meta: OscAppMeta,
    search_term: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OscAppMeta {
    #[serde(default)]
    pub slug: String,

    #[serde(default)]
    pub name: String,

    #[serde(default)]
    pub author: String,

    #[serde(default)]
    pub authors: Box<[String]>,

    #[serde(default)]
    pub category: String,

    #[serde(default)]
    pub contributors: Box<[String]>,

    #[serde(default)]
    pub description: Description,

    #[serde(default)]
    pub downloads: usize,

    #[serde(default)]
    pub assets: Assets,

    #[serde(default)]
    pub flags: Box<[Flag]>,

    #[serde(default)]
    pub package_type: PackageType,

    #[serde(default)]
    pub peripherals: Box<[Peripheral]>,

    #[serde(default)]
    pub subdirectories: Box<[String]>,

    #[serde(default)]
    pub supported_platforms: Box<[Platform]>,

    #[serde(deserialize_with = "size_to_string")]
    #[serde(default)]
    pub uncompressed_size: String,

    #[serde(default)]
    pub version: String,

    #[serde(deserialize_with = "time::serde::timestamp::deserialize")]
    #[serde(default = "unix_epoch")]
    pub release_date: OffsetDateTime,
}

pub fn size_to_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let size = Size::deserialize(deserializer)?;
    Ok(size.to_string())
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct Description {
    pub short: String,
    pub long: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct Assets {
    pub archive: Asset,
    pub binary: Asset,
    pub icon: Asset,
    pub meta: Asset,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct Asset {
    pub url: String,
    pub size: Option<Size>,
    pub hash: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PackageType {
    Dol,
    Elf,

    #[default]
    Unknown,
}

impl PackageType {
    pub fn as_str(&self) -> &str {
        match self {
            PackageType::Dol => "DOL",
            PackageType::Elf => "ELF",
            PackageType::Unknown => "Unknown",
        }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Peripheral {
    WiiRemote,
    GamecubeController,
    Nunchuk,
    ClassicController,
    Sdhc,
    UsbKeyboard,
    WiiZapper,

    #[default]
    Unknown,
}

impl Peripheral {
    pub fn as_str(&self) -> &str {
        match self {
            Peripheral::WiiRemote => "Wii Remote",
            Peripheral::GamecubeController => "GameCube Controller",
            Peripheral::Nunchuk => "Nunchuk",
            Peripheral::ClassicController => "Classic Controller",
            Peripheral::Sdhc => "SDHC Support",
            Peripheral::UsbKeyboard => "USB Keyboard",
            Peripheral::WiiZapper => "Wii Zapper",
            Peripheral::Unknown => "Unknown",
        }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Platform {
    Wii,
    WiiMini,
    Vwii,

    #[default]
    Unknown,
}

impl Platform {
    pub fn as_str(&self) -> &str {
        match self {
            Platform::Wii => "Wii",
            Platform::WiiMini => "Wii Mini",
            Platform::Vwii => "vWii",
            Platform::Unknown => "Unknown",
        }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Flag {
    WritesToNand,
    Deprecated,

    #[default]
    Unknown,
}

impl Flag {
    pub fn as_str(&self) -> &str {
        match self {
            Flag::WritesToNand => "Writes to NAND",
            Flag::Deprecated => "Deprecated",
            Flag::Unknown => "Unknown",
        }
    }
}

fn unix_epoch() -> OffsetDateTime {
    OffsetDateTime::UNIX_EPOCH
}
