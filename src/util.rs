// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{message::Message, state::State};
use anyhow::{Result, anyhow};
use iced::Task;
use size::Size;
use smol::{
    fs::{self, File},
    io::{self, BufReader},
    stream::StreamExt,
};
use std::path::{Path, PathBuf};
use sysinfo::Disks;
use tempfile::NamedTempFile;

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
    let mount_point = state.config.mount_point().to_path_buf();
    Task::perform(get_drive_usage(mount_point), Message::GotDriveUsage)
}

async fn get_drive_usage(mount_point: PathBuf) -> String {
    const GIB: f64 = 1024.0 * 1024.0 * 1024.0;

    if mount_point.as_os_str().is_empty() {
        return "0/0 GiB".to_string();
    }

    let disks = Disks::new_with_refreshed_list();

    disks
        .iter()
        .filter(|disk| mount_point.starts_with(disk.mount_point()))
        .max_by_key(|disk| disk.mount_point().as_os_str().len())
        .map(|disk| {
            let total = disk.total_space();
            let used = total - disk.available_space();

            let used = used as f64 / GIB;
            let total = total as f64 / GIB;

            format!("{:.2}/{:.2} GiB", used, total)
        })
        .unwrap_or("0/0 GiB".to_string())
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
pub async fn run_dot_clean(mount_point: PathBuf) -> Result<()> {
    let status = smol::process::Command::new("dot_clean")
        .arg("-m")
        .arg(mount_point)
        .status()
        .await;

    match status {
        Ok(status) => {
            if !status.success() {
                Err(anyhow!("dot_clean failed"))
            } else {
                Ok(())
            }
        }
        Err(e) => Err(anyhow!("dot_clean failed: {}", e)),
    }
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
        let entry = zip
            .file()
            .entries()
            .get(i)
            .ok_or_else(|| anyhow!("Failed to get entry"))?;

        let path = dest_dir.join(sanitize_file_path(entry.filename().as_str()?));

        let entry_is_dir = entry.dir()?;

        let mut entry_reader = zip.reader_without_entry(i).await?;

        if entry_is_dir {
            if !path.exists() {
                fs::create_dir_all(&path).await?;
            }
        } else {
            let parent = path
                .parent()
                .ok_or_else(|| anyhow!("Failed to get parent dir"))?;

            if !parent.is_dir() {
                fs::create_dir_all(parent).await?;
            }

            let mut file = File::create(&path).await?;
            io::copy(&mut entry_reader, &mut file).await?;
        }
    }

    Ok(())
}

pub async fn extract_zip_bytes(zip_bytes: Vec<u8>, dest_dir: &Path) -> Result<()> {
    println!("Extracting zip into \"{}\"", dest_dir.display());

    let zip = async_zip::base::read::mem::ZipFileReader::new(zip_bytes).await?;

    for i in 0..zip.file().entries().len() {
        let entry = zip
            .file()
            .entries()
            .get(i)
            .ok_or_else(|| anyhow!("Failed to get entry"))?;

        let path = dest_dir.join(sanitize_file_path(entry.filename().as_str()?));

        let entry_is_dir = entry.dir()?;

        let mut entry_reader = zip.reader_without_entry(i).await?;

        if entry_is_dir {
            if !path.exists() {
                fs::create_dir_all(&path).await?;
            }
        } else {
            let parent = path
                .parent()
                .ok_or_else(|| anyhow!("Failed to get parent dir"))?;

            if !parent.is_dir() {
                fs::create_dir_all(parent).await?;
            }

            let mut file = File::create(&path).await?;
            io::copy(&mut entry_reader, &mut file).await?;
        }
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

/// Returns a relative path without reserved names, redundant separators, ".", or "..".
fn sanitize_file_path(path: &str) -> PathBuf {
    // Replaces backwards slashes
    path.replace('\\', "/")
        // Sanitizes each component
        .split('/')
        .map(sanitize_filename::sanitize)
        .map(String::from)
        .collect()
}
