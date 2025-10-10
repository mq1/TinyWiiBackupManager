// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{ArchiveFormat, Config, SortBy, ViewAs, WiiOutputFormat};
use anyhow::Result;
use serde_json::{Map, Value};
use slint::{SharedString, ToSharedString};
use std::{fs, path::Path};

impl Config {
    pub fn load(data_dir: &Path) -> Self {
        let path = data_dir.join("config.json");
        let bytes = fs::read(&path).unwrap_or_default();

        let values = serde_json::from_slice::<Map<String, Value>>(&bytes).unwrap_or_default();

        let always_split = values
            .get("always_split")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let archive_format = values
            .get("archive_format")
            .and_then(|v| v.as_str())
            .and_then(|v| serde_json::from_str(v).ok())
            .unwrap_or(ArchiveFormat::Rvz);

        let mount_point = values
            .get("mount_point")
            .and_then(|v| v.as_str())
            .map(ToSharedString::to_shared_string)
            .unwrap_or_else(SharedString::default);

        let remove_sources_apps = values
            .get("remove_sources_apps")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let remove_sources_games = values
            .get("remove_sources_games")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let scrub_update_partition = values
            .get("scrub_update_partition")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let sort_by = values
            .get("sort_by")
            .and_then(|v| v.as_str())
            .and_then(|v| serde_json::from_str(v).ok())
            .unwrap_or(SortBy::NameAscending);

        let view_as = values
            .get("view_as")
            .and_then(|v| v.as_str())
            .and_then(|v| serde_json::from_str(v).ok())
            .unwrap_or(ViewAs::Grid);

        let wii_ip = values
            .get("wii_ip")
            .and_then(|v| v.as_str())
            .map(ToSharedString::to_shared_string)
            .unwrap_or_else(SharedString::default);

        let wii_output_format = values
            .get("wii_output_format")
            .and_then(|v| v.as_str())
            .and_then(|v| serde_json::from_str(v).ok())
            .unwrap_or(WiiOutputFormat::Wbfs);

        Config {
            always_split,
            archive_format,
            mount_point,
            remove_sources_apps,
            remove_sources_games,
            scrub_update_partition,
            sort_by,
            view_as,
            wii_ip,
            wii_output_format,
        }
    }

    pub fn save(&self, data_dir: &Path) -> Result<()> {
        let path = data_dir.join("config.json");
        let bytes = serde_json::to_vec(self)?;
        fs::write(path, bytes)?;

        Ok(())
    }
}
