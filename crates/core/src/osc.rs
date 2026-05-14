// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::util::AGENT;
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

#[derive(Debug, Clone)]
pub struct OscApp {
    pub meta: OscAppMeta,
    pub search_term: String,
}

impl OscApp {
    pub fn download_icon(&self, data_dir: &Path) -> Result<()> {
        let icon_path = data_dir.join(format!("osc-icons/{}.png", self.meta.slug));

        if icon_path.exists() {
            bail!("Icon already exists");
        }

        let body = AGENT
            .get(&self.meta.assets.icon.url)
            .call()?
            .body_mut()
            .read_to_vec()?;

        fs::write(&icon_path, &body)?;

        Ok(())
    }

    pub fn install(&self, root_dir: &Path) -> Result<()> {
        let body = AGENT
            .get(&self.meta.assets.archive.url)
            .call()?
            .body_mut()
            .with_config()
            .limit(100 * 1024 * 1024)
            .read_to_vec()?;

        let mut reader = Cursor::new(body);
        let mut archive = ZipArchive::new(&mut reader)?;
        archive.extract(root_dir)?;

        Ok(())
    }

    pub fn wiiload(&self, wii_ip: &str) -> Result<String> {
        crate::wiiload::download_then_send(wii_ip, &self.meta.assets.archive.url)
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

    let body = AGENT
        .get(CONTENTS_URL)
        .call()?
        .body_mut()
        .read_to_string()?;

    fs::write(&cache_path, body)?;

    Ok(())
}

pub fn load_contents(data_dir: &Path) -> Result<(Vec<OscApp>, i32, i32)> {
    let cached_contents_path = data_dir.join("osc-cache.json");

    let last_refresh = cached_contents_path.metadata()?.modified()?;

    let raw = fs::read_to_string(&cached_contents_path)?;
    let apps = serde_json::from_str::<Vec<OscAppMeta>>(&raw)?;
    let apps = apps
        .into_iter()
        .map(|meta| {
            let search_term = format!("{}\0{}", &meta.name, &meta.author);
            OscApp { meta, search_term }
        })
        .collect::<Vec<_>>();

    let elapsed_mins = last_refresh.elapsed().unwrap_or_default().as_secs() / 60;
    let elapsed_hours = (elapsed_mins / 60) as i32;
    let elapsed_mins = (elapsed_mins % 60) as i32;

    Ok((apps, elapsed_hours, elapsed_mins))
}
