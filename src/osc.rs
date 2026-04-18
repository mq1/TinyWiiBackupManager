// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{OscAppMeta, USER_AGENT, data_dir::DATA_DIR};
use anyhow::Result;
use slint::{SharedString, ToSharedString};
use std::{
    cell::RefCell,
    fs,
    rc::Rc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use time::UtcDateTime;

const CONTENTS_URL: &str = "https://hbb1.oscwii.org/api/v4/contents";

pub fn load_contents(force_refresh: bool) -> Result<(Vec<OscAppMeta>, i32, i32)> {
    let cached_contents_path = DATA_DIR.join("osc-cache.json");

    let last_refresh = cached_contents_path
        .metadata()
        .ok()
        .and_then(|m| m.modified().ok())
        .unwrap_or(UNIX_EPOCH);

    let a_day_ago = SystemTime::now() - Duration::from_hours(24);
    let should_refresh = force_refresh || last_refresh < a_day_ago;

    let (raw, last_refresh) = if should_refresh {
        let resp = minreq::get(CONTENTS_URL)
            .with_header("User-Agent", USER_AGENT)
            .send()?;

        let raw = String::from_utf8(resp.into_bytes())?;
        fs::write(cached_contents_path, &raw)?;

        (raw, SystemTime::now())
    } else {
        let raw = fs::read_to_string(cached_contents_path)?;
        (raw, last_refresh)
    };

    let escaped = escape_str(&raw);
    let mut apps = serde_json::from_str::<Vec<OscAppMeta>>(&escaped)?;

    #[allow(clippy::cast_possible_truncation)]
    for app in &mut apps {
        if let Ok(release_date) = app.release_date.parse()
            && let Ok(release_date) = UtcDateTime::from_unix_timestamp(release_date)
        {
            app.release_date = release_date.date().to_shared_string();
        }

        let search_term = format!("{}\0{}", app.slug, app.name)
            .to_lowercase()
            .to_shared_string();

        app.search_term = search_term;
    }

    let elapsed_mins = last_refresh.elapsed().unwrap_or_default().as_secs() / 60;
    let elapsed_hours = (elapsed_mins / 60) as i32;
    let elapsed_mins = (elapsed_mins % 60) as i32;

    Ok((apps, elapsed_hours, elapsed_mins))
}

// TODO
#[cfg(false)]
pub fn load_icons(apps: &ModelRc<OscAppMeta>, weak: Weak<State<'static>>) {
    let icon_urls = apps
        .iter()
        .map(|app| (app.i, app.slug.to_string(), app.assets.icon.url.to_string()))
        .collect::<Vec<_>>();

    let cache_dir = DATA_DIR.join("osc-icons");

    let _ = std::thread::spawn(move || {
        let res = || -> Result<()> {
            fs::create_dir_all(&cache_dir)?;

            for (i, slug, url) in icon_urls {
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
                    model.set_row_data(i as usize, icon);
                });
            }

            Ok(())
        }();

        if let Err(e) = res {
            let _ = weak.upgrade_in_event_loop(move |state| {
                state.invoke_notify_err(e.to_shared_string());
            });
        }
    });
}

// for some reason slint strings don't work without this
fn escape_str(s: &str) -> String {
    s.replace("\\\\", "/")
        .replace("\\\"", "'")
        .replace("\\n", "    ")
        .replace("\\t", "    ")
}

pub fn get_filter_fn(
    query_lowercase: Rc<RefCell<SharedString>>,
) -> Box<dyn Fn(&OscAppMeta) -> bool> {
    Box::new(move |app| {
        let query_lowercase = query_lowercase.borrow();

        if query_lowercase.is_empty() {
            return true;
        }

        app.search_term.contains(query_lowercase.as_str())
    })
}
