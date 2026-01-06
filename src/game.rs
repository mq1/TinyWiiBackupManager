// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{config::SortBy, game_id::GameID, message::Message, state::State, util};
use futures::TryFutureExt;
use iced::Task;
use size::Size;
use smol::{fs, future::try_zip, io, stream::StreamExt};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Game {
    pub path: PathBuf,
    pub size: Size,
    pub is_wii: bool,
    pub title: String,
    pub id: [u8; 6],
    search_term: String,
}

impl Game {
    pub async fn from_path(path: PathBuf, is_wii: bool) -> Option<Self> {
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
        let id = GameID::from_str(id_str);

        let size = util::get_dir_size(path.clone()).await.unwrap_or_default();

        let search_term = format!("{}{}", title, id_str).to_lowercase();

        Some(Self {
            path,
            size,
            is_wii,
            title,
            id,
            search_term,
        })
    }

    pub fn open_dir(&self) -> io::Result<()> {
        open::that(&self.path)
    }

    pub fn open_gametdb(&self) -> io::Result<()> {
        let url = format!("https://www.gametdb.com/Wii/{}", self.id.as_str());
        open::that(url)
    }

    pub fn get_delete_task(&self) -> Task<Message> {
        let path = self.path.clone();
        let title = self.title.clone();

        Task::perform(
            async move {
                fs::remove_dir_all(&path).await.map_err(|e| e.to_string())?;
                Ok(title)
            },
            Message::GameDeleted,
        )
    }

    pub fn matches_search(&self, filter: &str) -> bool {
        self.search_term.contains(filter)
    }

    pub fn get_path_str(&self) -> &str {
        self.path.to_str().unwrap_or("Invalid path")
    }
}

pub fn get_list_games_task(state: &State) -> Task<Message> {
    let drive_path = state.config.get_drive_path().to_path_buf();

    Task::perform(
        list(drive_path).map_err(|e| e.to_string()),
        Message::GotGames,
    )
}

async fn list(drive_path: PathBuf) -> io::Result<Box<[Game]>> {
    let wii_path = drive_path.join("wbfs");
    let gc_path = drive_path.join("games");

    let (mut wii_games, mut gc_games) =
        try_zip(read_game_dir(wii_path, true), read_game_dir(gc_path, false)).await?;

    wii_games.append(&mut gc_games);

    Ok(wii_games.into_boxed_slice())
}

async fn read_game_dir(game_dir: PathBuf, is_wii: bool) -> io::Result<Vec<Game>> {
    if !game_dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries = fs::read_dir(game_dir).await?;

    let mut games = Vec::new();
    while let Some(entry) = entries.try_next().await? {
        if let Some(game) = Game::from_path(entry.path(), is_wii).await {
            games.push(game);
        }
    }

    Ok(games)
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
