// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{extensions::SUPPORTED_DISC_EXTENSIONS, message::Message, state::State};
use anyhow::{Result, anyhow, bail};
use async_zip::base::read::seek::ZipFileReader;
use futures::future::join_all;
use iced::Task;
use size::Size;
use smol::{
    fs::{self, File},
    io::{self, BufReader},
    stream::StreamExt,
};
use soft_canonicalize::soft_canonicalize;
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    sync::LazyLock,
};
use sysinfo::Disks;
use tempfile::NamedTempFile;

#[cfg(target_os = "macos")]
use smol::process;

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
    use std::io::{Seek, SeekFrom, Write};

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
pub async fn run_dot_clean(mount_point: &Path) -> io::Result<process::ExitStatus> {
    process::Command::new("dot_clean")
        .arg("-m")
        .arg(mount_point)
        .status()
        .await
}

pub async fn scan_for_discs(path: PathBuf) -> io::Result<Box<[PathBuf]>> {
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

async fn keep_disc_file(path: PathBuf) -> Option<PathBuf> {
    if is_valid_disc_file(&path).await {
        Some(path)
    } else {
        None
    }
}

pub async fn is_valid_disc_file(path: &Path) -> bool {
    let stem = match path.file_stem().and_then(OsStr::to_str) {
        Some(s) => s,
        None => return false,
    };

    if stem.starts_with('.') {
        return false;
    }

    if stem.ends_with(".part1") {
        return false;
    }

    let ext = match path.extension().and_then(OsStr::to_str) {
        Some(s) => s,
        None => return false,
    };

    match ext {
        "zip" => does_this_zip_contain_a_disc(path).await,
        "gcm" | "iso" | "wbfs" | "wia" | "rvz" | "ciso" | "gcz" | "tgc" | "nfs" => true,
        _ => false,
    }
}

async fn does_this_zip_contain_a_disc(path: &Path) -> bool {
    let file = match File::open(path).await {
        Ok(f) => f,
        Err(_) => return false,
    };

    let reader = BufReader::new(file);

    let zip = match ZipFileReader::new(reader).await {
        Ok(a) => a,
        Err(_) => return false,
    };

    zip.file()
        .entries()
        .first()
        .and_then(|e| e.filename().as_str().ok())
        .is_some_and(|filename| {
            SUPPORTED_DISC_EXTENSIONS
                .iter()
                .any(|ext| filename.ends_with(ext))
        })
}

pub async fn get_files_and_dirs(base_dir: &Path) -> io::Result<(Vec<PathBuf>, Vec<PathBuf>)> {
    let mut files = Vec::new();
    let mut dirs = Vec::new();

    let mut entries = fs::read_dir(base_dir).await?;

    while let Some(entry) = entries.try_next().await? {
        let path = entry.path();

        if path.is_dir() {
            dirs.push(path);
        } else if path.is_file() {
            files.push(path);
        }
    }

    Ok((files, dirs))
}

pub async fn extract_zip(zip_path: &Path, dest_dir: &Path) -> Result<()> {
    println!(
        "Extracting \"{}\" into \"{}\"",
        zip_path.display(),
        dest_dir.display()
    );

    let zip_file_reader = BufReader::new(File::open(zip_path).await?);
    let mut zip = async_zip::base::read::seek::ZipFileReader::new(zip_file_reader).await?;

    for i in 0..zip.file().entries().len() {
        let entry = &zip.file().entries()[i];

        let filename = entry.filename().as_str()?;
        let is_dir = entry.dir()?;

        if is_dir {
            println!("  - Directory: {}", filename);
        } else {
            println!("  - File: {}", filename);
        }

        let rel_path = Path::new(filename);
        let dest_path = soft_canonicalize(dest_dir.join(rel_path))?;

        if !dest_path.starts_with(dest_dir) {
            bail!("Directory traversal attempt detected");
        }

        if is_dir {
            fs::create_dir_all(&dest_path).await?;
            continue;
        }

        let parent = dest_path
            .parent()
            .ok_or(anyhow!("Failed to get parent dir"))?;

        let mut reader = zip.reader_without_entry(i).await?;

        fs::create_dir_all(parent).await?;
        let mut writer = File::create(dest_path).await?;
        io::copy(&mut reader, &mut writer).await?;
    }

    Ok(())
}

pub async fn extract_zip_bytes(zip_bytes: Vec<u8>, dest_dir: &Path) -> Result<()> {
    println!("Extracting zip into \"{}\"", dest_dir.display());

    let zip = async_zip::base::read::mem::ZipFileReader::new(zip_bytes).await?;

    for (i, entry) in zip.file().entries().iter().enumerate() {
        let filename = entry.filename().as_str()?;
        let is_dir = entry.dir()?;

        if is_dir {
            println!("  - Directory: {}", filename);
        } else {
            println!("  - File: {}", filename);
        }

        let rel_path = Path::new(filename);
        let dest_path = soft_canonicalize(dest_dir.join(rel_path))?;

        if !dest_path.starts_with(dest_dir) {
            bail!("Directory traversal attempt detected");
        }

        if is_dir {
            fs::create_dir_all(&dest_path).await?;
            continue;
        }

        let parent = dest_path
            .parent()
            .ok_or(anyhow!("Failed to get parent dir"))?;

        let mut reader = zip.reader_without_entry(i).await?;

        fs::create_dir_all(parent).await?;
        let mut writer = File::create(dest_path).await?;
        io::copy(&mut reader, &mut writer).await?;
    }

    Ok(())
}

pub async fn get_dir_size(path: PathBuf) -> io::Result<Size> {
    let mut bytes = 0u64;
    let mut stack = vec![path];

    while let Some(current_path) = stack.pop() {
        let mut entries = fs::read_dir(&current_path).await?;

        while let Some(entry) = entries.try_next().await? {
            let entry_path = entry.path();

            if entry_path.is_dir() {
                stack.push(entry_path);
            } else if entry_path.is_file() {
                let meta = entry.metadata().await?;
                bytes += meta.len();
            }
        }
    }

    Ok(Size::from_bytes(bytes))
}
