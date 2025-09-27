// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::base_dir::BaseDir;
use crate::util::oscwii;
use crate::util::oscwii::AppCache;
use anyhow::{Result, anyhow, bail};
use path_slash::PathBufExt;
use serde::{Deserialize, Deserializer};
use std::fs;
use std::path::PathBuf;
use time::format_description::BorrowedFormatItem;
use time::{Date, macros::format_description};

const FORMAT: &[BorrowedFormatItem] = format_description!("[year][month][day]");

fn parse_date_only<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let str = String::deserialize(deserializer)?;
    let substr = str.chars().take(8).collect::<String>();

    if substr.len() < 8 {
        return Ok("Unknown".to_string());
    }

    if let Ok(date) = Date::parse(&substr, FORMAT) {
        Ok(date.to_string())
    } else {
        Ok("Unknown".to_string())
    }
}

#[derive(Clone, Deserialize)]
pub struct WiiAppMeta {
    pub name: String,
    pub coder: String,
    pub version: String,
    #[serde(deserialize_with = "parse_date_only")]
    pub release_date: String,
    pub short_description: String,
    pub long_description: String,
}

#[derive(Clone)]
pub struct WiiApp {
    pub path: PathBuf,
    pub size: u64,
    pub icon_uri: String,
    pub meta: WiiAppMeta,
    pub info_opened: bool,
    pub oscwii: String,
    pub oscwii_app: Option<oscwii::App>,
}

impl WiiApp {
    pub fn from_path(path: PathBuf, app_cache: &AppCache) -> Result<Self> {
        if !path.is_dir() {
            bail!("Path is not a directory");
        }

        let size = fs_extra::dir::get_size(&path)?;

        let icon_path = path.join("icon.png");
        let icon_uri = format!("file://{}", icon_path.to_slash_lossy());

        let meta_path = path.join("meta.xml");
        let meta_file = fs::read_to_string(meta_path)?;
        let meta = quick_xml::de::from_str(&meta_file)?;

        let file_name = path.file_name().ok_or(anyhow!("Failed to get file name"))?;
        let oscwii = format!(
            "https://oscwii.org/library/app/{}",
            file_name.to_string_lossy()
        );

        let oscwii_app = app_cache
            .apps
            .iter()
            .find(|app| app.slug == file_name.to_string_lossy())
            .cloned();

        Ok(Self {
            path,
            size,
            icon_uri,
            meta,
            info_opened: false,
            oscwii,
            oscwii_app,
        })
    }

    pub fn toggle_info(&mut self) {
        self.info_opened = !self.info_opened;
    }

    pub fn remove(&self) -> Result<()> {
        if rfd::MessageDialog::new()
            .set_title(format!("Remove {}", self.meta.name))
            .set_description(format!(
                "Are you sure you want to remove {}?",
                self.meta.name
            ))
            .set_buttons(rfd::MessageButtons::OkCancel)
            .show()
            == rfd::MessageDialogResult::Ok
        {
            fs::remove_dir_all(&self.path)?;
        }

        Ok(())
    }
}

pub fn get_installed(base_dir: &BaseDir, app_cache: &AppCache) -> Result<Vec<WiiApp>> {
    let apps = fs::read_dir(base_dir.apps_dir())?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter_map(|path| WiiApp::from_path(path, app_cache).ok())
        .collect();

    Ok(apps)
}
