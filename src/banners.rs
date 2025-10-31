// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{app::App, games::GameID, http::AGENT, tasks::BackgroundMessage};
use anyhow::Result;
use std::{fs, path::Path};

fn download_banner_for_game(cache_bnr_path: &Path, game_id: &GameID) -> Result<()> {
    let path = cache_bnr_path.join(game_id.as_str()).with_extension("bnr");

    if path.exists() {
        return Ok(());
    }

    let url = format!("https://banner.rc24.xyz/{}.bnr", game_id.as_str());
    let fallback_url = format!("https://banner.rc24.xyz/{}.bnr", game_id.as_partial());
    println!("Downloading banner for {} from {}", game_id.as_str(), url);
    println!("Fallback URL: {}", fallback_url);

    let bytes = match AGENT.get(&url).call() {
        Ok(mut resp) => resp.body_mut().read_to_vec()?,
        Err(_) => AGENT.get(&fallback_url).call()?.body_mut().read_to_vec()?,
    };

    fs::write(&path, bytes)?;

    Ok(())
}

pub fn spawn_download_banners_task(app: &mut App) {
    let cache_bnr_path = app.config.contents.mount_point.join("cache_bnr");
    let games = app.games.clone();

    app.task_processor.spawn(move |msg_sender| {
        msg_sender.send(BackgroundMessage::UpdateStatus(
            "🖻 Downloading banners...".to_string(),
        ))?;

        fs::create_dir_all(&cache_bnr_path)?;

        for game in games.iter().filter(|g| !g.is_wii) {
            msg_sender.send(BackgroundMessage::UpdateStatus(format!(
                "🖻 Downloading banners... ({})",
                &game.display_title
            )))?;

            if let Err(e) = download_banner_for_game(&cache_bnr_path, &game.id) {
                msg_sender.send(BackgroundMessage::NotifyError(format!(
                    "Failed to download banner for {}: {}",
                    &game.display_title, e
                )))?;
            }
        }

        msg_sender.send(BackgroundMessage::NotifyInfo(
            "🖻 Banners downloaded".to_string(),
        ))?;

        Ok(())
    });
}
