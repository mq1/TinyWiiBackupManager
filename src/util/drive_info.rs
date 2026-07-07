// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::util;
use anyhow::{Result, bail};
use serde::Deserialize;
use size::Size;
use std::path::Path;
use which_fs::FsKind;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct DriveInfo {
    pub used_size: Size,
    pub total_size: Size,
    games_size: Size,
    apps_size: Size,
    #[serde(skip)]
    fs_kind: FsKind,
    allocation_granularity: u64,
}

impl DriveInfo {
    pub async fn try_from_path(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        if !path.is_dir() {
            bail!("Not a directory");
        }

        let stat = fs4::statvfs(path)?;

        let total_size = Size::from_bytes(stat.total_space());
        let avail_size = Size::from_bytes(stat.available_space());
        let used_size = total_size - avail_size;

        let allocation_granularity = stat.allocation_granularity();

        let fs_kind = FsKind::try_from_path(path).unwrap_or(FsKind::Unknown);

        let wii_games_dir = path.join("wbfs");
        let wii_games_size = util::misc::get_dir_size(&wii_games_dir).await;
        let gc_games_dir = path.join("games");
        let gc_games_size = util::misc::get_dir_size(&gc_games_dir).await;
        let games_size = wii_games_size + gc_games_size;

        let apps_dir = path.join("apps");
        let apps_size = util::misc::get_dir_size(&apps_dir).await;

        Ok(Self {
            used_size,
            total_size,
            games_size,
            apps_size,
            fs_kind,
            allocation_granularity,
        })
    }
}
