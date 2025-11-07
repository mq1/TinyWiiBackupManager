// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    app::App,
    config::SortBy,
    http,
    tasks::{BackgroundMessage, TaskProcessor},
};
use anyhow::{Result, bail};
use path_slash::PathBufExt;
use serde::Deserialize;
use size::Size;
use std::{
    fs::{self, File},
    io::BufReader,
    path::{Path, PathBuf},
};
use zip::ZipArchive;

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct HbcAppMeta {
    pub name: String,
    pub coder: String,
    pub version: String,
    pub release_date: String,
    pub short_description: String,
    pub long_description: String,
}

impl HbcAppMeta {
    pub fn version_display(&self) -> String {
        if self.version.len() > 10 {
            format!("📌 {}...", &self.version[..10])
        } else {
            format!("📌 {}", &self.version)
        }
    }
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
    pub meta: HbcAppMeta,
    pub image_uri: String,
    pub size: Size,
    pub path: PathBuf,
    pub search_str: String,
}

impl HbcApp {
    pub fn from_path(path: PathBuf) -> Result<Self> {
        if !path.is_dir() {
            bail!("{} is not a directory", path.display());
        }

        if let Some(file_name) = path.file_name()
            && let Some(file_name) = file_name.to_str()
            && file_name.starts_with('.')
        {
            bail!("Skipping hidden directory {}", path.display());
        }

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

        Ok(Self {
            meta,
            path,
            size,
            search_str,
            image_uri,
        })
    }
}

pub fn list(mount_point: &Path) -> Box<[HbcApp]> {
    if mount_point.as_os_str().is_empty() {
        return Box::new([]);
    }

    let apps_dir = mount_point.join("apps");

    match fs::read_dir(&apps_dir) {
        Ok(entries) => {
            let mut apps = entries
                .filter_map(|entry| HbcApp::from_path(entry.ok()?.path()).ok())
                .collect::<Vec<_>>();
            apps.sort_by(|a, b| a.meta.name.cmp(&b.meta.name));
            apps.into_boxed_slice()
        }
        Err(_) => Box::new([]),
    }
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
    task_processor: &TaskProcessor,
    mount_point: PathBuf,
) {
    task_processor.spawn(move |msg_sender| {
        msg_sender.send(BackgroundMessage::UpdateStatus(format!(
            "📥 Downloading {}...",
            &zip_url
        )))?;

        http::download_and_extract_zip(&zip_url, &mount_point)?;

        msg_sender.send(BackgroundMessage::TriggerRefreshHbcApps)?;

        msg_sender.send(BackgroundMessage::NotifyInfo(format!(
            "📥 {} Downloaded",
            &zip_url
        )))?;

        Ok(())
    });
}

pub fn spawn_install_apps_task(app: &App, paths: Vec<PathBuf>) {
    let remove_sources = app.config.contents.remove_sources_apps;
    let mount_point = app.config.contents.mount_point.clone();

    app.task_processor.spawn(move |msg_sender| {
        msg_sender.send(BackgroundMessage::UpdateStatus(
            "🖴 Installing apps...".to_string(),
        ))?;

        for path in &paths {
            msg_sender.send(BackgroundMessage::UpdateStatus(format!(
                "🖴 Installing {}...",
                path.display()
            )))?;
            install_zip(&mount_point, path)?;

            if remove_sources {
                fs::remove_file(path)?;
            }

            msg_sender.send(BackgroundMessage::TriggerRefreshHbcApps)?;

            msg_sender.send(BackgroundMessage::NotifyInfo(format!(
                "🖴 Installed {}",
                path.display()
            )))?;
        }

        Ok(())
    });
}

pub fn sort(hbc_apps: &mut [HbcApp], sort_by: &SortBy) {
    match sort_by {
        SortBy::NameAscending => {
            hbc_apps.sort_by(|a, b| a.meta.name.cmp(&b.meta.name));
        }
        SortBy::NameDescending => {
            hbc_apps.sort_by(|a, b| b.meta.name.cmp(&a.meta.name));
        }
        SortBy::SizeAscending => {
            hbc_apps.sort_by(|a, b| a.size.cmp(&b.size));
        }
        SortBy::SizeDescending => {
            hbc_apps.sort_by(|a, b| b.size.cmp(&a.size));
        }
    }
}
