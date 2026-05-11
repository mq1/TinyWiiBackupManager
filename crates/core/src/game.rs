// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{game_id::GameID, id_map};
use std::{
    fs,
    path::{Path, PathBuf},
};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Game {
    pub uuid: Uuid,
    pub id: GameID,
    pub title: String,
    pub path: PathBuf,
    pub size: u64,
    pub is_wii: bool,
}

impl Game {
    pub fn from_path(path: PathBuf) -> Option<Self> {
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
            uuid: Uuid::now_v7(),
            id,
            title,
            path,
            size,
            is_wii,
        })
    }

    pub fn get_disc_path(&self) -> Option<PathBuf> {
        let wii_wbfs = format!("{}.wbfs", self.id);
        let wii_iso = format!("{}.iso", self.id);
        let wii_part0_iso = format!("{}.part0.iso", self.id);

        let possible_filenames = [
            wii_wbfs.as_str(),
            wii_iso.as_str(),
            wii_part0_iso.as_str(),
            "game.iso",
            "game.ciso",
        ];

        for filename in possible_filenames {
            let path = self.path.join(filename);
            if path.is_file() {
                return Some(path);
            }
        }

        None
    }
}

pub fn scan_dir(path: &Path) -> Vec<Game> {
    let Ok(entries) = fs::read_dir(path) else {
        return Vec::new();
    };

    entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            Game::from_path(entry.path())
        })
        .collect()
}
