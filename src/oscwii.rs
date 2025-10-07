// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::http::AGENT;
use anyhow::{Result, bail};
use serde::Deserialize;
use std::{fs, path::Path, time::Duration};

const CONTENTS_URL: &str = "https://hbb1.oscwii.org/api/v4/contents";

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Apps(pub Vec<App>);

impl Apps {
    pub fn load(data_dir: &Path) -> Result<Self> {
        let path = data_dir.join("oscwii.json");

        if let Ok(apps) = Self::load_cache(&path) {
            Ok(apps)
        } else {
            let bytes = AGENT.get(CONTENTS_URL).call()?.body_mut().read_to_vec()?;
            fs::write(&path, &bytes)?;
            let apps = serde_json::from_slice(&bytes)?;
            Ok(apps)
        }
    }

    fn load_cache(path: &Path) -> Result<Self> {
        // get file time
        let file_time = fs::metadata(&path)?.modified()?;

        // get difference
        let elapsed = file_time.elapsed()?;

        if elapsed > Duration::from_secs(60 * 60 * 24) {
            bail!("oscwii.json is too old");
        }

        let bytes = fs::read(&path)?;
        let apps = serde_json::from_slice(&bytes)?;

        Ok(apps)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct App {
    pub slug: String,
    pub name: String,
    pub author: String,
    pub authors: Vec<String>,
    pub category: String,
    pub contributors: Vec<String>,
    pub description: Description,
    pub assets: Assets,
    pub flags: Vec<String>,
    pub package_type: String,
    pub peripherals: Vec<String>,
    pub release_date: u64,
    pub shop: Shop,
    pub subdirectories: Vec<String>,
    pub supported_platforms: Vec<String>,
    pub uncompressed_size: u64,
    pub version: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Description {
    pub short: String,
    pub long: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Assets {
    pub icon: Asset,
    pub archive: AssetWithHash,
    pub binary: AssetWithHash,
    pub meta: MetaAsset,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Asset {
    pub url: String,
    pub size: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AssetWithHash {
    pub url: String,
    pub hash: String,
    pub size: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MetaAsset {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Shop {
    pub contents_size: u64,
    pub title_id: String,
    pub inodes: u32,
    pub title_version: u32,
    pub tmd_size: u32,
}
