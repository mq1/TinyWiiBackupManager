// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::PathBuf,
    sync::{Mutex, OnceLock},
};

use crate::{ArchiveFormat, WiiOutputFormat, dirs};

static CONFIG_PATH: OnceLock<PathBuf> = OnceLock::new();
static CONFIG_CACHE: OnceLock<Mutex<Config>> = OnceLock::new();

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub mount_point: PathBuf,
    pub remove_sources_games: bool,
    pub remove_sources_apps: bool,
    pub scrub_update_partition: bool,
    pub wii_output_format: WiiOutputFormat,
    pub archive_format: ArchiveFormat,
    pub wii_ip: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mount_point: PathBuf::new(),
            remove_sources_games: false,
            remove_sources_apps: false,
            scrub_update_partition: false,
            wii_output_format: WiiOutputFormat::WbfsAuto,
            archive_format: ArchiveFormat::Rvz,
            wii_ip: "192.168.1.100".to_string(),
        }
    }
}

pub fn init() -> Result<()> {
    let data_dir = dirs::data_dir()?;

    let path = data_dir.join("config.json");
    let bytes = fs::read(&path).unwrap_or_default();
    let config: Config = serde_json::from_slice(&bytes).unwrap_or_default();

    CONFIG_PATH
        .set(path)
        .map_err(|_| anyhow!("Failed to set CONFIG_PATH"))?;

    CONFIG_CACHE
        .set(Mutex::new(config))
        .map_err(|_| anyhow!("Failed to set CONFIG_CACHE"))?;

    Ok(())
}

pub fn get() -> Config {
    CONFIG_CACHE
        .get()
        .and_then(|mutex| mutex.lock().ok())
        .map(|guard| guard.clone())
        .unwrap_or_default()
}

fn set(config: Config) -> Result<()> {
    let cache = CONFIG_CACHE
        .get()
        .ok_or(anyhow!("CONFIG_CACHE not initialized"))?;
    let path = CONFIG_PATH
        .get()
        .ok_or(anyhow!("CONFIG_PATH not initialized"))?;

    // Update cache
    cache
        .lock()
        .map_err(|_| anyhow!("Mutex poisoned"))?
        .clone_from(&config);

    // Update config file
    let bytes = serde_json::to_vec(&config)?;
    fs::write(path, bytes)?;

    Ok(())
}

pub fn update(mutate: impl FnOnce(&mut Config)) -> Result<()> {
    let mut config = get();
    mutate(&mut config);
    set(config)
}
