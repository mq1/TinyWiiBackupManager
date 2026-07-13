// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::messages::Message;
use iced::{
    Element, Length,
    widget::{button, column, container, row, text},
};
use lucide_icons::Icon;
use std::{borrow::Cow, ffi::OsStr};

pub fn view<'a>(
    label: impl Into<Cow<'a, str>>,
    icon: Option<Icon>,
    url: &'a OsStr,
) -> Element<'a, Message> {
    let label = row![icon.unwrap_or(Icon::Globe).widget(), text(label.into())].spacing(5);

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
        .on_press_with(|| Message::Open(url.to_os_string()))
        .into()
}
