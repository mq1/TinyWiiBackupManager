// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::messages::Message;
use iced::{
    Background, Element, Theme, padding,
    widget::{Container, container},
};

fn style(theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(theme.palette().background)),
        border: container::bordered_box(theme).border.rounded(10),
        ..container::bordered_box(theme)
    }
}

pub fn my_card<'a>(contents: impl Into<Element<'a, Message>>) -> Container<'a, Message> {
    container(contents)
        .style(style)
        .padding(padding::horizontal(10).vertical(5))
}
