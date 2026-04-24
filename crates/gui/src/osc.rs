// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{DisplayedOscApp, Logic, data_dir::DATA_DIR, util::MIB};
use slint::{Image, SharedString, ToSharedString, Weak};
use std::{cell::RefCell, fs, rc::Rc};
use time::UtcDateTime;
use twbm_core::osc::OscAppMeta;

impl DisplayedOscApp {
    pub fn new(meta: &OscAppMeta, idx: usize) -> Self {
        let search_term = format!("{}\0{}", meta.name, meta.slug).to_lowercase();
        let icon_path = DATA_DIR.join(format!("osc-icons/{}.png", meta.slug));
        let icon = Image::load_from_path(&icon_path).unwrap_or_default();

        let release_date = match UtcDateTime::from_unix_timestamp(meta.release_date) {
            Ok(date) => date.date().to_shared_string(),
            Err(_) => meta.release_date.to_shared_string(),
        };

        Self {
            idx: idx as i32,
            slug: meta.slug.to_shared_string(),
            icon,
            name: meta.name.to_shared_string(),
            version: meta.version.to_shared_string(),
            release_date,
            short_description: meta.description.short.to_shared_string(),
            long_description: meta.description.long.to_shared_string(),
            search_term: search_term.to_shared_string(),
            author: meta.author.to_shared_string(),
            uncompressed_size_mib: meta.uncompressed_size as f32 / MIB,
        }
    }
}

pub fn download_icons(apps: Vec<(usize, OscAppMeta)>, weak: Weak<Logic<'static>>) {
    let _ = fs::create_dir_all(DATA_DIR.join("osc-icons"));

    for (i, app) in apps {
        if app.download_icon(&DATA_DIR).is_ok() {
            let _ = weak.upgrade_in_event_loop(move |logic| {
                logic.invoke_reload_osc_icon(i as i32);
            });
        }
    }
}

pub fn get_filter_fn(
    query_lowercase: Rc<RefCell<SharedString>>,
) -> impl Fn(&DisplayedOscApp) -> bool {
    move |app| {
        let query_lowercase = query_lowercase.borrow();

        if query_lowercase.is_empty() {
            return true;
        }

        app.search_term.contains(query_lowercase.as_str())
    }
}
