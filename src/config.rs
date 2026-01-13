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

    pub fn valid_mount_point(&self) -> bool {
        !self.contents.mount_point.as_os_str().is_empty() && self.contents.mount_point.exists()
    }

    pub fn get_drive_path_str(&self) -> &str {
        self.contents.mount_point.to_str().unwrap_or("Invalid Path")
    }

    pub fn get_drive_path(&self) -> &Path {
        &self.contents.mount_point
    }

    pub fn update_drive_path(&mut self, new_mount_point: PathBuf) -> Result<()> {
        self.contents.mount_point = new_mount_point;
        self.write()
    }

    pub fn get_sort_by(&self) -> SortBy {
        self.contents.sort_by
    }

    pub fn update_sort_by(&mut self, new_sort_by: SortBy) -> Result<()> {
        self.contents.sort_by = new_sort_by;
        self.write()
    }

    pub fn get_theme_pref(&self) -> ThemePreference {
        self.contents.theme_preference
    }

    pub fn update_theme_pref(&mut self, new_theme_pref: ThemePreference) -> Result<()> {
        self.contents.theme_preference = new_theme_pref;
        self.write()
    }

    pub fn get_wii_output_format(&self) -> nod::common::Format {
        self.contents.wii_output_format
    }

    pub fn update_wii_output_format(
        &mut self,
        new_wii_output_format: nod::common::Format,
    ) -> Result<()> {
        self.contents.wii_output_format = new_wii_output_format;
        self.write()
    }

    pub fn get_view_as(&self) -> ViewAs {
        self.contents.view_as
    }

    pub fn update_view_as(&mut self, new_view_as: ViewAs) -> Result<()> {
        self.contents.view_as = new_view_as;
        self.write()
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
