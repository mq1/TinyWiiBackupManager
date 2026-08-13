// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::messages::Message;
use iced::{
    Element, Length, Theme,
    widget::{button, column, row, rule, text, text::IntoFragment},
};
use lucide_icons::Icon;
use std::ffi::OsString;

pub fn my_link<'a, L, O, U, I>(label: L, url: U, icon: I) -> Element<'a, Message>
where
    L: IntoFragment<'a>,
    O: Into<OsString> + 'a,
    U: Fn() -> O + 'a,
    I: Into<Option<Icon>>,
{
    let icon = icon.into().unwrap_or(Icon::Globe).widget();
    let label = row![icon, text(label)].spacing(5);

    let underline = rule::horizontal(1).style(|theme: &Theme| rule::Style {
        color: theme.palette().primary,
        ..rule::default(theme)
    });

    button(column![label, underline].width(Length::Shrink))
        .style(|theme: &Theme, status| button::Style {
            text_color: theme.palette().primary,
            ..button::text(theme, status)
        })
        .padding(0)
        .on_press_with(move || Message::Open(url().into()))
        .into()
}
