// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{game_id::GameID, id_map};
use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub struct Game {
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
            id,
            title,
            path,
            size,
            is_wii,
        })
    }

    pub fn get_disc_path(&self) -> Option<PathBuf> {
        let entries = self.path.read_dir().ok()?;

        for entry in entries.filter_map(Result::ok) {
            if !entry.file_type().is_ok_and(|t| t.is_file()) {
                continue;
            }

            let path = entry.path();

            let Some(filename) = path.file_name().and_then(OsStr::to_str) else {
                continue;
            };

            if filename.starts_with('.') {
                continue;
            }

            if filename.ends_with(".part1.iso") {
                continue;
            }

            let Some(ext) = path.extension() else {
                continue;
            };

            if ext.eq_ignore_ascii_case("iso")
                || ext.eq_ignore_ascii_case("wbfs")
                || ext.eq_ignore_ascii_case("ciso")
            {
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
