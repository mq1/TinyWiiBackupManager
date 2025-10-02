// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::{Result, anyhow};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::PathBuf,
    sync::{LazyLock, Mutex},
};

static CONFIG_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    let proj = ProjectDirs::from("it", "mq1", env!("CARGO_PKG_NAME"))
        .expect("Failed to get project directory");

    let data_dir = proj.data_dir();
    let _ = fs::create_dir_all(data_dir);

    data_dir.join("config.json")
});

static CONFIG_CACHE: LazyLock<Mutex<Config>> = LazyLock::new(|| {
    let bytes = fs::read(&*CONFIG_PATH).unwrap_or_default();
    let data = serde_json::from_slice(&bytes).unwrap_or_default();

    Mutex::new(data)
});

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub mount_point: PathBuf,
    pub remove_sources_games: bool,
    pub remove_sources_apps: bool,
}

pub fn get() -> Config {
    match CONFIG_CACHE.lock() {
        Ok(config) => config.clone(),
        Err(_) => Config::default(),
    }
}

fn set(config: Config) -> Result<()> {
    // Update cache
    CONFIG_CACHE
        .lock()
        .map_err(|_| anyhow!("Mutex poisoned"))?
        .clone_from(&config);

    // Update config file
    let bytes = serde_json::to_vec(&config)?;
    fs::write(&*CONFIG_PATH, bytes)?;

    Ok(())
}

pub fn update(mutate: impl Fn(&mut Config)) -> Result<()> {
    let mut config = get();
    mutate(&mut config);
    set(config)
}
