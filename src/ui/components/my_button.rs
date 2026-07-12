// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::messages::Message;
use iced::{
    border::Border,
    widget::{Button, Row, button, text},
};
use lucide_icons::Icon;

pub fn view<'a>(label: Option<&'a str>, icon: Option<Icon>) -> Button<'a, Message> {
    let content = [
        label.map(|l| text(l).into()),
        icon.map(|i| i.widget().into()),
    ]
    .into_iter()
    .flatten()
    .collect::<Row<_>>()
    .spacing(5);

    button(content).style(move |theme, status| {
        let palette = theme.extended_palette();
        let mut base = button::subtle(theme, status);

        base.border = Border {
            width: 1.0,
            radius: 12.0.into(),
            color: palette.background.weak.color,
        };

        base
    })
}
