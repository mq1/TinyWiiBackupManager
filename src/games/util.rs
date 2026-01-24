// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;
use std::{
    ffi::OsStr,
    fs::File,
    path::{Path, PathBuf},
};
use walkdir::{DirEntry, WalkDir};
use zip::ZipArchive;

pub fn scan_for_discs(path: PathBuf) -> Vec<PathBuf> {
    WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(is_valid_entry)
        .map(|e| e.path().to_path_buf())
        .filter(|p| {
            p.extension()
                .and_then(OsStr::to_str)
                .is_some_and(|ext| ext != "zip")
                || does_this_zip_contain_a_disc(p)
        })
        .collect()
}

fn is_valid_entry(entry: &DirEntry) -> bool {
    if !entry.file_type().is_file() {
        return false;
    }

    let Some(filename) = entry.file_name().to_str() else {
        return false;
    };

    if filename.starts_with('.') {
        return false;
    }

    if filename.ends_with(".part1.iso") {
        return false;
    }

    filename.ends_with(".zip") || is_disc_filename(filename)
}

pub fn is_valid_disc_file(path: &Path) -> bool {
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
        does_this_zip_contain_a_disc(path)
    } else {
        is_disc_filename(filename)
    }
}

fn does_this_zip_contain_a_disc(path: &Path) -> bool {
    let Ok(file) = File::open(path) else {
        return false;
    };

    let Ok(mut archive) = ZipArchive::new(file) else {
        return false;
    };

    let Ok(first_file) = archive.by_index(0) else {
        return false;
    };

    is_disc_filename(first_file.name())
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
