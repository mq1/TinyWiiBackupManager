// SPDX-FileCopyrightText: 2023 Manuel Quarneti <manuel.quarneti@proton.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::types::message::Message;
use crate::TinyWiiBackupManager;
use iced::widget::{text, vertical_space, Column};
use iced::{Alignment, Element, Length};

pub fn view(_app: &TinyWiiBackupManager, remaining: usize) -> Element<Message> {
    Column::new()
        .push(vertical_space(Length::Fill))
        .push(text("Adding games...").size(30))
        .push(text(format!("{} games remaining", remaining)))
        .push(vertical_space(Length::Fill))
        .align_items(Alignment::Center)
        .width(Length::Fill)
        .spacing(8)
        .into()
}
