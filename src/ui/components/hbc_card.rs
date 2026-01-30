// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    hbc::app::HbcApp,
    message::Message,
    state::State,
    ui::{Screen, components::my_tooltip, style},
};
use iced::{
    Alignment, Element,
    widget::{button, column, container, image, row, space, text},
};
use iced_palace::widget::ellipsized_text;
use lucide_icons::iced::{icon_circle_arrow_up, icon_info, icon_pin, icon_trash};

pub fn view<'a>(state: &'a State, app: &'a HbcApp) -> Element<'a, Message> {
    let mut col = column![
        row![
            icon_pin().size(12),
            my_tooltip::view(
                ellipsized_text(app.meta().version())
                    .width(65)
                    .wrapping(text::Wrapping::None),
                app.meta().version()
            ),
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

    let mut actions = row![my_tooltip::view(
        button(row![icon_info(), text("Info")].spacing(5))
            .style(style::rounded_secondary_button)
            .on_press_with(|| Message::NavTo(Screen::HbcInfo(app.clone()))),
        "Show app info"
    ),]
    .spacing(5);

    if let Some(osc_i) = app.osc_i() {
        let osc_app = state.osc_app_list.get_unchecked(osc_i);

        if app.meta().version() != osc_app.version() {
            actions = actions.push(my_tooltip::view(
                button(icon_circle_arrow_up())
                    .style(style::rounded_secondary_button)
                    .on_press_with(|| Message::AskInstallOscApp(osc_app.clone())),
                "Update app to latest version",
            ));
        }
    }

    actions = actions.push(my_tooltip::view(
        button(icon_trash())
            .style(style::rounded_secondary_button)
            .on_press_with(|| Message::AskDeleteDirConfirmation(app.path().clone())),
        "Delete app from drive",
    ));

    col = col
        .push(my_tooltip::view(
            container(text(app.meta().name()).wrapping(text::Wrapping::None)).clip(true),
            app.meta().name(),
        ))
        .push(space::vertical())
        .push(actions);

    container(col).style(style::card).into()
}
