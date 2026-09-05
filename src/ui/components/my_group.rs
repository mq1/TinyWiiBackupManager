// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{messages::Message, ui::components::my_card::my_card};
use iced::{
    Element,
    widget::{Container, column, row, rule, space},
};
use lucide_icons::Icon;

pub fn my_group<'a>(
    title: &'a str,
    icon: Icon,
    top_right_content: impl Into<Element<'a, Message>>,
    content: impl Into<Element<'a, Message>>,
) -> Container<'a, Message> {
    my_card(
        column![
            row![
                icon.widget(),
                title,
                space::horizontal(),
                top_right_content.into()
            ]
            .spacing(5),
            rule::horizontal(1),
            content.into()
        ]
        .spacing(10)
        .padding(5),
    )
    .padding(5)
}
