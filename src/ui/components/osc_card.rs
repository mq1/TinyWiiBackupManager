// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    message::Message,
    state::State,
    ui::{components::my_tooltip, style},
};
use iced::{
    Alignment, Element,
    widget::{column, container, image, row, space, text},
};
use lucide_icons::iced::icon_pin;

pub fn view(state: &State, i: usize) -> Element<'_, Message> {
    let app = &state.osc_apps[i];

    let mut col = column![
        row![
            icon_pin(),
            text(&app.meta.version),
            space::horizontal(),
            text(&app.meta.uncompressed_size)
        ]
        .spacing(5)
        .align_y(Alignment::Center),
        space::vertical(),
    ]
    .spacing(5)
    .padding(10)
    .width(170)
    .height(220)
    .align_x(Alignment::Center);

    if let Some(icon) = state.get_osc_app_icon(app) {
        col = col.push(image(icon).height(100));
    }

    col = col
        .push(my_tooltip::view(
            container(text(&app.meta.name).wrapping(text::Wrapping::None)).clip(true),
            &app.meta.name,
        ))
        .push(space::vertical())
        .push(row![].spacing(5));

    container(col).style(style::card).into()
}
