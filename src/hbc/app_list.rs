// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{config::SortBy, hbc::app::HbcApp, message::Message, state::State};
use anyhow::Result;
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use iced::{Task, futures::TryFutureExt};
use itertools::Itertools;
use size::Size;
use smol::{fs, stream::StreamExt};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct HbcAppList {
    list: Box<[HbcApp]>,
    total_size: Size,
    filtered_indices: Box<[usize]>,
}

impl HbcAppList {
    pub fn empty() -> Self {
        Self {
            list: Box::new([]),
            total_size: Size::from_bytes(0),
            filtered_indices: Box::new([]),
        }
    }

    pub fn new(apps: Vec<HbcApp>) -> Self {
        let total_size = apps
            .iter()
            .fold(Size::from_bytes(0), |acc, app| acc + app.size());

        Self {
            list: apps.into_boxed_slice(),
            total_size,
            filtered_indices: Box::new([]),
        }
    }

    #[inline(always)]
    pub fn get(&self, i: usize) -> Option<&HbcApp> {
        self.list.get(i)
    }

    #[inline(always)]
    pub fn get_unchecked(&self, i: usize) -> &HbcApp {
        &self.list[i]
    }

    #[inline(always)]
    pub fn count(&self) -> usize {
        self.list.len()
    }

    #[inline(always)]
    pub fn iter(&self) -> impl Iterator<Item = &HbcApp> {
        self.list.iter()
    }

    #[inline(always)]
    pub fn total_size(&self) -> Size {
        self.total_size
    }

    #[inline(always)]
    pub fn filtered_indices(&self) -> impl Iterator<Item = usize> {
        self.filtered_indices.iter().copied()
    }

    pub fn fuzzy_search(&mut self, query: &str) {
        let matcher = SkimMatcherV2::default();

        self.filtered_indices = self
            .list
            .iter()
            .enumerate()
            .filter_map(|(i, app)| {
                matcher
                    .fuzzy_match(app.meta().name(), query)
                    .map(|score| (i, score))
            })
            .sorted_unstable_by_key(|(_, score)| *score)
            .map(|(i, _)| i)
            .collect();
    }

    pub fn sort(&mut self, prev_sort_by: SortBy, sort_by: SortBy) {
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
                self.list.reverse();
            }

            (SortBy::SizeAscending, SortBy::NameAscending)
            | (SortBy::SizeDescending, SortBy::NameAscending)
            | (SortBy::None, SortBy::NameAscending) => {
                self.list
                    .sort_unstable_by(|a, b| a.meta().name().cmp(b.meta().name()));
            }

            (SortBy::SizeAscending, SortBy::NameDescending)
            | (SortBy::SizeDescending, SortBy::NameDescending)
            | (SortBy::None, SortBy::NameDescending) => {
                self.list
                    .sort_unstable_by(|a, b| b.meta().name().cmp(a.meta().name()));
            }

            (SortBy::NameAscending, SortBy::SizeAscending)
            | (SortBy::NameDescending, SortBy::SizeAscending)
            | (SortBy::None, SortBy::SizeAscending) => {
                self.list.sort_unstable_by_key(|a| a.size());
            }

            (SortBy::NameAscending, SortBy::SizeDescending)
            | (SortBy::NameDescending, SortBy::SizeDescending)
            | (SortBy::None, SortBy::SizeDescending) => {
                self.list
                    .sort_unstable_by_key(|a| std::cmp::Reverse(a.size()));
            }
        }
    }
}

pub fn get_list_hbc_apps_task(state: &State) -> Task<Message> {
    let mount_point = state.config.mount_point().to_path_buf();

    Task::perform(
        list(mount_point).map_err(|e| e.to_string()),
        Message::GotHbcAppList,
    )
}

async fn list(mount_point: PathBuf) -> Result<HbcAppList> {
    let apps_dir = mount_point.join("apps");
    if !apps_dir.exists() {
        return Ok(HbcAppList::empty());
    }

    let mut entries = fs::read_dir(&apps_dir).await?;

    let mut hbc_apps = Vec::new();
    while let Some(entry) = entries.try_next().await? {
        if let Some(hbc_app) = HbcApp::from_path(entry.path()).await {
            hbc_apps.push(hbc_app);
        }
    }

    let hbc_app_list = HbcAppList::new(hbc_apps);
    Ok(hbc_app_list)
}
