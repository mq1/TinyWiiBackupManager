// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use super::homebrew_app::HomebrewApp;
use crate::{config::SortBy, errors::Error};
use either::Either;
use smol::{
    fs,
    stream::{self, Stream, StreamExt},
};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default)]
pub struct HomebrewAppList {
    apps: Vec<HomebrewApp>,
    order_by_name: Vec<usize>,
    order_by_size: Vec<usize>,
}

impl HomebrewAppList {
    pub async fn new(root_path: PathBuf) -> Result<Self, Error> {
        let apps_dir_path = root_path.join("apps");
        let apps = scan_dir(&apps_dir_path).await.collect::<Vec<_>>().await;

        let mut order_by_name = (0..apps.len()).collect::<Vec<_>>();
        let mut order_by_size = order_by_name.clone();

        order_by_name.sort_by_key(|&i| &apps[i].meta.name);
        order_by_size.sort_by_key(|&i| apps[i].size);

        Ok(Self {
            apps,
            order_by_name,
            order_by_size,
        })
    }

    pub fn iter_by(&self, sort_by: SortBy) -> impl Iterator<Item = &HomebrewApp> {
        let (order, reversed) = match sort_by {
            SortBy::NameAscending => (&self.order_by_name, false),
            SortBy::NameDescending => (&self.order_by_name, true),
            SortBy::SizeAscending => (&self.order_by_size, false),
            SortBy::SizeDescending => (&self.order_by_size, true),
        };

        let iter = order.iter().map(|&i| &self.apps[i]);

        if reversed {
            Either::Right(iter.rev())
        } else {
            Either::Left(iter)
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
