// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::TxtCodesSource,
    games::{game::Game, game_id::GameID, game_list::GameList, id_map},
    http_util,
    message::Message,
    state::State,
};
use anyhow::{Result, anyhow};
use iced::{
    Task,
    futures::TryFutureExt,
    task::{Sipper, sipper},
};
use std::{
    fs,
    path::{Path, PathBuf},
};

impl TxtCodesSource {
    pub fn get_txtcode(self, game_id: GameID) -> Result<Vec<u8>> {
        match self {
            Self::WebArchive => {
                let url = format!(
                    "https://raw.githubusercontent.com/mq1/GeckoArchive/refs/heads/main/codes/{}.txt",
                    game_id.as_str()
                );

                http_util::get(&url)
            }
            Self::GameHacking => {
                let gamehacking_id =
                    id_map::get_ghid(game_id).ok_or(anyhow!("Could not find gamehacks id"))?;

                let form = format!(
                    "format=Text&filename={}&sysID=22&gamID={}&download=true",
                    game_id.as_str(),
                    gamehacking_id
                );

                http_util::send_form("https://gamehacking.org/inc/sub.exportCodes.php", &form)
            }
            Self::Rc24 => {
                let url = format!("https://codes.rc24.xyz/txt.php?txt={}", game_id.as_str());
                http_util::get(&url)
            }
        }
    }
}

fn download_cheats_for_game(
    mount_point: &Path,
    source: TxtCodesSource,
    game_id: GameID,
) -> Result<()> {
    let parent = mount_point.join("txtcodes");
    let path = parent.join(game_id.as_str()).with_extension("txt");

    if path.exists() {
        return Ok(());
    }

    let txtcode = source.get_txtcode(game_id)?;

    fs::create_dir_all(parent)?;
    fs::write(&path, txtcode)?;

    Ok(())
}

pub fn get_download_cheats_for_game_task(state: &State, game: &Game) -> Task<Message> {
    let mount_point = state.config.mount_point().clone();
    let source = state.config.txt_codes_source();
    let game_id = game.id();
    let game_title = game.title().clone();

    Task::perform(
        async move { download_cheats_for_game(&mount_point, source, game_id) }
            .map_err(move |e| format!("Failed to download cheats for {game_title}: {e:#}")),
        Message::EmptyResult,
    )
}

fn get_download_chats_for_all_games_sipper(
    mount_point: PathBuf,
    game_list: GameList,
    source: TxtCodesSource,
) -> impl Sipper<String, String> {
    sipper(async move |mut progress| {
        for game in game_list.iter() {
            if let Err(e) = download_cheats_for_game(&mount_point, source, game.id()) {
                let msg = format!("Failed to download cheats for {}: {}", game.title(), e);
                progress.send(msg).await;
            }
        }

        "Finished downloading cheats for all games".to_string()
    })
}

pub fn get_download_cheats_for_all_games_task(state: &State) -> Task<Message> {
    Task::sip(
        get_download_chats_for_all_games_sipper(
            state.config.mount_point().clone(),
            state.game_list.clone(),
            state.config.txt_codes_source(),
        ),
        Message::GenericError,
        Message::GenericSuccess,
    )
}
