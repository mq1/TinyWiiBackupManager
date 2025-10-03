// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::http::AGENT;
use anyhow::Result;
use std::fs::{self, File};
use std::io;
use std::io::Cursor;
use std::path::Path;

const DOWNLOAD_URL: &str = "https://www.gametdb.com/wiitdb.zip";

/// Handles the blocking logic of downloading and extracting the database.
pub fn download_and_extract_database(mount_point: &Path) -> Result<()> {
    // Create the target directory.
    let target_dir = mount_point.join("apps").join("usbloader_gx");
    fs::create_dir_all(&target_dir)?;

    // Perform the download request.
    let mut response = AGENT.get(DOWNLOAD_URL).call()?;

    let buffer = response.body_mut().read_to_vec()?;

    // Create a cursor in memory.
    let cursor = Cursor::new(buffer);

    // Open the zip archive from the in-memory buffer.
    let mut archive = zip::ZipArchive::new(cursor)?;

    let mut zipped_file = archive.by_name("wiitdb.xml")?;

    // Extract the wiitdb.xml file to the target directory.
    let target_path = target_dir.join("wiitdb.xml");
    let _ = fs::remove_file(&target_path);
    let mut outfile = File::create(&target_path)?;

    io::copy(&mut zipped_file, &mut outfile)?;

    Ok(())
}
