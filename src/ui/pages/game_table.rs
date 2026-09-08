// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    messages::Message,
    state::{AppState, Ongoing},
    ui::components::{
        game_row::game_row, games_other_toolbar::games_other_toolbar,
        games_titlebar::games_titlebar, my_card::my_card,
    },
};
use iced::{
    Element,
    widget::{Column, column, rule},
};
use itertools::Itertools;

pub fn game_table(state: &AppState) -> Element<'_, Message> {
    let is_exporting = state.ongoing.contains(&Ongoing::ExportingGame);

    let content = state
        .games
        .iter_by(state.config.sort_by)
        .map(|game| game_row(game, is_exporting))
        .intersperse_with(|| rule::horizontal(1).into())
        .collect::<Column<'_, _>>();

    column![
        games_titlebar(state),
        games_other_toolbar(state),
        my_card(content).padding(0)
    ]
    .padding(10)
    .spacing(10)
    .into()
}
