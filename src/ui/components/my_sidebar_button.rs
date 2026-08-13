// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::messages::Message;
use iced::{
    Background,
    border::radius,
    widget::{Button, button, stack},
};
use lucide_icons::Icon;

pub struct MySidebarButton<'a> {
    icons: &'a [Icon],
    active: bool,
}

impl<'a> MySidebarButton<'a> {
    pub fn new(icons: &'a [Icon]) -> Self {
        Self {
            icons,
            active: false,
        }
    }

    pub fn active_if(mut self, condition: bool) -> Self {
        self.active = condition;
        self
    }

    pub fn view(self) -> Button<'a, Message> {
        let content = stack(
            self.icons
                .iter()
                .map(|i| i.widget().size(24).center().into()),
        );

        button(content)
            .width(42)
            .height(42)
            .style(move |theme, status| {
                let palette = theme.palette();

                let mut base = button::text(theme, status);

                base.border.radius = radius(24);

                if self.active {
                    base.background = Some(Background::Color(palette.primary.scale_alpha(0.5)));
                }

                base
            })
    }
}
