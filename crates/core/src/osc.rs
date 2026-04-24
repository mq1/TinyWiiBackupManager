// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::http;
use anyhow::{Result, bail};
use serde::Deserialize;
use std::{
    fs,
    io::Cursor,
    path::Path,
    time::{Duration, SystemTime},
};
use zip::ZipArchive;

const CONTENTS_URL: &str = "https://hbb1.oscwii.org/api/v4/contents";

#[derive(Debug, Clone, Deserialize)]
pub struct OscAppMetaAsset {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OscAppMetaAssets {
    icon: OscAppMetaAsset,
    archive: OscAppMetaAsset,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OscAppMetaDescription {
    pub short: String,
    pub long: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OscAppMeta {
    pub slug: String,
    pub name: String,
    pub author: String,
    pub version: String,
    pub assets: OscAppMetaAssets,
    pub uncompressed_size: u64,
    pub release_date: i64,
    pub description: OscAppMetaDescription,
}

impl OscAppMeta {
    pub fn download_icon(&self, data_dir: &Path) -> Result<()> {
        let icon_path = data_dir.join(format!("osc-icons/{}.png", self.slug));

        if icon_path.exists() {
            bail!("Icon already exists");
        }

        let body = http::get_vec(&self.assets.icon.url)?;
        fs::write(&icon_path, &body)?;

        Ok(())
    }

    pub fn install(&self, root_dir: &Path) -> Result<()> {
        let body = http::get_vec(&self.assets.archive.url)?;
        let mut reader = Cursor::new(body);
        let mut archive = ZipArchive::new(&mut reader)?;
        archive.extract(root_dir)?;

        Ok(())
    }
}

pub fn cache_contents(data_dir: &Path, force: bool) -> Result<()> {
    let cache_path = data_dir.join("osc-cache.json");

    if !force
        && cache_path
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .is_some_and(|t| t > SystemTime::now() - Duration::from_hours(24))
    {
        return Ok(());
    }

    let body = http::get_string(CONTENTS_URL)?;
    fs::write(&cache_path, body)?;

    Ok(())
}

pub fn load_contents(data_dir: &Path) -> Result<(Vec<OscAppMeta>, i32, i32)> {
    let cached_contents_path = data_dir.join("osc-cache.json");

    let last_refresh = cached_contents_path.metadata()?.modified()?;

    let raw = fs::read_to_string(&cached_contents_path)?;
    let apps = serde_json::from_str::<Vec<OscAppMeta>>(&raw)?;

    let elapsed_mins = last_refresh.elapsed().unwrap_or_default().as_secs() / 60;
    let elapsed_hours = (elapsed_mins / 60) as i32;
    let elapsed_mins = (elapsed_mins % 60) as i32;

    Ok((apps, elapsed_hours, elapsed_mins))
}
