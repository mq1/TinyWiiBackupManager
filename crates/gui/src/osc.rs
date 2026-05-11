// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{DisplayedOscApp, Logic, util::MIB};
use slint::{Image, SharedString, ToSharedString, Weak};
use std::{cell::RefCell, fs, rc::Rc};
use time::UtcDateTime;
use twbm_core::{data_dir::DATA_DIR, osc::OscApp};

impl From<&OscApp> for DisplayedOscApp {
    fn from(app: &OscApp) -> Self {
        let search_term = format!("{}\0{}", app.meta.name, app.meta.slug).to_lowercase();
        let icon_path = DATA_DIR.join(format!("osc-icons/{}.png", app.meta.slug));
        let icon = Image::load_from_path(&icon_path).unwrap_or_default();

        let release_date = match UtcDateTime::from_unix_timestamp(app.meta.release_date) {
            Ok(date) => date.date().to_shared_string(),
            Err(_) => app.meta.release_date.to_shared_string(),
        };

        Self {
            uid: app.uid,
            slug: app.meta.slug.to_shared_string(),
            icon,
            name: app.meta.name.to_shared_string(),
            version: app.meta.version.to_shared_string(),
            release_date,
            short_description: app.meta.description.short.to_shared_string(),
            long_description: app.meta.description.long.to_shared_string(),
            search_term: search_term.to_shared_string(),
            author: app.meta.author.to_shared_string(),
            uncompressed_size_mib: app.meta.uncompressed_size as f32 / MIB,
        }
    }
}

pub fn download_icons(apps: &[OscApp], weak: Weak<Logic<'static>>) {
    let _ = fs::create_dir_all(DATA_DIR.join("osc-icons"));

    for (i, app) in apps.iter().enumerate() {
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
