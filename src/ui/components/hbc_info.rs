// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    message::Message,
    state::State,
    ui::{components, style},
};
use iced::{
    Element, Length,
    widget::{button, column, image, row, rule, scrollable, space, stack, text},
};
use lucide_icons::iced::{
    icon_calendar, icon_clipboard_list, icon_folder, icon_globe, icon_tag, icon_trash, icon_waves,
    icon_weight,
};

pub fn view(state: &State, hbc_i: usize) -> Element<'_, Message> {
    let app = &state.hbc_apps[hbc_i];

    let col = column![
        row![icon_waves().size(18), text(&app.meta.name).size(18)].spacing(5),
        row![icon_folder(), text("Path:"), text(app.get_path_str())].spacing(5),
        rule::horizontal(1),
        row![icon_tag(), text("Version:"), text(&app.meta.version)].spacing(5),
        row![
            components::developers::get_icon(&app.meta.coder),
            text("Coder:"),
            text(&app.meta.coder)
        ]
        .spacing(5),
        row![
            icon_calendar(),
            text("Release Date:"),
            text(app.meta.release_date.to_string())
        ]
        .spacing(5),
        row![icon_weight(), text("Size:"), text(app.size.to_string())].spacing(5),
        row![
            icon_clipboard_list(),
            text("Short Description:"),
            text(&app.meta.short_description)
        ]
        .spacing(5),
        rule::horizontal(1),
        scrollable(text(&app.meta.long_description).width(Length::Fill)).height(Length::Fill),
        rule::horizontal(1),
        row![
            button(row![icon_globe(), text("Open OSC Page")].spacing(5))
                .style(style::rounded_button)
                .on_press(Message::OpenHbcPage(hbc_i)),
            button(row![icon_trash(), text("Delete")].spacing(5))
                .style(style::rounded_danger_button)
                .on_press(Message::AskDeleteHbcApp(hbc_i)),
        ]
        .spacing(5)
    ]
    .spacing(5)
    .padding(10);

    match &app.image_path {
        Some(image_path) => stack![
            col,
            row![
                space::horizontal(),
                image(image_path)
                    .height(96)
                    .expand(true)
                    .filter_method(image::FilterMethod::Linear)
            ]
            .padding(10)
        ]
        .into(),
        None => col.into(),
    }
}
