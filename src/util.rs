// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{message::Message, state::State};
use anyhow::{Result, anyhow};
use iced::Task;
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
    let mount_point = state.config.mount_point().clone();
    Task::perform(
        async move { get_drive_usage(&mount_point) },
        Message::GotDriveUsage,
    )
}

fn get_drive_usage(mount_point: &Path) -> String {
    const GIB: f64 = 1024.0 * 1024.0 * 1024.0;

    if mount_point.as_os_str().is_empty() {
        return "0/0 GiB".to_string();
    }

    let disks = Disks::new_with_refreshed_list();

    disks
        .iter()
        .filter(|disk| mount_point.starts_with(disk.mount_point()))
        .max_by_key(|disk| disk.mount_point().as_os_str().len())
        .map_or("0/0 GiB".to_string(), |disk| {
            let total = disk.total_space();
            let used = total - disk.available_space();

            #[allow(clippy::cast_precision_loss)]
            let (used, total) = (used as f64, total as f64);

            format!("{:.2}/{:.2} GiB", used / GIB, total / GIB)
        })
}

/// Returns Ok if we can create a file >4 GiB in this directory
pub fn can_write_over_4gb(mount_point: &Path) -> bool {
    use std::io::{Seek, SeekFrom, Write};

    if mount_point.as_os_str().is_empty() {
        return false;
    }

    // Create a temp file in the target directory
    let Ok(tmp) = &mut NamedTempFile::new_in(mount_point) else {
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
pub fn run_dot_clean(mount_point: PathBuf) -> Result<String> {
    let status = std::process::Command::new("dot_clean")
        .arg("-m")
        .arg(mount_point)
        .status();

    match status {
        Ok(status) => {
            if status.success() {
                Ok("dot_clean successful".to_string())
            } else {
                Err(anyhow!("dot_clean failed"))
            }
        }
        Err(e) => Err(anyhow!("dot_clean failed: {e}")),
    }
}
