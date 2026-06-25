// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::Config,
    drive_info::DriveInfo,
    games::game::Game,
    notifications::Notifications,
    plugins::{self, plugin::Plugin},
    ui::pages::Page,
};
use anyhow::Context;
use std::path::PathBuf;

pub(crate) struct AppState {
    pub data_dir: PathBuf,
    pub config: Config,
    pub notifications: Notifications,
    pub drive_info: Option<DriveInfo>,
    pub games: Vec<Game>,
    pub plugins: Vec<Plugin>,
    pub current_page: Page,
}

impl AppState {
    pub fn new(data_dir: PathBuf) -> Self {
        let config = Config::load(&data_dir);

        let mut initial = Self {
            data_dir,
            config,
            notifications: Notifications::new(),
            drive_info: None,
            games: Vec::new(),
            plugins: Vec::new(),
            current_page: Page::Games,
        };

        initial.reload_games();
        initial.reload_drive_info();
        initial.reload_plugins();

        initial
    }

    pub fn reload_drive_info(&mut self) {
        let mount_point = &self.config.contents.mount_point;

        if mount_point.as_os_str().is_empty() {
            self.drive_info = None;
            return;
        }

        let res = DriveInfo::try_from_path(mount_point).context("Failed to load drive info");

        match res {
            Ok(drive_info) => self.drive_info = Some(drive_info),
            Err(e) => {
                self.notifications.add(e);
                self.drive_info = None;
            }
        }
    }

    pub fn reload_games(&mut self) {
        let res = crate::games::list(
            &self.config.contents.mount_point,
            self.config.contents.sort_by,
        )
        .context("Failed to load games");

        match res {
            Ok(games) => self.games = games,
            Err(e) => {
                self.notifications.add(e);
                self.games.clear();
            }
        }
    }

    pub fn reload_plugins(&mut self) {
        let res = plugins::load(&self.data_dir).context("Failed to load plugins");

        match res {
            Ok(plugins) => self.plugins = plugins,
            Err(e) => {
                self.notifications.add(e);
                self.plugins = Vec::new();
            }
        }
    }
}
