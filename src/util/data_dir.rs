// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use directories::ProjectDirs;
use std::{fs, path::PathBuf};

pub fn get_data_dir() -> Option<PathBuf> {
    match is_portable() {
        true => get_portable_dir(),
        false => get_user_dir().or_else(get_portable_dir),
    }
}

fn is_portable() -> bool {
    let Ok(current_exe) = std::env::current_exe() else {
        return false;
    };

    let Some(stem) = current_exe.file_stem() else {
        return false;
    };
    let mut stem = stem.to_string_lossy().into_owned();
    stem.make_ascii_lowercase();

    if ["not-portable", "notportable", "no-portable", "noportable"]
        .into_iter()
        .any(|s| stem.contains(s))
    {
        return false;
    }

    if stem.contains("portable") {
        return true;
    }

    let Some(parent) = current_exe.parent() else {
        return false;
    };

    // if parent is a windows drive
    parent
        .to_string_lossy()
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_uppercase())
}

fn get_user_dir() -> Option<PathBuf> {
    ProjectDirs::from("it", "mq1", "TinyWiiBackupManager")
        .map(|proj| proj.data_dir().to_path_buf())
        .and_then(|data_dir| match data_dir.exists() {
            false => fs::create_dir_all(&data_dir).is_ok().then_some(data_dir),
            true => Some(data_dir),
        })
}

fn get_portable_dir() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()
        .map(|path| path.with_file_name("TinyWiiBackupManager-data"))
        .and_then(|data_dir| match data_dir.exists() {
            false => fs::create_dir_all(&data_dir).is_ok().then_some(data_dir),
            true => Some(data_dir),
        })
}
