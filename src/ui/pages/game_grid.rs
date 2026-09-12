// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    messages::Message,
    state::{AppState, Ongoing},
    ui::components::{
        game_card::game_card, games_other_toolbar::games_other_toolbar,
        games_titlebar::games_titlebar,
    },
};
use iced::{
    Element,
    widget::{Row, column},
};

pub fn game_grid(state: &AppState) -> Element<'_, Message> {
    let is_exporting = state.ongoing.contains(Ongoing::ExportingGame);

    let content = state
        .games
        .iter_by(state.config.sort_by)
        .map(|game| game_card(game, is_exporting))
        .collect::<Row<'_, _>>()
        .spacing(10);

    column![games_titlebar(state), games_other_toolbar(state), content]
        .padding(10)
        .spacing(10)
        .into()
}
