// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::USER_AGENT;
use crate::base_dir::BaseDir;
use anyhow::{Context, Result};
use std::fs::{self, File};
use std::io;
use std::io::Cursor;

const DOWNLOAD_URL: &str = "https://www.gametdb.com/wiitdb.zip";

/// Handles the blocking logic of downloading and extracting the database.
pub fn download_and_extract_database(base_dir: &BaseDir) -> Result<()> {
    // Create the target directory.
    let target_dir = base_dir.usbloadergx_dir();
    fs::create_dir_all(&target_dir)
        .with_context(|| format!("Failed to create directory at: {}", target_dir.display()))?;

    // Perform the download request.
    let response = minreq::get(DOWNLOAD_URL)
        .with_header("User-Agent", USER_AGENT)
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
        .with_context(|| format!("Failed to create output file at: {}", target_path.display()))?;
    io::copy(&mut zipped_file, &mut outfile).with_context(|| {
        format!(
            "Failed to extract 'wiitdb.xml' to {}",
            target_path.display()
        )
    })?;

    Ok(())
}
