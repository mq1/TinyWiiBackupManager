// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{http_util, message::Message};
use anyhow::Result;
use derive_getters::Getters;
use iced::{Task, futures::TryFutureExt};
use serde::Deserialize;
use size::Size;
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};
use time::OffsetDateTime;

impl OscAppMeta {
    pub fn oscwii_uri(&self) -> OsString {
        format!("https://oscwii.org/library/app/{}", &self.slug).into()
    }

    pub fn get_install_task(&self, mount_point: PathBuf) -> Task<Message> {
        let app = self.clone();

        Task::perform(
            async move { app.install(&mount_point) }
                .map_err(|e| format!("Failed to install app: {e:#}")),
            Message::AppInstalled,
        )
    }

    pub fn install(&self, mount_point: &Path) -> Result<String> {
        let url = self.assets.archive.url();
        http_util::download_and_extract_zip(url, mount_point)?;

        let msg = format!("App installed: {}", self.name());
        Ok(msg)
    }
}

#[derive(Debug, Clone, Deserialize, Getters)]
pub struct OscAppMeta {
    #[serde(default)]
    slug: String,

    #[serde(default)]
    name: String,

    #[serde(default)]
    author: String,

    #[serde(default)]
    authors: Box<[String]>,

    #[serde(default)]
    category: String,

    #[serde(default)]
    contributors: Box<[String]>,

    #[serde(default)]
    description: Description,

    #[serde(default)]
    #[getter(copy)]
    downloads: usize,

    #[serde(default)]
    assets: Assets,

    #[serde(default)]
    flags: Box<[Flag]>,

    #[serde(default)]
    package_type: PackageType,

    #[serde(default)]
    peripherals: Box<[Peripheral]>,

    #[serde(default)]
    subdirectories: Box<[String]>,

    #[serde(default)]
    supported_platforms: Box<[Platform]>,

    #[serde(default)]
    #[getter(copy)]
    uncompressed_size: Size,

    #[serde(default)]
    version: String,

    #[serde(deserialize_with = "time::serde::timestamp::deserialize")]
    #[serde(default = "unix_epoch")]
    #[getter(copy)]
    release_date: OffsetDateTime,
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

#[derive(Debug, Clone, Deserialize, Default, Getters)]
#[serde(default)]
pub struct Description {
    short: String,
    long: String,
}

#[derive(Debug, Clone, Deserialize, Default, Getters)]
#[serde(default)]
pub struct Assets {
    pub archive: Asset,
    pub binary: Asset,
    pub icon: Asset,
    pub meta: Asset,
}

#[derive(Debug, Clone, Deserialize, Default, Getters)]
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
