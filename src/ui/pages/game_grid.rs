// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    messages::Message,
    state::AppState,
    ui::components::{game_card, games_titlebar},
};
use iced::{
    Element,
    widget::{Row, column},
};

pub fn view<'a>(state: &'a AppState) -> Element<'a, Message> {
    let content = state
        .games
        .iter()
        .enumerate()
        .map(game_card::view)
        .collect::<Row<_>>()
        .spacing(10);

    column![games_titlebar::view(), content]
        .padding(10)
        .spacing(10)
        .into()
}
