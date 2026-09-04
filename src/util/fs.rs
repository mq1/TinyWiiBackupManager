// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::util::fp::VecExt;
use size::Size;
use smol::{
    fs,
    stream::{self, Stream, StreamExt},
};
use std::{
    convert::identity,
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
    stream::unfold(vec![path.into()], move |mut stack| async move {
        let current = stack.pop()?;

        // Skip ignored names (hidden files, split ISOs, non-utf8)
        let is_ignored = current
            .file_name()
            .and_then(OsStr::to_str)
            .is_none_or(|name| name.starts_with('.') || name.ends_with(".part1.iso"));

        if is_ignored {
            return Some((None, stack));
        }

        match fs::symlink_metadata(&current).await {
            Ok(meta) if meta.is_dir() => match fs::read_dir(&current).await {
                Ok(entries) => entries
                    .filter_map(Result::ok)
                    .map(|e| e.path())
                    .collect::<Vec<_>>()
                    .await
                    .appended_to(stack)
                    .pipe(|stack| Some((None, stack))),

                Err(_) => Some((None, stack)),
            },

            Ok(meta) if meta.is_file() => {
                let matches_ext = current
                    .extension()
                    .and_then(OsStr::to_str)
                    .is_some_and(|ext| exts.contains(&ext));

                Some((matches_ext.then_some(current), stack))
            }

            _ => Some((None, stack)),
        }
    })
    .filter_map(identity)
}
