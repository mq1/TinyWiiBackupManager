// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::app::App;
use crate::http;
use crate::messages::Message;
use anyhow::{Result, bail};
use egui_phosphor::regular as ph;
use path_slash::PathExt;
use serde::Deserialize;
use size::Size;
use std::{fs, path::Path, time::Duration};
use time::OffsetDateTime;

const CONTENTS_URL: &str = "https://hbb1.oscwii.org/api/v4/contents";

pub fn spawn_load_osc_apps_task(app: &App) {
    let cache_path = app.data_dir.join("osc-cache.json");
    let icons_dir = app.data_dir.join("osc-icons");

    app.task_processor.spawn(move |msg_sender| {
        msg_sender.send(Message::UpdateStatus(format!(
            "{} Loading OSC Meta...",
            ph::STOREFRONT
        )))?;

        fs::create_dir_all(&icons_dir)?;

        let cache = match load_cache(&cache_path) {
            Some(cache) => cache,
            None => {
                let bytes = http::get(CONTENTS_URL)?;
                fs::write(&cache_path, &bytes)?;
                serde_json::from_slice(&bytes)?
            }
        };

        let apps = cache
            .into_iter()
            .filter_map(|meta| OscApp::from_meta(meta, &icons_dir))
            .collect::<Box<[_]>>();

        msg_sender.send(Message::GotOscApps(apps))?;
        msg_sender.send(Message::NotifyInfo(format!(
            "{} OSC Apps loaded",
            ph::STOREFRONT
        )))?;

        Ok(())
    });
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

    http::download_file(&meta.assets.icon.url, &icon_path)?;

    Ok(())
}

impl OscApp {
    fn from_meta(meta: OscAppMeta, icons_dir: &Path) -> Option<Self> {
        let icon_path = icons_dir.join(&meta.slug).with_extension("png");
        let icon_uri = format!("file://{}", icon_path.to_slash()?);
        let search_str = (meta.name.clone() + &meta.slug).to_lowercase();

        Some(Self {
            meta,
            icon_uri,
            search_str,
        })
    }
}

#[derive(Debug, Clone)]
pub struct OscApp {
    pub meta: OscAppMeta,
    pub icon_uri: String,
    pub search_str: String,
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

    #[serde(default)]
    pub uncompressed_size: Size,

    #[serde(default)]
    pub version: String,

    #[serde(deserialize_with = "time::serde::timestamp::deserialize")]
    #[serde(default = "unix_epoch")]
    pub release_date: OffsetDateTime,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct Description {
    pub short: String,
    pub long: String,
}

impl OscAppMeta {
    pub fn trimmed_version(&self) -> &str {
        if self.version.len() > 10 {
            &self.version[..10]
        } else {
            &self.version
        }
    }
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
