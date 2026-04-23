// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    DriveInfo,
    util::{GIB, MIB},
};
use slint::ToSharedString;
use std::path::Path;
use which_fs::FsKind;

impl DriveInfo {
    #[must_use]
    pub fn from_path(path: &Path) -> Self {
        if !path.is_dir() {
            return Self::default();
        }

        let label = match path.file_name() {
            Some(name) => name.to_string_lossy().to_shared_string(),
            None => path.to_string_lossy().to_shared_string(),
        };

        let (used_gib, total_gib) = get_usage(path);
        let fs_kind = FsKind::try_from_path(path)
            .unwrap_or(FsKind::Unknown)
            .to_shared_string();

        let wii_games_dir = path.join("wbfs");
        let wii_games_bytes = fs_extra::dir::get_size(&wii_games_dir).unwrap_or(0);
        let gc_games_dir = path.join("games");
        let gc_games_bytes = fs_extra::dir::get_size(&gc_games_dir).unwrap_or(0);

        #[allow(clippy::cast_precision_loss)]
        let games_gib = (wii_games_bytes + gc_games_bytes) as f32 / GIB;

        let apps_dir = path.join("apps");
        let apps_bytes = fs_extra::dir::get_size(&apps_dir).unwrap_or(0);

        #[allow(clippy::cast_precision_loss)]
        let apps_mib = apps_bytes as f32 / MIB;

        Self {
            label,
            used_gib,
            total_gib,
            games_gib,
            apps_mib,
            fs_kind,
        }
    }
}

fn get_usage(path: &Path) -> (f32, f32) {
    let Ok(stat) = fs4::statvfs(path) else {
        return (0., 0.);
    };

    let total_bytes = stat.total_space();
    let avail_bytes = stat.available_space();
    let used_bytes = total_bytes.saturating_sub(avail_bytes);

    #[allow(clippy::cast_precision_loss)]
    let used_gib = used_bytes as f32 / GIB;

    #[allow(clippy::cast_precision_loss)]
    let total_gib = total_bytes as f32 / GIB;

    (used_gib, total_gib)
}
