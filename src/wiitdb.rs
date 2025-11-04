// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::app::App;
use crate::games::GameID;
use crate::http;
use crate::tasks::BackgroundMessage;
use anyhow::{Context, Result};
use capitalize::Capitalize;
use serde::Deserialize;
use std::fs::{self, File};
use std::io::BufReader;
use std::path::Path;

const DOWNLOAD_URL: &str = "https://www.gametdb.com/wiitdb.zip";

/// Handles the blocking logic of downloading and extracting the database.
pub fn spawn_download_task(app: &App) {
    let mount_point = app.config.contents.mount_point.clone();

    app.task_processor.spawn(move |msg_sender| {
        msg_sender.send(BackgroundMessage::UpdateStatus(
            "📥 Downloading wiitdb.xml...".to_string(),
        ))?;

        // Create the target directory.
        let target_dir = mount_point.join("apps").join("usbloader_gx");
        fs::create_dir_all(&target_dir)?;

        // Perform the download request and extract.
        http::download_and_extract_zip(DOWNLOAD_URL, &target_dir)?;

        msg_sender.send(BackgroundMessage::NotifyInfo(
            "📥 wiitdb.xml Downloaded Successfully".to_string(),
        ))?;

        Ok(())
    });
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct Datafile {
    #[serde(rename = "game")]
    pub games: Vec<GameInfo>,
}

impl Datafile {
    pub fn load(mount_point: &Path) -> Result<Self> {
        let path = mount_point
            .join("apps")
            .join("usbloader_gx")
            .join("wiitdb.xml");

        let file =
            File::open(&path).context("Failed to load wiitdb.xml, download it first (☰ button)")?;

        let reader = BufReader::new(file);

        let data = quick_xml::de::from_reader(reader)?;

        Ok(data)
    }

    pub fn get_game_info(&self, game_id: &[u8; 6]) -> Option<&GameInfo> {
        self.games.iter().find(|game| game.id == *game_id)
    }
}

#[derive(Debug, Deserialize, Default, Clone)]
#[serde(default)]
pub struct GameInfo {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(deserialize_with = "deser_id")]
    pub id: [u8; 6],
    pub region: Region,
    #[serde(deserialize_with = "deser_langs")]
    pub languages: Vec<Language>,
    pub developer: Option<String>,
    pub publisher: Option<String>,
    pub date: Date,
    #[serde(deserialize_with = "deser_genres")]
    pub genre: Vec<String>,
    pub rating: Rating,
    #[serde(rename = "wi-fi")]
    pub wifi: Wifi,
    pub input: Input,
    #[serde(rename = "rom")]
    pub roms: Vec<Rom>,
}

fn deser_id<'de, D>(deserializer: D) -> Result<[u8; 6], D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let id = <[u8; 6]>::from_id_str(&s);
    Ok(id)
}

fn deser_genres<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let genres = s.split(',').map(|s| s.capitalize_first_only()).collect();
    Ok(genres)
}

#[derive(Debug, Deserialize, Default, Clone)]
#[serde(default)]
pub struct Date {
    #[serde(rename = "@year")]
    pub year: String,
    #[serde(rename = "@month")]
    pub month: String,
    #[serde(rename = "@day")]
    pub day: String,
}

#[derive(Debug, Deserialize, Default, Clone)]
#[serde(default)]
pub struct Rating {
    #[serde(rename = "@type")]
    pub r#type: String,
    #[serde(rename = "@value")]
    pub value: String,
}

#[derive(Debug, Deserialize, Default, Clone)]
#[serde(default)]
pub struct Wifi {
    #[serde(rename = "@players")]
    pub players: u8,
    #[serde(rename = "feature")]
    pub features: Vec<String>,
}

#[derive(Debug, Deserialize, Default, Clone)]
#[serde(default)]
pub struct Input {
    #[serde(rename = "@players")]
    pub players: u8,
    #[serde(rename = "control")]
    pub controls: Vec<Control>,
}

#[derive(Debug, Deserialize, Default, Clone)]
#[serde(default)]
pub struct Control {
    #[serde(rename = "@required")]
    pub required: bool,
    #[serde(rename = "@type")]
    pub r#type: String,
}

#[derive(Debug, Deserialize, Default, Clone)]
#[serde(default)]
pub struct Rom {
    #[serde(rename = "@crc", deserialize_with = "deser_crc")]
    pub crc: Option<u32>,
}

fn deser_crc<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(u32::from_str_radix(&s, 16).ok())
}

#[rustfmt::skip]
#[derive(Debug, Default, Clone)]
pub enum Language { En, Fr, De, Es, It, Ja, Nl, Se, Dk, No, Ko, Pt, Zhtw, Zhcn, Fi, Tr, Gr, Ru, #[default] Unknown }

impl From<&str> for Language {
    fn from(s: &str) -> Self {
        match s {
            "EN" => Language::En,
            "FR" => Language::Fr,
            "DE" => Language::De,
            "ES" => Language::Es,
            "IT" => Language::It,
            "JA" => Language::Ja,
            "NL" => Language::Nl,
            "SE" => Language::Se,
            "DK" => Language::Dk,
            "NO" => Language::No,
            "KO" => Language::Ko,
            "PT" => Language::Pt,
            "ZHTW" => Language::Zhtw,
            "ZHCN" => Language::Zhcn,
            "FI" => Language::Fi,
            "TR" => Language::Tr,
            "GR" => Language::Gr,
            "RU" => Language::Ru,
            _ => Language::Unknown,
        }
    }
}

fn deser_langs<'de, D>(deserializer: D) -> Result<Vec<Language>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let langs = s.split(',').map(Language::from).collect();

    Ok(langs)
}

#[rustfmt::skip]
#[derive(Debug, Default, Clone)]
pub enum Region { NtscU, NtscJ, NtscK, NtscT, Pal, PalR, #[default] Unknown }

impl<'de> Deserialize<'de> for Region {
    fn deserialize<D>(deserializer: D) -> Result<Region, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(match s.as_str() {
            "NTSC-U" => Region::NtscU,
            "NTSC-J" => Region::NtscJ,
            "NTSC-K" => Region::NtscK,
            "NTSC-T" => Region::NtscT,
            "PAL" => Region::Pal,
            "PAL-R" => Region::PalR,
            _ => Region::Unknown,
        })
    }
}

impl AsRef<str> for Region {
    fn as_ref(&self) -> &str {
        match self {
            Region::NtscJ => "NTSC-J",
            Region::NtscU => "NTSC-U",
            Region::NtscK => "NTSC-K",
            Region::NtscT => "NTSC-T",
            Region::Pal => "PAL",
            Region::PalR => "PAL-R",
            Region::Unknown => "Unknown",
        }
    }
}
