// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::message::Message;
use iced::Element;
use lucide_iced::{
    bytes::{METRONOME, USER_STAR, WAND_SPARKLES},
    themed_icon,
};

pub fn get_dev_icon(name: &str) -> Element<'_, Message> {
    let icon_bytes = match name {
        "blackb0x" | "USB Loader GX Team" => WAND_SPARKLES,
        "Aep" => METRONOME,
        _ => USER_STAR,
    };

    themed_icon(icon_bytes, 14.)
}
