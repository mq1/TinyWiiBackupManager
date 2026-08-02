// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{messages::Message, ui::components::my_button};
use iced::{Element, widget::row};

pub fn view<'a>() -> Element<'a, Message> {
    row![
        my_button::view(Some("Grid"), None).on_press(Message::ViewAsGrid),
        my_button::view(Some("Table"), None).on_press(Message::ViewAsTable)
    ]
    .spacing(5)
    .into()
}
