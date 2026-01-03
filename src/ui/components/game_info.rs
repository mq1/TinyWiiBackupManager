// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{message::Message, state::State};
use iced::{Element, widget::text};

pub fn view(state: &State, game_i: usize) -> Element<'_, Message> {
    let game = &state.games[game_i];

    text(&game.title).into()
}
