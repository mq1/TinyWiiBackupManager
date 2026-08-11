// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    messages::Message,
    ui::components::{self, my_button::MyButton},
};
use iced::{
    Element, Length,
    widget::{column, container, row, rule, space, text},
};
use std::path::Path;

pub fn view(path: &Path) -> Element<'_, Message> {
    components::card::view(
        column![
            container(text!(
                "Are you sure you want to delete {}?\nThis action cannot be undone.",
                path.file_name().unwrap_or_default().display()
            ))
            .padding(10),
            space::vertical(),
            rule::horizontal(1),
            row![
                space::horizontal(),
                MyButton::new()
                    .label("Cancel")
                    .view()
                    .on_press(Message::CloseModal),
                MyButton::new()
                    .label("Ok")
                    .view()
                    .on_press_with(|| Message::DeleteDir(path.to_path_buf()))
            ]
            .spacing(10)
            .padding(10)
        ]
        .width(600)
        .height(Length::Shrink),
    )
    .padding(0)
    .into()
}
