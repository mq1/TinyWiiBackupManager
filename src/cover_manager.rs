use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

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

/// Manages game cover art from GameTDB
pub struct CoverManager {
    base_dir: PathBuf,
    /// Track which covers have permanently failed (404s)
    failed: Arc<Mutex<HashSet<(String, CoverType)>>>,
}

impl CoverManager {
    /// Create a new CoverManager with the given base directory
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            base_dir,
            failed: Arc::new(Mutex::new(HashSet::new())),
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

    /// Check if a cover has permanently failed (404)
    pub fn is_failed(&self, game_id: &str, cover_type: CoverType) -> bool {
        self.failed
            .lock()
            .unwrap()
            .contains(&(game_id.to_string(), cover_type))
    }

    /// Mark covers as permanently failed (404s)
    pub fn mark_failed(&self, game_ids: Vec<String>, cover_type: CoverType) {
        let mut failed = self.failed.lock().unwrap();
        for game_id in game_ids {
            failed.insert((game_id, cover_type));
        }
    }

    /// Get the base directory
    pub fn base_dir(&self) -> &PathBuf {
        &self.base_dir
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
}
