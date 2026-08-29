// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{games::game::Game, messages::Message, ui::components::my_card::my_card};
use iced::{
    Element, Length, padding,
    widget::{button, row, rule, space, text, tooltip},
};
use lucide_icons::iced::{icon_box, icon_info, icon_pointer, icon_trash};

pub fn game_row(game: &Game) -> Element<'_, Message> {
    row![
        tooltip(
            game.is_wii.then(icon_pointer).unwrap_or_else(icon_box),
            my_card(if game.is_wii { "Wii" } else { "GameCube" }),
            tooltip::Position::Top
        ),
        text!("{} [{}]", &game.title, game.id),
        space::horizontal(),
        text(game.size.to_string()),
        rule::vertical(1),
        tooltip(
            button(icon_trash().center())
                .padding(0)
                .on_press_with(|| Message::AskDeleteDir(game.path.clone()))
                .style(button::text)
                .width(20)
                .height(20),
            my_card("Delete game"),
            tooltip::Position::Top
        ),
        tooltip(
            button(icon_info().center())
                .padding(0)
                .on_press_with(|| Message::OpenGameInfo(game.clone()))
                .style(button::text)
                .width(20)
                .height(20),
            my_card("Game info"),
            tooltip::Position::Top
        ),
    ]
    .spacing(5)
    .height(Length::Shrink)
    .padding(padding::all(2).left(5))
    .into()
}
