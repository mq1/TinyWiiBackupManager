// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use std::{fs::File, io::BufReader, path::Path, sync::Arc};

use anyhow::Result;
use slint::ToSharedString;
use zip::ZipArchive;

use crate::{TaskType, tasks::TaskProcessor};

pub fn install_apps(mount_point: &Path, task_processor: &Arc<TaskProcessor>) -> Result<()> {
    let paths = rfd::FileDialog::new()
        .set_title("Select Wii HBC App(s)")
        .add_filter("Wii App", &["zip", "ZIP"])
        .pick_files();

    if let Some(paths) = paths {
        for path in paths {
            let mount_point = mount_point.to_path_buf();
            task_processor.spawn(Box::new(move |weak| {
                let status = format!("Installing {}...", path.display());
                weak.upgrade_in_event_loop(move |handle| {
                    handle.set_status(status.to_shared_string());
                    handle.set_task_type(TaskType::InstallingApps);
                })?;

                let file = File::open(path)?;
                let reader = BufReader::new(file);
                let mut archive = ZipArchive::new(reader)?;

                if archive.file_names().any(|n| n.starts_with("apps/")) {
                    archive.extract(&mount_point)?;
                } else {
                    archive.extract(mount_point.join("apps"))?;
                }

                Ok(())
            }))?;
        }
    }

    Ok(())
}
