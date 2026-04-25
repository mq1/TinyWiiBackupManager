// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::{Result, bail};
use arrayvec::ArrayString;
use std::fmt::Write;
use std::path::Path;
use which_fs::FsKind;

#[derive(Debug, Clone, Copy)]
pub struct DriveInfo {
    pub label: ArrayString<255>,
    pub used_bytes: u64,
    pub total_bytes: u64,
    pub games_bytes: u64,
    pub apps_bytes: u64,
    pub fs_kind: FsKind,
    pub allocation_granularity: u64,
}

impl DriveInfo {
    pub fn from_path(path: &Path) -> Result<Self> {
        if !path.is_dir() {
            bail!("Not a directory");
        }

        let label_osstr = path.file_name().unwrap_or(path.as_os_str());
        let mut label = ArrayString::<255>::new();
        write!(label, "{}", label_osstr.to_string_lossy())?;

        let stat = fs4::statvfs(path)?;
        let total_bytes = stat.total_space();
        let avail_bytes = stat.available_space();
        let used_bytes = total_bytes.saturating_sub(avail_bytes);
        let allocation_granularity = stat.allocation_granularity();

        let fs_kind = FsKind::try_from_path(path).unwrap_or(FsKind::Unknown);

        let wii_games_dir = path.join("wbfs");
        let wii_games_bytes = fs_extra::dir::get_size(&wii_games_dir).unwrap_or(0);
        let gc_games_dir = path.join("games");
        let gc_games_bytes = fs_extra::dir::get_size(&gc_games_dir).unwrap_or(0);
        let games_bytes = wii_games_bytes + gc_games_bytes;

        let apps_dir = path.join("apps");
        let apps_bytes = fs_extra::dir::get_size(&apps_dir).unwrap_or(0);

        Ok(Self {
            label,
            used_bytes,
            total_bytes,
            games_bytes,
            apps_bytes,
            fs_kind,
            allocation_granularity,
        })
    }

    pub fn empty() -> Self {
        Self {
            label: ArrayString::new(),
            used_bytes: 0,
            total_bytes: 0,
            games_bytes: 0,
            apps_bytes: 0,
            fs_kind: FsKind::Unknown,
            allocation_granularity: 0,
        }
    }
}
