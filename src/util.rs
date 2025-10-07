// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::{Result, bail};
use size::Size;
use std::{
    io::{Seek, SeekFrom, Write},
    path::Path,
};
use sysinfo::Disks;
use tempfile::NamedTempFile;

pub fn get_disk_usage(mount_point: &Path) -> Option<String> {
    if mount_point.as_os_str().is_empty() {
        return None;
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
}

/// Returns Ok if we can create a file >4 GiB in this directory
pub fn can_write_over_4gb(mount_point: &Path) -> Result<()> {
    if mount_point.as_os_str().is_empty() {
        bail!("Fat32 check failed: No mount point selected");
    }

    // Create a temp file in the target directory
    let mut tmp = NamedTempFile::new_in(mount_point)?;

    // Seek to 4 GiB
    tmp.as_file_mut()
        .seek(SeekFrom::Start(4 * 1024 * 1024 * 1024))?;

    // Write a single byte
    tmp.as_file_mut().write_all(&[0])?;

    Ok(())
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
