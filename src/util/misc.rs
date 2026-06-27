// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use size::Size;
use std::{io, path::PathBuf};

pub fn get_dir_size(path: impl Into<PathBuf>) -> io::Result<Size> {
    let mut size = 0;

    let mut entries = vec![path.into()];
    while let Some(entry) = entries.pop() {
        let meta = entry.symlink_metadata()?;

        if meta.is_file() {
            size += meta.len();
        } else if meta.is_dir() {
            entries.extend(entry.read_dir()?.filter_map(Result::ok).map(|e| e.path()));
        }
    }

    Ok(Size::from_bytes(size))
}
