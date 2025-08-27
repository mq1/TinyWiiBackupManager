use super::{Job, JobContext, JobResult, JobState, start_job, update_status};
use crate::util::download::download_with_progress;
use anyhow::{Context, Result};
use log::{error, info};
use std::fs::{self, File};
use std::io;
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;
use std::task::Waker;

const DOWNLOAD_URL: &str = "https://www.gametdb.com/wiitdb.zip";
const STATUS_MESSAGE: &str = "Downloading wiitdb.zip...";

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
    update_status(&context, STATUS_MESSAGE.to_string(), 0, 1, &cancel)?;

    match download_and_extract_database(&config.base_dir, &context, &cancel) {
        Ok(path) => {
            info!("GameTDB database updated successfully at: {:?}", path);
            update_status(
                &context,
                "GameTDB database updated successfully".to_string(),
                1,
                1,
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
fn download_and_extract_database(
    base_dir: &Path,
    context: &JobContext,
    cancel: &Receiver<()>,
) -> Result<PathBuf> {
    info!("Downloading GameTDB database from {DOWNLOAD_URL}");

    // Create the target directory.
    let target_dir = base_dir.join("apps/usbloader_gx");
    fs::create_dir_all(&target_dir)
        .with_context(|| format!("Failed to create directory at: {target_dir:?}"))?;

    // Perform the download request.
    let response = ureq::get(DOWNLOAD_URL)
        .call()
        .with_context(|| format!("Failed to download from {DOWNLOAD_URL}"))?;

    // Create an in-memory buffer and perform the download with progress updates.
    let mut buffer = if let Some(len) = response.body().content_length() {
        Vec::with_capacity(len as usize)
    } else {
        Vec::new()
    };
    download_with_progress(
        response,
        &mut buffer,
        STATUS_MESSAGE.to_string(),
        Some("wiitdb.zip".to_string()),
        0,
        1,
        context,
        cancel,
    )?;
    info!("Download complete, read {} bytes", buffer.len());

    // Update status to indicate extraction is starting.
    update_status(
        context,
        "Extracting wiitdb.xml...".to_string(),
        1,
        1,
        cancel,
    )?;

    // Open the zip archive from the in-memory buffer.
    let mut archive = zip::ZipArchive::new(Cursor::new(&*buffer))
        .with_context(|| "Failed to create zip archive from cursor")?;
    let mut zip_file = archive
        .by_name("wiitdb.xml")
        .context("Could not find 'wiitdb.xml' in the downloaded archive")?;

    // Extract the wiitdb.xml file to the target directory.
    let target_path = target_dir.join("wiitdb.xml");
    let mut outfile = File::create(&target_path)
        .with_context(|| format!("Failed to create output file at: {target_path:?}"))?;
    io::copy(&mut zip_file, &mut outfile)
        .with_context(|| format!("Failed to extract 'wiitdb.xml' to {target_path:?}"))?;
    outfile
        .flush()
        .with_context(|| format!("Failed to extract 'wiitdb.xml' to {target_path:?}"))?;

    info!("Successfully extracted wiitdb.xml to {target_path:?}");
    Ok(target_path)
}
