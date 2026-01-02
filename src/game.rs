// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use size::Size;
use std::{
    fs, io,
    path::{Path, PathBuf},
};

use crate::{game_id::GameID, wiitdb};

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

    #[inline]
    pub fn open_dir(&self) -> io::Result<()> {
        open::that(&self.path)
    }

    #[inline]
    pub fn matches_search(&self, filter: &str) -> bool {
        self.search_term.contains(filter)
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
