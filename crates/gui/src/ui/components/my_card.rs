// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

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

pub fn my_card<'a, Msg>(contents: impl Into<Element<'a, Msg>>) -> Container<'a, Msg> {
    container(contents)
        .style(style)
        .padding(padding::horizontal(10).vertical(5))
}
