// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    hbc::osc::OscAppMeta,
    message::Message,
    state::State,
    ui::{Screen, components::my_tooltip, style},
};
use iced::{
    Alignment, Element,
    widget::{button, column, container, image, row, space, text},
};
use iced_palace::widget::ellipsized_text;
use lucide_icons::iced::{icon_cloud_download, icon_info, icon_monitor_up, icon_pin};

pub fn view<'a>(state: &State, app: &'a OscAppMeta) -> Element<'a, Message> {
    let mut col = column![
        row![
            icon_pin().size(12),
            my_tooltip::view(
                ellipsized_text(app.version())
                    .width(65)
                    .wrapping(text::Wrapping::None),
                app.version()
            ),
            space::horizontal(),
            text(app.uncompressed_size().to_string())
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

    let mut actions = row![
        my_tooltip::view(
            button(row![icon_info(), text("Info")].spacing(5))
                .style(style::rounded_secondary_button)
                .on_press_with(|| Message::NavTo(Screen::OscInfo(app.clone()))),
            "Show app info",
        ),
        my_tooltip::view(
            button(icon_monitor_up())
                .style(style::rounded_secondary_button)
                .on_press_with(|| Message::WiiloadOsc(app.clone())),
            "Send app via Wiiload"
        ),
    ]
    .spacing(5);

    if state.config.is_mount_point_valid() {
        actions = actions.push(my_tooltip::view(
            button(icon_cloud_download())
                .style(style::rounded_secondary_button)
                .on_press_with(|| Message::AskInstallOscApp(app.clone())),
            "Install app",
        ));
    } else {
        actions = actions.push(my_tooltip::view(
            button(icon_cloud_download()).style(style::rounded_secondary_button),
            "Install app",
        ));
    }

    col = col
        .push(my_tooltip::view(
            container(text(app.name()).wrapping(text::Wrapping::None)).clip(true),
            app.name(),
        ))
        .push(space::vertical())
        .push(actions);

    container(col).style(style::card).into()
}
