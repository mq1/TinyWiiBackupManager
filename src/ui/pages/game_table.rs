// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    messages::Message,
    state::AppState,
    ui::components::{card, game_row, games_titlebar},
};
use iced::{
    Element,
    widget::{Column, column, rule},
};
use itertools::Itertools;

pub fn view(state: &AppState) -> Element<'_, Message> {
    let content = state
        .games
        .iter()
        .enumerate()
        .map(game_row::view)
        .intersperse_with(|| rule::horizontal(1).into())
        .collect::<Column<_>>();

    column![games_titlebar::view(), card::view(content).padding(0)]
        .padding(10)
        .spacing(10)
        .into()
}
