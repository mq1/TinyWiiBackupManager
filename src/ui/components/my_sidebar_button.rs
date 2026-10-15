// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::message::Message;
use iced::{
    Background, Element, Length,
    widget::{button, container, stack, svg},
};
use iced_aw::style::colors::WHITE;

pub struct MySidebarButton<'a> {
    icons: &'a [&'static [u8]],
    active: bool,
    on_press: Option<Message>,
    on_press_with: Option<fn() -> Message>,
}

impl<'a> MySidebarButton<'a> {
    pub fn new(icons: &'a [&'static [u8]]) -> Self {
        Self {
            icons,
            active: false,
            on_press: None,
            on_press_with: None,
        }
    }

    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    pub fn on_press(mut self, on_press: Message) -> Self {
        self.on_press = Some(on_press);
        self
    }

    pub fn on_press_with(mut self, on_press_with: fn() -> Message) -> Self {
        self.on_press_with = Some(on_press_with);
        self
    }
}

impl<'a> From<MySidebarButton<'a>> for Element<'_, Message> {
    fn from(mut value: MySidebarButton<'a>) -> Self {
        let content = stack(value.icons.iter().map(|&icon| {
            svg(svg::Handle::from_memory(icon))
                .width(24.)
                .height(24.)
                .style(|_, _| svg::Style { color: Some(WHITE) })
                .into()
        }));

        let mut btn = button(container(content).center(Length::Fill))
            .width(42)
            .height(42)
            .style(move |theme, status| {
                let mut base = button::text(theme, status);
                base.border.radius = 24.into();

                if value.active {
                    let color = theme.palette().primary.scale_alpha(0.5);
                    base.background = Some(Background::Color(color));
                }

                base
            });

        if let Some(on_press) = value.on_press.take() {
            btn = btn.on_press(on_press);
        }

        if let Some(on_press_with) = value.on_press_with.take() {
            btn = btn.on_press_with(on_press_with);
        }

        btn.into()
    }
}

pub fn my_sidebar_button<'a>(icons: &'a [&'static [u8]]) -> MySidebarButton<'a> {
    MySidebarButton::new(icons)
}
