// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{app::App, http, tasks::BackgroundMessage};
use anyhow::{Result, bail};
use path_slash::PathExt;
use serde::{Deserialize, Deserializer};
use size::Size;
use std::{fs, path::Path, time::Duration};

const CONTENTS_URL: &str = "https://hbb1.oscwii.org/api/v4/contents";

pub fn spawn_load_osc_apps_task(app: &App) {
    let cache_path = app.data_dir.join("osc-cache.json");
    let icons_dir = app.data_dir.join("osc-icons");

    app.task_processor.spawn(move |msg_sender| {
        msg_sender.send(BackgroundMessage::UpdateStatus(
            "📓 Downloading OSC Meta...".to_string(),
        ))?;

        fs::create_dir_all(&icons_dir)?;

        let cache = match load_cache(&cache_path) {
            Ok(cache) => cache,
            Err(_) => {
                let bytes = http::get(CONTENTS_URL, None)?;
                fs::write(&cache_path, &bytes)?;
                serde_json::from_slice(&bytes)?
            }
        };

        let apps = cache
            .into_iter()
            .filter_map(|meta| OscApp::from_meta(meta, &icons_dir))
            .collect::<Vec<_>>();

        msg_sender.send(BackgroundMessage::GotOscApps(apps))?;
        msg_sender.send(BackgroundMessage::NotifyInfo(
            "📓 OSC Apps loaded".to_string(),
        ))?;

        Ok(())
    });
}

fn load_cache(path: &Path) -> Result<Vec<OscAppMeta>> {
    // get file time
    let file_time = fs::metadata(path)?.modified()?;

    // get difference
    let elapsed = file_time.elapsed()?;

    if elapsed > Duration::from_secs(60 * 60 * 24) {
        bail!("osc-cache.json is too old");
    }

    let bytes = fs::read(path)?;
    let apps = serde_json::from_slice(&bytes)?;

    Ok(apps)
}

pub fn download_icon(meta: &OscAppMeta, icons_dir: &Path) -> Result<()> {
    let icon_path = icons_dir.join(&meta.slug).with_extension("png");

    if icon_path.exists() {
        bail!("{} already exists", icon_path.display());
    }

    http::download_file(&meta.assets.icon.url, &icon_path)?;

    Ok(())
}

impl OscApp {
    fn from_meta(meta: OscAppMeta, icons_dir: &Path) -> Option<Self> {
        let icon_path = icons_dir.join(&meta.slug).with_extension("png");
        let icon_uri = format!("file://{}", icon_path.to_slash()?);

        let search_str = (meta.name.clone() + &meta.slug).to_lowercase();

        Some(Self {
            meta,
            icon_uri,
            search_str,
        })
    }
}

#[derive(Clone)]
pub struct OscApp {
    pub meta: OscAppMeta,
    pub icon_uri: String,
    pub search_str: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct OscAppMeta {
    pub slug: String,
    pub name: String,
    pub author: String,
    pub assets: Assets,
    pub release_date: usize,
    #[serde(deserialize_with = "deser_size")]
    pub uncompressed_size: Size,
    pub version: String,
}

impl OscAppMeta {
    pub fn version_display(&self) -> String {
        if self.version.len() > 10 {
            format!("📌 {}...", &self.version[..10])
        } else {
            format!("📌 {}", &self.version)
        }
    }
}

fn deser_size<'de, D>(deserializer: D) -> Result<Size, D::Error>
where
    D: Deserializer<'de>,
{
    let size = usize::deserialize(deserializer)?;
    Ok(Size::from_bytes(size))
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct Assets {
    pub icon: Asset,
    pub archive: Asset,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct Asset {
    pub url: String,
}
