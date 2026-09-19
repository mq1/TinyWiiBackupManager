// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    games::game::Game,
    messages::Message,
    ui::components::{my_button::my_button, my_card::my_card},
};
use iced::{
    Alignment, Element,
    widget::{column, image, row, space, text, tooltip},
};
use iced_palace::widget::ellipsized_text;
use lucide_icons::{Icon, iced::icon_tag};

pub fn game_card(game: &Game, is_exporting: bool) -> Element<'_, Message> {
    let cover: Element<'_, Message> = match game.cover() {
        Some(cover) => image(cover).height(96).into(),
        None => space().height(96).into(),
    };

    my_card(
        column![
            row![
                icon_tag(),
                text(game.id_str()),
                space::horizontal(),
                text(game.size_str())
            ]
            .spacing(5),
            cover,
            ellipsized_text(game.title()).wrapping(text::Wrapping::None),
            row![
                my_button()
                    .label("Info")
                    .icon(Icon::Info)
                    .on_press_with(|| Message::OpenGameInfo(game.clone()))
                    .expand_width(),
                tooltip(
                    {
                        let btn = my_button().icon(Icon::HardDriveDownload);

                        if is_exporting {
                            btn
                        } else {
                            btn.on_press_with(|| Message::PickExportDest(game.clone()))
                        }
                    },
                    my_card("Export game"),
                    tooltip::Position::Bottom
                ),
                tooltip(
                    my_button()
                        .icon(Icon::Trash)
                        .on_press_with(|| Message::AskDeleteDir(game.path().to_path_buf())),
                    my_card("Delete game"),
                    tooltip::Position::Bottom
                )
            ]
            .spacing(5)
        ]
        .align_x(Alignment::Center)
        .padding(5)
        .spacing(10),
    )
    .width(169.5)
    .into()
}
