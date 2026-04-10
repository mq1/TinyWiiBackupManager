// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{HbcApp, HbcAppList, SortBy, hbc_app};
use anyhow::Result;
use slint::{ModelRc, VecModel};
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
