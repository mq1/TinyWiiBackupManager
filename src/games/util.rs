// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    disc_util,
    games::{extensions::ext_to_format, game_id::GameID},
};
use anyhow::Result;
use std::{
    fs::File,
    path::{Path, PathBuf},
};
use walkdir::{DirEntry, WalkDir};
use zip::ZipArchive;

pub fn scan_for_discs(path: PathBuf) -> Vec<(PathBuf, GameID)> {
    WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .map(DirEntry::into_path)
        .filter_map(maybe_path_to_entry)
        .collect()
}

pub fn maybe_path_to_entry(path: impl Into<PathBuf>) -> Option<(PathBuf, GameID)> {
    let path = path.into();

    if !path.is_file() {
        return None;
    }

    let filename = path.file_name()?.to_str()?;

    if filename.starts_with('.') {
        return None;
    }

    if filename.ends_with(".part1.iso") {
        return None;
    }

    let ext = path.extension()?;

    if ext.eq_ignore_ascii_case("zip") {
        let id = get_zipped_game_id(&path)?;
        Some((path, id))
    } else if let Some(format) = ext_to_format(ext) {
        let mut file = File::open(&path).ok()?;
        let id = disc_util::read_gameid(&mut file, format).unwrap_or_default();
        Some((path, id))
    } else {
        None
    }
}

fn get_zipped_game_id(path: &Path) -> Option<GameID> {
    let file = File::open(path).ok()?;
    let mut archive = ZipArchive::new(file).ok()?;

    let mut first_file = archive.by_index(0).ok()?;
    let enclosed_name = first_file.enclosed_name()?;
    let ext = enclosed_name.extension()?;

    let format = ext_to_format(ext)?;
    let id = disc_util::read_gameid(&mut first_file, format).unwrap_or_default();
    Some(id)
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
