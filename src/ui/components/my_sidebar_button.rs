// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::messages::Message;
use iced::{
    Background,
    widget::{Button, button, stack},
};
use lucide_icons::Icon;

pub fn my_sidebar_button<'a>(
    icons: impl IntoIterator<Item = Icon>,
    active: bool,
) -> Button<'a, Message> {
    let content = stack(
        icons
            .into_iter()
            .map(|i| i.widget().size(24).center().into()),
    );

    button(content)
        .width(42)
        .height(42)
        .style(move |theme, status| {
            let mut base = button::text(theme, status);
            base.border.radius = 24.into();

            if active {
                let color = theme.palette().primary.scale_alpha(0.5);
                base.background = Some(Background::Color(color));
            }

            base
        })
}
