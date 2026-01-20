// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;
use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

impl Config {
    pub fn load(data_dir: &Path) -> Self {
        let path = data_dir.join("config.json");
        let bytes = fs::read(&path).unwrap_or_default();
        let mut config = serde_json::from_slice::<Config>(&bytes).unwrap_or_default();
        config.path = path;

        // Invalidate invalid mount_point
        if !config.mount_point.exists() {
            config.mount_point.clear();
        }

        config
    }

    pub fn write(&self) -> Result<()> {
        let bytes = serde_json::to_vec_pretty(&self)?;
        fs::write(&self.path, &bytes)?;

        Ok(())
    }

    pub fn is_mount_point_valid(&self) -> bool {
        !self.mount_point.as_os_str().is_empty() && self.mount_point.exists()
    }

    pub fn with_always_split(&self, always_split: bool) -> Self {
        let mut config = self.clone();
        config.always_split = always_split;
        config
    }

    pub fn with_mount_point(&self, mount_point: PathBuf) -> Self {
        let mut config = self.clone();
        config.mount_point = mount_point;
        config
    }

    pub fn with_remove_sources_apps(&self, remove_sources_apps: bool) -> Self {
        let mut config = self.clone();
        config.remove_sources_apps = remove_sources_apps;
        config
    }

    pub fn with_remove_sources_games(&self, remove_sources_games: bool) -> Self {
        let mut config = self.clone();
        config.remove_sources_games = remove_sources_games;
        config
    }

    pub fn with_scrub_update_partition(&self, scrub_update_partition: bool) -> Self {
        let mut config = self.clone();
        config.scrub_update_partition = scrub_update_partition;
        config
    }

    pub fn with_sort_by(&self, sort_by: SortBy) -> Self {
        let mut config = self.clone();
        config.sort_by = sort_by;
        config
    }

    pub fn with_view_as(&self, view_as: ViewAs) -> Self {
        let mut config = self.clone();
        config.view_as = view_as;
        config
    }

    pub fn with_wii_ip(&self, wii_ip: String) -> Self {
        let mut config = self.clone();
        config.wii_ip = wii_ip;
        config
    }

    pub fn with_theme_preference(&self, theme_preference: ThemePreference) -> Self {
        let mut config = self.clone();
        config.theme_preference = theme_preference;
        config
    }

    pub fn with_archive_format(&self, archive_format: nod::common::Format) -> Self {
        let mut config = self.clone();
        config.archive_format = archive_format;
        config
    }

    pub fn with_wii_output_format(&self, wii_output_format: nod::common::Format) -> Self {
        let mut config = self.clone();
        config.wii_output_format = wii_output_format;
        config
    }

    pub fn with_gc_output_format(&self, gc_output_format: nod::common::Format) -> Self {
        let mut config = self.clone();
        config.gc_output_format = gc_output_format;
        config
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Getters)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    #[serde(skip)]
    path: PathBuf,

    #[serde(default)]
    #[getter(copy)]
    always_split: bool,

    #[serde(default)]
    mount_point: PathBuf,

    #[serde(default)]
    #[getter(copy)]
    remove_sources_apps: bool,

    #[serde(default)]
    #[getter(copy)]
    remove_sources_games: bool,

    #[serde(default)]
    #[getter(copy)]
    scrub_update_partition: bool,

    #[serde(default)]
    #[getter(copy)]
    sort_by: SortBy,

    #[serde(default)]
    #[getter(copy)]
    view_as: ViewAs,

    #[serde(default = "default_wii_ip")]
    wii_ip: String,
    //
    //pub accent_color: AccentColor,
    //pub txt_codes_source: TxtCodesSource,
    //
    #[serde(default)]
    #[getter(copy)]
    theme_preference: ThemePreference,

    #[serde(default = "default_archive_format", with = "FormatDef")]
    #[getter(copy)]
    archive_format: nod::common::Format,

    #[serde(default = "default_wii_output_format", with = "FormatDef")]
    #[getter(copy)]
    wii_output_format: nod::common::Format,

    #[serde(default = "default_gc_output_format", with = "FormatDef")]
    #[getter(copy)]
    gc_output_format: nod::common::Format,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            path: PathBuf::new(),
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
