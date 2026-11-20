// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{message::Message, ui::components::search_bar::search_bar};
use iced::Element;
use twbm_gamemanager::manager::GameList;

pub fn game_search(games: &GameList) -> Element<'_, Message> {
    search_bar(
        games.search_term(),
        Message::SearchGames,
        "Search by Title/ID",
    )
}
