// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;
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
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ArchiveFormat {
    #[default]
    Rvz,
    Iso,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SortBy {
    #[default]
    NameAscending,
    NameDescending,
    SizeAscending,
    SizeDescending,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ViewAs {
    #[default]
    Grid,
    List,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Copy)]
#[serde(rename_all = "snake_case")]
pub enum WiiOutputFormat {
    #[default]
    Wbfs,
    Iso,
}
