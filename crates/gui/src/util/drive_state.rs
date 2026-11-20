// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::state::LazyState;
use anyhow::bail;
use size::Size;
use smol::unblock;
use std::path::Path;
use which_fs::FsKind;

#[derive(Debug, Clone)]
pub struct DriveInfo {
    pub used_size: Size,
    pub total_size: Size,
    pub games_size: Size,
    pub apps_size: Size,
    pub fs_kind: FsKind,
    pub allocation_granularity: Size,
}

impl DriveInfo {
    pub async fn load(path: &Path) -> LazyState<DriveInfo> {
        let res = async move {
            if !path.is_dir() {
                bail!("{} is not a directory", path.display());
            }

            let stat = fs4::statvfs(path)?;

            let total_size = stat.total_space();
            let avail_size = stat.available_space();
            let used_size = total_size.saturating_sub(avail_size);
            let allocation_granularity = stat.allocation_granularity();

            let fs_kind = FsKind::try_from_path(path).unwrap_or(FsKind::Unknown);

            let (wii_games_size, gc_games_size, apps_size) = unblock({
                let path = <Box<Path>>::from(path);

                move || {
                    let wii_games_dir = path.join("wbfs");
                    let wii_games_size =
                        fs_extra::dir::get_size(&wii_games_dir).unwrap_or_default();

                    let gc_games_dir = path.join("games");
                    let gc_games_size = fs_extra::dir::get_size(&gc_games_dir).unwrap_or_default();

                    let apps_dir = path.join("apps");
                    let apps_size = fs_extra::dir::get_size(&apps_dir).unwrap_or_default();

                    (wii_games_size, gc_games_size, apps_size)
                }
            })
            .await;

            let games_size = wii_games_size.saturating_add(gc_games_size);

            Ok(DriveInfo {
                used_size: Size::from_bytes(used_size),
                total_size: Size::from_bytes(total_size),
                games_size: Size::from_bytes(games_size),
                apps_size: Size::from_bytes(apps_size),
                fs_kind,
                allocation_granularity: Size::from_bytes(allocation_granularity),
            })
        }
        .await;

        match res {
            Ok(info) => LazyState::Loaded(info),
            Err(err) => LazyState::Errored(err.to_string()),
        }
    }
}

impl DriveInfo {
    pub fn has_optimal_allocation_granularity(&self) -> bool {
        const _32_GIB: Size = Size::from_const(32 * 1024 * 1024 * 1024);
        const _32_KIB: Size = Size::from_const(32 * 1024);
        const _64_KIB: Size = Size::from_const(64 * 1024);

        let is_small = self.total_size <= _32_GIB;
        let optimal = if is_small { _32_KIB } else { _64_KIB };
        optimal == self.allocation_granularity
    }
}
