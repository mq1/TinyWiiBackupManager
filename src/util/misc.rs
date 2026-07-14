// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use size::Size;
use smol::{fs, stream::StreamExt};
use std::path::PathBuf;

pub async fn get_dir_size(path: impl Into<PathBuf>) -> Size {
    let mut size = 0;

    let mut entries = vec![path.into()];
    while let Some(entry) = entries.pop() {
        let Ok(meta) = fs::symlink_metadata(&entry).await else {
            continue;
        };

        if meta.is_file() {
            size += meta.len();
        } else if meta.is_dir()
            && let Ok(new_entries) = fs::read_dir(&entry).await
        {
            let new_entries = new_entries
                .collect::<Vec<_>>()
                .await
                .into_iter()
                .filter_map(Result::ok)
                .map(|entry| entry.path());

            entries.extend(new_entries);
        }
    }

    Size::from_bytes(size)
}
