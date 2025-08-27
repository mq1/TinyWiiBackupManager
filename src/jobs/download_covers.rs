use super::{Job, JobContext, JobResult, JobState, start_job, update_status};
use crate::cover_manager::CoverType;
use crate::util::regions::Region;
use anyhow::{Context, Result, bail};
use log::{debug, info, warn};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;
use std::task::Waker;
use std::time::Duration;
use std::{fs, io};
use tempfile::NamedTempFile;

pub struct DownloadCoversConfig {
    pub base_dir: PathBuf,
    pub cover_type: CoverType,
    pub game_ids: Vec<String>,
}

pub struct DownloadCoversResult {
    pub downloaded: usize,
    pub skipped: usize,
    pub failed: usize,
    pub failed_ids: Vec<String>,
    pub cover_type: CoverType,
}

pub fn start_download_covers(waker: Waker, config: DownloadCoversConfig) -> JobState {
    let title = format!(
        "Download {} covers",
        match config.cover_type {
            CoverType::Cover3D => "3D",
            CoverType::Cover2D => "2D",
            CoverType::CoverFull => "full",
            CoverType::Disc => "disc",
        }
    );
    start_job(
        waker,
        &title,
        Job::DownloadCovers,
        move |context, cancel| {
            download_covers(context, cancel, config).map(JobResult::DownloadCovers)
        },
    )
}

fn download_covers(
    context: JobContext,
    cancel: Receiver<()>,
    config: DownloadCoversConfig,
) -> Result<Box<DownloadCoversResult>> {
    let mut downloaded = 0;
    let mut skipped = 0;
    let mut failed = 0;
    let mut failed_ids = Vec::new();

    let total = config.game_ids.len() as u32;

    for (idx, game_id) in config.game_ids.iter().enumerate() {
        // Update status and check for cancellation
        update_status(
            &context,
            format!("Downloading cover for {}", game_id),
            idx as u32,
            total,
            &cancel,
        )?;

        // Check if already exists
        let cover_path = config
            .base_dir
            .join("apps/usbloader_gx")
            .join(config.cover_type.subdirectory())
            .join(format!("{}.png", game_id));

        if cover_path.exists() {
            skipped += 1;
            continue;
        }

        // Download with retries
        match download_with_retries(&config.base_dir, game_id, config.cover_type) {
            Ok(_) => {
                downloaded += 1;
                info!(
                    "Downloaded {} cover for {}",
                    config.cover_type.name(),
                    game_id
                );
            }
            Err(e) if e.to_string().contains("404") || e.to_string().contains("Not Found") => {
                failed += 1;
                failed_ids.push(game_id.clone());
                debug!("Cover not found for {}: {}", game_id, e);
            }
            Err(e) => {
                failed += 1;
                warn!("Failed to download {} cover: {}", game_id, e);
            }
        }
    }

    // Final status
    update_status(
        &context,
        format!(
            "Downloaded {} covers ({} skipped, {} failed)",
            downloaded, skipped, failed
        ),
        total,
        total,
        &cancel,
    )?;

    Ok(Box::new(DownloadCoversResult {
        downloaded,
        skipped,
        failed,
        failed_ids,
        cover_type: config.cover_type,
    }))
}

fn download_with_retries(base_dir: &Path, game_id: &str, cover_type: CoverType) -> Result<()> {
    let mut attempts = 0;
    loop {
        match download_single_cover(base_dir, game_id, cover_type) {
            Ok(_) => return Ok(()),
            Err(e) if e.to_string().contains("404") || e.to_string().contains("Not Found") => {
                return Err(e); // Don't retry 404s
            }
            Err(_) if attempts < 3 => {
                std::thread::sleep(Duration::from_secs(1 << attempts));
                attempts += 1;
            }
            Err(e) => return Err(e),
        }
    }
}

fn download_single_cover(base_dir: &Path, game_id: &str, cover_type: CoverType) -> Result<()> {
    // Determine language from game ID region
    let lang = Region::from_id(game_id).to_lang();

    // Construct GameTDB API URL
    let url = format!(
        "https://art.gametdb.com/wii/{}/{}/{}.png",
        cover_type.api_endpoint(),
        lang,
        game_id
    );

    // Download the cover to a temporary file
    debug!("Downloading cover from: {}", url);
    let mut response = ureq::get(&url)
        .call()
        .context("Failed to send HTTP request")?;

    if !response.status().is_success() {
        if response.status() == 404 {
            bail!("404 Not Found");
        } else {
            bail!("HTTP error: {}", response.status());
        }
    }

    // Write to temporary file first
    let mut temp_file =
        NamedTempFile::new().context("Failed to create temporary file for cover")?;

    io::copy(&mut response.body_mut().as_reader(), &mut temp_file)
        .context("Failed to copy cover to temporary file")?;

    temp_file
        .flush()
        .context("Failed to flush temporary file")?;

    // Create target directory structure
    let target_path = base_dir
        .join("apps/usbloader_gx")
        .join(cover_type.subdirectory())
        .join(format!("{}.png", game_id));

    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent).context("Failed to create cover directory")?;
    }

    // Copy to final location (safer than persist for cross-filesystem)
    fs::copy(temp_file.path(), &target_path).context("Failed to copy cover to final location")?;
    // temp_file will be automatically cleaned up when dropped

    Ok(())
}

impl CoverType {
    pub fn name(&self) -> &'static str {
        match self {
            CoverType::Cover3D => "3D",
            CoverType::Cover2D => "2D",
            CoverType::CoverFull => "full",
            CoverType::Disc => "disc",
        }
    }
}
