// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use size::Size;
use std::{
    fs, io,
    path::{Path, PathBuf},
};

use crate::{config::SortBy, game_id::GameID, wiitdb};

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
    pub fn from_path(path: PathBuf, is_wii: bool) -> Option<Self> {
        if !path.is_dir() {
            return None;
        }

        let (title_str, id_str) = path.file_name()?.to_str()?.split_once(" [")?;
        let id_str = id_str.strip_suffix(']')?;
        let title = title_str.to_string();
        let id = GameID::from_str(id_str);

        let bytes = fs_extra::dir::get_size(&path).unwrap_or(0);
        if bytes <= 1024 * 1024 * 8 {
            return None;
        }
        let size = Size::from_bytes(bytes);

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

    pub fn delete(&self) -> io::Result<()> {
        fs::remove_dir_all(&self.path)
    }

    pub fn matches_search(&self, filter: &str) -> bool {
        self.search_term.contains(filter)
    }

    pub fn get_path_str(&self) -> &str {
        self.path.to_str().unwrap_or("Invalid path")
    }
}

pub fn list(drive_path: &Path, wiitdb: &Option<wiitdb::Datafile>) -> Box<[Game]> {
    let mut games = Vec::new();

    let mut wii_games = read_game_dir(drive_path.join("wbfs"), true);
    let mut gc_games = read_game_dir(drive_path.join("games"), false);

    games.append(&mut wii_games);
    games.append(&mut gc_games);
    let mut games = games.into_boxed_slice();

    if let Some(wiitdb) = &wiitdb {
        for game in &mut games {
            if let Some(title) = wiitdb.get_title(game.id) {
                game.title = title;
            }
        }
    }

    games
}

fn read_game_dir(game_dir: PathBuf, is_wii: bool) -> Vec<Game> {
    let entries = match fs::read_dir(game_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    entries
        .filter_map(Result::ok)
        .filter_map(|e| Game::from_path(e.path(), is_wii))
        .collect()
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
