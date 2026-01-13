// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{message::Message, ui::style};
use iced::{
    Element,
    time::Duration,
    widget::{container, text, tooltip},
};

pub fn view<'a>(
    content: impl Into<Element<'a, Message>>,
    tooltip_str: &'a str,
) -> Element<'a, Message> {
    let popup: Element<'_, Message> = container(text(tooltip_str))
        .padding(5)
        .style(style::card)
        .into();

    tooltip(content, popup, tooltip::Position::FollowCursor)
        .delay(Duration::from_secs(1))
        .into()
}
