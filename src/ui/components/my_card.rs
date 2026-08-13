// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::messages::Message;
use iced::{
    Background, Element, padding,
    widget::{Container, container},
};

pub fn my_card<'a>(contents: impl Into<Element<'a, Message>>) -> Container<'a, Message> {
    container(contents)
        .style(|theme| {
            let mut base = container::bordered_box(theme);
            base.border.radius = 10.into();
            base.background = Some(Background::Color(theme.palette().background));
            base
        })
        .padding(padding::horizontal(10).vertical(5))
}
