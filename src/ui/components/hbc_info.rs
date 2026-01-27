// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    hbc::app::HbcApp,
    message::Message,
    state::State,
    ui::{components, style},
};
use iced::{
    Element, Length, padding,
    widget::{button, column, container, image, row, rule, scrollable, space, stack, text},
};
use lucide_icons::iced::{
    icon_calendar, icon_clipboard_list, icon_file_text, icon_folder, icon_globe, icon_info,
    icon_tag, icon_trash, icon_waves, icon_weight,
};

pub fn view<'a>(_state: &State, app: &'a HbcApp) -> Element<'a, Message> {
    let details = column![
        row![icon_info(), "Details"].spacing(5),
        rule::horizontal(1),
        space(),
        row![icon_tag(), text!("Version: {}", app.meta().version())].spacing(5),
        row![
            components::developers::get_icon(app.meta().coder()),
            text!("Coder: {}", app.meta().coder())
        ]
        .spacing(5),
        row![
            icon_calendar(),
            text!("Release date: {}", app.meta().release_date())
        ]
        .spacing(5),
        row![icon_weight(), text!("Size: {}", app.size())].spacing(5),
        row![
            icon_clipboard_list(),
            text!("Short description: {}", app.meta().short_description())
        ]
        .spacing(5),
    ]
    .padding(10)
    .width(Length::Fill)
    .spacing(5);

    let long_description = column![
        row![icon_file_text(), "Long description"].spacing(5),
        rule::horizontal(1),
        space(),
        text(app.meta().long_description()),
    ]
    .padding(10)
    .width(Length::Fill)
    .spacing(5);

    let col = column![
        row![icon_waves().size(18), text(app.meta().name()).size(18)]
            .spacing(5)
            .padding(padding::top(10).left(10)),
        button(
            row![
                icon_folder().style(text::primary),
                text!("Path: {}", app.path().display()).style(text::primary)
            ]
            .spacing(5)
            .padding(padding::left(10).bottom(10)),
        )
        .style(button::text)
        .padding(0)
        .on_press_with(|| Message::OpenThat(app.get_path_uri())),
        scrollable(
            column![
                container(details).style(style::card),
                container(long_description).style(style::card),
            ]
            .spacing(10)
            .padding(padding::horizontal(10))
        )
        .spacing(1)
        .height(Length::Fill),
        row![
            button(row![icon_globe(), text("Open OSC page")].spacing(5))
                .style(style::rounded_button)
                .on_press_with(|| Message::OpenThat(app.get_oscwii_uri())),
            button(row![icon_trash(), text("Delete")].spacing(5))
                .style(style::rounded_danger_button)
                .on_press_with(|| Message::AskDeleteDirConfirmation(app.path().clone())),
        ]
        .spacing(5)
        .padding(5)
    ];

    match app.image_path() {
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
