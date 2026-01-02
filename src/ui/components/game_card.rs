// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    game::Game,
    game_id::GameID,
    message::Message,
    state::State,
    ui::{components::my_tooltip, style},
};
use iced::{
    Alignment, Element,
    widget::{column, container, image, row, space, text},
};
use lucide_icons::iced::icon_tag;

pub fn view<'a>(state: &'a State, game: &'a Game) -> Element<'a, Message> {
    let mut col = column![
        row![
            icon_tag(),
            text(game.id.as_str()),
            space::horizontal(),
            text(game.size.to_string())
        ]
        .spacing(5)
        .align_y(Alignment::Center),
        space::vertical(),
    ]
    .spacing(10)
    .padding(10)
    .width(170)
    .height(200)
    .align_x(Alignment::Center);

    if let Some(cover) = state.get_game_cover(game) {
        col = col.push(image(cover).height(100));
    }

    col = col
        .push(my_tooltip::view(
            container(text(&game.title)).clip(true),
            &game.title,
        ))
        .push(space::vertical());

    container(col).style(style::card).into()
}
