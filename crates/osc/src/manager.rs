// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::app::OscApp;
use jiff::{SignedDuration, Unit};
use std::{
    collections::HashMap,
    fs,
    path::Path,
    time::{Duration, SystemTime},
};
use twbm_http::download_file;

const CONTENTS_URL: &str = "https://hbb1.oscwii.org/api/v4/contents";

#[derive(Clone, Debug)]
pub struct OscAppList {
    apps: HashMap<Box<str>, OscApp>,
    refreshed: SystemTime,
    search_term: String,
}

#[derive(Debug, Clone)]
pub enum OscAppManager {
    NotLoaded,
    Loading,
    Loaded(OscAppList),
    Errored(String),
}

impl OscAppManager {
    pub fn load(data_dir: &Path, force_refresh: bool) -> OscAppManager {
        let cache_path = data_dir.join("osc-cache.json");

        let res = (move || {
            let is_old = fs::metadata(&cache_path)
                .ok()
                .and_then(|meta| meta.modified().ok())
                .and_then(|modified| modified.elapsed().ok())
                .is_none_or(|d| d > Duration::from_hours(24));

            let should_refresh = force_refresh || is_old;

            if should_refresh {
                let new_path = data_dir.join("osc-cache-new.json");
                if download_file(CONTENTS_URL, &new_path).is_ok() {
                    fs::rename(&new_path, &cache_path)?;
                }
            }

            let refreshed = fs::metadata(&cache_path)?.modified()?;

            let contents = fs::read_to_string(&cache_path)?;
            let apps = serde_json::from_str::<Vec<OscApp>>(&contents)?;

            let apps = apps
                .into_iter()
                .map(|app| (app.slug().into(), app))
                .collect();

            let list = OscAppList {
                apps,
                refreshed,
                search_term: String::default(),
            };

            Ok::<_, anyhow::Error>(list)
        })();

        match res {
            Ok(apps) => OscAppManager::Loaded(apps),
            Err(err) => OscAppManager::Errored(err.to_string()),
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
            .values()
            .filter(|app| app.matches_search(&self.search_term))
    }

    pub fn last_refresh(&self) -> SignedDuration {
        SignedDuration::system_until(self.refreshed, SystemTime::now())
            .ok()
            .and_then(|d| d.round(Unit::Second).ok())
            .unwrap_or(SignedDuration::MAX)
    }

    pub fn reload_icon(&mut self, slug: &str, data_dir: &Path) {
        if let Some(app) = self.apps.get_mut(slug) {
            app.set_icon(data_dir);
        }
    }

    pub fn reload_all_icons(&mut self, data_dir: &Path) {
        for app in self.apps.values_mut() {
            app.set_icon(data_dir);
        }
    }
}
