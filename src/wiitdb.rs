// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::TaskType;
use crate::http::AGENT;
use crate::tasks::TaskProcessor;
use slint::ToSharedString;
use std::fs::{self, OpenOptions};
use std::io::{self, BufWriter, Cursor};
use std::path::PathBuf;
use std::sync::Arc;
use zip::ZipArchive;

const DOWNLOAD_URL: &str = "https://www.gametdb.com/wiitdb.zip";

/// Handles the blocking logic of downloading and extracting the database.
pub fn download(mount_point: PathBuf, task_processor: &Arc<TaskProcessor>) {
    task_processor.spawn(Box::new(move |weak| {
        weak.upgrade_in_event_loop(move |handle| {
            handle.set_status("Downloading wiitdb.xml...".to_shared_string());
            handle.set_task_type(TaskType::DownloadingFile);
        })?;

        // Create the target directory.
        let target_dir = mount_point.join("apps").join("usbloader_gx");
        fs::create_dir_all(&target_dir)?;

        // Perform the download request.
        let mut response = AGENT.get(DOWNLOAD_URL).call()?;

        let buffer = response
            .body_mut()
            .with_config()
            .limit(20 * 1024 * 1024) // 20 MiB
            .read_to_vec()?;

        // Create a cursor in memory.
        let cursor = Cursor::new(buffer);

        // Open the zip archive from the in-memory buffer.
        let mut archive = ZipArchive::new(cursor)?;
        let mut zipped_file = archive.by_name("wiitdb.xml")?;

        // Extract the wiitdb.xml file to the target directory.
        let target_path = target_dir.join("wiitdb.xml");
        let target_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&target_path)?;
        let mut writer = BufWriter::new(target_file);
        io::copy(&mut zipped_file, &mut writer)?;

        Ok("wiitbd.xml downloaded successfully".to_string())
    }));
}
