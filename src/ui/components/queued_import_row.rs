// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{messages::Message, ui::components::my_card::my_card};
use iced::{
    Element, Length, padding,
    widget::{button, row, space, text, tooltip},
};
use lucide_icons::iced::icon_x;
use std::path::PathBuf;

pub fn queued_import_row((i, path): (usize, &PathBuf)) -> Element<'_, Message> {
    row![
        text(path.to_string_lossy()),
        space::horizontal(),
        tooltip(
            button(icon_x().center())
                .padding(0)
                .on_press(Message::CancelImport(i))
                .style(button::text)
                .width(20)
                .height(20),
            my_card("Cancel"),
            tooltip::Position::Top
        ),
    ]
    .spacing(5)
    .height(Length::Shrink)
    .padding(padding::all(2).left(5))
    .into()
}
