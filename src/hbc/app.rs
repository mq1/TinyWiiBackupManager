// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::message::Message;
use crate::state::State;
use anyhow::Result;
use derive_getters::Getters;
use iced::Task;
use iced::futures::TryFutureExt;
use serde::{Deserialize, Deserializer};
use size::Size;
use std::ffi::OsString;
use std::fs::{self, File};
use std::path::PathBuf;
use std::sync::Arc;
use time::PrimitiveDateTime;
use time::macros::format_description;
use zip::ZipArchive;

#[derive(Debug, Clone, Deserialize, Default, Getters)]
#[serde(default)]
pub struct HbcAppMeta {
    name: String,
    coder: String,
    version: String,
    short_description: String,
    long_description: String,

    #[serde(deserialize_with = "deser_date")]
    release_date: String,
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

#[derive(Debug, Clone, Getters)]
pub struct HbcApp {
    meta: HbcAppMeta,
    #[getter(copy)]
    size: Size,
    path: PathBuf,
    image_path: Option<PathBuf>,
}

impl PartialEq for HbcApp {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl Eq for HbcApp {}

impl HbcApp {
    pub fn maybe_from_path(path: PathBuf) -> Option<Self> {
        if !path.is_dir() {
            return None;
        }

        let slug = path.file_name()?.to_str()?;

        if slug.starts_with('.') {
            return None;
        }

        let meta_path = path.join("meta").with_extension("xml");
        let meta = fs::read_to_string(&meta_path).unwrap_or_default();
        let mut meta = quick_xml::de::from_str::<HbcAppMeta>(&meta).unwrap_or_default();

        if meta.name.is_empty() {
            return None;
        }

        meta.name = meta.name.trim().to_string();

        let size = fs_extra::dir::get_size(&path).unwrap_or(0);

        let image_path = path.join("icon.png");
        let image_path = if image_path.exists() {
            Some(image_path)
        } else {
            None
        };

        Some(Self {
            meta,
            size: Size::from_bytes(size),
            path,
            image_path,
        })
    }

    pub fn get_path_uri(&self) -> OsString {
        self.path.as_os_str().to_os_string()
    }

    pub fn get_trimmed_version_str(&self) -> &str {
        let len = self.meta.version.len().min(8);
        &self.meta.version[..len]
    }

    pub fn get_oscwii_uri(&self) -> OsString {
        match self.path.file_name() {
            Some(file_name) => {
                format!("https://oscwii.org/library/app/{}", file_name.display()).into()
            }
            None => "https://oscwii.org/404".into(),
        }
    }
}

pub fn get_install_hbc_apps_task(state: &State, zip_paths: Box<[PathBuf]>) -> Task<Message> {
    let drive_path = state.config.mount_point().clone();

    Task::perform(
        async { install_hbc_apps(zip_paths, drive_path) }.map_err(Arc::new),
        Message::HbcAppsInstalled,
    )
}

fn install_hbc_apps(zip_paths: Box<[PathBuf]>, dest_dir: PathBuf) -> Result<String> {
    for zip_path in &zip_paths {
        let zip_file = File::open(zip_path)?;
        let mut archive = ZipArchive::new(zip_file)?;
        archive.extract(&dest_dir)?;
    }

    let msg = format!("Installed {} apps", zip_paths.len());
    Ok(msg)
}
