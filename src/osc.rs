// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{OscAppMeta, OscContents, State, USER_AGENT};
use anyhow::Result;
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use image::ImageFormat;
use slint::{Image, Model, ModelRc, Rgba8Pixel, SharedPixelBuffer, SharedString, VecModel, Weak};
use std::{
    fs,
    path::Path,
    rc::Rc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

const CONTENTS_URL: &str = "https://hbb1.oscwii.org/api/v4/contents";

impl OscContents {
    pub fn fetch(data_dir: &Path) -> Result<(String, SystemTime)> {
        let cached_contents_path = data_dir.join("osc-cache.json");

        let last_refresh = cached_contents_path
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .unwrap_or(UNIX_EPOCH);

        let should_refresh = last_refresh < SystemTime::now() - Duration::from_hours(24);

        if should_refresh {
            let resp = minreq::get(CONTENTS_URL)
                .with_header("User-Agent", USER_AGENT)
                .send()?;

            let raw = String::from_utf8(resp.into_bytes())?;
            fs::write(cached_contents_path, &raw)?;

            Ok((raw, SystemTime::now()))
        } else {
            let raw = fs::read_to_string(cached_contents_path)?;
            Ok((raw, last_refresh))
        }
    }

    pub fn load(raw: String, last_refresh: SystemTime) -> Result<Self> {
        let sanitized = raw.replace("\\\"", "”"); // for some reason slint doesn't like this
        let apps = serde_json::from_str::<Vec<OscAppMeta>>(&sanitized)?;
        let icons = vec![Image::default(); apps.len()];
        let apps = ModelRc::from(Rc::new(VecModel::from(apps)));
        let icons = ModelRc::from(Rc::new(VecModel::from(icons)));

        let elapsed_mins = last_refresh.elapsed().unwrap_or_default().as_secs() / 60;
        let elapsed_hours = (elapsed_mins / 60) as i32;
        let elapsed_mins = (elapsed_mins % 60) as i32;

        let contents = Self {
            apps,
            err: SharedString::new(),
            icons,
            last_refresh: (elapsed_hours, elapsed_mins),
            filter: SharedString::new(),
            filtered_apps: ModelRc::default(),
        };

        Ok(contents)
    }
}

pub fn load_icons(apps: &ModelRc<OscAppMeta>, data_dir: &Path, weak: Weak<State<'static>>) {
    let icon_urls = apps
        .iter()
        .map(|app| (app.slug.to_string(), app.assets.icon.url.to_string()))
        .collect::<Vec<_>>();

    let cache_dir = data_dir.join("osc-icons");

    let _ = std::thread::spawn(move || {
        let res = || -> Result<()> {
            fs::create_dir_all(&cache_dir)?;

            for (i, (slug, url)) in icon_urls.iter().enumerate() {
                let icon_path = cache_dir.join(format!("{slug}.png"));

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

pub fn fuzzy_search(apps: &ModelRc<OscAppMeta>, query: &str) -> ModelRc<OscAppMeta> {
    let matcher = SkimMatcherV2::default();

    let mut filtered_apps = Vec::new();
    for app in apps.iter() {
        let name_score = matcher.fuzzy_match(&app.name, query);
        let author_score = matcher.fuzzy_match(&app.author, query);

        let score = match (name_score, author_score) {
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
