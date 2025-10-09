// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{ArchiveFormat, Config, WiiOutputFormat};
use anyhow::Result;
use slint::{SharedString, ToSharedString};
use std::{fs, path::Path};

impl Config {
    pub fn load(data_dir: &Path) -> Self {
        let path = data_dir.join("config.json");
        let bytes = fs::read(&path).unwrap_or_default();

        serde_json::from_slice::<Config>(&bytes).unwrap_or(Self {
            mount_point: SharedString::new(),
            remove_sources_games: false,
            remove_sources_apps: false,
            scrub_update_partition: false,
            wii_output_format: WiiOutputFormat::Wbfs,
            archive_format: ArchiveFormat::Rvz,
            always_split: false,
            wii_ip: "192.168.1.100".to_shared_string(),
            as_list: false,
        })
    }

    pub fn save(&self, data_dir: &Path) -> Result<()> {
        let path = data_dir.join("config.json");
        let bytes = serde_json::to_vec(self)?;
        fs::write(path, bytes)?;

        Ok(())
    }
}
