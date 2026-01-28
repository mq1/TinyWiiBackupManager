// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::state::State;
use std::fmt::Write;
use std::fs;
use std::path::Path;

/// Returns true if the notification should be shown
pub fn check(state: &State) -> bool {
    let known_mount_points_path = state.data_dir.join("known_mount_points.txt");
    let mut contents = fs::read_to_string(&known_mount_points_path).unwrap_or_default();

    if !state.config.is_mount_point_valid() {
        return false;
    }

    let is_known = contents
        .lines()
        .any(|l| Path::new(l) == state.config.mount_point());

    // Add the mount point to the list of known mount points
    if !is_known {
        let _ = writeln!(&mut contents, "{}", state.config.mount_point().display());
        let _ = fs::write(&known_mount_points_path, contents);
    }

    !is_known
}
