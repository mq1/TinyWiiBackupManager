// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{app::App, games::GameID, http::AGENT, tasks::BackgroundMessage};
use anyhow::Result;
use std::{fs, path::Path};

fn download_cheats_for_game(txt_cheatcodespath: &Path, game_id: &GameID) -> Result<()> {
    let path = txt_cheatcodespath
        .join(game_id.as_str())
        .with_extension("txt");

    if path.exists() {
        return Ok(());
    }

    let url = format!("https://codes.rc24.xyz/txt.php?txt={}", game_id.as_str());
    let bytes = AGENT.get(&url).call()?.body_mut().read_to_vec()?;
    fs::write(&path, bytes)?;

    Ok(())
}

pub fn download_cheats(app: &mut App) {
    let txt_cheatcodespath = app.config.contents.mount_point.join("txtcodes");
    let games = app.games.clone();

    app.task_processor.spawn(move |msg_sender| {
        msg_sender.send(BackgroundMessage::UpdateStatus(
            "📓 Downloading cheats...".to_string(),
        ))?;

        fs::create_dir_all(&txt_cheatcodespath)?;

        for game in &games {
            msg_sender.send(BackgroundMessage::UpdateStatus(format!(
                "📓 Downloading cheats... ({})",
                &game.display_title
            )))?;

            if let Err(e) = download_cheats_for_game(&txt_cheatcodespath, &game.id) {
                msg_sender.send(BackgroundMessage::NotifyError(format!(
                    "Failed to download cheats for {}: {}",
                    &game.display_title, e
                )))?;
            }
        }

        msg_sender.send(BackgroundMessage::NotifyInfo(
            "📓 Cheats downloaded".to_string(),
        ))?;

        Ok(())
    });
}
