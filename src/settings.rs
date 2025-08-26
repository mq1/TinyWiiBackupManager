use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Persistent application settings
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Settings {
    /// The base directory path where games are stored
    pub base_dir_path: Option<PathBuf>,

    /// Whether to remove source files after conversion
    pub remove_sources: bool,
}

impl Settings {
    /// Load settings from eframe storage
    pub fn load(storage: &dyn eframe::Storage) -> Self {
        eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
    }

    /// Save settings to eframe storage
    pub fn save(&self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}
