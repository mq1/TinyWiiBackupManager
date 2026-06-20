// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::Config,
    games::game::Game,
    notifications::{Notification, NotificationLevel},
    plugins::Plugin,
    ui::pages::Page,
};
use getset::{CopyGetters, WithSetters};
use std::path::PathBuf;

#[derive(CopyGetters, WithSetters)]
pub(crate) struct AppState {
    data_dir: PathBuf,
    config: Config,
    notifications: Vec<Notification>,

    #[getset(set_with = "pub")]
    games: Vec<Game>,

    #[getset(set_with = "pub")]
    plugins: Vec<Plugin>,

    #[getset(get_copy = "pub", set_with = "pub")]
    current_page: Page,
}

impl AppState {
    pub fn new(data_dir: PathBuf) -> Self {
        let config = Config::load(&data_dir);

        let initial = Self {
            data_dir,
            config,
            notifications: Vec::new(),
            games: Vec::new(),
            plugins: Vec::new(),
            current_page: Page::Games,
        };

        initial.with_games_reloaded().with_plugins_reloaded()
    }

    pub fn with_notification(mut self, notification: Notification) -> Self {
        self.notifications.push(notification);
        self
    }

    pub fn with_games_reloaded(self) -> Self {
        let res = crate::games::list(
            self.config.contents.mount_point(),
            self.config.contents.sort_by(),
        );

        match res {
            Ok(games) => self.with_games(games),
            Err(e) => {
                let notification = Notification::new(
                    format!("Failed to load games: {e}"),
                    NotificationLevel::Error,
                );
                self.with_notification(notification)
            }
        }
    }

    pub fn with_plugins_reloaded(self) -> Self {
        let res = crate::plugins::list(&self.data_dir);

        match res {
            Ok(plugins) => self.with_plugins(plugins),
            Err(e) => {
                let label = format!("Failed to load plugins: {e}");
                let notification = Notification::new(label, NotificationLevel::Error);
                self.with_notification(notification)
            }
        }
    }
}
