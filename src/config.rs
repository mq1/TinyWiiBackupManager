// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{ArchiveFormat, Config, SortBy, ViewAs, WiiOutputFormat};
use anyhow::Result;
use serde_json::{Map, Value, json};
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
            .and_then(ArchiveFormat::from_str)
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
            .and_then(SortBy::from_str)
            .unwrap_or(SortBy::NameAscending);

        let view_as = values
            .get("view_as")
            .and_then(|v| v.as_str())
            .and_then(ViewAs::from_str)
            .unwrap_or(ViewAs::Grid);

        let wii_ip = values
            .get("wii_ip")
            .and_then(|v| v.as_str())
            .map(ToSharedString::to_shared_string)
            .unwrap_or("192.168.1.100".to_shared_string());

        let wii_output_format = values
            .get("wii_output_format")
            .and_then(|v| v.as_str())
            .and_then(WiiOutputFormat::from_str)
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

        let values = json!({
            "always_split": self.always_split,
            "archive_format": self.archive_format.as_str(),
            "mount_point": self.mount_point.as_str(),
            "remove_sources_apps": self.remove_sources_apps,
            "remove_sources_games": self.remove_sources_games,
            "scrub_update_partition": self.scrub_update_partition,
            "sort_by": self.sort_by.as_str(),
            "view_as": self.view_as.as_str(),
            "wii_ip": self.wii_ip.as_str(),
            "wii_output_format": self.wii_output_format.as_str(),
        });

        let bytes = serde_json::to_vec(&values)?;
        fs::write(&path, &bytes)?;

        Ok(())
    }
}

impl ArchiveFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            ArchiveFormat::Rvz => "rvz",
            ArchiveFormat::Iso => "iso",
        }
    }

    pub fn from_str(s: &str) -> Option<ArchiveFormat> {
        match s {
            "rvz" => Some(ArchiveFormat::Rvz),
            "iso" => Some(ArchiveFormat::Iso),
            _ => None,
        }
    }
}

impl SortBy {
    pub fn as_str(&self) -> &'static str {
        match self {
            SortBy::NameAscending => "name_ascending",
            SortBy::NameDescending => "name_descending",
            SortBy::SizeAscending => "size_ascending",
            SortBy::SizeDescending => "size_descending",
        }
    }

    pub fn from_str(s: &str) -> Option<SortBy> {
        match s {
            "name_ascending" => Some(SortBy::NameAscending),
            "name_descending" => Some(SortBy::NameDescending),
            "size_ascending" => Some(SortBy::SizeAscending),
            "size_descending" => Some(SortBy::SizeDescending),
            _ => None,
        }
    }
}

impl ViewAs {
    pub fn as_str(&self) -> &'static str {
        match self {
            ViewAs::Grid => "grid",
            ViewAs::List => "list",
        }
    }

    pub fn from_str(s: &str) -> Option<ViewAs> {
        match s {
            "grid" => Some(ViewAs::Grid),
            "list" => Some(ViewAs::List),
            _ => None,
        }
    }
}

impl WiiOutputFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            WiiOutputFormat::Wbfs => "wbfs",
            WiiOutputFormat::Iso => "iso",
        }
    }

    pub fn from_str(s: &str) -> Option<WiiOutputFormat> {
        match s {
            "wbfs" => Some(WiiOutputFormat::Wbfs),
            "iso" => Some(WiiOutputFormat::Iso),
            _ => None,
        }
    }
}
