// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::Config,
    errors::Error,
    games::{self, game::Game},
    homebrew::{self, homebrew_app::HomebrewApp},
    messages::Message,
    notifications::Notification,
    ui::{components::Modal, dialogs, pages::Page},
    util::drive_info::DriveInfo,
};
use iced::Task;
use smol::fs::File;
use std::path::PathBuf;

pub(crate) struct AppState {
    pub data_dir: PathBuf,
    pub config: Config,
    pub notifications: Vec<Notification>,
    pub drive_info: Option<DriveInfo>,
    pub games: Vec<Game>,
    pub homebrew_apps: Vec<HomebrewApp>,
    pub current_page: Page,
    pub current_modal: Option<Modal>,
}

impl AppState {
    pub fn new(data_dir: PathBuf) -> (Self, Task<Message>) {
        let state = Self {
            data_dir,
            config: Config::default(),
            notifications: Vec::new(),
            drive_info: None,
            games: Vec::new(),
            homebrew_apps: Vec::new(),
            current_page: Page::Games,
            current_modal: None,
        };

        let task = state.load_config_task();

        (state, task)
    }

    pub fn write_config_task(&self) -> Task<Message> {
        let config = self.config.clone();
        Task::perform(async move { config.write().await }, Message::WroteConfig)
    }

    pub fn load_config_task(&self) -> Task<Message> {
        let data_dir = self.data_dir.clone();
        Task::perform(Config::load(data_dir), Message::GotConfig)
    }

    pub fn pick_mount_point_task(&self) -> Task<Message> {
        iced::window::oldest()
            .and_then(|id| iced::window::run(id, dialogs::pick_mount_point))
            .map(Message::MountPointPicked)
    }

    pub fn get_games_task(&self) -> Task<Message> {
        let data_dir = self.data_dir.clone();
        let mount_point = self.config.contents.mount_point.clone();
        let sort_by = self.config.contents.sort_by;

        Task::perform(
            async move { games::list(&data_dir, &mount_point, sort_by).await },
            Message::GotGames,
        )
    }

    pub fn get_homebrew_apps_task(&self) -> Task<Message> {
        let mount_point = self.config.contents.mount_point.clone();
        let sort_by = self.config.contents.sort_by;

        Task::perform(
            async move { homebrew::list(&mount_point, sort_by).await },
            Message::GotHomebrewApps,
        )
    }

    pub fn get_drive_info_task(&self) -> Task<Message> {
        let mount_point = &self.config.contents.mount_point;
        if mount_point.as_os_str().is_empty() {
            return Task::none();
        }

        let mount_point = mount_point.clone();
        Task::perform(DriveInfo::try_from_path(mount_point), Message::GotDriveInfo)
    }

    pub fn get_disc_info_task(&self, game_i: usize) -> Task<Message> {
        let game = self.games[game_i].clone();

        Task::perform(
            async move {
                let disc_path = game.get_disc_path().await.ok_or(Error::DiscNotFound)?;
                let mut file = File::open(&disc_path).await?;
                let meta = wii_disc_info::Meta::read(&mut file).await?;
                Ok(Box::new(meta))
            },
            Message::GotDiscInfo,
        )
    }
}
