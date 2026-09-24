// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{osc::osc_app::OscApp, util::http::download_file};
use smol::fs;
use std::{
    path::Path,
    time::{Duration, SystemTime},
};

const CONTENTS_URL: &str = "https://hbb1.oscwii.org/api/v4/contents";

#[derive(Clone, Debug)]
pub struct OscAppList {
    apps: Box<[OscApp]>,
    refreshed: SystemTime,
    search_term: String,
}

#[derive(Clone, Debug)]
pub enum OscState {
    NotLoaded,
    Loading,
    Loaded(OscAppList),
    Errored(String),
}

impl OscState {
    pub async fn load(data_dir: &Path) -> Self {
        let cache_path = data_dir.join("osc-cache.json");

        let res = async move {
            let refreshed = match fs::metadata(&cache_path)
                .await
                .and_then(|meta| meta.modified())
            {
                Ok(time) => time,
                Err(_) => {
                    download_file(CONTENTS_URL, &cache_path).await?;
                    fs::metadata(&cache_path).await?.modified()?
                }
            };

            let contents = fs::read_to_string(&cache_path).await?;
            let apps = serde_json::from_str(&contents)?;

            let list = OscAppList {
                apps,
                refreshed,
                search_term: String::default(),
            };

            Ok::<_, anyhow::Error>(list)
        }
        .await;

        match res {
            Ok(apps) => Self::Loaded(apps),
            Err(err) => Self::Errored(err.to_string()),
        }
    }
}

impl OscAppList {
    pub fn search_term(&self) -> &str {
        &self.search_term
    }

    pub fn set_search_term(&mut self, search_term: String) {
        self.search_term = search_term;
    }

    pub fn iter(&self) -> impl Iterator<Item = &OscApp> {
        self.apps
            .iter()
            .filter(|app| app.matches_search(&self.search_term))
    }

    pub fn last_refresh(&self) -> Duration {
        self.refreshed
            .elapsed()
            .map(|d| Duration::from_mins(d.as_secs() / 60))
            .unwrap_or(Duration::MAX)
    }

    pub fn reload_icon(&mut self, idx: usize, data_dir: &Path) {
        self.apps[idx].load_icon_blocking(data_dir);
    }

    pub fn reload_all_icons(&mut self, data_dir: &Path) {
        for app in &mut self.apps {
            app.load_icon_blocking(data_dir);
        }
    }
}

impl std::ops::Index<usize> for OscAppList {
    type Output = OscApp;

    fn index(&self, index: usize) -> &Self::Output {
        &self.apps[index]
    }
}
