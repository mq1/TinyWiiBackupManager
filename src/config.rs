// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub struct Config {
    path: PathBuf,
    contents: Contents,
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

    fn write(&self) -> Result<()> {
        let bytes = serde_json::to_vec_pretty(&self.contents)?;
        fs::write(&self.path, &bytes)?;

        Ok(())
    }

    #[inline]
    pub fn get_drive_path_str(&self) -> &str {
        self.contents.mount_point.to_str().unwrap_or("Invalid Path")
    }

    #[inline]
    pub fn get_drive_path(&self) -> &Path {
        &self.contents.mount_point
    }

    #[inline]
    pub fn update_drive_path(&mut self, new_mount_point: PathBuf) -> Result<()> {
        self.contents.mount_point = new_mount_point;
        self.write()
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
    //pub accent_color: AccentColor,
    //pub txt_codes_source: TxtCodesSource,
    pub theme_preference: ThemePreference,

    #[serde(with = "FormatDef")]
    pub archive_format: nod::common::Format,

    #[serde(with = "FormatDef")]
    pub wii_output_format: nod::common::Format,

    #[serde(with = "FormatDef")]
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
#[serde(rename_all = "lowercase")]
pub enum ViewAs {
    Grid,
    List,
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ThemePreference {
    Light,
    Dark,
    System,
}
