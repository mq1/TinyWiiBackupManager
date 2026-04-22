// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{Logic, OscApp, OscAppMeta, UREQ_AGENT, data_dir::DATA_DIR};
use anyhow::{Result, bail};
use slint::{Image, SharedString, ToSharedString, Weak};
use std::{
    cell::RefCell,
    fs,
    rc::Rc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use time::UtcDateTime;

const CONTENTS_URL: &str = "https://hbb1.oscwii.org/api/v4/contents";

pub fn load_contents(force_refresh: bool) -> Result<(Vec<OscApp>, i32, i32)> {
    let cached_contents_path = DATA_DIR.join("osc-cache.json");

    let last_refresh = cached_contents_path
        .metadata()
        .ok()
        .and_then(|m| m.modified().ok())
        .unwrap_or(UNIX_EPOCH);

    let a_day_ago = SystemTime::now() - Duration::from_hours(24);
    let should_refresh = force_refresh || last_refresh < a_day_ago;

    let (raw, last_refresh) = if should_refresh {
        let body = UREQ_AGENT
            .get(CONTENTS_URL)
            .call()?
            .body_mut()
            .read_to_string()?;

        fs::write(cached_contents_path, &body)?;

        (body, SystemTime::now())
    } else {
        let raw = fs::read_to_string(cached_contents_path)?;
        (raw, last_refresh)
    };

    let escaped = escape_str(&raw);
    let apps = serde_json::from_str::<Vec<OscAppMeta>>(&escaped)?;

    let apps = apps
        .into_iter()
        .map(|meta| {
            let release_date = match UtcDateTime::from_unix_timestamp(meta.release_date) {
                Ok(datetime) => datetime.date().to_shared_string(),
                Err(_) => meta.release_date.to_shared_string(),
            };

            let search_term = format!("{}\0{}", meta.slug, meta.name).to_shared_string();

            let icon_path = DATA_DIR.join(format!("osc-icons/{}.png", meta.slug));
            let icon = Image::load_from_path(&icon_path).unwrap_or_default();

            OscApp {
                release_date,
                icon,
                search_term,
                meta,
            }
        })
        .collect();

    let elapsed_mins = last_refresh.elapsed().unwrap_or_default().as_secs() / 60;
    let elapsed_hours = (elapsed_mins / 60) as i32;
    let elapsed_mins = (elapsed_mins % 60) as i32;

    Ok((apps, elapsed_hours, elapsed_mins))
}

fn download_icon(slug: &str, icon_url: &str) -> Result<()> {
    let icon_path = DATA_DIR.join(format!("osc-icons/{slug}.png"));

    if icon_path.exists() {
        bail!("Icon already exists");
    }

    let body = UREQ_AGENT.get(icon_url).call()?.body_mut().read_to_vec()?;
    fs::write(&icon_path, &body)?;

    Ok(())
}

pub fn load_icons(apps: impl IntoIterator<Item = OscAppMeta>, weak: Weak<Logic<'static>>) {
    let icon_urls = apps
        .into_iter()
        .map(|app| (app.slug.to_string(), app.assets.icon.url.to_string()))
        .collect::<Vec<_>>();

    let cache_dir = DATA_DIR.join("osc-icons");
    let _ = fs::create_dir_all(&cache_dir);

    let _ = std::thread::spawn(move || {
        for (i, (slug, url)) in icon_urls.iter().enumerate() {
            if download_icon(slug, url).is_ok() {
                let _ = weak.upgrade_in_event_loop(move |logic| {
                    logic.invoke_reload_osc_icon(i as i32);
                });
            }
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

pub fn get_filter_fn(query_lowercase: Rc<RefCell<SharedString>>) -> impl Fn(&OscApp) -> bool {
    move |app| {
        let query_lowercase = query_lowercase.borrow();

        if query_lowercase.is_empty() {
            return true;
        }

        app.search_term.contains(query_lowercase.as_str())
    }
}
