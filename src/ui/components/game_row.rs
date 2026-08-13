// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{games::game::Game, messages::Message, ui::components::my_card::MyCard};
use iced::{
    Element, padding,
    widget::{button, row, space, text, tooltip},
};
use lucide_icons::iced::{icon_box, icon_info, icon_pointer, icon_trash};
use std::sync::Arc;

pub fn view(game: &Arc<Game>) -> Element<'_, Message> {
    row![
        tooltip(
            game.is_wii.then(icon_pointer).unwrap_or_else(icon_box),
            MyCard::new(text(if game.is_wii { "Wii" } else { "GameCube" })).view(),
            tooltip::Position::Top
        ),
        text!("{} [{}]", &game.title, game.id),
        space::horizontal(),
        tooltip(
            button(icon_trash().center())
                .padding(0)
                .on_press_with(|| Message::AskDeleteDir(game.path.clone()))
                .style(button::text)
                .width(20)
                .height(20),
            MyCard::new(text!("Delete game")).view(),
            tooltip::Position::Top
        ),
        tooltip(
            button(icon_info().center())
                .padding(0)
                .on_press_with(|| Message::OpenGameInfo(game.clone()))
                .style(button::text)
                .width(20)
                .height(20),
            MyCard::new(text!("Game info")).view(),
            tooltip::Position::Top
        ),
    ]
    .spacing(5)
    .padding(padding::all(2).left(5))
    .into()
}
