// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::message::Message;
use iced::{
    Element, Length,
    widget::{container, row},
};
use lucide_icons::iced::{icon_arrow_down_left, icon_hard_drive};

pub fn no_drive() -> Element<'static, Message> {
    container(row![
        icon_arrow_down_left(),
        "  Click on  ",
        icon_hard_drive(),
        "  to select a Drive/Mount Point"
    ])
    .center(Length::Fill)
    .into()
}
