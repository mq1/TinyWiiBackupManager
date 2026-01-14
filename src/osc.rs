// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{http_util, message::Message, state::State};
use anyhow::Result;
use iced::{Task, futures::TryFutureExt};
use serde::{Deserialize, Deserializer};
use size::Size;
use smol::{fs, io};
use std::{
    path::{Path, PathBuf},
    time::Duration,
};
use time::Date;

const CONTENTS_URL: &str = "https://hbb1.oscwii.org/api/v4/contents";

pub fn get_load_osc_apps_task(state: &State) -> Task<Message> {
    let data_dir = state.data_dir.clone();

    Task::perform(
        load_osc_apps(data_dir).map_err(|e| e.to_string()),
        Message::GotOscApps,
    )
}

async fn load_osc_apps(data_dir: PathBuf) -> Result<Box<[OscAppMeta]>> {
    let cache_path = data_dir.join("osc-cache.json");

    let apps = match load_cache(&cache_path).await {
        Some(cache) => cache,
        None => {
            let bytes = http_util::get(CONTENTS_URL).await?;
            fs::write(&cache_path, &bytes).await?;
            serde_json::from_slice(&bytes)?
        }
    };

    Ok(apps.into_boxed_slice())
}

async fn load_cache(path: &Path) -> Option<Vec<OscAppMeta>> {
    // get file time
    let file_time = fs::metadata(path).await.ok()?.modified().ok()?;

    // get difference
    let elapsed = file_time.elapsed().ok()?;

    if elapsed > Duration::from_secs(60 * 60 * 24) {
        return None;
    }

    let bytes = fs::read(path).await.ok()?;
    let apps = serde_json::from_slice(&bytes).ok()?;

    Some(apps)
}

pub fn get_download_icons_task(state: &State) -> Task<Message> {
    let osc_apps = state.osc_apps.clone();
    let icons_dir = state.data_dir.join("osc-icons");

    Task::perform(
        async move {
            fs::create_dir_all(&icons_dir)
                .await
                .map_err(|e| e.to_string())?;

            for app in &osc_apps {
                let icon_path = icons_dir.join(&app.slug).with_extension("png");
                if !icon_path.exists() {
                    let _ = http_util::download_file(&app.assets.icon.url, &icon_path).await;
                }
            }

            Ok(())
        },
        Message::EmptyResult,
    )
}

impl OscAppMeta {
    pub fn get_trimmed_version_str(&self) -> &str {
        let len = self.version.len().min(8);
        &self.version[..len]
    }

    pub fn open_page(&self) -> io::Result<()> {
        let url = format!("https://oscwii.org/library/app/{}", &self.slug);
        open::that(url)
    }

    pub fn get_install_task(&self, mount_point: PathBuf) -> Task<Message> {
        let url = self.assets.archive.url.clone();
        let name = self.name.clone();

        Task::perform(
            async move {
                http_util::download_and_extract_zip(&url, &mount_point)
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(name)
            },
            Message::AppInstalled,
        )
    }
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

    #[serde(deserialize_with = "timestamp_to_date")]
    #[serde(default = "min_date")]
    pub release_date: Date,
}

fn size_to_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let size = Size::deserialize(deserializer)?;
    Ok(size.to_string())
}

fn timestamp_to_date<'de, D>(deserializer: D) -> Result<Date, D::Error>
where
    D: Deserializer<'de>,
{
    let date_time = time::serde::timestamp::deserialize(deserializer)?;
    Ok(date_time.date())
}

fn min_date() -> Date {
    Date::MIN
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
