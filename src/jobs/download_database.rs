use super::{Job, JobContext, JobResult, JobState, start_job, update_status};
use anyhow::{Context, Result};
use std::fs::{self, File};
use std::io::{BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;
use std::task::Waker;
use tempfile::NamedTempFile;

pub struct DownloadDatabaseConfig {
    pub base_dir: PathBuf,
}

pub struct DownloadDatabaseResult {
    pub success: bool,
    pub path: Option<PathBuf>,
}

pub fn start_download_database(waker: Waker, config: DownloadDatabaseConfig) -> JobState {
    start_job(
        waker,
        "Download GameTDB Database",
        Job::DownloadDatabase,
        move |context, cancel| {
            download_database(context, cancel, config).map(JobResult::DownloadDatabase)
        },
    )
}

fn download_database(
    context: JobContext,
    cancel: Receiver<()>,
    config: DownloadDatabaseConfig,
) -> Result<Box<DownloadDatabaseResult>> {
    // Update status
    update_status(
        &context,
        "Downloading wiitdb.zip".to_string(),
        0,
        2,
        &cancel,
    )?;

    // Download logic
    match download_database_blocking(&config.base_dir) {
        Ok(path) => {
            update_status(&context, "Complete".to_string(), 2, 2, &cancel)?;
            Ok(Box::new(DownloadDatabaseResult {
                success: true,
                path: Some(path),
            }))
        }
        Err(e) => {
            log::error!("Failed to download GameTDB database: {}", e);
            Err(e)
        }
    }
}

fn download_database_blocking(base_dir: &Path) -> Result<PathBuf> {
    log::info!("Downloading GameTDB database from https://www.gametdb.com/wiitdb.zip");

    // Download the zip file to a temporary file
    let mut temp_zip = NamedTempFile::new().context("Failed to create temporary file for zip")?;

    let response = reqwest::blocking::get("https://www.gametdb.com/wiitdb.zip")
        .context("Failed to download wiitdb.zip")?;

    if !response.status().is_success() {
        anyhow::bail!("HTTP error downloading wiitdb.zip: {}", response.status());
    }

    let bytes = response
        .bytes()
        .context("Failed to read wiitdb.zip response")?;

    temp_zip
        .write_all(&bytes)
        .context("Failed to write temporary zip file")?;
    temp_zip
        .flush()
        .context("Failed to flush temporary zip file")?;

    // Create target directory
    let target_dir = base_dir.join("apps/usbloader_gx");
    fs::create_dir_all(&target_dir).context("Failed to create usbloader_gx directory")?;

    // Extract the zip file
    let file = File::open(temp_zip.path()).context("Failed to open temporary zip file")?;
    let mut archive =
        zip::ZipArchive::new(BufReader::new(file)).context("Failed to read zip archive")?;

    // Look for wiitdb.xml in the archive
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).context("Failed to access zip entry")?;
        let name = file.name();

        // Only extract wiitdb.xml
        if name == "wiitdb.xml" || name.ends_with("/wiitdb.xml") {
            // Extract directly to the target location
            let target_path = target_dir.join("wiitdb.xml");
            let mut outfile = File::create(&target_path).context("Failed to create wiitdb.xml")?;
            std::io::copy(&mut file, &mut outfile).context("Failed to extract wiitdb.xml")?;
            outfile.flush().context("Failed to flush wiitdb.xml")?;

            log::info!("Successfully installed wiitdb.xml to {:?}", target_path);
            return Ok(target_path);
        }
    }

    anyhow::bail!("wiitdb.xml not found in downloaded archive")
}
