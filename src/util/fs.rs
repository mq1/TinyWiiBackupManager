// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::errors::Error;
use async_zip::{base::read1::seek::ZipArchiveReader, error::ZipError};
use futures::stream::StreamExt;
use path_clean::PathClean;
use size::Size;
use smol::{
    fs::{self, File},
    io::{self, AsyncWriteExt, BufReader, BufWriter},
    stream::{self, Stream},
};
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

pub async fn get_dir_size(path: &Path) -> Size {
    let mut size = 0;

    let mut entries = vec![path.to_path_buf()];
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

pub async fn unzip(path: impl AsRef<Path>, target: &Path) -> Result<(), Error> {
    let mut zip = {
        let file = File::open(path).await?;
        let reader = BufReader::new(file);
        ZipArchiveReader::open(reader).await?
    };

    let target = target.clean();

    for i in 0..zip.cdrs().len() {
        let filename = zip.cdrs()[i]
            .insecure_file_name
            .as_str()
            .ok_or(ZipError::StringNotUtf8)?;

        let path = target.join(filename).clean();
        if !path.starts_with(&target) {
            return Err(Error::Zip("Path traversal detected".into()));
        }

        if filename.ends_with(|c| c == '/' || c == '\\') {
            fs::create_dir_all(&path).await?;
        } else {
            let mut entry_reader = zip.file(i).await?;

            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).await?;
            }

            let file = File::create(&path).await?;
            let mut writer = BufWriter::with_capacity(0x8000, file);

            io::copy(&mut entry_reader, &mut writer).await?;
            writer.flush().await?;
        }
    }

    Ok(())
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
