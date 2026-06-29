// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{messages::Message, state::AppState};
use iced::Task;

impl AppState {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::NoOp => Task::none(),
            Message::NavigateTo(page) => {
                self.current_page = page;
                Task::none()
            }
            Message::RefreshGamesAndApps => {
                self.reload_games();
                Task::none()
            }
            Message::RefreshPlugins => {
                self.reload_plugins();
                Task::none()
            }
            Message::RunTool(plugin_i, tool_i) => self.run_tool(plugin_i, tool_i),
            Message::Notify(notification) => {
                self.notifications.add(notification);
                Task::none()
            }
        }
    }
}
