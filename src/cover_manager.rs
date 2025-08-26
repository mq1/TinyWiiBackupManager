// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::messages::BackgroundMessage;
use anyhow::{Context, Result};
use egui_inbox::UiInboxSender;
use phf::phf_map;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;

// A static map to convert the region character from a game's ID to a language code
// used by the GameTDB API for fetching cover art.
static REGION_TO_LANG: phf::Map<char, &'static str> = phf_map! {
    'A' => "EN", // System Wii Channels (i.e. Mii Channel)
    'B' => "EN", // Ufouria: The Saga (NA)
    'D' => "DE", // Germany
    'E' => "US", // USA
    'F' => "FR", // France
    'H' => "NL", // Netherlands
    'I' => "IT", // Italy
    'J' => "JA", // Japan
    'K' => "KO", // Korea
    'L' => "EN", // Japanese import to Europe, Australia and other PAL regions
    'M' => "EN", // American import to Europe, Australia and other PAL regions
    'N' => "US", // Japanese import to USA and other NTSC regions
    'P' => "EN", // Europe and other PAL regions such as Australia
    'Q' => "KO", // Japanese Virtual Console import to Korea
    'R' => "RU", // Russia
    'S' => "ES", // Spain
    'T' => "KO", // American Virtual Console import to Korea
    'U' => "EN", // Australia / Europe alternate languages
    'V' => "EN", // Scandinavia
    'W' => "ZH", // Republic of China (Taiwan) / Hong Kong / Macau
    'X' => "EN", // Europe alternate languages / US special releases
    'Y' => "EN", // Europe alternate languages / US special releases
    'Z' => "EN", // Europe alternate languages / US special releases
};

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
    pub fn queue_download(
        &self,
        game_id: String,
        cover_type: CoverType,
        sender: UiInboxSender<BackgroundMessage>,
    ) {
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

            // Send result to UI
            match result {
                Ok(path) => {
                    let _ = sender.send(BackgroundMessage::CoverDownloaded {
                        game_id,
                        cover_type,
                        path,
                    });
                }
                Err(e) => {
                    log::debug!("Failed to download cover for {}: {}", game_id, e);
                    let _ = sender.send(BackgroundMessage::CoverDownloadFailed {
                        game_id,
                        cover_type,
                        error: e.to_string(),
                    });
                }
            }
        });
    }

    /// Download a cover from GameTDB API (blocking operation)
    fn download_cover(
        base_dir: &Path,
        game_id: &str,
        cover_type: CoverType,
    ) -> Result<PathBuf> {
        // Determine language from game ID region
        let region_char = game_id.chars().nth(3)
            .context("Game ID too short to determine region")?;
        let language = REGION_TO_LANG.get(&region_char).unwrap_or(&"EN");

        // Construct GameTDB API URL
        let url = format!(
            "https://art.gametdb.com/wii/{}/{}/{}.png",
            cover_type.api_endpoint(),
            language,
            game_id
        );

        // Create target directory structure
        let target_path = base_dir
            .join("apps/usbloader_gx")
            .join(cover_type.subdirectory())
            .join(format!("{}.png", game_id));
        
        if let Some(parent) = target_path.parent() {
            std::fs::create_dir_all(parent)
                .context("Failed to create cover directory")?;
        }

        // Download the cover
        log::debug!("Downloading cover from: {}", url);
        let response = reqwest::blocking::get(&url)
            .context("Failed to send HTTP request")?;

        if response.status().is_success() {
            let bytes = response.bytes()
                .context("Failed to read response body")?;
            
            std::fs::write(&target_path, bytes)
                .context("Failed to write cover file")?;
            
            log::info!("Downloaded cover: {:?}", target_path);
            Ok(target_path)
        } else {
            anyhow::bail!("HTTP error: {}", response.status());
        }
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
            temp_dir.path().join("apps/usbloader_gx/images/2D/RMGE01.png")
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