// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use std::fs;

use crate::{TaskType, config, games, http::AGENT, tasks};
use anyhow::Result;

fn get_lang(id: &str) -> &'static str {
    let region_code = id.chars().nth(3).unwrap_or_default();

    match region_code {
        'E' | 'N' => "US",
        'J' => "JA",
        'K' | 'Q' | 'T' => "KO",
        'R' => "RU",
        'W' => "ZH",
        _ => "EN",
    }
}

pub fn download_covers() -> Result<()> {
    let games = games::list()?;

    let images_dir = config::get()
        .mount_point
        .join("apps")
        .join("usbloader_gx")
        .join("images");

    fs::create_dir_all(&images_dir)?;

    for game in games {
        let path = images_dir.join(&game.id).with_extension("png");

        tasks::spawn(Box::new(move |weak| {
            weak.upgrade_in_event_loop(move |handle| {
                handle.set_task_type(TaskType::DownloadingCovers);
            })?;

            if !path.exists() {
                let url = format!(
                    "https://art.gametdb.com/wii/cover3D/{}/{}.png",
                    get_lang(&game.id),
                    &game.id
                );
                let bytes = AGENT.get(&url).call()?.body_mut().read_to_vec()?;
                fs::write(&path, bytes)?;
            }

            Ok(())
        }))?;
    }

    Ok(())
}
