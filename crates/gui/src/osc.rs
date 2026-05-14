// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{AppWindow, Dispatcher, DisplayedOscApp, Message, util::MIB};
use slint::{ComponentHandle, Image, ToSharedString, Weak};
use std::fs;
use time::UtcDateTime;
use twbm_core::{data_dir::DATA_DIR, osc::OscApp};

impl From<&OscApp> for DisplayedOscApp {
    fn from(app: &OscApp) -> Self {
        let icon_path = DATA_DIR.join(format!("osc-icons/{}.png", app.meta.slug));
        let icon = Image::load_from_path(&icon_path).unwrap_or_default();

        let release_date = match UtcDateTime::from_unix_timestamp(app.meta.release_date) {
            Ok(date) => date.date().to_shared_string(),
            Err(_) => app.meta.release_date.to_shared_string(),
        };

        Self {
            slug: app.meta.slug.to_shared_string(),
            icon,
            name: app.meta.name.to_shared_string(),
            version: app.meta.version.to_shared_string(),
            release_date,
            short_description: app.meta.description.short.to_shared_string(),
            long_description: app.meta.description.long.to_shared_string(),
            author: app.meta.author.to_shared_string(),
            uncompressed_size_mib: app.meta.uncompressed_size as f32 / MIB,
        }
    }
}

pub fn download_icons(apps: &[OscApp], weak: Weak<AppWindow>) {
    let _ = fs::create_dir_all(DATA_DIR.join("osc-icons"));

    for (i, app) in apps.iter().enumerate() {
        if app.download_icon(&DATA_DIR).is_ok() {
            let _ = weak.upgrade_in_event_loop(move |app| {
                app.global::<Dispatcher<'_>>()
                    .invoke_dispatch(Message::ReloadOscIcon, i.to_shared_string());
            });
        }
    }
}
