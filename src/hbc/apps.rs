// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::config::SortBy;
use crate::message::Message;
use crate::state::State;
use crate::util::{self, FuzzySearchable};
use anyhow::Result;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use iced::Task;
use iced::futures::TryFutureExt;
use iced::futures::future::join_all;
use itertools::Itertools;
use serde::{Deserialize, Deserializer};
use size::Size;
use smol::fs;
use smol::stream::StreamExt;
use std::ffi::OsString;
use std::path::PathBuf;
use time::PrimitiveDateTime;
use time::macros::format_description;

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
    pub path: PathBuf,
    pub image_path: Option<PathBuf>,
}

impl HbcApp {
    pub async fn from_path(path: PathBuf) -> Option<Self> {
        if !path.is_dir() {
            return None;
        }

        let slug = path.file_name()?.to_str()?;

        if slug.starts_with('.') {
            return None;
        }

        let meta_path = path.join("meta").with_extension("xml");
        let meta = fs::read_to_string(&meta_path).await.unwrap_or_default();
        let mut meta = quick_xml::de::from_str::<HbcAppMeta>(&meta).unwrap_or_default();

        if meta.name.is_empty() {
            return None;
        }

        meta.name = meta.name.trim().to_string();

        let size = util::get_dir_size(path.clone()).await.unwrap_or_default();

        let image_path = path.join("icon.png");
        let image_path = if image_path.exists() {
            Some(image_path)
        } else {
            None
        };

        Some(Self {
            meta,
            path,
            size,
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

pub fn get_list_hbc_apps_task(state: &State) -> Task<Message> {
    let mount_point = state.config.mount_point().to_path_buf();

    Task::perform(
        list(mount_point).map_err(|e| e.to_string()),
        Message::GotHbcApps,
    )
}

async fn list(mount_point: PathBuf) -> Result<Box<[HbcApp]>> {
    let apps_dir = mount_point.join("apps");
    if !apps_dir.exists() {
        return Ok(Box::new([]));
    }

    let mut entries = fs::read_dir(&apps_dir).await?;

    let mut hbc_apps = Vec::new();
    while let Some(entry) = entries.try_next().await? {
        let path = entry.path();
        hbc_apps.push(HbcApp::from_path(path));
    }

    Ok(join_all(hbc_apps).await.into_iter().flatten().collect())
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

impl FuzzySearchable for Box<[HbcApp]> {
    fn fuzzy_search(&self, query: &str) -> Box<[usize]> {
        let matcher = SkimMatcherV2::default();

        self.iter()
            .enumerate()
            .filter_map(|(i, app)| {
                matcher
                    .fuzzy_match(&app.meta.name, query)
                    .map(|score| (i, score))
            })
            .sorted_unstable_by_key(|(_, score)| *score)
            .map(|(i, _)| i)
            .collect()
    }
}

pub fn get_install_hbc_apps_task(state: &State, zip_paths: Box<[PathBuf]>) -> Task<Message> {
    let drive_path = state.config.mount_point().to_path_buf();

    Task::perform(
        install_hbc_apps(drive_path, zip_paths).map_err(|e| e.to_string()),
        Message::HbcAppsInstalled,
    )
}

async fn install_hbc_apps(dest_dir: PathBuf, zip_paths: Box<[PathBuf]>) -> Result<()> {
    for zip_path in zip_paths {
        util::extract_zip(&zip_path, &dest_dir).await?;
    }

    Ok(())
}
