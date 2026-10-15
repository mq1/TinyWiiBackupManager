// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::message::Message;
use iced::{
    Element, Length,
    widget::{container, row},
};
use lucide_iced::{
    bytes::{ARROW_DOWN_LEFT, HARD_DRIVE},
    themed_icon,
};

pub fn no_drive() -> Element<'static, Message> {
    container(row![
        themed_icon(ARROW_DOWN_LEFT, 14.),
        "  Click on  ",
        themed_icon(HARD_DRIVE, 14.),
        "  to select a Drive/Mount Point"
    ])
    .center(Length::Fill)
    .into()
}
