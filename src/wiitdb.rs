// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::game_id::GameID;
use crate::http_util;
use crate::message::Message;
use crate::state::State;
use anyhow::Result;
use capitalize::Capitalize;
use futures::TryFutureExt;
use iced::Task;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

const DOWNLOAD_URL: &str = "https://www.gametdb.com/wiitdb.zip";

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct Datafile {
    #[serde(rename = "game")]
    pub games: Box<[GameInfo]>,
}

impl Datafile {
    pub fn load(path: &Path) -> Result<Self> {
        let contents = fs::read_to_string(path)?;
        let mut data = quick_xml::de::from_str::<Datafile>(&contents)?;

        // We sort it now so we can binary search quickly
        data.games.sort_unstable_by_key(|game| game.id);

        Ok(data)
    }

    fn lookup(&self, game_id: [u8; 6]) -> Option<&GameInfo> {
        self.games
            .binary_search_by_key(&game_id, |game| game.id)
            .ok()
            .map(|i| &self.games[i])
    }

    pub fn get_game_info(&self, game_id: [u8; 6]) -> Option<GameInfo> {
        self.lookup(game_id).cloned()
    }

    pub fn get_title(&self, game_id: [u8; 6]) -> Option<String> {
        self.lookup(game_id)
            .and_then(|info| info.locales.first())
            .map(|locale| locale.title.clone())
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
    pub languages: Box<[Language]>,
    #[serde(rename = "locale")]
    pub locales: Box<[Locale]>,
    pub developer: Option<String>,
    pub publisher: Option<String>,
    pub date: Date,
    #[serde(deserialize_with = "deser_genres")]
    pub genre: Box<[String]>,
    pub rating: Rating,
    #[serde(rename = "wi-fi")]
    pub wifi: Wifi,
    pub input: Input,
    #[serde(rename = "rom")]
    pub roms: Box<[Rom]>,
}

fn deser_id<'de, D>(deserializer: D) -> Result<[u8; 6], D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let id = GameID::from_str(s.as_str());
    Ok(id)
}

fn deser_genres<'de, D>(deserializer: D) -> Result<Box<[String]>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let genres = s.split(',').map(|s| s.capitalize_first_only()).collect();
    Ok(genres)
}

#[derive(Debug, Deserialize, Default, Clone)]
#[serde(default)]
pub struct Locale {
    title: String,
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
    pub features: Box<[String]>,
}

#[derive(Debug, Deserialize, Default, Clone)]
#[serde(default)]
pub struct Input {
    #[serde(rename = "@players")]
    pub players: u8,
    #[serde(rename = "control")]
    pub controls: Box<[Control]>,
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

#[derive(Debug, Default, Clone)]
pub enum Language {
    En,
    Fr,
    De,
    Es,
    It,
    Ja,
    Nl,
    Se,
    Dk,
    No,
    Ko,
    Pt,
    Zhtw,
    Zhcn,
    Fi,
    Tr,
    Gr,
    Ru,

    #[default]
    Unknown,
}

impl Language {
    pub fn as_str(&self) -> &str {
        match self {
            Language::En => "English",
            Language::Fr => "French",
            Language::De => "German",
            Language::Es => "Spanish",
            Language::It => "Italian",
            Language::Ja => "Japanese",
            Language::Nl => "Dutch",
            Language::Se => "Swedish",
            Language::Dk => "Danish",
            Language::No => "Norwegian",
            Language::Ko => "Korean",
            Language::Pt => "Portuguese",
            Language::Zhtw => "Traditional Chinese",
            Language::Zhcn => "Simplified Chinese",
            Language::Fi => "Finnish",
            Language::Tr => "Turkish",
            Language::Gr => "Greek",
            Language::Ru => "Russian",
            Language::Unknown => "Unknown",
        }
    }
}

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

fn deser_langs<'de, D>(deserializer: D) -> Result<Box<[Language]>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let langs = s.split(',').map(Language::from).collect();

    Ok(langs)
}

#[derive(Debug, Default, Clone)]
pub enum Region {
    NtscU,
    NtscJ,
    NtscK,
    NtscT,
    Pal,
    PalR,

    #[default]
    Unknown,
}

impl Region {
    pub fn as_str(&self) -> &str {
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

pub fn get_load_wiitdb_task(state: &State) -> Task<Message> {
    let data_dir = state.data_dir.clone();

    Task::perform(
        load_wiitdb(data_dir).map_err(|e| e.to_string()),
        Message::GotWiitdbDatafile,
    )
}

async fn load_wiitdb(data_dir: PathBuf) -> Result<Datafile> {
    let wiitdb_path = data_dir.join("wiitdb.xml");

    if !wiitdb_path.exists() {
        http_util::download_and_extract_zip(DOWNLOAD_URL, &data_dir).await?;
    }

    Datafile::load(&wiitdb_path)
}

pub fn get_download_wiitdb_to_drive_task(state: &State) -> Task<Message> {
    let mount_point = state.config.get_drive_path().to_path_buf();

    Task::perform(
        async move {
            match download_wiitdb_to_drive(mount_point).await {
                Ok(()) => Ok("wiitdb.xml successfully downloaded to drive".to_string()),
                Err(e) => Err(e.to_string()),
            }
        },
        Message::GenericResult,
    )
}

async fn download_wiitdb_to_drive(mount_point: PathBuf) -> Result<()> {
    // Create the target directory.
    let target_dir = mount_point.join("apps").join("usbloader_gx");
    fs::create_dir_all(&target_dir)?;

    // Download wiitdb
    http_util::download_and_extract_zip(DOWNLOAD_URL, &target_dir).await?;

    Ok(())
}
