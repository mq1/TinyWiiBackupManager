// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::messages::Message;
use iced::{
    Element,
    widget::{row, text},
};
use lucide_icons::iced::icon_chevron_right;

pub fn view<'a>() -> Element<'a, Message> {
    row![icon_chevron_right().size(20), text("Plugins").size(20),]
        .spacing(5)
        .into()
}
