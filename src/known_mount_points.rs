// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::app::App;
use anyhow::{Result, bail};
use std::fs;
use std::path::Path;

pub fn check(app: &App) -> Result<bool> {
    let known_mount_points_path = app.data_dir.join("known_mount_points.txt");
    if !known_mount_points_path.exists() {
        fs::write(&known_mount_points_path, "")?;
    }

    // The "" mount point is always known, so we don't show the popup
    if app.config.contents.mount_point.as_os_str().is_empty() {
        bail!("Mount point is empty");
    }

    let mut is_known = false;

    let mut contents = fs::read_to_string(&known_mount_points_path)?;
    for known_mount_point in contents.lines() {
        if Path::new(known_mount_point) == app.config.contents.mount_point {
            is_known = true;
            break;
        }
    }

    // Add the mount point to the list of known mount points
    if !is_known {
        contents.push_str(&format!("{}\n", app.config.contents.mount_point.display()));
        fs::write(&known_mount_points_path, contents)?;
    }

    Ok(is_known)
}
