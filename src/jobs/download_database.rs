use super::{Job, JobContext, JobResult, JobState, start_job, update_status};
use anyhow::{Context, Result};
use log::{error, info};
use std::fs::{self, File};
use std::io;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;
use std::task::Waker;

const DOWNLOAD_URL: &str = "https://www.gametdb.com/wiitdb.zip";

pub struct DownloadDatabaseConfig {
    pub base_dir: PathBuf,
}

pub struct DownloadDatabaseResult {
    pub path: Option<PathBuf>,
}

/// Starts the job to download the GameTDB database.
pub fn start_download_database(waker: Waker, config: DownloadDatabaseConfig) -> JobState {
    start_job(
        waker,
        "Update GameTDB database",
        Job::DownloadDatabase,
        move |context, cancel| {
            download_database(context, cancel, config).map(JobResult::DownloadDatabase)
        },
    )
}

/// Orchestrates the database download and extraction process.
fn download_database(
    context: JobContext,
    cancel: Receiver<()>,
    config: DownloadDatabaseConfig,
) -> Result<Box<DownloadDatabaseResult>> {
    update_status(
        &context,
        "Downloading wiitdb.zip...".to_string(),
        0,
        2,
        &cancel,
    )?;

    let result = download_and_extract_database(&config.base_dir);

    match result {
        Ok(path) => {
            info!("GameTDB database updated successfully at: {:?}", path);
            update_status(
                &context,
                "GameTDB database updated successfully".to_string(),
                2,
                2,
                &cancel,
            )?;
            Ok(Box::new(DownloadDatabaseResult { path: Some(path) }))
        }
        Err(e) => {
            error!("Failed to download GameTDB database: {:?}", e);
            // Propagate the error to the job runner.
            Err(e)
        }
    }
}

/// Handles the blocking logic of downloading and extracting the database.
fn download_and_extract_database(base_dir: &Path) -> Result<PathBuf> {
    info!("Downloading GameTDB database from {}", DOWNLOAD_URL);

    // Create the target directory.
    let target_dir = base_dir.join("apps/usbloader_gx");
    fs::create_dir_all(&target_dir)
        .with_context(|| format!("Failed to create directory at: {:?}", target_dir))?;

    // Perform the download request.
    let response = ureq::get(DOWNLOAD_URL)
        .call()
        .with_context(|| format!("Failed to download from {}", DOWNLOAD_URL))?;

    // Make a buffer from the response body and create a cursor.
    let (_, body) = response.into_parts();
    let mut buffer = Vec::new();
    body.into_reader().read_to_end(&mut buffer)?;
    let cursor = Cursor::new(buffer);

    // Create a zip archive from the cursor.
    let mut archive =
        zip::ZipArchive::new(cursor).with_context(|| "Failed to create zip archive from cursor")?;

    // Look for wiitdb.xml and extract it.
    let mut zip_file = archive
        .by_name("wiitdb.xml")
        .context("Could not find 'wiitdb.xml' in the downloaded archive")?;

    let target_path = target_dir.join("wiitdb.xml");
    let mut outfile = File::create(&target_path)
        .with_context(|| format!("Failed to create output file at: {:?}", target_path))?;

    io::copy(&mut zip_file, &mut outfile)
        .with_context(|| format!("Failed to extract 'wiitdb.xml' to {:?}", target_path))?;

    info!("Successfully extracted wiitdb.xml to {:?}", target_path);

    Ok(target_path)
}
