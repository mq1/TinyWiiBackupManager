// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{games::ImportEntry, message::Message, ui::components::row_button::row_button};
use iced::{
    Element, Length, padding,
    widget::{row, space, text},
};
use lucide_icons::Icon;

pub fn queued_import_row((i, entry): (usize, &ImportEntry)) -> Element<'_, Message> {
    row![
        text(entry.path().to_string_lossy()),
        space::horizontal(),
        row_button(Icon::X, "Cancel", Some(move || Message::CancelImport(i)))
    ]
    .spacing(5)
    .height(Length::Shrink)
    .padding(padding::all(2).left(5))
    .into()
}
