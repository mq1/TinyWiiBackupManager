// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{DisplayedHomebrewApp, util::MIB};
use slint::{Image, Rgba8Pixel, SharedPixelBuffer, SharedString, ToSharedString};
use std::{cell::RefCell, cmp::Ordering, path::Path, rc::Rc};
use twbm_core::{
    config::{Config, SortBy},
    homebrew_app::HomebrewApp,
};

impl DisplayedHomebrewApp {
    pub fn new(app: &HomebrewApp, idx: usize) -> Self {
        let buffer = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
            app.icon_rgba8.as_raw(),
            app.icon_rgba8.width(),
            app.icon_rgba8.height(),
        );
        let icon = Image::from_rgba8(buffer);

        let slug = app.path.file_name().unwrap_or_default().to_string_lossy();
        let search_term = format!("{}\0{}", app.meta.name, slug).to_lowercase();

        Self {
            idx: idx as i32,
            slug: slug.to_shared_string(),
            path: app.path.to_string_lossy().to_shared_string(),
            size_mib: app.size as f32 / MIB,
            icon,
            name: app.meta.name.to_shared_string(),
            coder: app.meta.coder.to_shared_string(),
            version: app.meta.version.to_shared_string(),
            release_date: app.meta.release_date.to_shared_string(),
            short_description: app.meta.short_description.to_shared_string(),
            long_description: app.meta.long_description.to_shared_string(),
            search_term: search_term.to_shared_string(),
            osc_idx: app.osc_idx,
        }
    }
}

pub fn get_compare_fn(
    config: Rc<RefCell<Config>>,
) -> impl Fn(&DisplayedHomebrewApp, &DisplayedHomebrewApp) -> Ordering {
    move |a, b| {
        let config = config.borrow();

        match config.contents.sort_by {
            SortBy::NameDescending => a.name.cmp(&b.name),
            SortBy::NameAscending => b.name.cmp(&a.name),
            SortBy::SizeDescending => a.size_mib.total_cmp(&b.size_mib),
            SortBy::SizeAscending => b.size_mib.total_cmp(&a.size_mib),
        }
    }
}

pub fn get_filter_fn(
    query_lowercase: Rc<RefCell<SharedString>>,
) -> impl Fn(&DisplayedHomebrewApp) -> bool {
    move |app| {
        let query_lowercase = query_lowercase.borrow();

        if query_lowercase.is_empty() {
            return true;
        }

        app.search_term.contains(query_lowercase.as_str())
    }
}

pub fn scan_drive(root_path: &Path) -> Vec<HomebrewApp> {
    let apps_dir = root_path.join("apps");
    twbm_core::homebrew_app::scan_dir(&apps_dir)
}
