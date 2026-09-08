// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{messages::Message, state::AppState, ui::components::game_search::game_search};
use iced::{
    Alignment, Element,
    widget::{Text, row, space, text},
};

fn usage(state: &AppState) -> Text<'_> {
    text!(
        "{} game{}   ~ {}  ",
        state.games.count(),
        if state.games.count() == 1 { "" } else { "s" },
        state.games.total_size()
    )
}

pub fn games_other_toolbar(state: &AppState) -> Element<'_, Message> {
    row![game_search(state), space::horizontal(), usage(state)]
        .align_y(Alignment::Center)
        .into()
}
