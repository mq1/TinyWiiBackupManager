// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::USER_AGENT;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use time::UtcDateTime;

const CONTENTS_URL: &str = "https://hbb1.oscwii.org/api/v4/contents";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppCache {
    pub apps: Vec<App>,
    pub last_update: UtcDateTime,
}

impl Default for AppCache {
    fn default() -> Self {
        Self {
            apps: Vec::new(),
            last_update: UtcDateTime::MIN,
        }
    }
}

impl AppCache {
    pub fn new() -> Result<AppCache> {
        let apps = ureq::get(CONTENTS_URL)
            .header("User-Agent", USER_AGENT)
            .call()?
            .body_mut()
            .read_json()?;

        let last_update = UtcDateTime::now();

        Ok(AppCache { apps, last_update })
    }

    pub fn needs_update(&self) -> bool {
        let now = UtcDateTime::now();
        let diff = now - self.last_update;
        diff > Duration::from_secs(60 * 60 * 24)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Description {
    pub short: String,
    pub long: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Assets {
    pub icon: Asset,
    pub archive: AssetWithHash,
    pub binary: AssetWithHash,
    pub meta: MetaAsset,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Asset {
    pub url: String,
    pub size: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AssetWithHash {
    pub url: String,
    pub hash: String,
    pub size: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MetaAsset {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Shop {
    pub contents_size: u64,
    pub title_id: String,
    pub inodes: u32,
    pub title_version: u32,
    pub tmd_size: u32,
}
