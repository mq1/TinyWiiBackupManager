// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{config::SortBy, homebrew::homebrew_app::HomebrewApp};
use itertools::Either;
use smol::{
    fs,
    stream::{self, Stream, StreamExt},
};
use smol_str::{SmolStr, ToSmolStr};
use std::path::PathBuf;
use tap::Pipe;

#[derive(Debug, Clone, Default)]
pub struct HomebrewAppList {
    apps: Box<[HomebrewApp]>,
    order_by_name: Box<[usize]>,
    order_by_size: Box<[usize]>,
    search_term: SmolStr,
}

#[derive(Debug, Clone, Default)]
pub enum HomebrewState {
    #[default]
    NotLoaded,
    Loading,
    Loaded(HomebrewAppList),
    Errored(SmolStr),
}

impl HomebrewState {
    pub async fn load(root_path: PathBuf) -> Self {
        let res = async move {
            let apps = root_path
                .join("apps")
                .pipe_ref(scan_dir)
                .await
                .collect::<Vec<_>>()
                .await
                .into_boxed_slice();

            let mut order_by_name = (0..apps.len()).collect::<Box<[_]>>();
            let mut order_by_size = order_by_name.clone();

            order_by_name.sort_by_key(|&i| apps[i].name());
            order_by_size.sort_by_key(|&i| apps[i].size());

            let app_list = HomebrewAppList {
                apps,
                order_by_name,
                order_by_size,
                search_term: SmolStr::default(),
            };

            Ok::<_, anyhow::Error>(app_list)
        }
        .await;

        match res {
            Ok(app_list) => HomebrewState::Loaded(app_list),
            Err(err) => HomebrewState::Errored(err.to_smolstr()),
        }
    }

    pub fn iter_by(&self, sort_by: SortBy) -> impl Iterator<Item = &HomebrewApp> {
        match self {
            HomebrewState::Loaded(app_list) => {
                let (order, reversed) = match sort_by {
                    SortBy::NameAscending => (&app_list.order_by_name, false),
                    SortBy::NameDescending => (&app_list.order_by_name, true),
                    SortBy::SizeAscending => (&app_list.order_by_size, false),
                    SortBy::SizeDescending => (&app_list.order_by_size, true),
                };

                let matches_search = |app: &&HomebrewApp| app.matches_search(&app_list.search_term);

                let iter = order
                    .iter()
                    .map(|&i| &app_list.apps[i])
                    .filter(matches_search);

                Either::Left(if reversed {
                    Either::Right(iter.rev())
                } else {
                    Either::Left(iter)
                })
            }
            _ => Either::Right(std::iter::empty()),
        }
    }

    pub fn count(&self) -> usize {
        match self {
            HomebrewState::Loaded(app_list) => app_list.apps.len(),
            _ => 0,
        }
    }

    pub fn search_term(&self) -> &str {
        match self {
            HomebrewState::Loaded(app_list) => &app_list.search_term,
            _ => "",
        }
    }

    pub fn set_search_term(&mut self, search_term: SmolStr) {
        if let HomebrewState::Loaded(app_list) = self {
            app_list.search_term = search_term;
        }
    }
}

async fn scan_dir(dir_path: &PathBuf) -> impl Stream<Item = HomebrewApp> {
    stream::iter(fs::read_dir(dir_path).await.ok())
        .flatten()
        .then(move |entry| async move {
            let path = entry.ok()?.path();
            HomebrewApp::try_from_path(path).await.ok()
        })
        .filter_map(std::convert::identity)
}
