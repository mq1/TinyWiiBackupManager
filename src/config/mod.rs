// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::config::message::ConfigMessage;
use anyhow::Result;
use iced::{Task, futures::TryFutureExt};
use serde::{Deserialize, Serialize};
use smol::fs;
use std::path::{Path, PathBuf};
use strum_macros::{EnumIter, IntoStaticStr};

pub mod message;
pub mod ui;
pub mod update;

#[derive(Debug, Clone, Default)]
pub struct Config {
    path: PathBuf,
    contents: ConfigContents,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigContents {
    #[serde(default)]
    always_split: bool,

    #[serde(default)]
    mount_point: PathBuf,

    #[serde(default)]
    remove_sources_apps: bool,

    #[serde(default)]
    remove_sources_games: bool,

    #[serde(default)]
    scrub_update_partition: bool,

    #[serde(default)]
    sort_by: SortBy,

    #[serde(default)]
    view_as: ViewAs,

    #[serde(default = "yes")]
    show_wii: bool,

    #[serde(default = "yes")]
    show_gc: bool,

    #[serde(default = "default_wii_ip")]
    wii_ip: String,

    #[serde(default)]
    txt_codes_source: TxtCodesSource,

    #[serde(default)]
    theme_preference: ThemePreference,

    #[serde(default)]
    wii_output_format: WiiOutputFormat,

    #[serde(default)]
    gc_output_format: GcOutputFormat,

    #[serde(default)]
    known_drives: Vec<PathBuf>,

    #[serde(default = "system_language")]
    preferred_language: PreferredLanguage,
}

impl Config {
    pub async fn load(data_dir: &'static Path) -> Self {
        let path = data_dir.join("config.json");

        let contents = fs::read_to_string(&path)
            .await
            .ok()
            .and_then(|s| serde_json::from_str::<ConfigContents>(&s).ok())
            .map(|mut contents| async move {
                if !contents.mount_point.as_os_str().is_empty()
                    && fs::read_dir(&contents.mount_point).await.is_err()
                {
                    contents.mount_point = PathBuf::new();
                }

                contents
            });

        let contents = match contents {
            Some(contents) => contents.await,
            None => ConfigContents::default(),
        };

        Self { path, contents }
    }

    async fn write_inner(&self) -> Result<()> {
        let contents = serde_json::to_string_pretty(&self.contents)?;
        fs::write(&self.path, contents).await?;
        Ok(())
    }

    fn write(&self) -> Task<ConfigMessage> {
        let config = self.clone();

        Task::perform(
            async move { config.write_inner().await }.map_err(|e| e.to_string()),
            ConfigMessage::Written,
        )
    }

    /// Returns true if the notification should be shown
    pub fn set_mount_point(&mut self, path: PathBuf) -> Task<ConfigMessage> {
        self.contents.mount_point = path;

        if self.contents.mount_point.as_os_str().is_empty() {
            return self.write();
        }

        let new = self
            .contents
            .known_drives
            .iter()
            .all(|p| p != &self.contents.mount_point);

        if new {
            self.contents
                .known_drives
                .push(self.contents.mount_point.clone());

            Task::batch([self.write(), Task::done(ConfigMessage::NewMountPoint)])
        } else {
            self.write()
        }
    }

    pub fn always_split(&self) -> bool {
        self.contents.always_split
    }

    pub fn mount_point(&self) -> &Path {
        &self.contents.mount_point
    }

    pub fn remove_sources_apps(&self) -> bool {
        self.contents.remove_sources_apps
    }

    pub fn remove_sources_games(&self) -> bool {
        self.contents.remove_sources_games
    }

    pub fn scrub_update_partition(&self) -> bool {
        self.contents.scrub_update_partition
    }

    pub fn sort_by(&self) -> SortBy {
        self.contents.sort_by
    }

    pub fn view_as(&self) -> ViewAs {
        self.contents.view_as
    }

    pub fn show_wii(&self) -> bool {
        self.contents.show_wii
    }

    pub fn show_gc(&self) -> bool {
        self.contents.show_gc
    }

    pub fn wii_ip(&self) -> &str {
        &self.contents.wii_ip
    }

    pub fn txt_codes_source(&self) -> TxtCodesSource {
        self.contents.txt_codes_source
    }

    pub fn theme_preference(&self) -> ThemePreference {
        self.contents.theme_preference
    }

    pub fn wii_output_format(&self) -> WiiOutputFormat {
        self.contents.wii_output_format
    }

    pub fn gc_output_format(&self) -> GcOutputFormat {
        self.contents.gc_output_format
    }

    pub fn preferred_language(&self) -> PreferredLanguage {
        self.contents.preferred_language
    }
}

impl Default for ConfigContents {
    fn default() -> Self {
        Self {
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
            preferred_language: system_language(),
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

fn system_language() -> PreferredLanguage {
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
