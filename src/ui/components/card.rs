// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::messages::Message;
use iced::{
    Element,
    border::radius,
    widget::{Container, container},
};

pub fn view<'a>(content: impl Into<Element<'a, Message>>) -> Container<'a, Message> {
    container(content)
        .style(|theme| {
            let mut base = container::bordered_box(theme);
            base.border.radius = radius(10);
            base
        })
        .padding(15)
}
