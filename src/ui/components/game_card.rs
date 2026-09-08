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
use tap::Pipe;

pub fn game_card(game: &Game, is_exporting: bool) -> Element<'_, Message> {
    let cover: Element<'_, Message> = match &game.cover {
        Some(cover) => {
            let (w, h) = cover.dimensions();
            let bytes = cover.as_raw().clone();
            let handle = iced::widget::image::Handle::from_rgba(w, h, bytes);
            image(handle).height(96).into()
        }
        None => space().height(96).into(),
    };

    my_card(
        column![
            row![
                icon_tag(),
                text(game.id.as_str()),
                space::horizontal(),
                text(game.size.to_string())
            ]
            .spacing(5),
            cover,
            ellipsized_text(&game.title).wrapping(text::Wrapping::None),
            row![
                my_button()
                    .label("Info")
                    .icon(Icon::Info)
                    .on_press_with(|| Message::OpenGameInfo(game.clone()))
                    .expand_width(),
                tooltip(
                    my_button()
                        .icon(Icon::HardDriveDownload)
                        .pipe(|btn| match is_exporting {
                            false => btn.on_press_with(|| Message::PickExportDest(game.clone())),
                            true => btn,
                        }),
                    my_card("Export game"),
                    tooltip::Position::Bottom
                ),
                tooltip(
                    my_button()
                        .icon(Icon::Trash)
                        .on_press_with(|| Message::AskDeleteDir(game.path.clone())),
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
    .width(172)
    .into()
}
