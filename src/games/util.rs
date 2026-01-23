// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;
use async_zip::base::read::seek::ZipFileReader;
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
            } else if is_valid_disc_file(&entry_path).await {
                discs.push(entry_path);
            }
        }
    }

    Ok(discs)
}

pub async fn is_valid_disc_file(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }

    let Some(filename) = path.file_name().and_then(OsStr::to_str) else {
        return false;
    };

    if filename.starts_with('.') {
        return false;
    }

    if filename.ends_with(".part1.iso") {
        return false;
    }

    if filename.ends_with(".zip") {
        does_this_zip_contain_a_disc(path).await
    } else {
        is_disc_filename(filename)
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
        .is_some_and(is_disc_filename)
}

fn is_disc_filename(filename: &str) -> bool {
    filename.ends_with(".gcm")
        || filename.ends_with(".iso")
        || filename.ends_with(".wbfs")
        || filename.ends_with(".wia")
        || filename.ends_with(".rvz")
        || filename.ends_with(".ciso")
        || filename.ends_with(".gcz")
        || filename.ends_with(".tgc")
        || filename.ends_with(".nfs")
}
