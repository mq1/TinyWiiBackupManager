// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{Logic, UREQ_AGENT, data_dir::DATA_DIR};
use anyhow::{Result, bail};
use arrayvec::ArrayString;
use slint::Weak;
use std::fs;

#[must_use]
fn lang_str(game_id: ArrayString<6>) -> &'static str {
    match game_id.chars().nth(3) {
        Some('E' | 'N') => "US",
        Some('J') => "JA",
        Some('K' | 'Q' | 'T') => "KO",
        Some('R') => "RU",
        Some('W') => "ZH",
        _ => "EN",
    }
}

fn download_cover(game_id: ArrayString<6>) -> Result<()> {
    let cover_path = DATA_DIR.join(format!("covers/{game_id}.png"));

    if cover_path.exists() {
        bail!("Cover already exists");
    }

    let cover_url = format!(
        "https://art.gametdb.com/wii/cover3D/{}/{}.png",
        lang_str(game_id),
        game_id,
    );

    let body = UREQ_AGENT.get(cover_url).call()?.body_mut().read_to_vec()?;
    fs::write(&cover_path, &body)?;

    Ok(())
}

pub fn download_covers(ids: Vec<ArrayString<6>>, weak: Weak<Logic<'static>>) {
    let _ = fs::create_dir_all(DATA_DIR.join("covers"));

    let _ = std::thread::spawn(move || {
        for (i, game_id) in ids.into_iter().enumerate() {
            if download_cover(game_id).is_ok() {
                let _ = weak.upgrade_in_event_loop(move |logic| {
                    logic.invoke_reload_cover(i as i32);
                });
            }
        }

        let _ = weak.upgrade_in_event_loop(move |logic| {
            logic.invoke_finished_downloading_covers();
        });
    });
}
