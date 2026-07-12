// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    games::game::Game,
    messages::Message,
    ui::components::{self, Modal},
};
use iced::{
    Alignment, Element,
    widget::{column, image, row, space, text},
};
use iced_palace::widget::ellipsized_text;
use lucide_icons::{Icon, iced::icon_tag};

pub fn view<'a>((idx, game): (usize, &'a Game)) -> Element<'a, Message> {
    components::card::view(
        column![
            row![
                icon_tag(),
                game.id.as_str(),
                space::horizontal(),
                text!("{}", game.size)
            ]
            .spacing(5),
            image(&game.cached_cover_path).height(96),
            ellipsized_text(&game.title).wrapping(text::Wrapping::None),
            row![
                components::my_button::view(Some("Info"), Some(Icon::Info))
                    .on_press(Message::OpenModal(Modal::GameInfo(idx))),
                components::my_button::view(None, Some(Icon::HardDriveDownload)),
                components::my_button::view(None, Some(Icon::Trash))
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
