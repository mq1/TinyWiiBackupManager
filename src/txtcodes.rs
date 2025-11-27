// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::app::App;
use crate::games::Game;
use crate::messages::Message;
use crate::{games::GameID, http, id_map};
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TxtCodesSource {
    WebArchive,
    Rc24,
    GameHacking,
}

impl TxtCodesSource {
    pub fn get_txtcode(&self, game_id: [u8; 6], game_id_str: &str) -> Result<Vec<u8>> {
        match self {
            Self::WebArchive => {
                let url = format!(
                    "https://web.archive.org/web/202009if_/geckocodes.org/txt.php?txt={}",
                    game_id_str
                );

                http::get(&url).map_err(Into::into)
            }
            Self::Rc24 => {
                let url = format!("https://codes.rc24.xyz/txt.php?txt={}", game_id_str);
                http::get(&url).map_err(Into::into)
            }
            Self::GameHacking => {
                let gamehacking_id = id_map::get_gamehacking_id(game_id)
                    .ok_or(anyhow!("Could not find gamehacks id"))?;

                let form = [
                    ("format", "Text"),
                    ("filename", game_id_str),
                    ("sysID", "22"),
                    ("gamID", &gamehacking_id.to_string()),
                    ("download", "true"),
                ];

                http::send_form("https://gamehacking.org/inc/sub.exportCodes.php", form)
                    .map_err(Into::into)
            }
        }
    }
}

fn download_cheats_for_game(
    txt_cheatcodespath: &Path,
    source: TxtCodesSource,
    game_id: [u8; 6],
) -> Result<()> {
    let game_id_str = game_id.as_str();
    let path = txt_cheatcodespath.join(game_id_str).with_extension("txt");

    if path.exists() {
        return Ok(());
    }

    let txtcode = source.get_txtcode(game_id, game_id_str)?;
    fs::write(&path, txtcode)?;

    Ok(())
}

pub fn spawn_download_all_cheats_task(app: &App) {
    let txt_cheatcodespath = app.config.contents.mount_point.join("txtcodes");
    let source = app.config.contents.txt_codes_source;

    let games = app
        .games
        .iter()
        .map(|game| (game.id, game.display_title.clone()))
        .collect::<Box<[_]>>();

    app.task_processor.spawn(move |msg_sender| {
        msg_sender.send(Message::UpdateStatus(
            "📓 Downloading cheats...".to_string(),
        ))?;

        fs::create_dir_all(&txt_cheatcodespath)?;

        for game in &games {
            msg_sender.send(Message::UpdateStatus(format!(
                "📓 Downloading cheats... ({})",
                &game.1
            )))?;

            if let Err(e) = download_cheats_for_game(&txt_cheatcodespath, source, game.0) {
                let context = format!("Failed to download cheats for {}", &game.1);
                msg_sender.send(Message::NotifyError(e.context(context)))?;
            }
        }

        msg_sender.send(Message::NotifyInfo("📓 Cheats downloaded".to_string()))?;

        Ok(())
    });
}

pub fn spawn_download_cheats_task(app: &App, game: &Game) {
    let txt_cheatcodespath = app.config.contents.mount_point.join("txtcodes");
    let source = app.config.contents.txt_codes_source;

    let game_id = game.id;
    let display_title = game.display_title.clone();

    app.task_processor.spawn(move |msg_sender| {
        msg_sender.send(Message::UpdateStatus(format!(
            "📓 Downloading cheats... ({})",
            &display_title
        )))?;

        fs::create_dir_all(&txt_cheatcodespath)?;

        if let Err(e) = download_cheats_for_game(&txt_cheatcodespath, source, game_id) {
            let context = format!("Failed to download cheats for {}", &display_title);
            msg_sender.send(Message::NotifyError(e.context(context)))?;
        } else {
            msg_sender.send(Message::NotifyInfo("📓 Cheats downloaded".to_string()))?;
        }

        Ok(())
    });
}
