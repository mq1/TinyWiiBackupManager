// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::app::App;
use crate::messages::Message;
use crate::{games::GameID, http};
use anyhow::Result;
use std::{fs, path::Path};

fn download_banner_for_game(cache_bnr_path: &Path, game_id: GameID) -> Result<()> {
    let path = cache_bnr_path.join(game_id.as_str()).with_extension("bnr");

    if path.exists() {
        return Ok(());
    }

    let url = format!("https://banner.rc24.xyz/{}.bnr", game_id.as_str());
    let fallback_url = format!("https://banner.rc24.xyz/{}.bnr", game_id.as_partial());

    let bytes = match http::get(&url) {
        Ok(body) => body,
        Err(_) => http::get(&fallback_url)?,
    };

    fs::write(&path, bytes)?;

    Ok(())
}

pub fn spawn_download_banners_task(app: &App) {
    let cache_bnr_path = app.config.contents.mount_point.join("cache_bnr");

    let gc_games = app
        .games
        .iter()
        .filter(|g| !g.is_wii)
        .map(|g| (g.id, g.display_title.clone()))
        .collect::<Box<[_]>>();

    app.task_processor.spawn(move |msg_sender| {
        msg_sender.send(Message::UpdateStatus(format!(
            "{} Downloading banners...",
            egui_phosphor::regular::IMAGE
        )))?;

        fs::create_dir_all(&cache_bnr_path)?;

        for game in &gc_games {
            msg_sender.send(Message::UpdateStatus(format!(
                "{} Downloading banners... ({})",
                egui_phosphor::regular::IMAGE,
                &game.1
            )))?;

            if let Err(e) = download_banner_for_game(&cache_bnr_path, game.0) {
                let context = format!(
                    "{} Failed to download banner for {}",
                    egui_phosphor::regular::IMAGE,
                    &game.1
                );
                msg_sender.send(Message::NotifyError(e.context(context)))?;
            }
        }

        msg_sender.send(Message::NotifyInfo(format!(
            "{} Banners downloaded",
            egui_phosphor::regular::IMAGE
        )))?;

        Ok(())
    });
}
