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
use lucide_icons::iced::{icon_cloud_download, icon_info, icon_monitor_up, icon_pin};

pub fn view(state: &State, i: usize) -> Element<'_, Message> {
    let app = &state.osc_apps[i];

    let mut col = column![
        row![
            icon_pin().size(12),
            my_tooltip::view(text(app.get_trimmed_version_str()), &app.meta.version),
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
    .height(160)
    .align_x(Alignment::Center);

    if let Some(icon) = state.get_osc_app_icon(app) {
        col = col.push(image(icon).height(48));
    }

    col = col
        .push(my_tooltip::view(
            container(text(&app.meta.name).wrapping(text::Wrapping::None)).clip(true),
            &app.meta.name,
        ))
        .push(space::vertical())
        .push(
            row![
                button(row![icon_info(), text("Info")].spacing(5))
                    .style(style::rounded_secondary_button)
                    .on_press(Message::NavigateTo(Screen::OscInfo(i))),
                button(icon_monitor_up()).style(style::rounded_secondary_button),
                button(icon_cloud_download())
                    .style(style::rounded_secondary_button)
                    .on_press(Message::AskInstallOscApp(i)),
            ]
            .spacing(5),
        );

    container(col).style(style::card).into()
}
