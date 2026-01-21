// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    message::Message,
    state::State,
    ui::{Screen, components::my_tooltip, style},
};
use iced::{
    Alignment, Element,
    widget::{button, column, container, image, row, space, text},
};
use lucide_icons::iced::{icon_info, icon_pin, icon_trash};

pub fn view(state: &State, i: usize) -> Element<'_, Message> {
    let app = state.hbc_app_list.get_unchecked(i);

    let mut col = column![
        row![
            icon_pin().size(12),
            my_tooltip::view(text(app.get_trimmed_version_str()), app.meta().version()),
            space::horizontal(),
            text(app.size().to_string())
        ]
        .spacing(5)
        .align_y(Alignment::Center),
        space::vertical(),
    ]
    .spacing(5)
    .padding(10)
    .width(170)
    .height(160)
    .align_x(Alignment::Center);

    if let Some(image_path) = app.image_path() {
        col = col.push(image(image_path).height(48));
    }

    col = col
        .push(my_tooltip::view(
            container(text(app.meta().name()).wrapping(text::Wrapping::None)).clip(true),
            app.meta().name(),
        ))
        .push(space::vertical())
        .push(
            row![
                button(row![icon_info(), text("Info")].spacing(5))
                    .style(style::rounded_secondary_button)
                    .on_press(Message::NavigateTo(Screen::HbcInfo(i))),
                button(icon_trash())
                    .style(style::rounded_secondary_button)
                    .on_press_with(|| Message::AskDeleteDirConfirmation(app.path().to_path_buf())),
            ]
            .spacing(5),
        );

    container(col).style(style::card).into()
}
