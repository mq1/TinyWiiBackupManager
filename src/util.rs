// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{message::Message, state::State};
use derive_getters::Getters;
use iced::Task;
use std::{fs, path::Path};
use sysinfo::Disks;

const GIB: u64 = 1024 * 1024 * 1024;

#[derive(Debug, Clone, Getters)]
pub struct DriveInfo {
    #[getter(copy)]
    used_space: u64,
    #[getter(copy)]
    total_space: u64,
    #[getter(copy)]
    is_fat32: bool,
}

impl DriveInfo {
    pub fn maybe_from_path(path: &Path) -> Option<Self> {
        let disks = Disks::new_with_refreshed_list();
        let path = path.canonicalize().ok()?;

        let disk = disks
            .iter()
            .filter(|disk| path.starts_with(disk.mount_point()))
            .max_by_key(|disk| disk.mount_point().as_os_str().len())?;

        let total_space = disk.total_space();
        let used_space = total_space - disk.available_space();

        let is_fat32 = matches!(disk.file_system().to_str()?, "msdos" | "vfat" | "FAT32");

        let info = Self {
            used_space,
            total_space,
            is_fat32,
        };

        Some(info)
    }

    pub fn get_usage_string(&self) -> String {
        let used_whole = self.used_space / GIB;
        let total_whole = self.total_space / GIB;

        let used_fract = ((self.used_space % GIB) * 100) / GIB;
        let total_fract = ((self.total_space % GIB) * 100) / GIB;

        format!("{used_whole}.{used_fract:02}/{total_whole}.{total_fract:02} GiB")
    }

    pub fn get_task(state: &State) -> Task<Message> {
        let mount_point = state.config.mount_point().clone();

        Task::perform(
            async move { DriveInfo::maybe_from_path(&mount_point) },
            Message::GotDriveInfo,
        )
    }
}

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

#[cfg(target_os = "macos")]
pub fn run_dot_clean(mount_point: std::path::PathBuf) -> anyhow::Result<String> {
    let status = std::process::Command::new("dot_clean")
        .arg("-m")
        .arg(mount_point)
        .status();

    match status {
        Ok(status) => {
            if status.success() {
                Ok("dot_clean successful".to_string())
            } else {
                Err(anyhow::anyhow!("dot_clean failed"))
            }
        }
        Err(e) => Err(anyhow::anyhow!("dot_clean failed: {e}")),
    }
}

/// Leftovers from previous versions
pub fn clean_old_files(data_dir: &Path) {
    let wiitdb_path = data_dir.join("wiitdb.xml");
    let _ = fs::remove_file(wiitdb_path);

    let titles_path = data_dir.join("titles.txt");
    let _ = fs::remove_file(titles_path);

    let redump_wii_path = data_dir.join("redump-wii.dat");
    let _ = fs::remove_file(redump_wii_path);

    let redump_gc_path = data_dir.join("redump-gc.dat");
    let _ = fs::remove_file(redump_gc_path);
}
