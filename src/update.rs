// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{messages::Message, state::AppState};
use iced::Task;

impl AppState {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::NavigateTo(page) => self.current_page = page,
            Message::RefreshGamesAndApps => self.reload_games(),
            Message::RefreshPlugins => self.reload_plugins(),
            Message::RunTool(id) => self.run_tool(id),
            Message::Notify(notification) => self.notifications.add(notification),
        }

        Task::none()
    }
}
