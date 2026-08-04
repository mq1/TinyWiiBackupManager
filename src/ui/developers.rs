// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::messages::Message;
use iced::Element;
use lucide_icons::Icon;

pub fn get_dev_icon(name: &str) -> Element<'_, Message> {
    let icon = match name {
        "blackb0x" | "USB Loader GX Team" => Icon::Wand2,
        "Aep" => Icon::Metronome,
        _ => Icon::UserStar,
    };

    icon.widget().into()
}
