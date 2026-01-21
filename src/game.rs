// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    disc_info::DiscInfo,
    game_id::GameID,
    message::Message,
    util::{self},
    wiitdb::{Datafile, GameInfo},
};
use derive_getters::Getters;
use iced::{Task, futures::TryFutureExt};
use size::Size;
use std::{ffi::OsString, path::PathBuf};

#[derive(Debug, Clone, Getters)]
pub struct Game {
    path: PathBuf,
    #[getter(copy)]
    size: Size,
    title: String,
    #[getter(copy)]
    id: GameID,
    disc_info: Option<Result<DiscInfo, String>>,
    wiitdb_info: Option<GameInfo>,
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
        let mut uri = OsString::from("https://www.gametdb.com/Wii/");
        uri.push(self.id.as_os_str());
        uri
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

    pub fn update_wiitdb_info(&mut self, wiitdb: &Datafile) {
        self.wiitdb_info = wiitdb.get_game_info(self.id);
    }

    pub fn update_title(&mut self, wiitdb: &Datafile) {
        if let Some(title) = wiitdb.get_title(self.id) {
            self.title = title;
        }
    }

    pub fn update_disc_info(&mut self, res: Result<DiscInfo, String>) {
        self.disc_info = Some(res);
    }
}
