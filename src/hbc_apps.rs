// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::HbcApp;
use anyhow::Result;
use serde::Deserialize;
use size::Size;
use slint::{Image, ToSharedString};
use std::{
    fs,
    path::{Path, PathBuf},
};

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

impl HbcApp {
    pub fn from_path(path: PathBuf) -> Self {
        let slug = path
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default();

        let meta_path = path.join("meta").with_extension("xml");
        let meta = fs::read_to_string(&meta_path).unwrap_or_default();
        let meta = quick_xml::de::from_str::<HbcAppMeta>(&meta).unwrap_or_default();

        let size = fs_extra::dir::get_size(&path).unwrap_or_default();
        let size = Size::from_bytes(size);

        let image_path = path.join("icon.png");
        let image = Image::load_from_path(&image_path);

        Self {
            slug: slug.to_shared_string(),
            name: meta.name.to_shared_string(),
            coder: meta.coder.to_shared_string(),
            version: meta.version.to_shared_string(),
            release_date: meta.release_date.to_shared_string(),
            short_description: meta.short_description.to_shared_string(),
            long_description: meta.long_description.to_shared_string(),
            path: path.to_str().unwrap_or_default().to_shared_string(),
            image: image.unwrap_or_default(),
            size: size.to_shared_string(),
        }
    }
}

pub fn list(mount_point: &Path) -> Result<Vec<HbcApp>> {
    if mount_point.as_os_str().is_empty() {
        return Ok(vec![]);
    }

    let apps_dir = mount_point.join("apps");
    fs::create_dir_all(&apps_dir)?;

    let apps = fs::read_dir(&apps_dir)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .map(HbcApp::from_path)
        .collect::<Vec<_>>();

    Ok(apps)
}
