// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{message::Message, state::State};
use derive_getters::Getters;
use iced::Task;
use std::{fs, path::Path};

#[cfg(target_os = "linux")]
const FAT32_MAGIC: rustix::fs::FsWord = 0x4d44;

#[cfg(target_os = "macos")]
const FAT32_MAGIC: [i8; 16] = [109, 115, 100, 111, 115, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

#[cfg(windows)]
const FAT32_MAGIC: &str = "FAT32";

#[derive(Debug, Clone, Getters)]
pub struct DriveInfo {
    #[getter(copy)]
    used_bytes: u64,
    #[getter(copy)]
    total_bytes: u64,
    #[getter(copy)]
    is_fat32: bool,
}

impl DriveInfo {
    #[cfg(unix)]
    pub fn maybe_from_path(path: &Path) -> Option<Self> {
        let stat = rustix::fs::statfs(path).ok()?;

        #[cfg(target_os = "linux")]
        let block_size = stat.f_frsize as u64;

        #[cfg(target_os = "macos")]
        let block_size = u64::from(stat.f_bsize);

        let total_bytes = stat.f_blocks * block_size;
        let avail_bytes = stat.f_bavail * block_size;
        let used_bytes = total_bytes.saturating_sub(avail_bytes);

        #[cfg(target_os = "linux")]
        let is_fat32 = stat.f_type == FAT32_MAGIC;

        #[cfg(target_os = "macos")]
        let is_fat32 = stat.f_fstypename == FAT32_MAGIC;

        let info = Self {
            used_bytes,
            total_bytes,
            is_fat32,
        };

        println!("FSINFO: {info:?}");

        Some(info)
    }

    #[cfg(windows)]
    pub fn maybe_from_path(path: &Path) -> Option<Self> {
        let path = path.to_str()?;
        let root_path = winsafe::GetVolumePathName(path).ok()?;

        let mut total_bytes = 0;
        let mut avail_bytes = 0;
        winsafe::GetDiskFreeSpaceEx(
            Some(root_path.as_str()),
            Some(&mut avail_bytes),
            Some(&mut total_bytes),
            None,
        )
        .ok()?;
        let used_bytes = total_bytes.saturating_sub(avail_bytes);

        let mut file_system_name = String::new();
        winsafe::GetVolumeInformation(
            Some(root_path.as_str()),
            None,
            None,
            None,
            None,
            Some(&mut file_system_name),
        )
        .ok()?;
        let is_fat32 = file_system_name == FAT32_MAGIC;

        let info = Self {
            used_bytes,
            total_bytes,
            is_fat32,
        };

        println!("FSINFO: {info:?}");

        Some(info)
    }

    pub fn get_usage_string(&self) -> String {
        const GIB: f64 = 1024. * 1024. * 1024.;

        #[allow(clippy::cast_precision_loss)]
        let (used_bytes, total_bytes) = (self.used_bytes as f64, self.total_bytes as f64);

        format!("{:.2}/{:.2} GiB", used_bytes / GIB, total_bytes / GIB)
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
