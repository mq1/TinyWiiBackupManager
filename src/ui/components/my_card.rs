// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{messages::Message, ui::my_palette};
use iced::{
    Element,
    border::radius,
    padding,
    widget::{Container, container},
};

pub struct MyCard<'a> {
    contents: Element<'a, Message>,
}

impl<'a> MyCard<'a> {
    pub fn new(contents: impl Into<Element<'a, Message>>) -> Self {
        Self {
            contents: contents.into(),
        }
    }

    pub fn view(self) -> Container<'a, Message> {
        container(self.contents)
            .style(|theme| {
                let mut base = container::bordered_box(theme);
                base.border.radius = radius(10);
                base.background = Some(my_palette::card_bg(theme));
                base
            })
            .padding(padding::horizontal(10).vertical(5))
    }
}
