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
    pub async fn load(data_dir: &Path, force_refresh: bool) -> Self {
        let cache_path = data_dir.join("osc-cache.json");

        let res = async move {
            let is_old = fs::metadata(&cache_path)
                .await
                .ok()
                .and_then(|meta| meta.modified().ok())
                .and_then(|modified| modified.elapsed().ok())
                .is_none_or(|d| d > Duration::from_hours(24));

            let should_refresh = force_refresh || is_old;

            if should_refresh {
                let new_path = data_dir.join("osc-cache-new.json");
                if download_file(CONTENTS_URL, &new_path).await.is_ok() {
                    let valid = fs::read_to_string(&new_path)
                        .await
                        .ok()
                        .and_then(|contents| serde_json::from_str::<Box<[OscApp]>>(&contents).ok())
                        .is_some();
                    if valid {
                        fs::rename(&new_path, &cache_path).await?;
                    } else {
                        // Keep the last usable catalog, and allow a later refresh
                        // to download again instead of reusing the invalid file.
                        let _ = fs::remove_file(&new_path).await;
                    }
                }
            }

            let refreshed = fs::metadata(&cache_path).await?.modified()?;
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

    pub fn into_iter(self) -> impl Iterator<Item = OscApp> {
        self.apps.into_iter()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_pending_catalog_does_not_replace_a_usable_cache() {
        for invalid in [
            "<html>service unavailable</html>",
            "{invalid JSON",
            r#"[{"slug":"missing-required-fields"}]"#,
        ] {
            let dir = tempfile::tempdir().unwrap();
            let cache = dir.path().join("osc-cache.json");
            let pending = dir.path().join("osc-cache-new.json");
            std::fs::write(&cache, "[]").unwrap();
            std::fs::write(&pending, invalid).unwrap();

            // An existing pending file avoids the network and exercises the
            // same validation used after a completed download.
            let state = smol::block_on(OscState::load(dir.path(), true));
            assert!(matches!(state, OscState::Loaded(_)));
            assert_eq!(std::fs::read_to_string(cache).unwrap(), "[]");
            assert!(!pending.exists());
        }
    }

    #[test]
    fn valid_pending_catalog_can_replace_an_invalid_old_catalog() {
        let dir = tempfile::tempdir().unwrap();
        let cache = dir.path().join("osc-cache.json");
        let pending = dir.path().join("osc-cache-new.json");
        std::fs::write(&cache, "invalid old catalog").unwrap();
        std::fs::write(&pending, "[]").unwrap();

        let state = smol::block_on(OscState::load(dir.path(), true));
        assert!(matches!(state, OscState::Loaded(_)));
        assert_eq!(std::fs::read_to_string(cache).unwrap(), "[]");
        assert!(!pending.exists());
    }
}
