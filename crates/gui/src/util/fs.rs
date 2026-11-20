// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use itertools::Itertools;
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};
use walkdir::{DirEntry, WalkDir};

fn should_keep(entry: &DirEntry, exts: &[&str]) -> bool {
    let Some(filename) = entry.file_name().to_str() else {
        return false;
    };

    if filename.starts_with('.') {
        return false;
    }

    if entry.file_type().is_file() {
        if filename.ends_with(".part1.iso") {
            return false;
        }

        let Some(file_ext) = entry.path().extension().and_then(OsStr::to_str) else {
            return false;
        };

        if !exts.contains(&file_ext) {
            return false;
        }
    }

    true
}

pub fn recursive_file_scan(path: &Path, exts: &[&str]) -> Vec<PathBuf> {
    WalkDir::new(path)
        .into_iter()
        .filter_entry(|e| should_keep(e, exts))
        .filter_map(Result::ok)
        .collect_vec()
        .into_iter()
        .filter(|e| e.file_type().is_file()) // keep only files
        .map(DirEntry::into_path)
        .collect()
}
