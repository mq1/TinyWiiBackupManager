// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::messages::BackgroundMessage;
use crate::util::regions::REGION_TO_LANG;
use anyhow::{Context, Result};
use egui_inbox::UiInboxSender;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::{fs, thread};
use tempfile::NamedTempFile;

/// Types of cover art available from GameTDB
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoverType {
    /// 3D box art (default)
    Cover3D,
    /// 2D flat cover art
    Cover2D,
    /// Full cover art (front + back)
    CoverFull,
    /// Disc art
    Disc,
}

impl CoverType {
    /// Get the subdirectory name for this cover type in USB Loader GX structure
    pub fn subdirectory(&self) -> &'static str {
        match self {
            CoverType::Cover3D => "images",
            CoverType::Cover2D => "images/2D",
            CoverType::CoverFull => "images/full",
            CoverType::Disc => "images/disc",
        }
    }

    /// Get the GameTDB API endpoint for this cover type
    pub fn api_endpoint(&self) -> &'static str {
        match self {
            CoverType::Cover3D => "cover3D",
            CoverType::Cover2D => "cover",
            CoverType::CoverFull => "coverfull",
            CoverType::Disc => "disc",
        }
    }
}

/// Manages downloading and caching of game cover art from GameTDB
pub struct CoverManager {
    base_dir: PathBuf,
    /// Track which covers are currently being downloaded to avoid duplicates
    downloading: Arc<Mutex<HashSet<(String, CoverType)>>>,
}

impl CoverManager {
    /// Create a new CoverManager with the given base directory
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            base_dir,
            downloading: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    /// Get the local path where a cover should be stored for USB Loader GX compatibility
    pub fn get_cover_path(&self, game_id: &str, cover_type: CoverType) -> PathBuf {
        self.base_dir
            .join("apps/usbloader_gx")
            .join(cover_type.subdirectory())
            .join(format!("{}.png", game_id))
    }

    /// Check if a cover exists locally
    pub fn has_cover(&self, game_id: &str, cover_type: CoverType) -> bool {
        self.get_cover_path(game_id, cover_type).exists()
    }

    /// Check if a cover is currently being downloaded
    pub fn is_downloading(&self, game_id: &str, cover_type: CoverType) -> bool {
        self.downloading
            .lock()
            .unwrap()
            .contains(&(game_id.to_string(), cover_type))
    }

    /// Queue a cover download in a background thread
    pub fn queue_download(&self, game_id: String, cover_type: CoverType) {
        // Check if already downloading
        {
            let mut downloading = self.downloading.lock().unwrap();
            let key = (game_id.clone(), cover_type);
            if downloading.contains(&key) {
                return; // Already downloading
            }
            downloading.insert(key);
        }

        // Check if file already exists
        if self.has_cover(&game_id, cover_type) {
            // File exists, remove from downloading set and return
            self.downloading
                .lock()
                .unwrap()
                .remove(&(game_id, cover_type));
            return;
        }

        let base_dir = self.base_dir.clone();
        let downloading = Arc::clone(&self.downloading);

        thread::spawn(move || {
            let result = Self::download_cover(&base_dir, &game_id, cover_type);

            // Remove from downloading set
            downloading
                .lock()
                .unwrap()
                .remove(&(game_id.clone(), cover_type));

            // Log result
            match result {
                Ok(path) => {
                    log::info!("Downloaded cover for {} to {:?}", game_id, path);
                }
                Err(e) => {
                    // Log as debug since cover download failures are not critical
                    log::debug!("Failed to download cover for {}: {}", game_id, e);
                }
            }
        });
    }

    /// Download covers for multiple games
    pub fn download_all_covers(&self, game_ids: Vec<String>, cover_type: CoverType) {
        for game_id in game_ids {
            self.queue_download(game_id, cover_type);
        }
    }

    /// Download the GameTDB database (wiitdb.xml) in a background thread
    pub fn download_database(&self, sender: UiInboxSender<BackgroundMessage>) -> Result<()> {
        let base_dir = self.base_dir.clone();

        thread::spawn(move || match Self::download_database_blocking(&base_dir) {
            Ok(path) => {
                log::info!("Successfully downloaded GameTDB database to {:?}", path);
                let _ = sender.send(BackgroundMessage::GameTDBDownloadComplete);
            }
            Err(e) => {
                log::error!("Failed to download GameTDB database: {}", e);
                let _ = sender.send(BackgroundMessage::Error(e));
            }
        });

        Ok(())
    }

    /// Download the GameTDB database (blocking operation)
    fn download_database_blocking(base_dir: &Path) -> Result<PathBuf> {
        log::info!("Downloading GameTDB database from https://www.gametdb.com/wiitdb.zip");

        // Download the zip file to a temporary file
        let mut temp_zip =
            NamedTempFile::new().context("Failed to create temporary file for zip")?;

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
                let mut outfile = File::create(&target_path)
                    .context("Failed to create wiitdb.xml")?;
                std::io::copy(&mut file, &mut outfile)
                    .context("Failed to extract wiitdb.xml")?;
                outfile.flush()
                    .context("Failed to flush wiitdb.xml")?;

                log::info!("Successfully installed wiitdb.xml to {:?}", target_path);
                return Ok(target_path);
            }
        }

        anyhow::bail!("wiitdb.xml not found in downloaded archive")
    }

    /// Download a cover from GameTDB API (blocking operation)
    fn download_cover(base_dir: &Path, game_id: &str, cover_type: CoverType) -> Result<PathBuf> {
        // Determine language from game ID region
        let region_char = game_id
            .chars()
            .nth(3)
            .context("Game ID too short to determine region")?;
        let language = REGION_TO_LANG.get(&region_char).unwrap_or(&"EN");

        // Construct GameTDB API URL
        let url = format!(
            "https://art.gametdb.com/wii/{}/{}/{}.png",
            cover_type.api_endpoint(),
            language,
            game_id
        );

        // Download the cover to a temporary file
        log::debug!("Downloading cover from: {}", url);
        let response = reqwest::blocking::get(&url).context("Failed to send HTTP request")?;

        if !response.status().is_success() {
            anyhow::bail!("HTTP error: {}", response.status());
        }

        let bytes = response.bytes().context("Failed to read response body")?;

        // Write to temporary file first
        let mut temp_file =
            NamedTempFile::new().context("Failed to create temporary file for cover")?;
        temp_file
            .write_all(&bytes)
            .context("Failed to write cover to temporary file")?;
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
        fs::copy(temp_file.path(), &target_path)
            .context("Failed to copy cover to final location")?;
        // temp_file will be automatically cleaned up when dropped

        log::info!("Downloaded cover: {:?}", target_path);
        Ok(target_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_cover_type_subdirectory() {
        assert_eq!(CoverType::Cover3D.subdirectory(), "images");
        assert_eq!(CoverType::Cover2D.subdirectory(), "images/2D");
        assert_eq!(CoverType::CoverFull.subdirectory(), "images/full");
        assert_eq!(CoverType::Disc.subdirectory(), "images/disc");
    }

    #[test]
    fn test_cover_type_api_endpoint() {
        assert_eq!(CoverType::Cover3D.api_endpoint(), "cover3D");
        assert_eq!(CoverType::Cover2D.api_endpoint(), "cover");
        assert_eq!(CoverType::CoverFull.api_endpoint(), "coverfull");
        assert_eq!(CoverType::Disc.api_endpoint(), "disc");
    }

    #[test]
    fn test_get_cover_path() {
        let temp_dir = TempDir::new().unwrap();
        let manager = CoverManager::new(temp_dir.path().to_path_buf());

        let path = manager.get_cover_path("RMGE01", CoverType::Cover3D);
        assert_eq!(
            path,
            temp_dir.path().join("apps/usbloader_gx/images/RMGE01.png")
        );

        let path = manager.get_cover_path("RMGE01", CoverType::Cover2D);
        assert_eq!(
            path,
            temp_dir
                .path()
                .join("apps/usbloader_gx/images/2D/RMGE01.png")
        );
    }

    #[test]
    fn test_has_cover() {
        let temp_dir = TempDir::new().unwrap();
        let manager = CoverManager::new(temp_dir.path().to_path_buf());

        // Initially no cover
        assert!(!manager.has_cover("RMGE01", CoverType::Cover3D));

        // Create the cover file
        let cover_path = manager.get_cover_path("RMGE01", CoverType::Cover3D);
        fs::create_dir_all(cover_path.parent().unwrap()).unwrap();
        fs::write(&cover_path, b"fake image data").unwrap();

        // Now it should exist
        assert!(manager.has_cover("RMGE01", CoverType::Cover3D));
    }

    #[test]
    fn test_region_to_language_mapping() {
        assert_eq!(REGION_TO_LANG.get(&'E'), Some(&"US"));
        assert_eq!(REGION_TO_LANG.get(&'P'), Some(&"EN"));
        assert_eq!(REGION_TO_LANG.get(&'J'), Some(&"JA"));
        assert_eq!(REGION_TO_LANG.get(&'D'), Some(&"DE"));
    }
}
