// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{extensions::SUPPORTED_DISC_EXTENSIONS, message::Message, state::State};
use anyhow::Result;
use iced::Task;
use size::Size;
use std::{
    ffi::OsStr,
    fs::{self, File},
    io::{Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    sync::LazyLock,
};
use sysinfo::Disks;
use tempfile::NamedTempFile;
use walkdir::WalkDir;
use zip::ZipArchive;

#[cfg(target_os = "macos")]
use anyhow::Context;

#[cfg(target_os = "macos")]
use std::process::Command;

static NUM_CPUS: LazyLock<usize> = LazyLock::new(num_cpus::get);

pub static PRELOADER_THREADS: LazyLock<usize> = LazyLock::new(|| match *NUM_CPUS {
    0..=4 => 1,
    5..=8 => 2,
    _ => 4,
});

pub static PROCESSOR_THREADS: LazyLock<usize> = LazyLock::new(|| match *NUM_CPUS {
    0..=4 => *NUM_CPUS - 1,
    5..=8 => *NUM_CPUS - 2,
    _ => *NUM_CPUS - 4,
});

pub fn sanitize(s: &str) -> String {
    let opts = sanitize_filename::Options {
        truncate: true,
        windows: true,
        replacement: "",
    };

    sanitize_filename::sanitize_with_options(s, opts)
        .trim()
        .to_string()
}

pub fn get_drive_usage_task(state: &State) -> Task<Message> {
    let drive_path = state.config.get_drive_path().to_path_buf();

    Task::perform(
        async move { get_drive_usage(drive_path) },
        Message::GotDriveUsage,
    )
}

fn get_drive_usage(mount_point: PathBuf) -> String {
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

#[cfg(target_os = "macos")]
pub fn run_dot_clean(mount_point: &Path) -> Result<()> {
    Command::new("dot_clean")
        .arg("-m")
        .arg(mount_point)
        .status()
        .context("Failed to run dot_clean")?;

    Ok(())
}

pub fn scan_for_discs(dir: &Path) -> Box<[PathBuf]> {
    WalkDir::new(dir)
        .sort_by_file_name()
        .same_file_system(true)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .map(|e| e.into_path())
        .filter(|p| is_valid_disc_file(p))
        .collect()
}

pub fn is_valid_disc_file(path: &Path) -> bool {
    let stem = match path.file_stem().and_then(OsStr::to_str) {
        Some(s) => s,
        None => return false,
    };

    if stem.ends_with(".part1") {
        return false;
    }

    let ext = match path.extension().and_then(OsStr::to_str) {
        Some(s) => s,
        None => return false,
    };

    match ext {
        "zip" => does_this_zip_contain_a_disc(path),
        "gcm" | "iso" | "wbfs" | "wia" | "rvz" | "ciso" | "gcz" | "tgc" | "nfs" => true,
        _ => false,
    }
}

fn does_this_zip_contain_a_disc(path: &Path) -> bool {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return false,
    };

    let mut archive = match ZipArchive::new(file) {
        Ok(a) => a,
        Err(_) => return false,
    };

    let disc_file = match archive.by_index(0) {
        Ok(f) => f,
        Err(_) => return false,
    };

    let disc_name = disc_file.name();
    SUPPORTED_DISC_EXTENSIONS
        .iter()
        .any(|ext| disc_name.ends_with(ext))
}

pub fn get_files_and_dirs(base_dir: &Path) -> (Vec<PathBuf>, Vec<PathBuf>) {
    let mut files = Vec::new();
    let mut dirs = Vec::new();

    let entries = match fs::read_dir(base_dir) {
        Ok(e) => e,
        Err(_) => return (files, dirs),
    };

    let iterator = entries.filter_map(Result::ok);

    for entry in iterator {
        if let Ok(file_type) = entry.file_type() {
            if file_type.is_file() {
                files.push(entry.path());
            } else if file_type.is_dir() {
                dirs.push(entry.path());
            }
        }
    }

    (files, dirs)
}
