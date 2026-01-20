// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::SortBy,
    disc_info::DiscInfo,
    game_id::GameID,
    message::Message,
    state::State,
    util::{self, FuzzySearchable},
    wiitdb::GameInfo,
};
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use iced::{Task, futures::TryFutureExt};
use itertools::Itertools;
use size::Size;
use smol::{fs, io, stream::StreamExt};
use std::{ffi::OsString, path::PathBuf};

#[derive(Debug, Clone)]
pub struct Game {
    pub path: PathBuf,
    pub size: Size,
    pub title: String,
    pub id: GameID,
    pub disc_info: Option<Result<DiscInfo, String>>,
    pub wiitdb_info: Option<GameInfo>,
}

impl Game {
    pub async fn from_path(path: PathBuf) -> Option<Self> {
        if !path.is_dir() {
            return None;
        }

        let filename = path.file_name()?.to_str()?;
        if filename.starts_with('.') {
            return None;
        }

        let (title_str, id_str) = filename.split_once(" [")?;
        let id_str = id_str.strip_suffix(']')?;
        let title = title_str.to_string();
        let id = GameID::try_from(id_str).ok()?;

        let size = util::get_dir_size(path.clone()).await.unwrap_or_default();

        Some(Self {
            path,
            size,
            title,
            id,
            disc_info: None,
            wiitdb_info: None,
        })
    }

    pub fn get_path_uri(&self) -> OsString {
        self.path.as_os_str().to_os_string()
    }

    pub fn get_gametdb_uri(&self) -> OsString {
        format!("https://www.gametdb.com/Wii/{}", self.id.as_str()).into()
    }

    pub fn get_path_str(&self) -> &str {
        self.path.to_str().unwrap_or("Invalid path")
    }

    pub fn get_load_disc_info_task(&mut self, i: usize) -> Task<Message> {
        self.disc_info = None;

        let path = self.path.clone();

        Task::perform(
            DiscInfo::from_game_dir(path).map_err(|e| e.to_string()),
            move |res| Message::GotDiscInfo(i, res),
        )
    }
}

pub fn get_list_games_task(state: &State) -> Task<Message> {
    let drive_path = state.config.mount_point().to_path_buf();

    Task::perform(
        list(drive_path).map_err(|e| e.to_string()),
        Message::GotGames,
    )
}

async fn list(drive_path: PathBuf) -> io::Result<Box<[Game]>> {
    let wii_path = drive_path.join("wbfs");
    let gc_path = drive_path.join("games");

    let mut games = Vec::new();
    read_game_dir(wii_path, &mut games).await?;
    read_game_dir(gc_path, &mut games).await?;

    Ok(games.into_boxed_slice())
}

async fn read_game_dir(game_dir: PathBuf, games: &mut Vec<Game>) -> io::Result<()> {
    if !game_dir.exists() {
        return Ok(());
    }

    let mut entries = fs::read_dir(game_dir).await?;

    while let Some(entry) = entries.try_next().await? {
        if let Some(game) = Game::from_path(entry.path()).await {
            games.push(game);
        }
    }

    Ok(())
}

pub trait Games {
    fn sort(&mut self, prev_sort_by: SortBy, sort_by: SortBy);
}

impl Games for Box<[Game]> {
    fn sort(&mut self, prev_sort_by: SortBy, sort_by: SortBy) {
        match (prev_sort_by, sort_by) {
            (SortBy::NameAscending, SortBy::NameAscending)
            | (SortBy::NameDescending, SortBy::NameDescending)
            | (SortBy::SizeAscending, SortBy::SizeAscending)
            | (SortBy::SizeDescending, SortBy::SizeDescending)
            | (_, SortBy::None) => {
                // Do nothing, already sorted
            }

            (SortBy::NameDescending, SortBy::NameAscending)
            | (SortBy::NameAscending, SortBy::NameDescending)
            | (SortBy::SizeDescending, SortBy::SizeAscending)
            | (SortBy::SizeAscending, SortBy::SizeDescending) => {
                self.reverse();
            }

            (SortBy::SizeAscending, SortBy::NameAscending)
            | (SortBy::SizeDescending, SortBy::NameAscending)
            | (SortBy::None, SortBy::NameAscending) => {
                self.sort_unstable_by(|a, b| a.title.cmp(&b.title));
            }

            (SortBy::SizeAscending, SortBy::NameDescending)
            | (SortBy::SizeDescending, SortBy::NameDescending)
            | (SortBy::None, SortBy::NameDescending) => {
                self.sort_unstable_by(|a, b| b.title.cmp(&a.title));
            }

            (SortBy::NameAscending, SortBy::SizeAscending)
            | (SortBy::NameDescending, SortBy::SizeAscending)
            | (SortBy::None, SortBy::SizeAscending) => {
                self.sort_unstable_by(|a, b| a.size.cmp(&b.size));
            }

            (SortBy::NameAscending, SortBy::SizeDescending)
            | (SortBy::NameDescending, SortBy::SizeDescending)
            | (SortBy::None, SortBy::SizeDescending) => {
                self.sort_unstable_by(|a, b| b.size.cmp(&a.size));
            }
        }
    }
}

impl FuzzySearchable for Box<[Game]> {
    fn fuzzy_search(&self, query: &str) -> Box<[usize]> {
        let matcher = SkimMatcherV2::default();

        self.iter()
            .enumerate()
            .filter_map(|(i, game)| {
                let title_score = matcher.fuzzy_match(&game.title, query);
                let id_score = matcher.fuzzy_match(game.id.as_str(), query);

                match (title_score, id_score) {
                    (Some(s1), Some(s2)) => Some((i, s1 + s2)),
                    (Some(s1), None) | (None, Some(s1)) => Some((i, s1)),
                    (None, None) => None,
                }
            })
            .sorted_unstable_by_key(|(_, score)| *score)
            .map(|(i, _)| i)
            .collect()
    }
}
