// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::Config,
    games::{self, game::Game},
    messages::Message,
    notifications::Notification,
    plugins::{self, plugin::Plugin},
    ui::pages::Page,
    util::drive_info::DriveInfo,
};
use iced::Task;
use std::path::PathBuf;

pub(crate) struct AppState {
    pub data_dir: PathBuf,
    pub config: Config,
    pub notifications: Vec<Notification>,
    pub drive_info: Option<DriveInfo>,
    pub games: Vec<Game>,
    pub plugins: Vec<Plugin>,
    pub current_page: Page,
}

impl AppState {
    pub fn new(data_dir: PathBuf) -> (Self, Task<Message>) {
        let config = Config::load(&data_dir);

        let state = Self {
            data_dir,
            config,
            notifications: Vec::new(),
            drive_info: None,
            games: Vec::new(),
            plugins: Vec::new(),
            current_page: Page::Games,
        };

        let task = Task::batch([
            state.get_games_task(),
            state.get_drive_info_task(),
            state.get_plugins_task(),
        ]);

        (state, task)
    }

    pub fn get_games_task(&self) -> Task<Message> {
        let mount_point = self.config.contents.mount_point.clone();
        let sort_by = self.config.contents.sort_by;

        Task::perform(games::list(mount_point, sort_by), |res| match res {
            Ok(games) => Message::GotGames(games),
            Err(e) => Message::CouldNotGetGames(e.to_string()),
        })
    }

    pub fn get_plugins_task(&self) -> Task<Message> {
        let data_dir = self.data_dir.clone();

        Task::perform(plugins::load(data_dir), |res| match res {
            Ok(plugins) => Message::GotPlugins(plugins),
            Err(e) => Message::CouldNotGetPlugins(e.to_string()),
        })
    }

    pub fn get_drive_info_task(&self) -> Task<Message> {
        let mount_point = self.config.contents.mount_point.clone();

        if mount_point.as_os_str().is_empty() {
            return Task::none();
        }

        Task::perform(DriveInfo::try_from_path(mount_point), |res| match res {
            Ok(drive_info) => Message::GotDriveInfo(drive_info),
            Err(e) => Message::CouldNotGetDriveInfo(e.to_string()),
        })
    }

    pub fn run_tool_task(&self, plugin_i: usize, tool_i: usize) -> Task<Message> {
        let plugin = self.plugins[plugin_i].clone();
        let straw = plugins::run::run_tool(plugin, tool_i);

        Task::sip(straw, std::convert::identity, |res| {
            if let Err(e) = res {
                Message::Notify(Notification::error(e))
            } else {
                Message::NoOp
            }
        })
    }
}
