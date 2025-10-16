// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{Config, HbcApp, MainWindow, TaskType, http::AGENT};
use anyhow::{Result, bail};
use serde::Deserialize;
use size::Size;
use slint::{Image, ToSharedString, Weak};
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
        let image = if image_path.exists()
            && let Ok(image) = Image::load_from_path(&image_path)
        {
            image
        } else {
            Image::load_from_svg_data(include_bytes!("../mdi/image-frame.svg"))
                .expect("Failed to load default icon")
        };

        let search_str = (meta.name.clone() + slug).to_lowercase().to_shared_string();

        Self {
            slug: slug.to_shared_string(),
            name: meta.name.trim().to_shared_string(),
            coder: meta.coder.to_shared_string(),
            version: meta.version.to_shared_string(),
            release_date: meta.release_date.to_shared_string(),
            short_description: meta.short_description.to_shared_string(),
            long_description: meta.long_description.to_shared_string(),
            path: path.to_str().unwrap_or_default().to_shared_string(),
            image,
            size: size.to_shared_string(),
            size_mib: (size.bytes() / 1024 / 1024) as i32,
            search_str,
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

pub fn add_app_from_url(
    mount_point_str: &str,
    zip_url: &str,
    zip_size: usize,
    weak: &Weak<MainWindow>,
) -> Result<()> {
    let mount_point = PathBuf::from(mount_point_str);
    let url = zip_url.to_string();

    let status = format!("Downloading {}...", &url);
    weak.upgrade_in_event_loop(move |handle| {
        handle.set_status(status.to_shared_string());
        handle.set_task_type(TaskType::DownloadingFolder);
    })?;

    let (_, body) = AGENT.get(&url).call()?.into_parts();
    let mut buffer = Vec::with_capacity(zip_size);
    body.into_reader().read_to_end(&mut buffer)?;

    let cursor = Cursor::new(buffer);
    let mut archive = ZipArchive::new(cursor)?;
    archive.extract(mount_point)?;

    weak.upgrade_in_event_loop(move |handle| {
        handle.invoke_refresh_hbc_apps();
    })?;

    Ok(())
}

pub fn add_apps(config: &Config, weak: &Weak<MainWindow>) -> Result<()> {
    let remove_sources = config.remove_sources_apps;
    let mount_point = PathBuf::from(&config.mount_point);
    fs::create_dir_all(mount_point.join("apps"))?;

    let paths = rfd::FileDialog::new()
        .set_title("Select Wii HBC App(s)")
        .add_filter("Wii App", &["zip", "ZIP"])
        .pick_files();

    if let Some(paths) = paths {
        for path in paths.iter() {
            {
                let status = format!("Installing {}...", path.display());
                weak.upgrade_in_event_loop(move |handle| {
                    handle.set_status(status.to_shared_string());
                    handle.set_task_type(TaskType::InstallingApps);
                })?;

                install_zip(&mount_point, path)?;
            }

            if remove_sources {
                fs::remove_file(path)?;
            }
        }

        weak.upgrade_in_event_loop(move |handle| {
            handle.invoke_refresh_hbc_apps();
        })?;

        Ok(())
    } else {
        bail!("No files selected");
    }
}
