// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{games::game::Game, messages::Message, ui::components};
use iced::{
    Element,
    widget::{button, column, row, text},
};

pub fn view<'a>(game: &'a Game) -> Element<'a, Message> {
    components::card::view(column![
        text(&game.title),
        components::link::view(game.path.to_string_lossy(), game.path.as_os_str()),
        row![button("Close").on_press(Message::CloseModal)]
    ])
    .into()
}
