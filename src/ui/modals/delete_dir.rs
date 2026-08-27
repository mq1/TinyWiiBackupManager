// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    messages::Message,
    ui::components::{
        my_button::{MyButtonKind, my_button},
        my_card::my_card,
    },
};
use iced::{
    Element, Length,
    widget::{column, container, row, rule, space, text},
};
use std::path::Path;

pub fn delete_dir(path: &Path) -> Element<'_, Message> {
    my_card(
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
                my_button("Cancel", None, MyButtonKind::Secondary).on_press(Message::CloseModal),
                my_button("Ok", None, MyButtonKind::Danger)
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
