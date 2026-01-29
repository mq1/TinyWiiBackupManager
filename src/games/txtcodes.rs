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
    sync::Arc,
};

impl TxtCodesSource {
    pub fn get_txtcode(self, game_id: GameID) -> Result<Vec<u8>> {
        match self {
            Self::GameHacking => {
                let gamehacking_id = id_map::get_gamehacking_id(game_id)
                    .ok_or(anyhow!("Could not find gamehacks id"))?;

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
    game: &Game,
) -> Result<String> {
    let parent = mount_point.join("txtcodes");
    let path = parent.join(game.id().as_str()).with_extension("txt");

    if path.exists() {
        return Ok(format!("Cheats for {} already present", game.title()));
    }

    let txtcode = source.get_txtcode(game.id())?;
    fs::create_dir_all(parent)?;
    fs::write(&path, txtcode)?;

    Ok(format!("Cheats for {} downloaded", game.title()))
}

pub fn get_download_cheats_for_game_task(state: &State, game: Game) -> Task<Message> {
    let mount_point = state.config.mount_point().clone();
    let source = state.config.txt_codes_source();

    Task::perform(
        async move { download_cheats_for_game(&mount_point, source, &game) }.map_err(Arc::new),
        Message::GenericResult,
    )
}

fn get_download_chats_for_all_games_sipper(
    mount_point: PathBuf,
    game_list: GameList,
    source: TxtCodesSource,
) -> impl Sipper<String, Arc<anyhow::Error>> {
    sipper(async move |mut progress| {
        for game in game_list.into_iter() {
            if let Err(e) = download_cheats_for_game(&mount_point, source, &game) {
                progress.send(Arc::new(e)).await;
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
