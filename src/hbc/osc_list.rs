// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{hbc::osc::OscAppMeta, http_util, message::Message, state::State};
use anyhow::Result;
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use iced::{Task, futures::TryFutureExt};
use itertools::Itertools;
use smol::fs;
use std::{
    path::{Path, PathBuf},
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

    #[inline(always)]
    pub fn get_unchecked(&self, i: usize) -> &OscAppMeta {
        &self.list[i]
    }

    #[inline(always)]
    pub fn position_by_slug(&self, slug: &str) -> Option<usize> {
        self.list.iter().position(|a| a.slug == slug)
    }

    #[inline(always)]
    pub fn count(&self) -> usize {
        self.list.len()
    }

    #[inline(always)]
    pub fn iter(&self) -> impl Iterator<Item = &OscAppMeta> {
        self.list.iter()
    }

    #[inline(always)]
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
                    .fuzzy_match(&app.name, query)
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
        load_osc_apps(data_dir).map_err(|e| e.to_string()),
        Message::GotOscAppList,
    )
}

async fn load_osc_apps(data_dir: PathBuf) -> Result<OscAppList> {
    let cache_path = data_dir.join("osc-cache.json");

    let apps = if let Some(cache) = load_cache(&cache_path).await {
        cache
    } else {
        let bytes = http_util::get(CONTENTS_URL.to_string()).await?;
        fs::write(&cache_path, &bytes).await?;
        serde_json::from_slice(&bytes)?
    };

    let osc_app_list = OscAppList::new(apps);
    Ok(osc_app_list)
}

async fn load_cache(path: &Path) -> Option<Vec<OscAppMeta>> {
    // get file time
    let file_time = fs::metadata(path).await.ok()?.modified().ok()?;

    // get difference
    let elapsed = file_time.elapsed().ok()?;

    if elapsed > Duration::from_secs(60 * 60 * 24) {
        return None;
    }

    let bytes = fs::read(path).await.ok()?;
    let apps = serde_json::from_slice(&bytes).ok()?;

    Some(apps)
}

pub fn get_download_icons_task(state: &State) -> Task<Message> {
    let list = state.osc_app_list.iter().cloned().collect_vec();
    let icons_dir = state.data_dir.join("osc-icons");

    Task::perform(
        async move {
            fs::create_dir_all(&icons_dir)
                .await
                .map_err(|e| e.to_string())?;

            for app in list {
                let icon_path = icons_dir.join(app.slug).with_extension("png");
                if !icon_path.exists() {
                    let _ = http_util::download_file(app.assets.icon.url, &icon_path).await;
                }
            }

            Ok(())
        },
        Message::EmptyResult,
    )
}
