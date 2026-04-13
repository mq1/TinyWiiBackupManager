// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{OscAppMeta, OscContents, State, USER_AGENT};
use anyhow::Result;
use image::ImageFormat;
use slint::{Image, Model, ModelRc, Rgba8Pixel, SharedPixelBuffer, SharedString, VecModel, Weak};
use std::{
    fs,
    path::Path,
    rc::Rc,
    time::{Duration, SystemTime},
};

const CONTENTS_URL: &str = "https://hbb1.oscwii.org/api/v4/contents";

impl OscContents {
    pub fn fetch(data_dir: &Path) -> Result<String> {
        let cached_contents_path = data_dir.join("osc-cache.json");

        let needs_to_be_downloaded = !cached_contents_path.exists()
            || (cached_contents_path.metadata()?.modified()?
                < (SystemTime::now() - Duration::from_hours(24)));

        let raw = if needs_to_be_downloaded {
            let resp = minreq::get(CONTENTS_URL)
                .with_header("User-Agent", USER_AGENT)
                .send()?;

            let raw = String::from_utf8(resp.into_bytes())?;
            fs::write(cached_contents_path, &raw)?;

            raw
        } else {
            fs::read_to_string(cached_contents_path)?
        };

        Ok(raw)
    }

    pub fn load(raw: String) -> Result<Self> {
        let sanitized = raw.replace("\\\"", "”");
        let apps = serde_json::from_str::<Vec<OscAppMeta>>(&sanitized)?;
        let icons = vec![Image::default(); apps.len()];
        let apps = ModelRc::from(Rc::new(VecModel::from(apps)));
        let icons = ModelRc::from(Rc::new(VecModel::from(icons)));

        let contents = Self {
            apps,
            err: SharedString::new(),
            icons,
        };

        Ok(contents)
    }
}

pub fn load_icons(apps: &ModelRc<OscAppMeta>, data_dir: &Path, weak: Weak<State<'static>>) {
    let icon_urls = apps
        .iter()
        .map(|app| app.assets.icon.url.to_string())
        .collect::<Vec<_>>();

    let cache_dir = data_dir.join("osc-icons");

    let _ = std::thread::spawn(move || {
        let res = || -> Result<()> {
            fs::create_dir_all(&cache_dir)?;

            for (i, url) in icon_urls.iter().enumerate() {
                let icon_path = cache_dir.join(format!("{i}.png"));

                let bytes = if !icon_path.exists() {
                    let resp = minreq::get(url)
                        .with_header("User-Agent", USER_AGENT)
                        .send()?;

                    let bytes = resp.into_bytes();
                    fs::write(&icon_path, &bytes)?;
                    bytes
                } else {
                    fs::read(&icon_path)?
                };

                let bytes =
                    image::load_from_memory_with_format(&bytes, ImageFormat::Png)?.into_rgba8();

                let _ = weak.upgrade_in_event_loop(move |state| {
                    let buffer = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
                        bytes.as_raw(),
                        bytes.width(),
                        bytes.height(),
                    );
                    let icon = Image::from_rgba8(buffer);

                    let model = state.get_osc_contents().icons;
                    model.set_row_data(i, icon);
                });
            }

            Ok(())
        }();

        if let Err(e) = res {
            let _ = weak.upgrade_in_event_loop(move |state| {
                state.push_notification(e.into());
            });
        }
    });
}
