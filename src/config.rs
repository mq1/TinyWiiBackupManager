// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{ArchiveFormat, WiiOutputFormat};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(skip)]
    pub path: PathBuf,

    pub mount_point: PathBuf,
    pub remove_sources_games: bool,
    pub remove_sources_apps: bool,
    pub scrub_update_partition: bool,
    pub wii_output_format: WiiOutputFormat,
    pub archive_format: ArchiveFormat,
    pub always_split: bool,
    pub wii_ip: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            path: PathBuf::new(),
            mount_point: PathBuf::new(),
            remove_sources_games: false,
            remove_sources_apps: false,
            scrub_update_partition: false,
            wii_output_format: WiiOutputFormat::Wbfs,
            archive_format: ArchiveFormat::Rvz,
            always_split: false,
            wii_ip: "192.168.1.100".to_string(),
        }
    }
}

impl Config {
    pub fn load(data_dir: &Path) -> Self {
        let path = data_dir.join("config.json");
        let bytes = fs::read(&path).unwrap_or_default();

        let mut config = serde_json::from_slice::<Config>(&bytes).unwrap_or_default();
        config.path = path;

        config
    }

    pub fn save(&self) -> Result<()> {
        let bytes = serde_json::to_vec(self)?;
        fs::write(&self.path, bytes)?;
        Ok(())
    }
}
