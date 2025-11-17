// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::messages::Message;
use crate::{
    games::{Game, GameID},
    http,
    tasks::TaskProcessor,
};
use anyhow::Result;
use std::{fs, path::Path};

fn download_cheats_for_game(txt_cheatcodespath: &Path, game_id: &[u8; 6]) -> Result<()> {
    let path = txt_cheatcodespath
        .join(game_id.as_str())
        .with_extension("txt");

    if path.exists() {
        return Ok(());
    }

    let url = format!(
        "https://web.archive.org/web/202009if_/geckocodes.org/txt.php?txt={}",
        game_id.as_str()
    );
    http::download_file(&url, &path)?;

    Ok(())
}

pub fn spawn_download_cheats_task(
    task_processor: &TaskProcessor,
    mount_point: &Path,
    games: Box<[Game]>,
) {
    let txt_cheatcodespath = mount_point.join("txtcodes");

    task_processor.spawn(move |msg_sender| {
        msg_sender.send(Message::UpdateStatus(
            "📓 Downloading cheats...".to_string(),
        ))?;

        fs::create_dir_all(&txt_cheatcodespath)?;

        for game in &games {
            msg_sender.send(Message::UpdateStatus(format!(
                "📓 Downloading cheats... ({})",
                &game.display_title
            )))?;

            if let Err(e) = download_cheats_for_game(&txt_cheatcodespath, &game.id) {
                let context = format!("Failed to download cheats for {}", &game.display_title);
                msg_sender.send(Message::NotifyError(e.context(context)))?;
            }
        }

        msg_sender.send(Message::NotifyInfo("📓 Cheats downloaded".to_string()))?;

        Ok(())
    });
}
