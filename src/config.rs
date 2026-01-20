// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub struct Config {
    path: PathBuf,
    pub contents: Contents,
}

impl Config {
    pub fn load(data_dir: &Path) -> Self {
        let path = data_dir.join("config.json");
        let bytes = fs::read(&path).unwrap_or_default();
        let mut contents = serde_json::from_slice::<Contents>(&bytes).unwrap_or_default();

        // Invalidate invalid mount_point
        if !contents.mount_point.exists() {
            contents.mount_point = PathBuf::new();
        }

        // load mount_point from args
        if let Some(mount_point) = std::env::args().nth(1).map(PathBuf::from)
            && mount_point.exists()
        {
            contents.mount_point = mount_point;
        }

        Self { path, contents }
    }

    pub fn write(&self) -> Result<()> {
        let bytes = serde_json::to_vec_pretty(&self.contents)?;
        fs::write(&self.path, &bytes)?;

        Ok(())
    }

    pub fn valid_mount_point(&self) -> bool {
        !self.contents.mount_point.as_os_str().is_empty() && self.contents.mount_point.exists()
    }

    pub fn with_sort_by(&self, sort_by: SortBy) -> Self {
        let mut config = self.clone();
        config.contents.sort_by = sort_by;
        config
    }

    pub fn with_view_as(&self, view_as: ViewAs) -> Self {
        let mut config = self.clone();
        config.contents.view_as = view_as;
        config
    }

    pub fn with_wii_output_format(&self, wii_output_format: nod::common::Format) -> Self {
        let mut config = self.clone();
        config.contents.wii_output_format = wii_output_format;
        config
    }

    pub fn with_theme_preference(&self, theme_preference: ThemePreference) -> Self {
        let mut config = self.clone();
        config.contents.theme_preference = theme_preference;
        config
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Contents {
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

    #[serde(default = "default_wii_ip")]
    pub wii_ip: String,
    //
    //pub accent_color: AccentColor,
    //pub txt_codes_source: TxtCodesSource,
    //
    #[serde(default)]
    pub theme_preference: ThemePreference,

    #[serde(default = "default_archive_format", with = "FormatDef")]
    pub archive_format: nod::common::Format,

    #[serde(default = "default_wii_output_format", with = "FormatDef")]
    pub wii_output_format: nod::common::Format,

    #[serde(default = "default_gc_output_format", with = "FormatDef")]
    pub gc_output_format: nod::common::Format,
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
            //accent_color: AccentColor::System,
            //txt_codes_source: TxtCodesSource::WebArchive,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum SortBy {
    #[default]
    NameAscending,
    NameDescending,
    SizeAscending,
    SizeDescending,
    None,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ViewAs {
    #[default]
    Grid,
    Table,
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "nod::common::Format", rename_all = "lowercase")]
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

fn default_archive_format() -> nod::common::Format {
    nod::common::Format::Rvz
}

fn default_wii_output_format() -> nod::common::Format {
    nod::common::Format::Wbfs
}

fn default_gc_output_format() -> nod::common::Format {
    nod::common::Format::Iso
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ThemePreference {
    #[default]
    System,
    Light,
    Dark,
}

fn default_wii_ip() -> String {
    "192.168.1.100".to_string()
}
