// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::app::App;
use crate::http::AGENT;
use std::fs::{self, OpenOptions};
use std::io::{self, BufWriter, Cursor, Read};
use zip::ZipArchive;

const DOWNLOAD_URL: &str = "https://www.gametdb.com/wiitdb.zip";

/// Handles the blocking logic of downloading and extracting the database.
pub fn spawn_download_task(app: &App) {
    let mount_point = app.config.contents.mount_point.clone();

    app.task_processor.spawn(move |status, toasts| {
        *status.lock() = "📥 Downloading wiitdb.xml...".to_string();

        // Create the target directory.
        let target_dir = mount_point.join("apps").join("usbloader_gx");
        fs::create_dir_all(&target_dir)?;

        // Perform the download request.
        let (_, body) = AGENT.get(DOWNLOAD_URL).call()?.into_parts();
        let mut buffer = Vec::with_capacity(body.content_length().unwrap_or(0) as usize);
        body.into_reader().read_to_end(&mut buffer)?;

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

        toasts
            .lock()
            .info("📥 wiitdb.xml Downloaded Successfully".to_string());

        Ok(())
    });
}
