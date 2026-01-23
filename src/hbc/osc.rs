// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{http_util, message::Message};
use iced::Task;
use serde::Deserialize;
use size::Size;
use std::{ffi::OsString, path::PathBuf};
use time::OffsetDateTime;

impl OscAppMeta {
    pub fn get_trimmed_version_str(&self) -> &str {
        let len = self.version.len().min(8);
        &self.version[..len]
    }

    pub fn get_oscwii_uri(&self) -> OsString {
        format!("https://oscwii.org/library/app/{}", &self.slug).into()
    }

    pub fn get_install_task(&self, mount_point: PathBuf) -> Task<Message> {
        let url = self.assets.archive.url.clone();
        let name = self.name.clone();

        Task::perform(
            async move {
                http_util::download_and_extract_zip(url, &mount_point)
                    .await
                    .map_err(|e| e.to_string())?;

                let msg = format!("App installed: {name}");
                Ok(msg)
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

    #[serde(default)]
    pub uncompressed_size: Size,

    #[serde(default)]
    pub version: String,

    #[serde(deserialize_with = "time::serde::timestamp::deserialize")]
    #[serde(default = "unix_epoch")]
    pub release_date: OffsetDateTime,
}

impl PartialEq for OscAppMeta {
    fn eq(&self, other: &Self) -> bool {
        self.slug == other.slug
    }
}

impl Eq for OscAppMeta {}

const fn unix_epoch() -> OffsetDateTime {
    OffsetDateTime::UNIX_EPOCH
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
    pub const fn as_str(&self) -> &str {
        match self {
            Self::Dol => "DOL",
            Self::Elf => "ELF",
            Self::Unknown => "Unknown",
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
    pub const fn as_str(&self) -> &str {
        match self {
            Self::WiiRemote => "Wii Remote",
            Self::GamecubeController => "GameCube Controller",
            Self::Nunchuk => "Nunchuk",
            Self::ClassicController => "Classic Controller",
            Self::Sdhc => "SDHC Support",
            Self::UsbKeyboard => "USB Keyboard",
            Self::WiiZapper => "Wii Zapper",
            Self::Unknown => "Unknown",
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
    pub const fn as_str(&self) -> &str {
        match self {
            Self::Wii => "Wii",
            Self::WiiMini => "Wii Mini",
            Self::Vwii => "vWii",
            Self::Unknown => "Unknown",
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
    pub const fn as_str(&self) -> &str {
        match self {
            Self::WritesToNand => "Writes to NAND",
            Self::Deprecated => "Deprecated",
            Self::Unknown => "Unknown",
        }
    }
}
