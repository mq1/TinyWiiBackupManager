// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::util::fs::get_dir_size;
use anyhow::bail;
use compact_str::{CompactString, ToCompactString};
use size::Size;
use std::path::PathBuf;
use which_fs::FsKind;

#[derive(Debug, Clone, Default)]
pub struct DriveInfo {
    used_size_str: CompactString,
    total_size: u64,
    total_size_str: CompactString,
    games_size_str: CompactString,
    apps_size_str: CompactString,
    fs_kind: FsKind,
    allocation_granularity: u64,
    allocation_granularity_str: CompactString,
}

#[derive(Debug, Clone, Default)]
pub enum DriveState {
    #[default]
    NotLoaded,
    Loading,
    Loaded(DriveInfo),
    Errored(CompactString),
}

impl DriveState {
    pub async fn load(path: PathBuf) -> Self {
        let res = async move {
            if !path.is_dir() {
                bail!("{} is not a directory", path.display());
            }

            let stat = fs4::statvfs(&path)?;

            let total_size = stat.total_space();
            let avail_size = stat.available_space();
            let used_size = total_size.saturating_sub(avail_size);
            let allocation_granularity = stat.allocation_granularity();

            let used_size_str = Size::from_bytes(used_size).to_compact_string();
            let total_size_str = Size::from_bytes(total_size).to_compact_string();
            let allocation_granularity_str =
                Size::from_bytes(allocation_granularity).to_compact_string();

            let fs_kind = FsKind::try_from_path(&path).unwrap_or(FsKind::Unknown);

            let wii_games_dir = path.join("wbfs");
            let wii_games_size = get_dir_size(&wii_games_dir).await;
            let gc_games_dir = path.join("games");
            let gc_games_size = get_dir_size(&gc_games_dir).await;
            let games_size = wii_games_size + gc_games_size;
            let games_size_str = Size::from_bytes(games_size).to_compact_string();

            let apps_dir = path.join("apps");
            let apps_size = get_dir_size(&apps_dir).await;
            let apps_size_str = Size::from_bytes(apps_size).to_compact_string();

            Ok(DriveInfo {
                used_size_str,
                total_size,
                total_size_str,
                games_size_str,
                apps_size_str,
                fs_kind,
                allocation_granularity,
                allocation_granularity_str,
            })
        }
        .await;

        match res {
            Ok(info) => DriveState::Loaded(info),
            Err(err) => DriveState::Errored(err.to_compact_string()),
        }
    }
}

impl DriveInfo {
    pub fn used_size_str(&self) -> &str {
        &self.used_size_str
    }

    pub fn total_size_str(&self) -> &str {
        &self.total_size_str
    }

    pub fn games_size_str(&self) -> &str {
        &self.games_size_str
    }

    pub fn apps_size_str(&self) -> &str {
        &self.apps_size_str
    }

    pub fn allocation_granularity_str(&self) -> &str {
        &self.allocation_granularity_str
    }

    pub fn fs_kind(&self) -> FsKind {
        self.fs_kind
    }

    pub fn has_optimal_allocation_granularity(&self) -> bool {
        let is_small = self.total_size <= (32 * 1024 * 1024 * 1024); // 32 GiB
        let optimal = if is_small { 32 * 1024 } else { 64 * 1024 };
        optimal == self.allocation_granularity
    }
}
