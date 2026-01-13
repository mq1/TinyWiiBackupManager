// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::{Result, anyhow};
use directories::ProjectDirs;
use std::{env, ffi::OsStr, fs, path::PathBuf};

pub fn get_data_dir() -> Result<PathBuf> {
    let data_dir = if is_portable() {
        let parent = get_exe_parent()?;
        parent.join("TinyWiiBackupManager-data")
    } else {
        let proj = ProjectDirs::from("it", "mq1", env!("CARGO_PKG_NAME"))
            .ok_or(anyhow!("Failed to get project dirs"))?;

        proj.data_dir().to_path_buf()
    };

    fs::create_dir_all(&data_dir)?;
    Ok(data_dir)
}

fn is_portable() -> bool {
    match env::current_exe() {
        Ok(exe) => exe
            .file_name()
            .and_then(OsStr::to_str)
            .is_some_and(|name| name.contains("portable")),
        Err(_) => false,
    }
}

fn get_exe_parent() -> Result<PathBuf> {
    let exe_path = env::current_exe()?;
    let parent = exe_path
        .parent()
        .ok_or(anyhow!("Failed to get exe parent"))?;

    Ok(parent.to_path_buf())
}
