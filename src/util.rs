// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{message::Message, state::State};
use derive_getters::Getters;
use iced::Task;
use std::{fs, path::Path};

const GIB: u64 = 1024 * 1024 * 1024;

#[derive(Debug, Clone, Getters)]
pub struct DriveInfo {
    #[getter(copy)]
    used_bytes: u64,
    #[getter(copy)]
    total_bytes: u64,
    #[getter(copy)]
    is_fat: bool,
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
        let used_bytes = total_bytes - avail_bytes;

        #[cfg(target_os = "linux")]
        let is_fat = {
            let f_type = stat.f_type as u32;
            f_type == 0x4d44
        };

        #[cfg(target_os = "macos")]
        let is_fat = {
            let fs_type = unsafe { std::ffi::CStr::from_ptr(stat.f_fstypename.as_ptr()) };
            fs_type.to_str().is_ok_and(|fs_type| fs_type == "msdos")
        };

        let info = Self {
            used_bytes,
            total_bytes,
            is_fat,
        };

        Some(info)
    }

    pub fn get_usage_string(&self) -> String {
        let used_whole = self.used_bytes / GIB;
        let total_whole = self.total_bytes / GIB;

        let used_fract = ((self.used_bytes % GIB) * 100) / GIB;
        let total_fract = ((self.total_bytes % GIB) * 100) / GIB;

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
