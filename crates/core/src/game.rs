// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{game_id::GameID, id_map};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub struct Game {
    pub id: GameID,
    pub title: String,
    pub path: PathBuf,
    pub size: u64,
    pub is_wii: bool,
}

impl Game {
    pub fn try_from_path(path: PathBuf) -> Option<Self> {
        let file_name = path.file_name()?.to_str()?;

        if file_name.starts_with(".") || !path.is_dir() {
            return None;
        }

        let (title, id) = file_name.split_once('[')?;
        let is_wii = matches!(id.chars().next(), Some('R' | 'S'));
        let id = GameID::new(&id[..id.len() - 1])?;

        let title = match id_map::get(id) {
            Some(entry) => entry.title.to_string(),
            None => title.trim().to_string(),
        };

        let size = fs_extra::dir::get_size(&path).ok()?;

        Some(Self {
            id,
            title,
            path,
            size,
            is_wii,
        })
    }
}

pub fn scan_dir(path: &Path) -> Vec<Game> {
    let Ok(entries) = fs::read_dir(path) else {
        return Vec::new();
    };

    entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            Game::try_from_path(entry.path())
        })
        .collect()
}
