// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    games::game::Game,
    message::Message,
    state::State,
    ui::{Screen, components::my_tooltip, style},
};
use iced::{
    Alignment, Element,
    widget::{button, column, container, image, row, space, text},
};
use iced_palace::widget::ellipsized_text;
use lucide_icons::iced::{icon_hard_drive_download, icon_info, icon_tag, icon_trash};

pub fn view<'a>(state: &State, game: &'a Game) -> Element<'a, Message> {
    let mut col = column![
        row![
            icon_tag().size(12),
            text(game.id().as_str().to_string()),
            space::horizontal(),
            text(game.size().to_string())
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
            ellipsized_text(game.title()).wrapping(text::Wrapping::None),
            game.title(),
        ))
        .push(space::vertical())
        .push(
            row![
                my_tooltip::view(
                    button(row![icon_info(), text("Info")].spacing(5))
                        .style(style::rounded_secondary_button)
                        .on_press_with(|| Message::NavTo(Screen::GameInfo(game.clone()))),
                    "Show game info"
                ),
                my_tooltip::view(
                    button(icon_hard_drive_download())
                        .style(style::rounded_secondary_button)
                        .on_press_with(|| Message::ChooseArchiveDest(
                            game.get_disc_path().unwrap_or_default(),
                            game.title().clone()
                        )),
                    "Archive game to PC"
                ),
                my_tooltip::view(
                    button(icon_trash())
                        .style(style::rounded_secondary_button)
                        .on_press_with(|| Message::AskDeleteDirConfirmation(game.path().clone())),
                    "Delete game"
                )
            ]
            .spacing(5),
        );

    container(col).style(style::card).into()
}
