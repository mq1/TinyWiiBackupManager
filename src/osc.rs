// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::app::App;
use crate::http;
use crate::messages::Message;
use anyhow::{Result, bail};
use path_slash::PathExt;
use serde::Deserialize;
use size::Size;
use std::{fs, path::Path, time::Duration};
use time::OffsetDateTime;

const CONTENTS_URL: &str = "https://hbb1.oscwii.org/api/v4/contents";

pub fn spawn_load_osc_apps_task(app: &App) {
    let cache_path = app.data_dir.join("osc-cache.json");
    let icons_dir = app.data_dir.join("osc-icons");

    app.task_processor.spawn(move |msg_sender| {
        msg_sender.send(Message::UpdateStatus(
            "📓 Downloading OSC Meta...".to_string(),
        ))?;

        fs::create_dir_all(&icons_dir)?;

        let cache = match load_cache(&cache_path) {
            Ok(cache) => cache,
            Err(_) => {
                let bytes = http::get(CONTENTS_URL)?;
                fs::write(&cache_path, &bytes)?;
                serde_json::from_slice(&bytes)?
            }
        };

        let apps = cache
            .into_iter()
            .filter_map(|meta| OscApp::from_meta(meta, &icons_dir))
            .collect::<Box<[_]>>();

        msg_sender.send(Message::GotOscApps(apps))?;
        msg_sender.send(Message::NotifyInfo("📓 OSC Apps loaded".to_string()))?;

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

#[derive(Debug, Clone)]
pub struct OscApp {
    pub meta: OscAppMeta,
    pub icon_uri: String,
    pub search_str: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OscAppMeta {
    pub slug: String,
    pub name: String,
    pub author: String,
    pub authors: Box<[String]>,
    pub category: String,
    pub contributors: Box<[String]>,
    pub description: Description,
    pub assets: Assets,
    pub flags: Box<[String]>,
    pub package_type: String,
    pub peripherals: Box<[String]>,
    //pub shop: ShopInformation,
    pub subdirectories: Box<[String]>,
    pub supported_platforms: Box<[String]>,
    pub uncompressed_size: Size,
    pub version: String,

    #[serde(deserialize_with = "time::serde::timestamp::deserialize")]
    pub release_date: OffsetDateTime,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Description {
    pub short: String,
    pub long: String,
}

impl OscAppMeta {
    pub fn trimmed_version(&self) -> &str {
        if self.version.len() > 10 {
            &self.version[..10]
        } else {
            &self.version
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Assets {
    pub archive: Asset,
    pub binary: Asset,
    pub icon: Asset,
    pub meta: Asset,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Asset {
    pub url: String,
    pub size: Option<Size>,
    pub hash: Option<String>,
}
