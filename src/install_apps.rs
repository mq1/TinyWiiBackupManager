// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{Config, TaskType, tasks::TaskProcessor};
use anyhow::Result;
use slint::ToSharedString;
use std::{
    fs::{self, File},
    io::BufReader,
    path::PathBuf,
    sync::Arc,
};
use zip::ZipArchive;

pub fn install_apps(config: &Config, task_processor: &Arc<TaskProcessor>) -> Result<()> {
    let remove_sources = config.remove_sources_apps;
    let mount_point = PathBuf::from(&config.mount_point);

    let paths = rfd::FileDialog::new()
        .set_title("Select Wii HBC App(s)")
        .add_filter("Wii App", &["zip", "ZIP"])
        .pick_files();

    if let Some(paths) = paths {
        task_processor.spawn(Box::new(move |weak| {
            for path in paths {
                {
                    let status = format!("Installing {}...", path.display());
                    weak.upgrade_in_event_loop(move |handle| {
                        handle.set_status(status.to_shared_string());
                        handle.set_task_type(TaskType::InstallingApps);
                    })?;

                    let file = File::open(&path)?;
                    let reader = BufReader::new(file);
                    let mut archive = ZipArchive::new(reader)?;

                    if archive.file_names().any(|n| n.starts_with("apps/")) {
                        archive.extract(&mount_point)?;
                    } else {
                        archive.extract(mount_point.join("apps"))?;
                    }
                }

                if remove_sources {
                    fs::remove_file(path)?;
                }
            }

            Ok(())
        }))?;
    }

    Ok(())
}
