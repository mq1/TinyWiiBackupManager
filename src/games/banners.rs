// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    games::{game_id::GameID, game_list::GameList},
    http_util,
    message::Message,
    state::State,
};
use anyhow::Result;
use iced::{
    Task,
    task::{Sipper, sipper},
};
use std::{
    fs,
    path::{Path, PathBuf},
};

fn download_banner_for_game(mount_point: &Path, game_id: GameID) -> Result<()> {
    let parent = mount_point.join("cache_bnr");
    let path = parent.join(game_id.as_str()).with_extension("bnr");

    if path.exists() {
        return Ok(());
    }

    let url = format!("https://banner.rc24.xyz/{}.bnr", game_id.as_str());
    let fallback_url = format!("https://banner.rc24.xyz/{}.bnr", game_id.as_partial_str());

    let bytes = match http_util::get(&url) {
        Ok(body) => body,
        Err(_) => http_util::get(&fallback_url)?,
    };

    fs::create_dir_all(&parent)?;
    fs::write(&path, bytes)?;

    Ok(())
}

fn get_download_banners_sipper(
    mount_point: PathBuf,
    game_list: GameList,
) -> impl Sipper<String, String> {
    sipper(async move |mut progress| {
        for game in game_list.iter().filter(|g| !g.is_wii()) {
            if let Err(e) = download_banner_for_game(&mount_point, game.id()) {
                let msg = format!("Failed to download banner for {}: {:#}", game.title(), e);
                progress.send(msg).await;
            }
        }

        "Banners downloaded".to_string()
    })
}

pub fn get_download_banners_task(state: &State) -> Task<Message> {
    Task::sip(
        get_download_banners_sipper(state.config.mount_point().clone(), state.game_list.clone()),
        Message::GenericError,
        Message::GenericSuccess,
    )
}
