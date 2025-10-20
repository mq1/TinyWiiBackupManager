// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{app::App, http::AGENT};
use anyhow::Result;
use path_slash::PathBufExt;
use serde::Deserialize;
use size::Size;
use std::{
    fs::{self, File},
    io::{BufReader, Cursor, Read},
    path::{Path, PathBuf},
};
use zip::ZipArchive;

#[derive(Debug, Clone, Deserialize)]
pub struct HbcAppMeta {
    pub name: String,
    pub coder: String,
    pub version: String,
    pub release_date: String,
    pub short_description: String,
    pub long_description: String,
}

impl Default for HbcAppMeta {
    fn default() -> Self {
        Self {
            name: "Unknown Name".to_string(),
            coder: "Unknown Coder".to_string(),
            version: "Unknown Version".to_string(),
            release_date: "Unknown Release Date".to_string(),
            short_description: "Unknown Short Description".to_string(),
            long_description: "Unknown Long Description".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct HbcApp {
    pub slug: String,
    pub name: String,
    pub coder: String,
    pub version: String,
    pub release_date: String,
    pub short_description: String,
    pub long_description: String,
    pub image_uri: String,
    pub size: Size,
    pub path: PathBuf,
    pub search_str: String,
}

impl HbcApp {
    pub fn from_path(path: PathBuf) -> Self {
        let slug = path
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default()
            .to_string();

        let meta_path = path.join("meta").with_extension("xml");
        let meta = fs::read_to_string(&meta_path).unwrap_or_default();
        let meta = quick_xml::de::from_str::<HbcAppMeta>(&meta).unwrap_or_default();

        let size = fs_extra::dir::get_size(&path).unwrap_or_default();
        let size = Size::from_bytes(size);

        let image_path = path.join("icon.png");
        let image_uri = format!("file://{}", image_path.to_slash_lossy());

        let search_str = (meta.name.clone() + &slug).to_lowercase();

        Self {
            slug,
            name: meta.name.trim().to_string(),
            coder: meta.coder,
            version: meta.version,
            release_date: meta.release_date,
            short_description: meta.short_description,
            long_description: meta.long_description,
            path,
            size,
            search_str,
            image_uri,
        }
    }
}

pub fn list(mount_point: &Path) -> Result<Vec<HbcApp>> {
    if mount_point.as_os_str().is_empty() {
        return Ok(vec![]);
    }

    let apps_dir = mount_point.join("apps");
    fs::create_dir_all(&apps_dir)?;

    let mut apps = fs::read_dir(&apps_dir)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .map(HbcApp::from_path)
        .collect::<Vec<_>>();

    apps.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(apps)
}

fn install_zip(mount_point: &Path, path: &Path) -> Result<()> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut archive = ZipArchive::new(reader)?;
    archive.extract(mount_point)?;

    Ok(())
}

pub fn spawn_install_app_from_url_task(
    zip_url: String,
    zip_size: usize,
    app: &mut App,
) -> Result<()> {
    let mount_point = app.config.contents.mount_point.clone();

    app.task_processor.spawn(move |status, toasts| {
        *status.lock() = format!("📥 Downloading {}...", &zip_url);

        let (_, body) = AGENT.get(&zip_url).call()?.into_parts();
        let mut buffer = Vec::with_capacity(zip_size);
        body.into_reader().read_to_end(&mut buffer)?;

        let cursor = Cursor::new(buffer);
        let mut archive = ZipArchive::new(cursor)?;
        archive.extract(mount_point)?;

        toasts.lock().info("📥 App installed".to_string());

        Ok(())
    });

    Ok(())
}

pub fn spawn_install_apps_task(app: &mut App, paths: Vec<PathBuf>) {
    let remove_sources = app.config.contents.remove_sources_apps;
    let mount_point = app.config.contents.mount_point.clone();

    app.task_processor.spawn(move |status, toasts| {
        *status.lock() = "🖴 Installing apps...".to_string();

        for path in &paths {
            *status.lock() = format!("🖴 Installing {}...", path.display());
            install_zip(&mount_point, path)?;

            if remove_sources {
                fs::remove_file(path)?;
            }

            toasts
                .lock()
                .info(format!("🖴 Installed {}", path.display()));
        }

        Ok(())
    });
}
