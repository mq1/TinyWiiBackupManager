// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::messages::Message;
use iced::{
    Alignment, Length,
    advanced::Widget,
    border::Border,
    widget::{Button, Row, button, column, text},
};
use lucide_icons::Icon;

pub fn view(label: Option<&str>, icon: Option<Icon>) -> Button<'_, Message> {
    let content = [
        icon.map(|i| i.widget().into()),
        label.map(|l| text(l).into()),
    ]
    .into_iter()
    .flatten()
    .collect::<Row<'_, _>>()
    .spacing(5);

    let content_width = content.size_hint().width;

    button(
        column![content]
            .align_x(Alignment::Center)
            .width(Length::Fill),
    )
    .style(move |theme, status| {
        let palette = theme.extended_palette();
        let mut base = button::subtle(theme, status);

        base.border = Border {
            width: 1.0,
            radius: 12.0.into(),
            color: palette.background.weak.color,
        };

        base
    })
    .width(content_width)
}

pub fn primary(label: Option<&str>, icon: Option<Icon>) -> Button<'_, Message> {
    view(label, icon).style(|theme, status| {
        let mut base = button::primary(theme, status);
        base.border.radius = 12.into();
        base
    })
}

pub fn danger(label: Option<&str>, icon: Option<Icon>) -> Button<'_, Message> {
    view(label, icon).style(|theme, status| {
        let mut base = button::danger(theme, status);
        base.border.radius = 12.into();
        base
    })
}
