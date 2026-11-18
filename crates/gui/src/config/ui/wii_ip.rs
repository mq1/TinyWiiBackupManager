// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::config::{Config, message::ConfigMessage};
use iced::{
    Alignment, Element,
    widget::{row, text_input},
};

pub fn wii_ip(config: &Config) -> Element<'_, ConfigMessage> {
    row![
        "Wii IP →",
        text_input("192.168.1.100", config.wii_ip())
            .on_input(ConfigMessage::SetWiiIp)
            .width(150)
    ]
    .spacing(5)
    .align_y(Alignment::Center)
    .into()
}
