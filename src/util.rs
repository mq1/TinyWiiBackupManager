// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::Path;

pub const GIB: f32 = 1024. * 1024. * 1024.;

pub fn get_drive_usage(path: &Path) -> (f32, f32) {
    let Ok(stat) = fs4::statvfs(path) else {
        return (0., 0.);
    };

    let total_bytes = stat.total_space();
    let avail_bytes = stat.available_space();
    let used_bytes = total_bytes.saturating_sub(avail_bytes);

    #[allow(clippy::cast_precision_loss)]
    let used_gib = (used_bytes as f32 / GIB * 100.).round() / 100.;

    #[allow(clippy::cast_precision_loss)]
    let total_gib = (total_bytes as f32 / GIB * 100.).round() / 100.;

    (used_gib, total_gib)
}
