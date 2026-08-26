// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use super::homebrew_app::HomebrewApp;
use crate::{config::SortBy, errors::Error};
use either::Either;
use smol::{
    fs,
    stream::{self, Stream, StreamExt},
};
use std::{path::Path, sync::Arc};

#[derive(Debug, Clone, Default)]
pub struct HomebrewAppList {
    sorted_by_name: Vec<Arc<HomebrewApp>>,
    sorted_by_size: Vec<Arc<HomebrewApp>>,
}

impl HomebrewAppList {
    pub async fn new(root_path: impl AsRef<Path>) -> Result<Self, Error> {
        let apps_dir_path = root_path.as_ref().join("apps");
        let apps = scan_dir(&apps_dir_path)
            .await
            .map(Arc::new)
            .collect::<Vec<_>>()
            .await;

        let mut sorted_by_name = apps.clone();
        let mut sorted_by_size = apps;

        sorted_by_name.sort_by(|a, b| a.meta.name.cmp(&b.meta.name));
        sorted_by_size.sort_by_key(|g| g.size);

        Ok(Self {
            sorted_by_name,
            sorted_by_size,
        })
    }

    pub fn iter_by(&self, sort_by: SortBy) -> impl Iterator<Item = &Arc<HomebrewApp>> {
        match sort_by {
            SortBy::NameAscending => Either::Left(self.sorted_by_name.iter()),
            SortBy::NameDescending => Either::Right(self.sorted_by_name.iter().rev()),
            SortBy::SizeAscending => Either::Left(self.sorted_by_size.iter()),
            SortBy::SizeDescending => Either::Right(self.sorted_by_size.iter().rev()),
        }
    }
}

async fn scan_dir(dir_path: &Path) -> impl Stream<Item = HomebrewApp> {
    stream::iter(fs::read_dir(dir_path).await.ok())
        .flatten()
        .then(move |entry| async move {
            let path = entry.ok()?.path();
            HomebrewApp::try_from_path(path).await.ok()
        })
        .filter_map(std::convert::identity)
}
