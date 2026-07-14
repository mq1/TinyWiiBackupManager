// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    games::game::Game,
    messages::Message,
    ui::components::{self, my_button},
};
use iced::{
    Element,
    widget::{column, row, rule, space, text},
};
use lucide_icons::Icon;

pub fn view<'a>(
    game: &'a Game,
    disc_info: Option<&'a wii_disc_info::Meta>,
) -> Element<'a, Message> {
    let content = if let Some(disc_info) = disc_info {
        text!("Region: {}", disc_info.region())
    } else {
        text("No disc info available")
    };

    components::card::view(
        column![
            column![
                text(&game.title).size(18),
                components::link::view(
                    game.path.to_string_lossy(),
                    Some(Icon::Folder),
                    game.path.as_os_str()
                ),
            ]
            .spacing(10)
            .padding(20),
            space::vertical(),
            content,
            space::vertical(),
            rule::horizontal(1),
            row![
                space::horizontal(),
                my_button::primary(Some("Close"), None).on_press(Message::CloseModal)
            ]
            .spacing(10)
            .padding(10)
        ]
        .width(600)
        .height(400),
    )
    .padding(0)
    .into()
}
