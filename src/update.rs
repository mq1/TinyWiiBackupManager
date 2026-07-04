// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{messages::Message, notifications::Notification, state::AppState};
use iced::Task;

impl AppState {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::NoOp => Task::none(),
            Message::NavigateTo(page) => {
                self.current_page = page;
                Task::none()
            }
            Message::Notify(notification) => {
                self.notifications.push(notification);
                Task::none()
            }
            Message::CloseNotification(idx) => {
                self.notifications.remove(idx);
                Task::none()
            }
            Message::RefreshGamesAndApps => self.get_games_task(),
            Message::GotConfig(config) => {
                self.config = config;
                Task::none()
            }
            Message::GotGames(games) => {
                self.games = games;
                Task::none()
            }
            Message::CouldNotGetGames(e) => {
                self.games.clear();
                self.notifications.push(Notification::error(e));
                Task::none()
            }
            Message::RefreshPlugins => self.get_plugins_task(),
            Message::GotPlugins(plugins) => {
                self.plugins = plugins;
                Task::none()
            }
            Message::CouldNotGetPlugins(e) => {
                self.plugins.clear();
                self.notifications.push(Notification::error(e));
                Task::none()
            }
            Message::GotDriveInfo(drive_info) => {
                self.drive_info = Some(drive_info);
                Task::none()
            }
            Message::CouldNotGetDriveInfo(e) => {
                self.drive_info = None;
                self.notifications.push(Notification::error(e));
                Task::none()
            }
            Message::RunTool(plugin_i, tool_i) => self.run_tool_task(plugin_i, tool_i),
        }
    }
}
