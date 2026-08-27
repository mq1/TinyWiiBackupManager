// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    messages::Message,
    state::AppState,
    ui::components::{game_card::game_card, games_titlebar::games_titlebar},
};
use iced::{
    Element,
    widget::{Row, column},
};

pub fn game_grid(state: &AppState) -> Element<'_, Message> {
    let content = state
        .games
        .iter_by(state.config.sort_by)
        .map(game_card)
        .collect::<Row<'_, _>>()
        .spacing(10);

    column![games_titlebar(state), content]
        .padding(10)
        .spacing(10)
        .into()
}
