// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::games::game_id::GameID;
use crate::http_util;
use crate::message::Message;
use crate::state::State;
use anyhow::Result;
use iced::Task;
use iced::futures::TryFutureExt;
use serde::Deserialize;
use serde_with::{
    DefaultOnError, DisplayFromStr, StringWithSeparator, formats::CommaSeparator, serde_as,
};
use std::fs::{self, File};
use std::io::{self, Cursor};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;
use zip::ZipArchive;

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
        let mut data = quick_xml::de::from_str::<Self>(&contents)?;

        // We sort it now so we can binary search quickly
        data.games.sort_unstable_by_key(|game| game.id);

        Ok(data)
    }

    fn lookup(&self, game_id: GameID) -> Option<&GameInfo> {
        self.games
            .binary_search_by_key(&game_id, |game| game.id)
            .ok()
            .map(|i| &self.games[i])
    }

    pub fn get_game_info(&self, game_id: GameID) -> Option<GameInfo> {
        self.lookup(game_id).cloned()
    }

    pub fn get_title(&self, game_id: GameID) -> Option<String> {
        self.lookup(game_id)
            .and_then(|info| info.locales.first())
            .map(|locale| locale.title.clone())
    }
}

#[serde_as]
#[derive(Debug, Deserialize, Default, Clone)]
#[serde(default)]
pub struct GameInfo {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(deserialize_with = "deser_id")]
    pub id: GameID,
    #[serde_as(as = "DisplayFromStr")]
    pub region: Region,
    #[serde_as(as = "StringWithSeparator::<CommaSeparator, Language>")]
    pub languages: Box<[Language]>,
    #[serde(rename = "locale")]
    pub locales: Box<[Locale]>,
    pub developer: Option<String>,
    pub publisher: Option<String>,
    pub date: Date,
    #[serde_as(as = "StringWithSeparator::<CommaSeparator, String>")]
    pub genre: Box<[String]>,
    pub rating: Rating,
    #[serde(rename = "wi-fi")]
    pub wifi: Wifi,
    pub input: Input,
    #[serde(rename = "rom")]
    pub roms: Box<[Rom]>,
}

fn deser_id<'de, D>(deserializer: D) -> Result<GameID, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let id = GameID::try_from(s.as_str()).map_err(serde::de::Error::custom)?;
    Ok(id)
}

#[derive(Debug, Deserialize, Default, Clone)]
#[serde(default)]
pub struct Locale {
    title: String,
}

#[serde_as]
#[derive(Debug, Deserialize, Default, Clone)]
#[serde(default)]
pub struct Date {
    #[serde(rename = "@year")]
    #[serde_as(as = "DefaultOnError<DisplayFromStr>")]
    pub year: u16,
    #[serde(rename = "@month")]
    #[serde_as(as = "DefaultOnError<DisplayFromStr>")]
    pub month: u8,
    #[serde(rename = "@day")]
    #[serde_as(as = "DefaultOnError<DisplayFromStr>")]
    pub day: u8,
}

#[derive(Debug, Deserialize, Default, Clone)]
#[serde(default)]
pub struct Rating {
    #[serde(rename = "@type")]
    pub r#type: String,
    #[serde(rename = "@value")]
    pub value: String,
}

#[serde_as]
#[derive(Debug, Deserialize, Default, Clone)]
#[serde(default)]
pub struct Wifi {
    #[serde(rename = "@players")]
    pub players: u8,
    #[serde(rename = "feature")]
    #[serde_as(as = "Box<[DisplayFromStr]>")]
    pub features: Box<[WifiFeature]>,
}

#[derive(
    Debug, Clone, Deserialize, Default, strum_macros::Display, strum_macros::IntoStaticStr,
)]
pub enum WifiFeature {
    NintendoDS,
    Online,
    Wiimmfi,
    Score,
    Download,
    MessageBoard,

    #[default]
    Unknown,
}

impl FromStr for WifiFeature {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let f = match s {
            "nintendods" => Self::NintendoDS,
            "online" => Self::Online,
            "wiimmfi" => Self::Wiimmfi,
            "score" => Self::Score,
            "download" => Self::Download,
            "messageboard" => Self::MessageBoard,
            _ => Self::Unknown,
        };

        Ok(f)
    }
}

#[derive(Debug, Deserialize, Default, Clone)]
#[serde(default)]
pub struct Input {
    #[serde(rename = "@players")]
    pub players: u8,
    #[serde(rename = "control")]
    pub controls: Box<[Control]>,
}

#[serde_as]
#[derive(Debug, Deserialize, Default, Clone)]
#[serde(default)]
pub struct Control {
    #[serde(rename = "@required")]
    pub required: bool,
    #[serde(rename = "@type")]
    #[serde_as(as = "DefaultOnError<DisplayFromStr>")]
    pub r#type: ControlType,
}

#[derive(
    Debug, Clone, Deserialize, Default, strum_macros::Display, strum_macros::IntoStaticStr,
)]
pub enum ControlType {
    Wiimote,
    Nunchuk,
    GameCube,
    MotionPlus,
    BalanceBoard,
    Mii,
    #[strum(serialize = "Classic Controller")]
    ClassicController,
    Wheel,
    Zapper,
    Drums,
    Guitar,
    Microphone,
    WiiSpeak,
    #[strum(serialize = "3D Glasses")]
    _3dGlasses,
    NintendoDS,
    DancePad,
    Keyboard,
    UDraw,
    #[strum(serialize = "Gameboy Advance")]
    GameboyAdvance,

    #[default]
    Unknown,
}

impl FromStr for ControlType {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let f = match s {
            "wiimote" => Self::Wiimote,
            "nunchuk" => Self::Nunchuk,
            "gamecube" => Self::GameCube,
            "motionplus" => Self::MotionPlus,
            "balanceboard" => Self::BalanceBoard,
            "mii" => Self::Mii,
            "classiccontroller" => Self::ClassicController,
            "wheel" => Self::Wheel,
            "zapper" => Self::Zapper,
            "drums" => Self::Drums,
            "guitar" => Self::Guitar,
            "microphone" => Self::Microphone,
            "wiispeak" => Self::WiiSpeak,
            "3dglasses" => Self::_3dGlasses,
            "nintendods" => Self::NintendoDS,
            "dancepad" => Self::DancePad,
            "keyboard" => Self::Keyboard,
            "udraw" => Self::UDraw,
            "gameboy advance" => Self::GameboyAdvance,
            _ => return Err("Unknown"),
        };

        Ok(f)
    }
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

#[derive(Debug, Default, Clone, strum_macros::Display, strum_macros::IntoStaticStr)]
pub enum Language {
    English,
    French,
    German,
    Spanish,
    Italian,
    Japanese,
    Dutch,
    Swedish,
    Danish,
    Norwegian,
    Korean,
    Portuguese,
    TraditionalChinese,
    SimplifiedChinese,
    Finnish,
    Turkish,
    Greek,
    Russian,

    #[default]
    Unknown,
}

impl FromStr for Language {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let l = match s {
            "EN" => Self::English,
            "FR" => Self::French,
            "DE" => Self::German,
            "ES" => Self::Spanish,
            "IT" => Self::Italian,
            "JA" => Self::Japanese,
            "NL" => Self::Dutch,
            "SE" => Self::Swedish,
            "DK" => Self::Danish,
            "NO" => Self::Norwegian,
            "KO" => Self::Korean,
            "PT" => Self::Portuguese,
            "ZHTW" => Self::TraditionalChinese,
            "ZHCN" => Self::SimplifiedChinese,
            "FI" => Self::Finnish,
            "TR" => Self::Turkish,
            "GR" => Self::Greek,
            "RU" => Self::Russian,
            _ => return Err("Unknown"),
        };

        Ok(l)
    }
}

#[derive(
    Debug, Default, Clone, Deserialize, strum_macros::Display, strum_macros::IntoStaticStr,
)]
pub enum Region {
    #[strum(serialize = "NTSC-U")]
    NtscU,

    #[strum(serialize = "NTSC-J")]
    NtscJ,

    #[strum(serialize = "NTSC-K")]
    NtscK,

    #[strum(serialize = "NTSC-T")]
    NtscT,

    #[strum(serialize = "PAL")]
    Pal,

    #[strum(serialize = "PAL-R")]
    PalR,

    #[default]
    Unknown,
}

impl FromStr for Region {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let r = match s {
            "NTSC-U" => Self::NtscU,
            "NTSC-J" => Self::NtscJ,
            "NTSC-K" => Self::NtscK,
            "NTSC-T" => Self::NtscT,
            "PAL" => Self::Pal,
            "PAL-R" => Self::PalR,
            _ => Self::Unknown,
        };

        Ok(r)
    }
}

pub fn get_load_wiitdb_task(state: &State) -> Task<Message> {
    let data_dir = state.data_dir.clone();

    Task::perform(
        async { load_wiitdb(data_dir) }.map_err(Arc::new),
        Message::GotWiitdbDatafile,
    )
}

fn load_wiitdb(data_dir: PathBuf) -> Result<(Datafile, bool)> {
    let mut downloaded = false;
    let wiitdb_path = data_dir.join("wiitdb.xml");

    if !wiitdb_path.exists() {
        let body = http_util::get(DOWNLOAD_URL)?;
        let mut archive = ZipArchive::new(Cursor::new(body))?;
        let mut datafile = archive.by_name("wiitdb.xml")?;
        let mut out_file = File::create(&wiitdb_path)?;
        io::copy(&mut datafile, &mut out_file)?;
        downloaded = true;
    }

    let datafile = Datafile::load(&wiitdb_path)?;

    Ok((datafile, downloaded))
}

pub fn get_download_wiitdb_to_drive_task(state: &State) -> Task<Message> {
    let mount_point = state.config.mount_point().clone();

    Task::perform(
        async move { download_wiitdb_to_drive(mount_point) }.map_err(Arc::new),
        Message::GenericResult,
    )
}

fn download_wiitdb_to_drive(mount_point: PathBuf) -> Result<String> {
    // Download wiitdb
    let body = http_util::get(DOWNLOAD_URL)?;

    // Open the archive
    let mut archive = ZipArchive::new(Cursor::new(body))?;
    let mut datafile = archive.by_name("wiitdb.xml")?;

    // Create the target directory.
    let target_dir = mount_point.join("apps").join("usbloader_gx");
    fs::create_dir_all(&target_dir)?;

    // Extract wiitdb.xml
    let target_path = target_dir.join("wiitdb.xml");
    let mut out_file = File::create(&target_path)?;
    io::copy(&mut datafile, &mut out_file)?;

    Ok("wiitdb.xml successfully downloaded to drive".to_string())
}
