// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::http::AGENT;
use anyhow::{Result, bail};
use serde::Deserialize;
use size::Size;
use std::{fs, io::Read, path::Path, rc::Rc, time::Duration};

const CONTENTS_URL: &str = "https://hbb1.oscwii.org/api/v4/contents";

pub fn load_osc_apps(data_dir: &Path, weak: &Weak<MainWindow>) -> Result<()> {
    let cache_path = data_dir.join("osc-cache.json");
    let icons_dir = data_dir.join("osc-icons");

    weak.upgrade_in_event_loop(|handle| {
        handle.set_osc_load_status("Loading OSC Apps...".to_shared_string());
    })?;

    let cache = match load_cache(&cache_path) {
        Ok(cache) => cache,
        Err(_) => {
            let bytes = AGENT.get(CONTENTS_URL).call()?.body_mut().read_to_vec()?;
            fs::write(&cache_path, &bytes)?;
            serde_json::from_slice(&bytes)?
        }
    };

    fs::create_dir_all(&icons_dir)?;
    let len = cache.len();
    for (i, app) in cache.iter().enumerate() {
        let status = format!("Downloading OSC App icons... {}/{}", i + 1, len).to_shared_string();

        weak.upgrade_in_event_loop(move |handle| {
            handle.set_osc_load_status(status);
        })?;

        let _ = download_icon(app, &icons_dir);
    }

    weak.upgrade_in_event_loop(move |handle| {
        let apps = cache
            .iter()
            .map(|app| OscApp::from_app(app, &icons_dir))
            .collect::<VecModel<_>>();

        let model = ModelRc::from(Rc::new(apps));
        handle.set_osc_apps(model);
        handle.set_osc_load_status(SharedString::new());
    })?;

    Ok(())
}

fn load_cache(path: &Path) -> Result<Vec<App>> {
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

fn download_icon(app: &App, icons_dir: &Path) -> Result<()> {
    let icon_path = icons_dir.join(&app.slug).with_extension("png");

    if icon_path.exists() {
        return Ok(());
    }

    let (_, body) = AGENT.get(&app.assets.icon.url).call()?.into_parts();
    let mut icon = Vec::with_capacity(app.assets.icon.size as usize);
    body.into_reader().read_to_end(&mut icon)?;

    fs::write(&icon_path, &icon)?;

    Ok(())
}

impl OscApp {
    fn from_app(app: &App, icons_dir: &Path) -> Self {
        let size = Size::from_bytes(app.uncompressed_size);

        let icon_path = icons_dir.join(&app.slug).with_extension("png");
        let icon = if icon_path.exists()
            && let Ok(icon) = Image::load_from_path(&icon_path)
        {
            icon
        } else {
            Image::load_from_svg_data(include_bytes!("../mdi/image-frame.svg"))
                .expect("Failed to load default icon")
        };

        let search_str = (app.name.clone() + &app.slug)
            .to_lowercase()
            .to_shared_string();

        Self {
            slug: app.slug.to_shared_string(),
            name: app.name.to_shared_string(),
            author: app.author.to_shared_string(),
            version: app.version.to_shared_string(),
            release_date: app.release_date.to_shared_string(),
            size: size.to_shared_string(),
            zip_url: app.assets.archive.url.to_shared_string(),
            zip_size: app.assets.archive.size as i32,
            icon,
            search_str,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct App {
    pub slug: String,
    pub name: String,
    pub author: String,
    pub assets: Assets,
    pub release_date: u64,
    pub uncompressed_size: u64,
    pub version: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Assets {
    pub icon: Asset,
    pub archive: Asset,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Asset {
    pub url: String,
    pub size: u64,
}
