// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::app::App;
use crate::base_dir::BaseDir;
use crate::messages::BackgroundMessage;
use anyhow::{Context, Result, anyhow};
use std::fs::{self, File};
use std::io;
use std::io::Cursor;

const DOWNLOAD_URL: &str = "https://www.gametdb.com/wiitdb.zip";

/// Handles the blocking logic of downloading and extracting the database.
fn download_and_extract_database(base_dir: &BaseDir) -> Result<()> {
    // Create the target directory.
    let target_dir = base_dir.usbloadergx_dir();
    fs::create_dir_all(&target_dir)
        .with_context(|| format!("Failed to create directory at: {target_dir:?}"))?;

    // Perform the download request.
    let response = minreq::get(DOWNLOAD_URL)
        .send()
        .with_context(|| format!("Failed to download from {DOWNLOAD_URL}"))?;

    let buffer = response.as_bytes();

    // Create a cursor in memory.
    let cursor = Cursor::new(buffer);

    // Open the zip archive from the in-memory buffer.
    let mut archive =
        zip::ZipArchive::new(cursor).context("Failed to create zip archive from cursor")?;

    let mut zipped_file = archive
        .by_name("wiitdb.xml")
        .context("Could not find 'wiitdb.xml' in the downloaded archive")?;

    // Extract the wiitdb.xml file to the target directory.
    let target_path = target_dir.join("wiitdb.xml");
    let mut outfile = File::create(&target_path)
        .with_context(|| format!("Failed to create output file at: {target_path:?}"))?;
    io::copy(&mut zipped_file, &mut outfile)
        .with_context(|| format!("Failed to extract 'wiitdb.xml' to {target_path:?}"))?;

    Ok(())
}

pub fn spawn_download_database_task(app: &App) {
    let base_dir = app.base_dir.clone();
    app.task_processor.spawn_task(move |ui_sender| {
        // Send an initial message to the UI.
        let _ = ui_sender.send(BackgroundMessage::UpdateStatus(
            "Downloading wiitdb.zip...".to_string(),
        ));

        let base_dir = base_dir.ok_or_else(|| anyhow!("Base directory is not set"))?;

        download_and_extract_database(&base_dir)
            .context("Failed to download and extract the database")?;

        let _ = ui_sender.send(BackgroundMessage::Info(
            "wiitdb.zip downloaded and extracted successfully.".to_string(),
        ));

        Ok(())
    });
}
