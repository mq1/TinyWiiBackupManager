// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{games::game::Game, messages::Message};
use iced::{
    Element, padding,
    widget::{button, row, space, text},
};
use lucide_icons::iced::{icon_box, icon_info, icon_pointer, icon_trash};

pub fn view(game: &Game) -> Element<'_, Message> {
    row![
        game.is_wii().then(icon_pointer).unwrap_or_else(icon_box),
        text!("{} [{}]", game.title(), game.id()),
        space::horizontal(),
        button(icon_trash().center())
            .padding(0)
            .on_press_with(|| Message::AskDeleteDir(game.path().clone()))
            .style(button::text)
            .width(20)
            .height(20),
        button(icon_info().center())
            .padding(0)
            .on_press_with(|| Message::OpenGameInfo(game.path().clone()))
            .style(button::text)
            .width(20)
            .height(20)
    ]
    .spacing(5)
    .padding(padding::all(2).left(5))
    .into()
}
