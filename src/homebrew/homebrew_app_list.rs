// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use super::homebrew_app::HomebrewApp;
use crate::{config::SortBy, errors::Error};
use futures::{Stream, StreamExt, TryFutureExt};
use smol::fs;
use std::{ops::Index, path::Path};

#[derive(Debug, Clone, Default)]
pub struct HomebrewAppList {
    inner: Vec<HomebrewApp>,
}

impl HomebrewAppList {
    pub async fn new(root_path: impl AsRef<Path>) -> Result<Self, Error> {
        let apps_dir_path = root_path.as_ref().join("apps");
        let apps = scan_dir(&apps_dir_path).await.collect().await;

        Ok(Self { inner: apps })
    }

    pub fn sorted_by(mut self, sort_by: SortBy) -> Self {
        let compare: fn(&HomebrewApp, &HomebrewApp) -> _ = match sort_by {
            SortBy::NameDescending => |a, b| a.meta.name.cmp(&b.meta.name),
            SortBy::NameAscending => |a, b| b.meta.name.cmp(&a.meta.name),
            SortBy::SizeDescending => |a, b| a.size.cmp(&b.size),
            SortBy::SizeAscending => |a, b| b.size.cmp(&a.size),
        };

        self.inner.sort_unstable_by(compare);
        self
    }

    pub fn iter(&self) -> impl Iterator<Item = &HomebrewApp> {
        self.inner.iter()
    }
}

async fn scan_dir(dir_path: &Path) -> impl Stream<Item = HomebrewApp> {
    fs::read_dir(dir_path)
        .try_flatten_stream()
        .filter_map(move |entry| async move {
            let path = entry.ok()?.path();
            HomebrewApp::try_from_path(path).await.ok()
        })
}

impl Index<usize> for HomebrewAppList {
    type Output = HomebrewApp;

    fn index(&self, index: usize) -> &Self::Output {
        &self.inner[index]
    }
}
