// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use directories::ProjectDirs;
use std::{fs, path::PathBuf};

pub fn get_data_dir() -> Option<PathBuf> {
    let data_dir = if is_portable() {
        get_portable_dir()?
    } else {
        get_user_dir().or(get_portable_dir())?
    };

    Some(data_dir)
}

fn is_portable() -> bool {
    std::env::current_exe().is_ok_and(|exe_path| {
        exe_path.file_name().is_some_and(|name| {
            name.to_string_lossy()
                .to_ascii_lowercase()
                .contains("portable")
        })
    })
}

fn get_user_dir() -> Option<PathBuf> {
    let proj = ProjectDirs::from("it", "mq1", "TinyWiiBackupManager")?;
    let data_dir = proj.data_dir().to_path_buf();

    fs::create_dir_all(&data_dir).ok()?;
    Some(data_dir)
}

fn get_portable_dir() -> Option<PathBuf> {
    let exe_path = std::env::current_exe().ok()?;
    let parent = exe_path.parent()?;
    let data_dir = parent.join("TinyWiiBackupManager-data");

    fs::create_dir_all(&data_dir).ok()?;
    Some(data_dir)
}
