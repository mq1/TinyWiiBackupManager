// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

#![allow(unused_mut, unused_assignments)]

use crate::{message::Message, state::State};
use derive_getters::Getters;
use iced::Task;
use std::{fs, path::Path};
use which_fs::FsKind;

#[derive(Debug, Clone, Getters)]
pub struct DriveInfo {
    #[getter(copy)]
    used_bytes: u64,
    #[getter(copy)]
    total_bytes: u64,
    #[getter(copy)]
    allocation_granularity: u64,
    #[getter(copy)]
    fs_kind: FsKind,
}

impl DriveInfo {
    pub fn maybe_from_path(path: &Path) -> Option<Self> {
        let stat = fs4::statvfs(path).ok()?;

        let total_bytes = stat.total_space();
        let avail_bytes = stat.available_space();
        let used_bytes = total_bytes.saturating_sub(avail_bytes);
        let allocation_granularity = stat.allocation_granularity();

        // My library :)
        let fs_kind = FsKind::try_from_path(path).ok()?;

        let info = Self {
            used_bytes,
            total_bytes,
            allocation_granularity,
            fs_kind,
        };

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
