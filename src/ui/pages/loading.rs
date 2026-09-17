// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::messages::Message;
use iced::{
    Alignment, Element, Length,
    widget::{column, container, text},
};
use iced_aw::Spinner;

pub fn loading() -> Element<'static, Message> {
    container(
        column![text("Loading..."), Spinner::new()]
            .spacing(10)
            .align_x(Alignment::Center),
    )
    .center(Length::Fill)
    .into()
}
