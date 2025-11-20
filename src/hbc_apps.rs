// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::app::App;
use crate::messages::Message;
use crate::osc::OscApp;
use crate::{config::SortBy, http};
use anyhow::{Result, bail};
use path_slash::PathBufExt;
use serde::{Deserialize, Deserializer};
use size::Size;
use std::{
    fs::{self, File},
    io::BufReader,
    path::{Path, PathBuf},
};
use time::PrimitiveDateTime;
use time::macros::format_description;
use zip::ZipArchive;

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct HbcAppMeta {
    pub name: String,
    pub coder: String,
    pub version: String,

    pub short_description: String,
    pub long_description: String,

    #[serde(deserialize_with = "deser_date")]
    pub release_date: String,
}

fn deser_date<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    const FORMAT_DESCRIPTION: &[time::format_description::FormatItem<'_>] =
        format_description!("[year][month][day][hour][minute][second]");

    let s = String::deserialize(deserializer)?;

    let date = PrimitiveDateTime::parse(&s, &FORMAT_DESCRIPTION)
        .map(|dt| dt.date().to_string())
        .unwrap_or(s);

    Ok(date)
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

#[derive(Debug, Clone)]
pub struct HbcApp {
    pub meta: HbcAppMeta,
    pub image_uri: String,
    pub size: Size,
    pub path: PathBuf,
    pub search_str: String,
    pub osc_app_i: Option<u16>,
}

impl HbcApp {
    pub fn from_path(path: PathBuf, osc_apps: &[OscApp]) -> Result<Self> {
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
        let mut meta = quick_xml::de::from_str::<HbcAppMeta>(&meta).unwrap_or_default();

        if meta.name.is_empty() {
            bail!("No name found in {}", path.display());
        }

        meta.name = meta.name.trim().to_string();

        let size = fs_extra::dir::get_size(&path).unwrap_or_default();
        let size = Size::from_bytes(size);

        let image_path = path.join("icon.png");
        let image_uri = format!("file://{}", image_path.to_slash_lossy());

        let search_str = (meta.name.clone() + &slug).to_lowercase();

        let osc_app_i = osc_apps
            .iter()
            .position(|osc_app| osc_app.meta.name == meta.name)
            .map(|i| i as u16);

        Ok(Self {
            meta,
            path,
            size,
            search_str,
            image_uri,
            osc_app_i,
        })
    }

    pub fn get_path_str(&self) -> &str {
        self.path.to_str().unwrap_or("Invalid Path")
    }
}

pub fn list(mount_point: &Path, osc_apps: &[OscApp]) -> Vec<HbcApp> {
    let mut hbc_apps = Vec::new();

    if mount_point.as_os_str().is_empty() {
        return hbc_apps;
    }

    let apps_dir = mount_point.join("apps");

    if let Ok(entries) = fs::read_dir(&apps_dir) {
        for entry in entries.filter_map(Result::ok) {
            if let Ok(hbc_app) = HbcApp::from_path(entry.path(), osc_apps) {
                hbc_apps.push(hbc_app);
            }
        }
    }

    hbc_apps
}

fn install_zip(mount_point: &Path, path: &Path) -> Result<()> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut archive = ZipArchive::new(reader)?;
    archive.extract(mount_point)?;

    Ok(())
}

pub fn spawn_install_app_from_url_task(app: &App, zip_url: String) {
    let mount_point = app.config.contents.mount_point.clone();

    app.task_processor.spawn(move |msg_sender| {
        msg_sender.send(Message::UpdateStatus(format!(
            "📥 Downloading {}...",
            &zip_url
        )))?;

        http::download_and_extract_zip(&zip_url, &mount_point)?;

        msg_sender.send(Message::TriggerRefreshHbcApps)?;

        msg_sender.send(Message::NotifyInfo(format!("📥 {} Downloaded", &zip_url)))?;

        Ok(())
    });
}

pub fn spawn_install_apps_task(app: &App, paths: Box<[PathBuf]>) {
    let remove_sources = app.config.contents.remove_sources_apps;
    let mount_point = app.config.contents.mount_point.clone();

    app.task_processor.spawn(move |msg_sender| {
        msg_sender.send(Message::UpdateStatus("🖴 Installing apps...".to_string()))?;

        for path in &paths {
            msg_sender.send(Message::UpdateStatus(format!(
                "🖴 Installing {}...",
                path.display()
            )))?;
            install_zip(&mount_point, path)?;

            if remove_sources {
                fs::remove_file(path)?;
            }

            msg_sender.send(Message::TriggerRefreshHbcApps)?;

            msg_sender.send(Message::NotifyInfo(format!(
                "🖴 Installed {}",
                path.display()
            )))?;
        }

        Ok(())
    });
}

pub fn sort(hbc_apps: &mut [HbcApp], prev_sort_by: SortBy, sort_by: SortBy) {
    match (prev_sort_by, sort_by) {
        (SortBy::NameAscending, SortBy::NameAscending)
        | (SortBy::NameDescending, SortBy::NameDescending)
        | (SortBy::SizeAscending, SortBy::SizeAscending)
        | (SortBy::SizeDescending, SortBy::SizeDescending)
        | (_, SortBy::None) => {
            // Do nothing, already sorted
        }

        (SortBy::NameDescending, SortBy::NameAscending)
        | (SortBy::NameAscending, SortBy::NameDescending)
        | (SortBy::SizeDescending, SortBy::SizeAscending)
        | (SortBy::SizeAscending, SortBy::SizeDescending) => {
            hbc_apps.reverse();
        }

        (SortBy::SizeAscending, SortBy::NameAscending)
        | (SortBy::SizeDescending, SortBy::NameAscending)
        | (SortBy::None, SortBy::NameAscending) => {
            hbc_apps.sort_unstable_by(|a, b| a.meta.name.cmp(&b.meta.name));
        }

        (SortBy::SizeAscending, SortBy::NameDescending)
        | (SortBy::SizeDescending, SortBy::NameDescending)
        | (SortBy::None, SortBy::NameDescending) => {
            hbc_apps.sort_unstable_by(|a, b| b.meta.name.cmp(&a.meta.name));
        }

        (SortBy::NameAscending, SortBy::SizeAscending)
        | (SortBy::NameDescending, SortBy::SizeAscending)
        | (SortBy::None, SortBy::SizeAscending) => {
            hbc_apps.sort_unstable_by(|a, b| a.size.cmp(&b.size));
        }

        (SortBy::NameAscending, SortBy::SizeDescending)
        | (SortBy::NameDescending, SortBy::SizeDescending)
        | (SortBy::None, SortBy::SizeDescending) => {
            hbc_apps.sort_unstable_by(|a, b| b.size.cmp(&a.size));
        }
    }
}
