// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::bail;
use size::Size;
use std::path::Path;
use which_fs::FsKind;

#[derive(Debug, Clone, Copy)]
pub struct DriveInfo {
    pub used_size: Size,
    pub total_size: Size,
    pub games_size: Size,
    pub apps_size: Size,
    fs_kind: FsKind,
    pub allocation_granularity: Size,
}

impl DriveInfo {
    pub fn has_4gib_filesize_limit(&self) -> bool {
        self.fs_kind == FsKind::Fat32
    }

    pub fn fs_tier(&self) -> i8 {
        match self.fs_kind {
            FsKind::Unknown => -1,
            FsKind::Fat32 => 1,
            FsKind::Ntfs | FsKind::ExFat => 2,
            _ => i8::MAX,
        }
    }

    pub fn fs_name(&self) -> &'static str {
        self.fs_kind.into_str()
    }

    pub fn has_optimal_allocation_granularity(&self) -> bool {
        const _32_GIB: Size = Size::from_const(32 * 1024 * 1024 * 1024);
        const _32_KIB: Size = Size::from_const(32 * 1024);
        const _64_KIB: Size = Size::from_const(64 * 1024);

        let is_small = self.total_size <= _32_GIB;
        let optimal = if is_small { _32_KIB } else { _64_KIB };
        optimal == self.allocation_granularity
    }
}

#[derive(Debug, Clone)]
pub enum DriveInfoState {
    Loading,
    Loaded(DriveInfo),
    Errored(String),
}

impl DriveInfoState {
    pub fn load(path: &Path) -> Self {
        let res = (|| {
            if !path.is_dir() {
                bail!("{} is not a directory", path.display());
            }

            let stat = fs4::statvfs(path)?;

            let total_size = stat.total_space();
            let avail_size = stat.available_space();
            let used_size = total_size.saturating_sub(avail_size);
            let allocation_granularity = stat.allocation_granularity();

            let fs_kind = FsKind::try_from_path(path).unwrap_or(FsKind::Unknown);

            let wii_games_dir = path.join("wbfs");
            let wii_games_size = fs_extra::dir::get_size(&wii_games_dir).unwrap_or_default();

            let gc_games_dir = path.join("games");
            let gc_games_size = fs_extra::dir::get_size(&gc_games_dir).unwrap_or_default();

            let games_size = wii_games_size.saturating_add(gc_games_size);

            let apps_dir = path.join("apps");
            let apps_size = fs_extra::dir::get_size(&apps_dir).unwrap_or_default();

            Ok(DriveInfo {
                used_size: Size::from_bytes(used_size),
                total_size: Size::from_bytes(total_size),
                games_size: Size::from_bytes(games_size),
                apps_size: Size::from_bytes(apps_size),
                fs_kind,
                allocation_granularity: Size::from_bytes(allocation_granularity),
            })
        })();

        match res {
            Ok(info) => DriveInfoState::Loaded(info),
            Err(err) => DriveInfoState::Errored(err.to_string()),
        }
    }
}
