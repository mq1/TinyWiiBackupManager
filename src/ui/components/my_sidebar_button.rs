// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::messages::Message;
use iced::{
    Background, Element,
    widget::{button, stack},
};
use lucide_icons::Icon;

pub struct MySidebarButton<'a> {
    icons: &'a [Icon],
    active: bool,
    on_press: Option<Message>,
}

impl<'a> MySidebarButton<'a> {
    pub fn new(icons: &'a [Icon]) -> Self {
        Self {
            icons,
            active: false,
            on_press: None,
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
}

impl<'a> From<MySidebarButton<'a>> for Element<'_, Message> {
    fn from(mut value: MySidebarButton<'a>) -> Self {
        let content = stack(
            value
                .icons
                .iter()
                .map(|i| i.widget().size(24).center().into()),
        );

        let mut btn = button(content)
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

        btn.into()
    }
}

pub fn my_sidebar_button(icons: &[Icon]) -> MySidebarButton<'_> {
    MySidebarButton::new(icons)
}
