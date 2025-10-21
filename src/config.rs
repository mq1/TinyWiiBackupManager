// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;
use eframe::egui::ThemePreference;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub struct Config {
    pub path: PathBuf,
    pub contents: Contents,
}

impl Config {
    pub fn load(data_dir: &Path) -> Self {
        let path = data_dir.join("config.json");
        let bytes = fs::read(&path).unwrap_or_default();
        let mut contents = serde_json::from_slice::<Contents>(&bytes).unwrap_or_default();

        // Invalidate invalid mount_point
        if !matches!(fs::exists(&contents.mount_point), Ok(true)) {
            contents.mount_point = PathBuf::new();
        }

        // load mount_point from args
        if let Some(mount_point) = std::env::args().nth(1) {
            contents.mount_point = PathBuf::from(mount_point);
        }

        Self { path, contents }
    }

    pub fn write(&self) -> Result<()> {
        let bytes = serde_json::to_vec(&self.contents)?;
        fs::write(&self.path, &bytes)?;

        Ok(())
    }

    pub fn get_drive_name(&self) -> &str {
        self.contents
            .mount_point
            .file_name()
            .unwrap_or(self.contents.mount_point.as_os_str())
            .to_str()
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "snake_case")]
pub struct Contents {
    pub always_split: bool,
    pub archive_format: ArchiveFormat,
    pub mount_point: PathBuf,
    pub remove_sources_apps: bool,
    pub remove_sources_games: bool,
    pub scrub_update_partition: bool,
    pub sort_by: SortBy,
    pub view_as: ViewAs,
    pub wii_ip: String,
    pub wii_output_format: WiiOutputFormat,
    #[serde(serialize_with = "ser_theme", deserialize_with = "deser_theme")]
    pub theme_preference: ThemePreference,
}

impl Default for Contents {
    fn default() -> Self {
        Self {
            always_split: false,
            archive_format: ArchiveFormat::Rvz,
            mount_point: PathBuf::new(),
            remove_sources_apps: false,
            remove_sources_games: false,
            scrub_update_partition: false,
            sort_by: SortBy::NameAscending,
            view_as: ViewAs::Grid,
            wii_ip: "192.168.1.100".to_string(),
            wii_output_format: WiiOutputFormat::Wbfs,
            theme_preference: ThemePreference::System,
        }
    }
}

fn ser_theme<S>(theme: &ThemePreference, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(match theme {
        ThemePreference::System => "system",
        ThemePreference::Light => "light",
        ThemePreference::Dark => "dark",
    })
}

fn deser_theme<'de, D>(deserializer: D) -> Result<ThemePreference, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(match s.as_str() {
        "light" => ThemePreference::Light,
        "dark" => ThemePreference::Dark,
        _ => ThemePreference::System,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ArchiveFormat {
    Rvz,
    Iso,
}

impl AsRef<str> for ArchiveFormat {
    fn as_ref(&self) -> &str {
        match self {
            ArchiveFormat::Rvz => "RVZ",
            ArchiveFormat::Iso => "ISO",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SortBy {
    NameAscending,
    NameDescending,
    SizeAscending,
    SizeDescending,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ViewAs {
    Grid,
    List,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Copy)]
#[serde(rename_all = "snake_case")]
pub enum WiiOutputFormat {
    Wbfs,
    Iso,
}
