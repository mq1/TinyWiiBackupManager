// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    disc_util::read_disc_header,
    games::{extensions::ext_to_format, game_id::GameID},
};
use nod::common::Format;
use std::{
    fs::File,
    path::{Path, PathBuf},
};
use zip::ZipArchive;

pub fn maybe_path_to_entry(path: impl Into<PathBuf>) -> Option<(PathBuf, Format, GameID, String)> {
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
        let (format, id, title) = read_zipped_disc_header(&path)?;
        Some((path, format, id, title))
    } else if ext_to_format(ext).is_some() {
        let mut file = File::open(&path).ok()?;
        let (format, id, title) = read_disc_header(&mut file).unwrap_or_default();
        Some((path, format, id, title))
    } else {
        None
    }
}

fn read_zipped_disc_header(path: &Path) -> Option<(Format, GameID, String)> {
    let file = File::open(path).ok()?;
    let mut archive = ZipArchive::new(file).ok()?;

    let mut first_file = archive.by_index(0).ok()?;
    let enclosed_name = first_file.enclosed_name()?;
    let ext = enclosed_name.extension()?;

    let _ = ext_to_format(ext)?;

    let (format, id, title) = read_disc_header(&mut first_file).unwrap_or_default();
    Some((format, id, title))
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
