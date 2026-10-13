// SPDX-FileCopyrightText: 2026 TinyWiiBackupManager contributors
// SPDX-License-Identifier: GPL-3.0-only

use crate::config::message::ConfigMessage;
use iced::{
    Element, Length,
    widget::{checkbox, column, text},
};

pub fn fast_transfer(enabled: bool, can_change: bool) -> Element<'static, ConfigMessage> {
    column![
        checkbox(enabled)
            .label("Fast transfer")
            .on_toggle_maybe(can_change.then_some(ConfigMessage::SetFastTransfer)),
        text("Applies to imports and exports. Uses extra memory; speed depends on your drive.")
            .size(12),
        text("Keeps the chosen format and game data unchanged.").size(12),
    ]
    .spacing(4)
    .width(Length::Fill)
    .into()
}
