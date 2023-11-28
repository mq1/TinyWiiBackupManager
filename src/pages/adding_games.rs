// SPDX-FileCopyrightText: 2023 Manuel Quarneti <manuelquarneti@protonmail.com>
// SPDX-License-Identifier: GPL-2.0-only

use crate::types::message::Message;
use crate::TinyWiiBackupManager;
use iced::widget::{column, text, vertical_space};
use iced::{Alignment, Element, Length};

pub fn view(_app: &TinyWiiBackupManager, remaining: usize) -> Element<Message> {
    column![
        vertical_space(Length::Fill),
        text("Adding games...").size(30),
        text(format!("{} games remaining", remaining)),
        vertical_space(Length::Fill),
    ]
    .align_items(Alignment::Center)
    .width(Length::Fill)
    .spacing(8)
    .into()
}
