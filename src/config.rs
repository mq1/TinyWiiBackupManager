// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::txtcodes::TxtCodesSource;
use crate::ui::accent::AccentColor;
use anyhow::Result;
use eframe::egui::ThemePreference;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub struct Config {
    pub path: PathBuf,
    pub contents: Contents,
}

impl Config {
    pub fn load(data_dir: &Path) -> Self {
        let path = data_dir.join("config.json");
        let bytes = fs::read(&path).unwrap_or_default();
        let mut contents = serde_json::from_slice::<Contents>(&bytes).unwrap_or_default();

        // Strip \\?\ from mount_point (I made a mess in v3, this fixes it)
        contents.mount_point = contents
            .mount_point
            .strip_prefix(r"\\?\")
            .unwrap_or(&contents.mount_point)
            .to_path_buf();

        // Invalidate invalid mount_point
        if !contents.mount_point.exists() {
            contents.mount_point = PathBuf::new();
        }

        // load mount_point from args
        if let Some(mount_point) = std::env::args().nth(1) {
            contents.mount_point = PathBuf::from(mount_point);
        }

        Self { path, contents }
    }

    pub fn write(&self) -> Result<()> {
        let bytes = serde_json::to_vec_pretty(&self.contents)?;
        fs::write(&self.path, &bytes)?;

        Ok(())
    }

    pub fn get_drive_path_str(&self) -> &str {
        if self.contents.mount_point.as_os_str().is_empty() {
            return "No Drive Selected";
        }

        self.contents.mount_point.to_str().unwrap_or("Invalid Path")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "snake_case")]
pub struct Contents {
    pub always_split: bool,
    pub mount_point: PathBuf,
    pub remove_sources_apps: bool,
    pub remove_sources_games: bool,
    pub scrub_update_partition: bool,
    pub sort_by: SortBy,
    pub view_as: ViewAs,
    pub wii_ip: String,
    pub accent_color: AccentColor,
    pub txt_codes_source: TxtCodesSource,

    #[serde(with = "FormatDef")]
    pub archive_format: nod::common::Format,

    #[serde(with = "FormatDef")]
    pub wii_output_format: nod::common::Format,

    #[serde(with = "FormatDef")]
    pub gc_output_format: nod::common::Format,

    #[serde(serialize_with = "ser_theme", deserialize_with = "deser_theme")]
    pub theme_preference: ThemePreference,
}

impl Default for Contents {
    fn default() -> Self {
        Self {
            always_split: false,
            archive_format: nod::common::Format::Rvz,
            mount_point: PathBuf::new(),
            remove_sources_apps: false,
            remove_sources_games: false,
            scrub_update_partition: false,
            sort_by: SortBy::NameAscending,
            view_as: ViewAs::Grid,
            wii_ip: "192.168.1.100".to_string(),
            wii_output_format: nod::common::Format::Wbfs,
            gc_output_format: nod::common::Format::Iso,
            theme_preference: ThemePreference::System,
            accent_color: AccentColor::System,
            txt_codes_source: TxtCodesSource::WebArchive,
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SortBy {
    None,
    NameAscending,
    NameDescending,
    SizeAscending,
    SizeDescending,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ViewAs {
    Grid,
    List,
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "nod::common::Format", rename_all = "snake_case")]
pub enum FormatDef {
    Iso,
    Ciso,
    Gcz,
    Nfs,
    Rvz,
    Wbfs,
    Wia,
    Tgc,
}
