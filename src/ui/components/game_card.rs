// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    message::Message,
    state::State,
    ui::{components::my_tooltip, style},
};
use iced::{
    Alignment, Element,
    widget::{button, column, container, image, row, space, text},
};
use lucide_icons::iced::{icon_hard_drive_download, icon_info, icon_tag, icon_trash};

pub fn view(state: &State, i: usize) -> Element<'_, Message> {
    let game = state.game_list.get_unchecked(i);

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
            container(text(game.title()).wrapping(text::Wrapping::None)).clip(true),
            game.title(),
        ))
        .push(space::vertical())
        .push(
            row![
                button(row![icon_info(), text("Info")].spacing(5))
                    .style(style::rounded_secondary_button)
                    .on_press(Message::OpenGameInfo(i)),
                button(icon_hard_drive_download()).style(style::rounded_secondary_button),
                button(icon_trash())
                    .style(style::rounded_secondary_button)
                    .on_press_with(|| Message::AskDeleteDirConfirmation(game.path().to_path_buf()))
            ]
            .spacing(5),
        );

    container(col).style(style::card).into()
}
