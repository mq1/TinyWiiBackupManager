// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use iced::widget::Text;
use lucide_icons::iced::{icon_metronome, icon_user, icon_wand_sparkles};

/// Map of developers to their icons
/// If you want to add yourself, open a PR or an issue :)
pub fn get_icon(developer: &str) -> Text<'_> {
    match developer {
        "blackb0x" | "USB Loader GX Team" => icon_wand_sparkles(),
        "Aep" => icon_metronome(),
        _ => icon_user(),
    }
}
