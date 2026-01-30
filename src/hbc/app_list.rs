// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{config::SortBy, hbc::app::HbcApp, message::Message, state::State};
use anyhow::Result;
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use iced::{Task, futures::TryFutureExt};
use size::Size;
use std::{fs, path::Path};

#[derive(Debug, Clone)]
pub struct HbcAppList {
    list: Box<[HbcApp]>,
    total_size: Size,
    filtered_indices: Box<[(usize, i64)]>,
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

    #[inline]
    pub fn count(&self) -> usize {
        self.list.len()
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &HbcApp> {
        self.list.iter()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut HbcApp> {
        self.list.iter_mut()
    }

    #[inline]
    pub const fn total_size(&self) -> Size {
        self.total_size
    }

    #[inline]
    pub fn iter_filtered(&self) -> impl Iterator<Item = &HbcApp> {
        self.filtered_indices
            .iter()
            .copied()
            .map(|(i, _score)| &self.list[i])
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
            .collect();

        self.filtered_indices
            .sort_unstable_by_key(|(_, score)| *score);
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

            (
                SortBy::SizeAscending | SortBy::SizeDescending | SortBy::None,
                SortBy::NameAscending,
            ) => {
                self.list
                    .sort_unstable_by(|a, b| a.meta().name().cmp(b.meta().name()));
            }

            (
                SortBy::SizeAscending | SortBy::SizeDescending | SortBy::None,
                SortBy::NameDescending,
            ) => {
                self.list
                    .sort_unstable_by(|a, b| b.meta().name().cmp(a.meta().name()));
            }

            (
                SortBy::NameAscending | SortBy::NameDescending | SortBy::None,
                SortBy::SizeAscending,
            ) => {
                self.list.sort_unstable_by_key(super::app::HbcApp::size);
            }

            (
                SortBy::NameAscending | SortBy::NameDescending | SortBy::None,
                SortBy::SizeDescending,
            ) => {
                self.list
                    .sort_unstable_by_key(|a| std::cmp::Reverse(a.size()));
            }
        }
    }
}

pub fn get_list_hbc_apps_task(state: &State) -> Task<Message> {
    let mount_point = state.config.mount_point().clone();

    Task::perform(
        async move { list(&mount_point) }.map_err(|e| format!("Failed to list apps: {e:#}")),
        Message::GotHbcAppList,
    )
}

fn list(mount_point: &Path) -> Result<HbcAppList> {
    let apps_dir = mount_point.join("apps");
    if !apps_dir.exists() {
        return Ok(HbcAppList::empty());
    }

    let hbc_apps = fs::read_dir(apps_dir)?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter_map(HbcApp::maybe_from_path)
        .collect();

    let hbc_app_list = HbcAppList::new(hbc_apps);
    Ok(hbc_app_list)
}
