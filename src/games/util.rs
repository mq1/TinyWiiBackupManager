// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;
use async_zip::base::read::seek::ZipFileReader;
use iced::futures::future::join_all;
use smol::{
    fs::{self, File},
    io::BufReader,
    stream::StreamExt,
};
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

pub async fn scan_for_discs(path: PathBuf) -> Result<Vec<PathBuf>> {
    let mut discs = Vec::new();
    let mut stack = vec![path];

    while let Some(current_path) = stack.pop() {
        let mut entries = fs::read_dir(&current_path).await?;

        while let Some(entry) = entries.try_next().await? {
            let entry_path = entry.path();

            if entry_path.is_dir() {
                stack.push(entry_path);
            } else if entry_path.is_file() {
                discs.push(keep_disc_file(entry_path));
            }
        }
    }

    let discs = join_all(discs).await.into_iter().flatten().collect();

    Ok(discs)
}

pub async fn keep_disc_file(path: PathBuf) -> Option<PathBuf> {
    if is_valid_disc_file(&path).await {
        Some(path)
    } else {
        None
    }
}

pub async fn is_valid_disc_file(path: &Path) -> bool {
    let Some(stem) = path.file_stem().and_then(OsStr::to_str) else {
        return false;
    };

    if stem.starts_with('.') {
        return false;
    }

    if stem.ends_with("part1") {
        return false;
    }

    let Some(ext) = path.extension().and_then(OsStr::to_str) else {
        return false;
    };

    match ext {
        "zip" => does_this_zip_contain_a_disc(path).await,
        "gcm" | "iso" | "wbfs" | "wia" | "rvz" | "ciso" | "gcz" | "tgc" | "nfs" => true,
        _ => false,
    }
}

async fn does_this_zip_contain_a_disc(path: &Path) -> bool {
    let Ok(file) = File::open(path).await else {
        return false;
    };

    let reader = BufReader::new(file);

    let Ok(zip) = ZipFileReader::new(reader).await else {
        return false;
    };

    zip.file()
        .entries()
        .first()
        .and_then(|e| e.filename().as_str().ok())
        .is_some_and(|filename| {
            [
                ".gcm", ".iso", ".wbfs", ".wia", ".rvz", ".ciso", ".gcz", ".tgc", ".nfs",
            ]
            .iter()
            .any(|ext| filename.ends_with(ext))
        })
}
