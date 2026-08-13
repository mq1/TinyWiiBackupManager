// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::messages::Message;
use iced::{
    Element, Length,
    widget::{button, column, container, row, text, text::IntoFragment},
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

    let underline = container(row![].height(1).width(Length::Fill)).style(|theme| {
        let mut base = container::bordered_box(theme);
        base.border.color = theme.palette().primary;
        base
    });

    button(column![label, underline].width(Length::Shrink))
        .style(|theme, status| {
            let mut base = button::text(theme, status);
            base.text_color = theme.palette().primary;
            base
        })
        .padding(0)
        .on_press_with(move || Message::Open(url().into()))
        .into()
}
