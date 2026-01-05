// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::config::SortBy;
use crate::message::Message;
use crate::state::State;
use anyhow::Result;
use iced::Task;
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

#[derive(Debug, Clone)]
pub struct HbcApp {
    pub meta: HbcAppMeta,
    pub size: Size,
    pub size_str: String,
    pub path: PathBuf,
    pub search_str: String,
    pub image_path: PathBuf,
}

impl HbcApp {
    pub fn from_path(path: PathBuf) -> Option<Self> {
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

        let size = Size::from_bytes(fs_extra::dir::get_size(&path).unwrap_or_default());
        let size_str = size.to_string();

        let image_path = path.join("icon.png");

        let search_str = format!("{}{}", &meta.name, &slug).to_lowercase();

        Some(Self {
            meta,
            path,
            size,
            size_str,
            search_str,
            image_path,
        })
    }

    pub fn get_path_str(&self) -> &str {
        self.path.to_str().unwrap_or("Invalid Path")
    }
}

pub fn get_list_hbc_apps_task(state: &State) -> Task<Message> {
    let mount_point = state.config.get_drive_path().to_path_buf();

    Task::perform(
        async move { list(mount_point).map_err(|e| e.to_string()) },
        Message::GotHbcApps,
    )
}

fn list(mount_point: PathBuf) -> Result<Box<[HbcApp]>> {
    let apps_dir = mount_point.join("apps");

    let entries = fs::read_dir(&apps_dir)?;

    let hbc_apps = entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter_map(HbcApp::from_path)
        .collect();

    Ok(hbc_apps)
}

fn install_zip(mount_point: &Path, path: &Path) -> Result<()> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut archive = ZipArchive::new(reader)?;
    archive.extract(mount_point)?;

    Ok(())
}

pub trait HbcApps {
    fn sort(&mut self, prev_sort_by: SortBy, sort_by: SortBy);
}

impl HbcApps for Box<[HbcApp]> {
    fn sort(&mut self, prev_sort_by: SortBy, sort_by: SortBy) {
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
                self.reverse();
            }

            (SortBy::SizeAscending, SortBy::NameAscending)
            | (SortBy::SizeDescending, SortBy::NameAscending)
            | (SortBy::None, SortBy::NameAscending) => {
                self.sort_unstable_by(|a, b| a.meta.name.cmp(&b.meta.name));
            }

            (SortBy::SizeAscending, SortBy::NameDescending)
            | (SortBy::SizeDescending, SortBy::NameDescending)
            | (SortBy::None, SortBy::NameDescending) => {
                self.sort_unstable_by(|a, b| b.meta.name.cmp(&a.meta.name));
            }

            (SortBy::NameAscending, SortBy::SizeAscending)
            | (SortBy::NameDescending, SortBy::SizeAscending)
            | (SortBy::None, SortBy::SizeAscending) => {
                self.sort_unstable_by(|a, b| a.size.cmp(&b.size));
            }

            (SortBy::NameAscending, SortBy::SizeDescending)
            | (SortBy::NameDescending, SortBy::SizeDescending)
            | (SortBy::None, SortBy::SizeDescending) => {
                self.sort_unstable_by(|a, b| b.size.cmp(&a.size));
            }
        }
    }
}
