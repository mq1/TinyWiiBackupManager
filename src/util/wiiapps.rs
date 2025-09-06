// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::base_dir::BaseDir;
use anyhow::{Result, anyhow, bail};
use std::fs;
use std::path::PathBuf;

pub struct WiiApp {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
}

impl WiiApp {
    pub fn from_path(path: PathBuf) -> Result<Self> {
        if !path.is_dir() {
            bail!("Path is not a directory");
        }

        let name = path
            .file_name()
            .ok_or(anyhow!("Invalid path"))?
            .to_string_lossy()
            .to_string();

        let size = fs_extra::dir::get_size(&path)?;

        Ok(Self { name, path, size })
    }
}

pub fn get_installed(base_dir: &BaseDir) -> Result<Vec<WiiApp>> {
    let apps = fs::read_dir(base_dir.apps_dir())?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter_map(|path| WiiApp::from_path(path).ok())
        .collect();

    Ok(apps)
}
