// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::util::fp::VecExt;
use size::Size;
use smol::{
    fs,
    stream::{self, Stream, StreamExt},
};
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};
use tap::Pipe;

pub async fn get_dir_size(path: &Path) -> Size {
    stream::unfold(vec![path.to_path_buf()], |mut stack| async move {
        let current = stack.pop()?;

        match fs::symlink_metadata(&current).await {
            Ok(meta) if meta.is_file() => Some((meta.len(), stack)),
            Ok(meta) if meta.is_dir() => match fs::read_dir(&current).await {
                Ok(entries) => entries
                    .filter_map(Result::ok)
                    .map(|entry| entry.path())
                    .collect::<Vec<_>>()
                    .await
                    .appended_to(stack)
                    .pipe(|stack| Some((0, stack))),

                Err(_) => Some((0, stack)),
            },
            _ => Some((0, stack)),
        }
    })
    .fold(0, u64::saturating_add)
    .await
    .pipe(Size::from_bytes)
}

pub fn recursive_file_scan<'a>(
    path: impl Into<PathBuf>,
    exts: &'a [&'a str],
) -> impl Stream<Item = PathBuf> + 'a {
    stream::unfold(vec![path.into()], move |mut entries| async move {
        while let Some(entry) = entries.pop() {
            let Ok(meta) = fs::symlink_metadata(&entry).await else {
                continue;
            };

            let Some(filename) = entry.file_name().and_then(OsStr::to_str) else {
                continue;
            };

            // Skip hidden files
            if filename.starts_with('.') {
                continue;
            }

            // Skip iso second split file
            if filename.ends_with(".part1.iso") {
                continue;
            }

            if meta.is_dir() {
                if let Ok(new_entries) = fs::read_dir(&entry).await {
                    let new_entries = new_entries
                        .collect::<Vec<_>>()
                        .await
                        .into_iter()
                        .filter_map(Result::ok)
                        .map(|e| e.path());
                    entries.extend(new_entries);
                }

                continue;
            }

            let Some(ext) = entry.extension().and_then(OsStr::to_str) else {
                continue;
            };

            if meta.is_file() && exts.contains(&ext) {
                return Some((entry, entries));
            }
        }

        None
    })
}
