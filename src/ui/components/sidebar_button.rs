// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::messages::Message;
use iced::{
    Background,
    border::Radius,
    widget::{Button, button},
};
use lucide_icons::Icon;

pub fn view<'a>(icon: Icon, active: bool) -> Button<'a, Message> {
    button(icon.widget().size(24).center())
        .width(42)
        .height(42)
        .style(move |theme, status| {
            let palette = theme.palette();

            let mut base = button::text(theme, status);

            base.border.radius = Radius::new(24);

            if active {
                base.background = Some(Background::Color(palette.primary.scale_alpha(0.5)));
            }

            base
        })
}
