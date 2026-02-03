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
        .map(DirEntry::into_path)
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

    let path = entry.path();

    let Some(ext) = path.extension() else {
        return false;
    };

    if ext.eq_ignore_ascii_case("zip") {
        does_this_zip_contain_a_disc(path)
    } else {
        is_disc_ext(ext)
    }
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

    let Some(ext) = path.extension() else {
        return false;
    };

    if ext.eq_ignore_ascii_case("zip") {
        does_this_zip_contain_a_disc(path)
    } else {
        is_disc_ext(ext)
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

    let Some(enclosed_name) = first_file.enclosed_name() else {
        return false;
    };

    let Some(ext) = enclosed_name.extension() else {
        return false;
    };

    is_disc_ext(ext)
}

fn is_disc_ext(ext: &OsStr) -> bool {
    ext.eq_ignore_ascii_case("gcm")
        || ext.eq_ignore_ascii_case("iso")
        || ext.eq_ignore_ascii_case("wbfs")
        || ext.eq_ignore_ascii_case("wia")
        || ext.eq_ignore_ascii_case("rvz")
        || ext.eq_ignore_ascii_case("ciso")
        || ext.eq_ignore_ascii_case("gcz")
        || ext.eq_ignore_ascii_case("tgc")
        || ext.eq_ignore_ascii_case("nfs")
}

pub fn get_threads_num() -> (usize, usize) {
    let cpus = num_cpus::get();

    let preloader_threads = match cpus {
        0..=4 => 1,
        5..=8 => 2,
        _ => 4,
    };

    let processor_threads = cpus - preloader_threads;

    (preloader_threads, processor_threads)
}
