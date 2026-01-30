// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    games::{disc_info::DiscInfo, game_id::GameID, id_map::ID_MAP, wiitdb::GameInfo},
    message::Message,
};
use anyhow::{Result, anyhow};
use derive_getters::Getters;
use iced::{Task, futures::TryFutureExt};
use size::Size;
use std::{
    ffi::{OsStr, OsString},
    fs,
    path::PathBuf,
    sync::Arc,
};

#[derive(Debug, Clone, Getters)]
pub struct Game {
    path: PathBuf,
    #[getter(copy)]
    size: Size,
    title: String,
    #[getter(copy)]
    id: GameID,
    disc_info: Option<Result<DiscInfo, String>>,
    game_info: Option<Result<GameInfo, String>>,
}

impl PartialEq for Game {
    fn eq(&self, other: &Self) -> bool {
        self.path() == other.path()
    }
}

impl Eq for Game {}

impl Game {
    pub fn maybe_from_path(path: PathBuf) -> Option<Self> {
        if !path.is_dir() {
            return None;
        }

        let filename = path.file_name()?.to_str()?;
        if filename.starts_with('.') {
            return None;
        }

        let (title_str, id_str) = filename.split_once(" [")?;
        let id_str = id_str.strip_suffix(']')?;
        let id = GameID::try_from(id_str).ok()?;

        let title = match ID_MAP.get_title(id) {
            Some(title) => title.to_string(),
            None => title_str.to_string(),
        };

        let size = fs_extra::dir::get_size(&path).unwrap_or(0);

        Some(Self {
            path,
            size: Size::from_bytes(size),
            title,
            id,
            disc_info: None,
            game_info: None,
        })
    }

    pub fn get_disc_path(&self) -> Result<PathBuf> {
        let entries = fs::read_dir(&self.path)?;

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

            if filename.ends_with(".iso")
                || filename.ends_with(".wbfs")
                || filename.ends_with(".ciso")
            {
                return Ok(path);
            }
        }

        Err(anyhow!("No disc found"))
    }

    pub fn get_path_uri(&self) -> OsString {
        self.path.as_os_str().to_os_string()
    }

    pub fn get_gametdb_uri(&self) -> OsString {
        let mut uri = OsString::from("https://www.gametdb.com/Wii/");
        uri.push(self.id.as_os_str());
        uri
    }

    pub fn get_load_disc_info_task(&mut self) -> Task<Message> {
        self.disc_info = None;
        let path = self.path.clone();

        Task::perform(
            async move { DiscInfo::try_from_game_dir(&path) }.map_err(Arc::new),
            Message::GotDiscInfo,
        )
    }

    pub fn set_game_info(&mut self, res: Result<GameInfo, String>) {
        self.game_info = Some(res);
    }

    pub fn set_disc_info(&mut self, res: Result<DiscInfo, String>) {
        self.disc_info = Some(res);
    }
}
