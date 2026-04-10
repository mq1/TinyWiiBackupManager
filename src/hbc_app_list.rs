// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{HbcApp, HbcAppList, SortBy, hbc_app};
use anyhow::Result;
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use slint::{Model, ModelRc, SharedString, VecModel};
use std::{fs, path::Path, rc::Rc};

impl HbcAppList {
    #[must_use]
    pub fn new(drive_path: &Path, sort_by: SortBy) -> Self {
        let apps_path = drive_path.join("apps");

        let mut apps = Vec::new();
        let _ = read_apps_dir(&apps_path, &mut apps);

        let total_size = apps.iter().fold(0., |acc, app| acc + app.size_mib);

        apps.sort_by(hbc_app::get_compare_fn(sort_by));
        let model = VecModel::from(apps);

        Self {
            apps: ModelRc::from(Rc::new(model)),
            filter: SharedString::new(),
            filtered_apps: ModelRc::default(),
            total_size,
        }
    }
}

fn read_apps_dir(apps_dir: &Path, apps: &mut Vec<HbcApp>) -> Result<()> {
    let entries = fs::read_dir(apps_dir)?;
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if let Some(game) = HbcApp::maybe_from_path(&path) {
            apps.push(game);
        }
    }

    Ok(())
}

pub fn fuzzy_search(apps: &ModelRc<HbcApp>, query: &str) -> ModelRc<HbcApp> {
    let matcher = SkimMatcherV2::default();

    let mut filtered_apps = Vec::new();
    for app in apps.iter() {
        let name_score = matcher.fuzzy_match(&app.meta.name, query);
        let coder_score = matcher.fuzzy_match(&app.meta.coder, query);

        let score = match (name_score, coder_score) {
            (Some(a), Some(b)) => a.saturating_add(b),
            (Some(a), None) | (None, Some(a)) => a,
            (None, None) => continue,
        };

        filtered_apps.push((app, score));
    }

    filtered_apps.sort_unstable_by_key(|(_, score)| *score);

    let filtered_apps = filtered_apps
        .into_iter()
        .map(|(app, _)| app)
        .collect::<VecModel<_>>();

    ModelRc::from(Rc::new(filtered_apps))
}
