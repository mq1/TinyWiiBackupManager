// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{hbc::osc::OscAppMeta, http_util, message::Message, state::State};
use anyhow::Result;
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use iced::{Task, futures::TryFutureExt};
use itertools::Itertools;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

const CONTENTS_URL: &str = "https://hbb1.oscwii.org/api/v4/contents";

#[derive(Debug, Clone)]
pub struct OscAppList {
    list: Box<[OscAppMeta]>,
    filtered_indices: Box<[(usize, i64)]>,
}

impl OscAppList {
    pub fn empty() -> Self {
        Self {
            list: Box::new([]),
            filtered_indices: Box::new([]),
        }
    }

    pub fn new(osc_apps: Vec<OscAppMeta>) -> Self {
        Self {
            list: osc_apps.into_boxed_slice(),
            filtered_indices: Box::new([]),
        }
    }

    #[allow(unused)]
    #[inline]
    pub fn count(&self) -> usize {
        self.list.len()
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &OscAppMeta> {
        self.list.iter()
    }

    #[inline]
    pub fn iter_filtered(&self) -> impl Iterator<Item = &OscAppMeta> {
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
                    .fuzzy_match(app.name(), query)
                    .map(|score| (i, score))
            })
            .collect();

        self.filtered_indices
            .sort_unstable_by_key(|(_, score)| *score);
    }
}

pub fn get_load_osc_apps_task(state: &State) -> Task<Message> {
    let data_dir = state.data_dir.clone();

    Task::perform(
        async { load_osc_apps(data_dir) }.map_err(Arc::new),
        Message::GotOscAppList,
    )
}

fn load_osc_apps(data_dir: PathBuf) -> Result<OscAppList> {
    let cache_path = data_dir.join("osc-cache.json");

    let apps = if let Some(cache) = load_cache(&cache_path) {
        cache
    } else {
        let bytes = http_util::get(CONTENTS_URL)?;
        fs::write(&cache_path, &bytes)?;
        serde_json::from_slice(&bytes)?
    };

    let osc_app_list = OscAppList::new(apps);
    Ok(osc_app_list)
}

fn load_cache(path: &Path) -> Option<Vec<OscAppMeta>> {
    // get file time
    let file_time = fs::metadata(path).ok()?.modified().ok()?;

    // get difference
    let elapsed = file_time.elapsed().ok()?;

    if elapsed > Duration::from_secs(60 * 60 * 24) {
        return None;
    }

    let bytes = fs::read(path).ok()?;
    let apps = serde_json::from_slice(&bytes).ok()?;

    Some(apps)
}

pub fn get_download_icons_task(state: &State) -> Task<Message> {
    let app_list = state.osc_app_list.clone();
    let icons_dir = state.data_dir.join("osc-icons");

    Task::perform(
        async move { download_icons(&app_list, &icons_dir) }.map_err(Arc::new),
        Message::EmptyResult,
    )
}

fn download_icons(app_list: &OscAppList, dest_dir: &Path) -> Result<()> {
    fs::create_dir_all(dest_dir)?;

    for app in app_list.iter() {
        let icon_path = dest_dir.join(app.slug()).with_extension("png");
        if !icon_path.exists() {
            let _ = http_util::download_file(app.assets().icon().url(), &icon_path);
        }
    }

    Ok(())
}
