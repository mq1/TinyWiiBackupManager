// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::util;
use anyhow::{Result, bail};
use getset::{CopyGetters, Getters};
use size::Size;
use std::path::Path;
use which_fs::FsKind;

#[derive(Debug, Clone, Default, Getters, CopyGetters)]
pub struct DriveInfo {
    #[getset(get = "pub")]
    label: String,

    #[getset(get_copy = "pub")]
    used_size: Size,

    #[getset(get_copy = "pub")]
    total_size: Size,

    #[getset(get_copy = "pub")]
    games_size: Size,

    #[getset(get_copy = "pub")]
    apps_size: Size,

    #[getset(get_copy = "pub")]
    fs_kind: FsKind,

    #[getset(get_copy = "pub")]
    allocation_granularity: u64,
}

impl DriveInfo {
    pub fn try_from_path(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        if !path.is_dir() {
            bail!("Not a directory");
        }

        let label_osstr = path.file_name().unwrap_or(path.as_os_str());
        let label = label_osstr.to_string_lossy().to_string();

        let stat = fs4::statvfs(path)?;

        let total_size = Size::from_bytes(stat.total_space());
        let avail_size = Size::from_bytes(stat.available_space());
        let used_size = total_size - avail_size;

        let allocation_granularity = stat.allocation_granularity();

        let fs_kind = FsKind::try_from_path(path).unwrap_or(FsKind::Unknown);

        let wii_games_dir = path.join("wbfs");
        let wii_games_size = util::get_dir_size(&wii_games_dir).unwrap_or_default();
        let gc_games_dir = path.join("games");
        let gc_games_size = util::get_dir_size(&gc_games_dir).unwrap_or_default();
        let games_size = wii_games_size + gc_games_size;

        let apps_dir = path.join("apps");
        let apps_size = util::get_dir_size(&apps_dir).unwrap_or_default();

        Ok(Self {
            label,
            used_size,
            total_size,
            games_size,
            apps_size,
            fs_kind,
            allocation_granularity,
        })
    }
}
