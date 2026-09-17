// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{errors::Error, osc::osc_app::OscApp, util::http::download_file};
use smol::fs;
use smol_str::SmolStr;
use std::{
    path::Path,
    time::{Duration, SystemTime},
};

const CONTENTS_URL: &str = "https://hbb1.oscwii.org/api/v4/contents";

#[derive(Debug, Clone, Default)]
pub struct OscAppFilter {
    pub search_term: SmolStr,
}

#[derive(Clone, Debug)]
pub struct OscAppList {
    apps: Vec<OscApp>,
    refreshed: SystemTime,
    pub filter: OscAppFilter,
}

impl OscAppList {
    pub fn iter(&self) -> impl Iterator<Item = &OscApp> {
        self.apps
            .iter()
            .filter(|app| app.matches_search(&self.filter.search_term))
    }

    pub fn last_refresh(&self) -> Duration {
        self.refreshed
            .elapsed()
            .map(|d| Duration::from_mins(d.as_secs() / 60))
            .unwrap_or(Duration::MAX)
    }
}

#[derive(Default, Clone, Debug)]
pub enum OscContents {
    #[default]
    NotYetLoaded,
    Loaded(OscAppList),
    Errored(Error),
}

impl OscContents {
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
                filter: OscAppFilter::default(),
            };

            Ok(list)
        }
        .await;

        match res {
            Ok(apps) => Self::Loaded(apps),
            Err(err) => Self::Errored(err),
        }
    }
}
