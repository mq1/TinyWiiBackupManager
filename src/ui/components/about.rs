// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{APP_ICON, message::Message, ui::style};
use iced::{
    Alignment, Element,
    widget::{button, column, container, image, row, space, text},
};
use iced_fonts::lucide;

const COPYRIGHT_TEXT: &str = "Copyright © 2026 Manuel Quarneti";

pub fn view() -> Element<'static, Message> {
    let icon_handle = image::Handle::from_rgba(512, 512, &APP_ICON[..]);

    column![
        space::vertical(),
        container(
            column![
                image(icon_handle).width(100).height(100),
                text(env!("CARGO_PKG_NAME")).size(20),
                text(env!("CARGO_PKG_VERSION")),
                text(COPYRIGHT_TEXT),
            ]
            .spacing(10)
            .align_x(Alignment::Center)
        )
        .padding(40)
        .style(style::card),
        space::vertical(),
        row![
            button(row![lucide::github(), text("Source Code")].spacing(5))
                .style(style::rounded_button)
                .on_press(Message::OpenProjectRepo),
            space::horizontal(),
        ]
        .padding(10),
    ]
    .align_x(Alignment::Center)
    .into()
}
