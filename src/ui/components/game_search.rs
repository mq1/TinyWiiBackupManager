// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{messages::Message, state::AppState, ui::components::search_bar::search_bar};
use iced::Element;

pub fn game_search(state: &AppState) -> Element<'_, Message> {
    search_bar(
        state.games.search_term(),
        Message::SearchGames,
        "Search by Title/ID",
    )
}
