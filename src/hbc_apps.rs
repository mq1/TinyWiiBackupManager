// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{Config, HbcApp, TaskType, http::AGENT, tasks::TaskProcessor};
use anyhow::Result;
use serde::Deserialize;
use size::Size;
use slint::{Image, ToSharedString};
use std::{
    fs::{self, File},
    io::{self, BufReader, Cursor},
    path::{Path, PathBuf},
    sync::Arc,
};
use zip::{ZipArchive, result::ZipResult};

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
            name: meta.name.trim().to_shared_string(),
            name_lower: meta.name.trim().to_lowercase().to_shared_string(),
            coder: meta.coder.to_shared_string(),
            version: meta.version.to_shared_string(),
            release_date: meta.release_date.to_shared_string(),
            short_description: meta.short_description.to_shared_string(),
            long_description: meta.long_description.to_shared_string(),
            path: path.to_str().unwrap_or_default().to_shared_string(),
            image: image.unwrap_or_default(),
            size: size.to_shared_string(),
            size_mib: (size.bytes() / 1024 / 1024) as i32,
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

    apps.sort_by(|a, b| a.name_lower.cmp(&b.name_lower));

    Ok(apps)
}

/// we check if in the zip there is an "apps" directory
/// if so, we extract it to the base directory
/// otherwise, we extract the zip to the apps directory
fn extract_app(
    mount_point: &Path,
    archive: &mut ZipArchive<impl io::Read + io::Seek>,
) -> ZipResult<()> {
    if archive.file_names().any(|n| n.starts_with("apps/")) {
        archive.extract(mount_point)
    } else {
        archive.extract(mount_point.join("apps"))
    }
}

fn install_zip(mount_point: &Path, path: &Path) -> Result<()> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut archive = ZipArchive::new(reader)?;
    extract_app(mount_point, &mut archive)?;

    Ok(())
}

pub fn add_app_from_url(mount_point_str: &str, url: &str, task_processor: Arc<TaskProcessor>) {
    let mount_point = PathBuf::from(mount_point_str);
    let mount_point_str = mount_point_str.to_shared_string();
    let url = url.to_string();

    task_processor.spawn(Box::new(move |weak| {
        let status = format!("Downloading {}...", &url);
        weak.upgrade_in_event_loop(move |handle| {
            handle.set_status(status.to_shared_string());
            handle.set_task_type(TaskType::DownloadingFolder);
        })?;

        let mut response = AGENT.get(&url).call()?;

        let buffer = response
            .body_mut()
            .with_config()
            .limit(50 * 1024 * 1024) // 50MB
            .read_to_vec()?;

        let cursor = Cursor::new(buffer);
        let mut archive = ZipArchive::new(cursor)?;
        extract_app(&mount_point, &mut archive)?;

        weak.upgrade_in_event_loop(move |handle| {
            handle.invoke_refresh(mount_point_str);
        })?;

        Ok(format!("Downloaded {}", url))
    }));
}

pub fn add_apps(config: &Config, task_processor: Arc<TaskProcessor>) -> Result<()> {
    let remove_sources = config.remove_sources_apps;
    let mount_point = PathBuf::from(&config.mount_point);
    let mount_point_str = config.mount_point.to_shared_string();
    fs::create_dir_all(mount_point.join("apps"))?;

    let paths = rfd::FileDialog::new()
        .set_title("Select Wii HBC App(s)")
        .add_filter("Wii App", &["zip", "ZIP"])
        .pick_files();

    if let Some(paths) = paths {
        task_processor.spawn(Box::new(move |weak| {
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
                handle.invoke_refresh(mount_point_str);
            })?;

            Ok(format!("Installed {} app(s)", paths.len()))
        }));
    }

    Ok(())
}
