// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    games::game::Game,
    messages::Message,
    ui::components::{my_button::MyButton, my_card::MyCard},
};
use iced::{
    Alignment, Element, Length,
    widget::{column, image, row, space, text},
};
use iced_palace::widget::ellipsized_text;
use lucide_icons::{Icon, iced::icon_tag};
use std::sync::Arc;

pub fn view(game: &Arc<Game>) -> Element<'_, Message> {
    MyCard::new(
        column![
            row![
                icon_tag(),
                text(game.id.as_str()),
                space::horizontal(),
                text(game.size.to_string())
            ]
            .spacing(5),
            image(&game.cached_cover_path).height(96),
            ellipsized_text(&game.title).wrapping(text::Wrapping::None),
            row![
                MyButton::new()
                    .label("Info")
                    .icon(Icon::Info)
                    .view()
                    .on_press_with(|| Message::OpenGameInfo(game.clone()))
                    .width(Length::Fill),
                MyButton::new().icon(Icon::HardDriveDownload).view(),
                MyButton::new()
                    .icon(Icon::Trash)
                    .view()
                    .on_press_with(|| Message::AskDeleteDir(game.path.clone()))
            ]
            .spacing(5)
        ]
        .align_x(Alignment::Center)
        .padding(5)
        .spacing(10),
    )
    .view()
    .width(172)
    .into()
}
