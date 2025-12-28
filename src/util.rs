// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::extensions::SUPPORTED_INPUT_EXTENSIONS;
use anyhow::{Context, Result};
use size::Size;
use std::{
    ffi::OsStr,
    fs::{self, File},
    io::{Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    process::Command,
};
use sysinfo::Disks;
use tempfile::NamedTempFile;
use zip::ZipArchive;

fn is_valid_char(c: &char) -> bool {
    matches!(*c, 'a'..='z' | 'A'..='Z' | '0'..='9' | ' ' | '+' | '-')
}

pub fn sanitize(s: &str) -> String {
    s.chars()
        .filter(is_valid_char)
        .collect::<String>()
        .trim()
        .to_string()
}

pub fn get_disk_usage(mount_point: &Path) -> String {
    if mount_point.as_os_str().is_empty() {
        return "0 bytes/0 bytes".to_string();
    }

    let disks = Disks::new_with_refreshed_list();

    disks
        .iter()
        .filter(|disk| mount_point.starts_with(disk.mount_point()))
        .max_by_key(|disk| disk.mount_point().as_os_str().len())
        .map(|disk| {
            let total = disk.total_space();
            let used = total - disk.available_space();

            format!("{}/{}", Size::from_bytes(used), Size::from_bytes(total))
        })
        .unwrap_or("0 bytes/0 bytes".to_string())
}

/// Returns Ok if we can create a file >4 GiB in this directory
pub fn can_write_over_4gb(mount_point: &Path) -> bool {
    if mount_point.as_os_str().is_empty() {
        return false;
    }

    // Create a temp file in the target directory
    let mut tmp = if let Ok(tmp) = NamedTempFile::new_in(mount_point) {
        tmp
    } else {
        return false;
    };

    // Seek to 4 GiB
    if tmp
        .as_file_mut()
        .seek(SeekFrom::Start(4 * 1024 * 1024 * 1024))
        .is_err()
    {
        return false;
    }

    // Write a single byte
    if tmp.as_file_mut().write_all(&[0]).is_err() {
        return false;
    }

    true
}

pub fn get_threads_num() -> (usize, usize) {
    let cpus = num_cpus::get();

    let preloader_threads = if cpus <= 4 {
        1
    } else if cpus <= 8 {
        2
    } else {
        4
    };

    let processor_threads = cpus - preloader_threads;

    (preloader_threads, processor_threads)
}

pub fn run_dot_clean(mount_point: &Path) -> Result<()> {
    Command::new("dot_clean")
        .arg("-m")
        .arg(mount_point)
        .status()
        .context("Failed to run dot_clean")?;

    Ok(())
}

pub fn scan_for_discs(dir: &Path) -> Vec<PathBuf> {
    let mut disc_paths = Vec::new();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();

            if path.is_dir() {
                let mut new = scan_for_discs(&path);
                disc_paths.append(&mut new);
            } else if path.is_file()
                && path
                    .extension()
                    .and_then(OsStr::to_str)
                    .is_some_and(|ext| SUPPORTED_INPUT_EXTENSIONS.contains(&ext))
                && path
                    .file_name()
                    .and_then(OsStr::to_str)
                    .is_some_and(|file_name| !file_name.ends_with(".part1.iso"))
                && (path.extension() != Some(OsStr::new("zip"))
                    || does_this_zip_contain_a_disc(&path))
            {
                disc_paths.push(path);
            }
        }
    }

    disc_paths
}

pub fn does_this_zip_contain_a_disc(path: &Path) -> bool {
    let file = if let Ok(file) = File::open(path) {
        file
    } else {
        return false;
    };

    let mut archive = if let Ok(archive) = ZipArchive::new(file) {
        archive
    } else {
        return false;
    };

    let disc_file = if let Ok(disc_file) = archive.by_index(0) {
        disc_file
    } else {
        return false;
    };

    let disc_name = disc_file.name();
    SUPPORTED_INPUT_EXTENSIONS
        .iter()
        .any(|ext| disc_name.ends_with(ext))
}
