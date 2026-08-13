// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    messages::Message,
    state::AppState,
    ui::components::{game_row, games_titlebar, my_card::my_card},
};
use iced::{
    Element,
    widget::{Column, column, rule},
};
use itertools::Itertools;

pub fn view(state: &AppState) -> Element<'_, Message> {
    let content = state
        .games
        .iter_by(state.config.sort_by)
        .map(game_row::view)
        .intersperse_with(|| rule::horizontal(1).into())
        .collect::<Column<'_, _>>();

    column![games_titlebar::view(state), my_card(content).padding(0)]
        .padding(10)
        .spacing(10)
        .into()
}
