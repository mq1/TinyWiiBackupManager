// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::base_dir::BaseDir;
use anyhow::{Result, bail};
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Deserialize)]
pub struct WiiAppMeta {
    pub name: String,
    pub coder: String,
    pub version: String,
    pub release_date: String,
    pub short_description: String,
    pub long_description: String,
}

#[derive(Clone)]
pub struct WiiApp {
    pub path: PathBuf,
    pub size: u64,
    pub icon_uri: String,
    pub meta: WiiAppMeta,
    pub info_opened: bool,
}

impl WiiApp {
    pub fn from_path(path: PathBuf) -> Result<Self> {
        if !path.is_dir() {
            bail!("Path is not a directory");
        }

        let size = fs_extra::dir::get_size(&path)?;

        let icon_uri = format!("file://{}", path.join("icon.png").display());

        let meta_path = path.join("meta.xml");
        let meta_file = fs::read_to_string(meta_path)?;
        let meta = quick_xml::de::from_str(&meta_file)?;

        Ok(Self {
            path,
            size,
            icon_uri,
            meta,
            info_opened: false,
        })
    }

    pub fn toggle_info(&mut self) {
        self.info_opened = !self.info_opened;
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
