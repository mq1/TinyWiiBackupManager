// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::homebrew_app::HomebrewApp;
use itertools::Either;
use std::{fs, io, path::Path};
use twbm_common::config_contents::SortBy;

#[derive(Debug, Clone, Default)]
pub struct HomebrewAppFilter {
    search_term: String,
}

#[derive(Debug, Clone)]
pub struct HomebrewAppList {
    apps: Vec<HomebrewApp>,
    order_by_name: Box<[usize]>,
    order_by_size: Box<[usize]>,
    filter: HomebrewAppFilter,
}

#[derive(Debug, Clone)]
pub enum HomebrewAppManager {
    Loading,
    Loaded(HomebrewAppList),
    Errored(String),
}

impl HomebrewAppManager {
    pub fn load(root_path: &Path) -> HomebrewAppManager {
        let apps_path = root_path.join("apps");

        let apps = match scan_dir(&apps_path) {
            Ok(apps) => apps,
            Err(e) if e.kind() == io::ErrorKind::NotFound => vec![],
            Err(e) => return Self::Errored(e.to_string()),
        };

        let mut order_by_name = (0..apps.len()).collect::<Box<[_]>>();
        let mut order_by_size = order_by_name.clone();

        order_by_name.sort_by_key(|&i| apps[i].meta().name());
        order_by_size.sort_by_key(|&i| apps[i].size());

        Self::Loaded(HomebrewAppList {
            apps,
            order_by_name,
            order_by_size,
            filter: HomebrewAppFilter::default(),
        })
    }
}

impl HomebrewAppList {
    pub fn iter(&self) -> impl Iterator<Item = &HomebrewApp> {
        self.apps.iter()
    }

    pub fn iter_by(&self, sort_by: SortBy) -> impl Iterator<Item = &HomebrewApp> {
        let (order, reversed) = match sort_by {
            SortBy::NameAscending => (self.order_by_name.as_ref(), false),
            SortBy::NameDescending => (self.order_by_name.as_ref(), true),
            SortBy::SizeAscending => (self.order_by_size.as_ref(), false),
            SortBy::SizeDescending => (self.order_by_size.as_ref(), true),
        };

        let iter = order
            .iter()
            .map(|&i| &self.apps[i])
            .filter(|app| app.matches_search(&self.filter.search_term));

        if reversed {
            Either::Right(iter.rev())
        } else {
            Either::Left(iter)
        }
    }

    pub fn count(&self) -> usize {
        self.apps.len()
    }

    pub fn search_term(&self) -> &str {
        &self.filter.search_term
    }

    pub fn set_search_term(&mut self, search_term: String) {
        self.filter.search_term = search_term;
    }
}

fn scan_dir(dir_path: &Path) -> io::Result<Vec<HomebrewApp>> {
    fs::read_dir(dir_path).and_then(|read_dir| {
        read_dir
            .filter_map(|entry| {
                entry
                    .map(|e| HomebrewApp::try_from_path(e.path()).ok())
                    .transpose()
            })
            .collect()
    })
}
