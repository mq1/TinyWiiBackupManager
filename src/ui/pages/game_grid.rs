// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{messages::Message, state::AppState, ui::components::game_card};
use iced::{
    Element,
    widget::{Row, column, text},
};

pub fn view<'a>(state: &'a AppState) -> Element<'a, Message> {
    let content = state.games.iter().map(game_card::view).collect::<Row<_>>();

    column![text("Games"), content]
        .padding(10)
        .spacing(10)
        .into()
}
