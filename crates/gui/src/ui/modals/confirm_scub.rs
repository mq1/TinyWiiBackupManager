// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    games::game::Game,
    message::Message,
    ui::components::{
        my_button::{MyButtonKind, my_button},
        my_card::my_card,
    },
};
use iced::{
    Element, Length,
    widget::{column, container, row, rule, space, text},
};

pub fn confirm_scrub(game: &Game) -> Element<'_, Message> {
    my_card(
        column![
            container(text!(
                "Are you sure you want to remove the update partition from {}?\nThis action cannot be undone.",
                game.title()
            ))
            .padding(10),
            space::vertical(),
            rule::horizontal(1),
            row![
                space::horizontal(),
                my_button().label("Cancel").on_press(Message::CloseModal),
                my_button()
                    .label("Ok")
                    .kind(MyButtonKind::Danger)
                    .on_press_with(|| Message::Scrub(game.clone()))
            ]
            .spacing(10)
            .padding(10)
        ]
        .width(600)
        .height(Length::Shrink),
    )
    .padding(0)
    .into()
}
