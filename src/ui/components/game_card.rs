// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    game::Game,
    game_id::GameID,
    message::Message,
    state::State,
    ui::{Screen, components::my_tooltip, style},
};
use iced::{
    Alignment, Element,
    widget::{button, column, container, image, row, space, text},
};
use lucide_icons::iced::{icon_hard_drive_download, icon_info, icon_tag, icon_trash};

pub fn view<'a>(state: &State, game: &'a Game, i: usize) -> Element<'a, Message> {
    let mut col = column![
        row![
            icon_tag().size(12),
            text(game.id.as_str()),
            space::horizontal(),
            text(game.size.to_string())
        ]
        .spacing(5)
        .align_y(Alignment::Center),
        space::vertical(),
    ]
    .spacing(5)
    .padding(10)
    .width(170)
    .height(205)
    .align_x(Alignment::Center);

    if let Some(cover) = state.get_game_cover(game) {
        col = col.push(
            image(cover)
                .height(93)
                .filter_method(image::FilterMethod::Linear),
        );
    }

    col = col
        .push(my_tooltip::view(
            container(text(&game.title).wrapping(text::Wrapping::None)).clip(true),
            &game.title,
        ))
        .push(space::vertical())
        .push(
            row![
                button(row![icon_info(), text("Info")].spacing(5))
                    .style(style::rounded_secondary_button)
                    .on_press(Message::NavigateTo(Screen::GameInfo(i))),
                button(icon_hard_drive_download()).style(style::rounded_secondary_button),
                button(icon_trash())
                    .style(style::rounded_secondary_button)
                    .on_press(Message::AskDeleteGame(i))
            ]
            .spacing(5),
        );

    container(col).style(style::card).into()
}
