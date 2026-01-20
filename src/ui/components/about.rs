// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{APP_ICON, message::Message, state::State, ui::style};
use iced::{
    Alignment, Element,
    widget::{button, column, container, image, row, space, text},
};
use lucide_icons::iced::{icon_folder, icon_github};

const COPYRIGHT_TEXT: &str = "Copyright © 2026 Manuel Quarneti";
const REPO_URI: &str = env!("CARGO_PKG_REPOSITORY");

pub fn view(state: &State) -> Element<'_, Message> {
    let icon_handle = image::Handle::from_rgba(256, 256, &APP_ICON[..]);

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
            button(row![icon_folder(), text("Data Directory")].spacing(5))
                .style(style::rounded_button)
                .on_press_with(|| Message::OpenThat(state.config.mount_point().as_os_str().into())),
            button(row![icon_github(), text("Source Code")].spacing(5))
                .style(style::rounded_button)
                .on_press_with(|| Message::OpenThat(REPO_URI.into())),
            space::horizontal(),
        ]
        .spacing(5)
        .padding(10),
    ]
    .align_x(Alignment::Center)
    .into()
}
