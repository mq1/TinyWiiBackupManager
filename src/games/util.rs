// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::games::game_id::GameID;
use anyhow::Result;
use nod::read::{DiscOptions, DiscReader};
use std::{
    ffi::OsStr,
    fs::File,
    io::{Cursor, Read},
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
    } else if is_disc_ext(ext) {
        let disc_reader = DiscReader::new(&path, &DiscOptions::default()).ok()?;
        let id = GameID::from(disc_reader.header().game_id);
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
    if !is_disc_ext(ext) {
        return None;
    }

    let mut buf = vec![0u8; 8 * 1024 * 1024].into_boxed_slice();
    first_file.read_exact(&mut buf).ok()?;
    let cursor = Cursor::new(buf);

    let disc_reader =
        DiscReader::new_from_non_cloneable_read(cursor, &DiscOptions::default()).ok()?;
    let id = GameID::from(disc_reader.header().game_id);
    Some(id)
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
