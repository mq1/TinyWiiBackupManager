// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{messages::Message, state::AppState};
use iced::Task;
use replace_with::replace_with_or_abort_and_return;

impl AppState {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        replace_with_or_abort_and_return(self, |state| match message {
            Message::NavigateTo(page) => (Task::none(), state.with_current_page(page)),
            Message::RefreshGamesAndApps => (Task::none(), state.with_games_reloaded()),
            Message::RefreshPlugins => (Task::none(), state.with_plugins_reloaded()),
        })
    }
}
