// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{DriveInfo, util::GIB};
use slint::ToSharedString;
use std::path::Path;
use which_fs::FsKind;

impl DriveInfo {
    pub fn from_path(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref();

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

        Self {
            label,
            used_gib,
            total_gib,
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
