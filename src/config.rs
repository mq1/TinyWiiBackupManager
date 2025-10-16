// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{ArchiveFormat, Config, SortBy, ViewAs, WiiOutputFormat};
use anyhow::Result;
use serde_json::{Map, Value, json};
use slint::ToSharedString;
use std::{fmt, fs, path::Path, str::FromStr};

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
            .and_then(|v| ArchiveFormat::from_str(v).ok())
            .unwrap_or(ArchiveFormat::Rvz);

        let mount_point = values
            .get("mount_point")
            .and_then(|v| v.as_str())
            .map(ToSharedString::to_shared_string)
            .unwrap_or_default();

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
            .and_then(|v| SortBy::from_str(v).ok())
            .unwrap_or(SortBy::NameAscending);

        let view_as = values
            .get("view_as")
            .and_then(|v| v.as_str())
            .and_then(|v| ViewAs::from_str(v).ok())
            .unwrap_or(ViewAs::Grid);

        let wii_ip = values
            .get("wii_ip")
            .and_then(|v| v.as_str())
            .map(ToSharedString::to_shared_string)
            .unwrap_or("192.168.1.100".to_shared_string());

        let wii_output_format = values
            .get("wii_output_format")
            .and_then(|v| v.as_str())
            .and_then(|v| WiiOutputFormat::from_str(v).ok())
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
            "archive_format": self.archive_format.to_string(),
            "mount_point": self.mount_point.as_str(),
            "remove_sources_apps": self.remove_sources_apps,
            "remove_sources_games": self.remove_sources_games,
            "scrub_update_partition": self.scrub_update_partition,
            "sort_by": self.sort_by.to_string(),
            "view_as": self.view_as.to_string(),
            "wii_ip": self.wii_ip.as_str(),
            "wii_output_format": self.wii_output_format.to_string(),
        });

        let bytes = serde_json::to_vec(&values)?;
        fs::write(&path, &bytes)?;

        Ok(())
    }
}

impl fmt::Display for ArchiveFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArchiveFormat::Rvz => write!(f, "rvz"),
            ArchiveFormat::Iso => write!(f, "iso"),
        }
    }
}

impl FromStr for ArchiveFormat {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "rvz" => Ok(ArchiveFormat::Rvz),
            "iso" => Ok(ArchiveFormat::Iso),
            _ => Err(()),
        }
    }
}

impl fmt::Display for SortBy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SortBy::NameAscending => write!(f, "name_ascending"),
            SortBy::NameDescending => write!(f, "name_descending"),
            SortBy::SizeAscending => write!(f, "size_ascending"),
            SortBy::SizeDescending => write!(f, "size_descending"),
        }
    }
}

impl FromStr for SortBy {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "name_ascending" => Ok(SortBy::NameAscending),
            "name_descending" => Ok(SortBy::NameDescending),
            "size_ascending" => Ok(SortBy::SizeAscending),
            "size_descending" => Ok(SortBy::SizeDescending),
            _ => Err(()),
        }
    }
}

impl fmt::Display for ViewAs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ViewAs::Grid => write!(f, "grid"),
            ViewAs::List => write!(f, "list"),
        }
    }
}

impl FromStr for ViewAs {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "grid" => Ok(ViewAs::Grid),
            "list" => Ok(ViewAs::List),
            _ => Err(()),
        }
    }
}

impl fmt::Display for WiiOutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WiiOutputFormat::Wbfs => write!(f, "wbfs"),
            WiiOutputFormat::Iso => write!(f, "iso"),
        }
    }
}

impl FromStr for WiiOutputFormat {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "wbfs" => Ok(WiiOutputFormat::Wbfs),
            "iso" => Ok(WiiOutputFormat::Iso),
            _ => Err(()),
        }
    }
}
