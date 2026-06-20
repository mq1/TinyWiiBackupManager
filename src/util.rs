// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use directories::ProjectDirs;
use size::Size;
use std::{io, path::PathBuf};

pub fn get_data_dir() -> Option<PathBuf> {
    let data_dir = if let Some(parent) = is_portable() {
        parent.join("TinyWiiBackupManager-data")
    } else {
        let proj = ProjectDirs::from("it", "mq1", "TinyWiiBackupManager")?;
        proj.data_dir().to_path_buf()
    };

    Some(data_dir)
}

fn is_portable() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let name = exe.file_name()?;
    let parent = exe.parent()?;

    name.to_string_lossy()
        .to_ascii_lowercase()
        .contains("portable")
        .then(|| parent.to_path_buf())
}

pub fn get_dir_size(path: impl Into<PathBuf>) -> io::Result<Size> {
    let mut size = 0;

    let mut entries = vec![path.into()];
    while let Some(entry) = entries.pop() {
        let meta = entry.symlink_metadata()?;

        if meta.is_file() {
            size += meta.len();
        } else if meta.is_dir() {
            let new = entry.read_dir()?.filter_map(Result::ok).map(|e| e.path());
            entries.extend(new);
        }
    }

    Ok(Size::from_bytes(size))
}
