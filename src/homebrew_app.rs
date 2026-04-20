// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    AppWindow, Config, HomebrewApp, HomebrewAppMeta, SortBy, mirrored::Mirrored, util::MIB,
};
use anyhow::Result;
use slint::{Image, SharedString, ToSharedString};
use std::{cell::RefCell, cmp::Ordering, fs, path::Path, rc::Rc};

impl HomebrewApp {
    #[must_use]
    pub fn maybe_from_path(path: &Path) -> Option<Self> {
        if !path.is_dir() {
            return None;
        }

        let filename = path.file_name()?.to_str()?;
        if filename.starts_with('.') {
            return None;
        }

        let meta_path = path.join("meta.xml");
        let meta = fs::read_to_string(&meta_path).ok()?;
        let mut meta = quick_xml::de::from_str::<HomebrewAppMeta>(&meta).ok()?;

        // some apps seem to place " " in front of the name to prioritize themselves when sorting
        meta.name = meta.name.trim().to_shared_string();

        let size = fs_extra::dir::get_size(path).ok()?;

        #[allow(clippy::cast_precision_loss)]
        let size_mib = size as f32 / MIB;

        let icon_path = path.join("icon.png");
        let icon = Image::load_from_path(&icon_path).unwrap_or_default();

        let search_term = format!("{}\0{}", filename, meta.name)
            .to_lowercase()
            .to_shared_string();

        let app = Self {
            path: path.to_string_lossy().to_shared_string(),
            slug: filename.to_shared_string(),
            meta,
            size_mib,
            icon,
            search_term,
        };

        Some(app)
    }
}

pub fn get_compare_fn(
    config: Rc<Mirrored<Config, AppWindow>>,
) -> Box<dyn Fn(&HomebrewApp, &HomebrewApp) -> Ordering> {
    Box::new(move |a, b| {
        let config = config.borrow();

        match config.contents.sort_by {
            SortBy::NameAscending => a.meta.name.cmp(&b.meta.name),
            SortBy::NameDescending => b.meta.name.cmp(&a.meta.name),
            SortBy::SizeAscending => a.size_mib.total_cmp(&b.size_mib),
            SortBy::SizeDescending => b.size_mib.total_cmp(&a.size_mib),
        }
    })
}

pub fn get_filter_fn(
    query_lowercase: Rc<RefCell<SharedString>>,
) -> Box<dyn Fn(&HomebrewApp) -> bool> {
    Box::new(move |app| {
        let query_lowercase = query_lowercase.borrow();

        if query_lowercase.is_empty() {
            return true;
        }

        app.search_term.contains(query_lowercase.as_str())
    })
}

pub fn scan_drive(root_dir: &Path) -> Vec<HomebrewApp> {
    let mut apps = Vec::new();
    let apps_dir = root_dir.join("apps");

    let Ok(entries) = fs::read_dir(&apps_dir) else {
        return apps;
    };

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if let Some(game) = HomebrewApp::maybe_from_path(&path) {
            apps.push(game);
        }
    }

    apps
}
