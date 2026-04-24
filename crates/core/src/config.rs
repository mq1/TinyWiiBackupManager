// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub path: PathBuf,
    pub contents: ConfigContents,
}

impl Config {
    pub fn load(data_dir: &Path) -> Self {
        let path = data_dir.join("config.json");
        let s = fs::read_to_string(&path).unwrap_or_default();
        let contents = serde_json::from_str(&s).unwrap_or_default();

        Self { path, contents }
    }

    pub fn write(&self) -> Result<()> {
        let s = serde_json::to_string_pretty(&self.contents)?;
        fs::write(&self.path, s)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigContents {
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
        }
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Default,
    IntoPrimitive,
    TryFromPrimitive,
)]
#[serde(rename_all = "snake_case")]
#[repr(i32)]
pub enum SortBy {
    #[default]
    NameDescending,
    NameAscending,
    SizeDescending,
    SizeAscending,
}

#[derive(
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Default,
    IntoPrimitive,
    TryFromPrimitive,
)]
#[serde(rename_all = "lowercase")]
#[repr(i32)]
pub enum ViewAs {
    #[default]
    Grid,
    Table,
}

#[derive(
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Default,
    IntoPrimitive,
    TryFromPrimitive,
)]
#[serde(rename_all = "lowercase")]
#[repr(i32)]
pub enum ThemePreference {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Default,
    IntoPrimitive,
    TryFromPrimitive,
)]
#[serde(rename_all = "snake_case")]
#[repr(i32)]
pub enum TxtCodesSource {
    #[default]
    WebArchive,
    GameHacking,
    Rc24,
}

#[derive(
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Default,
    IntoPrimitive,
    TryFromPrimitive,
)]
#[serde(rename_all = "lowercase")]
#[repr(i32)]
pub enum WiiOutputFormat {
    #[default]
    Wbfs,
    Iso,
}

#[derive(
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Default,
    IntoPrimitive,
    TryFromPrimitive,
)]
#[serde(rename_all = "lowercase")]
#[repr(i32)]
pub enum GcOutputFormat {
    #[default]
    Iso,
    Ciso,
}

fn default_wii_ip() -> String {
    "192.168.1.100".to_string()
}

fn yes() -> bool {
    true
}
