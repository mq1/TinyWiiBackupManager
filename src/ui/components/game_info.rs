// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{game::Game, message::Message};
use iced::{Element, widget::text};

pub fn view<'a>(game: &'a Game) -> Element<'a, Message> {
    text(&game.title).into()
}
