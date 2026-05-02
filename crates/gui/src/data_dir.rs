// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use directories::ProjectDirs;
use std::{env, fs, path::PathBuf, sync::LazyLock};

pub static DATA_DIR: LazyLock<PathBuf> = LazyLock::new(|| get_data_dir().unwrap_or_default());

fn get_data_dir() -> Option<PathBuf> {
    let data_dir = if let Some(parent) = is_portable() {
        parent.join("TinyWiiBackupManager-data")
    } else {
        let proj = ProjectDirs::from("it", "mq1", "TinyWiiBackupManager")?;
        proj.data_dir().to_path_buf()
    };

    fs::create_dir_all(&data_dir).ok()?;
    Some(data_dir)
}

fn is_portable() -> Option<PathBuf> {
    let exe = env::current_exe().ok()?;
    let name = exe.file_name()?;
    let parent = exe.parent()?;

    name.to_string_lossy()
        .to_ascii_lowercase()
        .contains("portable")
        .then(|| parent.to_path_buf())
}
