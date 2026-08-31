// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::errors::Error;
use serde::{Deserialize, Serialize};
use smol::fs;
use std::path::{Path, PathBuf};
use strum_macros::{EnumIter, IntoStaticStr};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(skip)]
    pub path: PathBuf,

    #[serde(default)]
    pub always_split: bool,

    #[serde(default)]
    pub mount_point: PathBuf,

    #[serde(default)]
    pub remove_sources_apps: bool,

    #[serde(default)]
    pub remove_sources_games: bool,

    #[serde(default)]
    pub scrub_update_partition: bool,

    #[serde(default)]
    pub sort_by: SortBy,

    #[serde(default)]
    pub view_as: ViewAs,

    #[serde(default = "yes")]
    pub show_wii: bool,

    #[serde(default = "yes")]
    pub show_gc: bool,

    #[serde(default = "default_wii_ip")]
    pub wii_ip: String,

    #[serde(default)]
    pub txt_codes_source: TxtCodesSource,

    #[serde(default)]
    pub theme_preference: ThemePreference,

    #[serde(default)]
    pub wii_output_format: WiiOutputFormat,

    #[serde(default)]
    pub gc_output_format: GcOutputFormat,

    #[serde(default)]
    pub known_drives: Vec<PathBuf>,

    #[serde(default)]
    pub preferred_language: PreferredLanguage,
}

impl Config {
    pub async fn load(data_dir: impl AsRef<Path>) -> Self {
        let path = data_dir.as_ref().join("config.json");
        let s = fs::read_to_string(&path).await.unwrap_or_default();
        let config = serde_json::from_str(&s).unwrap_or_default();

        Self { path, ..config }
    }

    pub async fn write(&self) -> Result<(), Error> {
        let s = serde_json::to_string_pretty(&self)?;
        fs::write(&self.path, s).await?;
        Ok(())
    }

    /// Returns true if the notification should be shown
    pub async fn check_mount_point(&mut self) -> bool {
        let drive = &self.mount_point;

        if drive.as_os_str().is_empty() {
            return false;
        }

        let new = self.known_drives.iter().all(|p| p != drive);

        if new {
            self.known_drives.push(drive.clone());
            let _ = self.write().await;
        }

        new
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            path: PathBuf::new(),
            always_split: false,
            mount_point: PathBuf::new(),
            remove_sources_apps: false,
            remove_sources_games: false,
            scrub_update_partition: false,
            sort_by: SortBy::NameDescending,
            view_as: ViewAs::Grid,
            wii_ip: default_wii_ip(),
            txt_codes_source: TxtCodesSource::WebArchive,
            theme_preference: ThemePreference::System,
            wii_output_format: WiiOutputFormat::Wbfs,
            gc_output_format: GcOutputFormat::Iso,
            show_wii: true,
            show_gc: true,
            known_drives: Vec::new(),
            preferred_language: PreferredLanguage::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum SortBy {
    #[default]
    NameDescending,
    NameAscending,
    SizeDescending,
    SizeAscending,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ViewAs {
    #[default]
    Grid,
    Table,
}

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default, EnumIter, IntoStaticStr,
)]
#[serde(rename_all = "lowercase")]
pub enum ThemePreference {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum TxtCodesSource {
    #[default]
    WebArchive,
    GameHacking,
    Rc24,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum WiiOutputFormat {
    #[default]
    Wbfs,
    Iso,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum GcOutputFormat {
    #[default]
    Iso,
    Ciso,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, EnumIter, IntoStaticStr)]
#[serde(rename_all = "lowercase")]
pub enum PreferredLanguage {
    English,
    French,
    German,
    Spanish,
    Italian,
    Dutch,
    Portuguese,
    Swedish,
    Danish,
    Finnish,
}

impl Default for PreferredLanguage {
    fn default() -> Self {
        let locale = sys_locale::get_locale();

        match locale.as_ref().and_then(|l| l.get(..2)) {
            Some("fr") => PreferredLanguage::French,
            Some("de") => PreferredLanguage::German,
            Some("es") => PreferredLanguage::Spanish,
            Some("it") => PreferredLanguage::Italian,
            Some("nl") => PreferredLanguage::Dutch,
            Some("pt") => PreferredLanguage::Portuguese,
            Some("sv") => PreferredLanguage::Swedish,
            Some("da") => PreferredLanguage::Danish,
            Some("fi") => PreferredLanguage::Finnish,
            _ => PreferredLanguage::English,
        }
    }
}

impl PreferredLanguage {
    pub fn as_str(&self) -> &'static str {
        match self {
            PreferredLanguage::English => "EN",
            PreferredLanguage::French => "FR",
            PreferredLanguage::German => "DE",
            PreferredLanguage::Spanish => "ES",
            PreferredLanguage::Italian => "IT",
            PreferredLanguage::Dutch => "NL",
            PreferredLanguage::Portuguese => "PT",
            PreferredLanguage::Swedish => "SW",
            PreferredLanguage::Danish => "DK",
            PreferredLanguage::Finnish => "FI",
        }
    }
}

#[inline]
fn default_wii_ip() -> String {
    String::from("192.168.1.100")
}

#[inline]
fn yes() -> bool {
    true
}
