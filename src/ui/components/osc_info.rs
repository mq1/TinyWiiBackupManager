// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    message::Message,
    osc::{Flag, Peripheral, Platform},
    state::State,
    ui::{components, style},
};
use iced::{
    Element, Length,
    widget::{button, column, image, row, rule, scrollable, space, stack, text},
};
use iced_fonts::lucide;

pub fn view(state: &State, osc_i: usize) -> Element<'_, Message> {
    let app = &state.osc_apps[osc_i];

    let col = column![
        row![
            lucide::shopping_bag().size(18),
            text(&app.meta.name).size(18)
        ]
        .spacing(5),
        rule::horizontal(1),
        row![lucide::tag(), text("Version:"), text(&app.meta.version)].spacing(5),
        row![
            components::developers::get_icon(&app.meta.author),
            text("Author:"),
            text(&app.meta.author)
        ]
        .spacing(5),
        row![
            lucide::users(),
            text("Authors:"),
            text(app.meta.authors.join(", "))
        ]
        .spacing(5),
        row![lucide::tag(), text("Category:"), text(&app.meta.category)].spacing(5),
        row![
            lucide::users(),
            text("contributors:"),
            text(app.meta.contributors.join(", "))
        ]
        .spacing(5),
        row![
            lucide::cloud_download(),
            text("Downloads:"),
            text(app.meta.downloads)
        ]
        .spacing(5),
        row![
            lucide::flag(),
            text("Flags:"),
            text(
                app.meta
                    .flags
                    .iter()
                    .map(Flag::as_str)
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        ]
        .spacing(5),
        row![
            lucide::package(),
            text("Package Type:"),
            text(app.meta.package_type.as_str())
        ]
        .spacing(5),
        row![
            lucide::usb(),
            text("Peripherals:"),
            text(
                app.meta
                    .peripherals
                    .iter()
                    .map(Peripheral::as_str)
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        ]
        .spacing(5),
        row![
            lucide::calendar(),
            text("Release Date:"),
            text(app.meta.release_date.to_string())
        ]
        .spacing(5),
        row![
            lucide::folders(),
            text("Subdirectories:"),
            text(app.meta.subdirectories.join(", "))
        ]
        .spacing(5),
        row![
            lucide::computer(),
            text("Supported Platforms:"),
            text(
                app.meta
                    .supported_platforms
                    .iter()
                    .map(Platform::as_str)
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        ]
        .spacing(5),
        row![
            lucide::weight(),
            text("Uncompressed Size:"),
            text(&app.meta.uncompressed_size)
        ]
        .spacing(5),
        row![
            lucide::clipboard_list(),
            text("Short Description:"),
            text(&app.meta.description.short)
        ]
        .spacing(5),
        rule::horizontal(1),
        scrollable(text(&app.meta.description.long))
            .width(Length::Fill)
            .height(Length::Fill),
        row![
            button(row![lucide::globe(), text("Open OSC Page")].spacing(5))
                .style(style::rounded_button)
                .on_press(Message::OpenOscPage(osc_i)),
            button(row![lucide::monitor_up(), text("Send via Wiiload")].spacing(5))
                .style(style::rounded_button),
            button(row![lucide::cloud_download(), text("Download")].spacing(5))
                .style(style::rounded_button)
        ]
        .spacing(5)
    ]
    .spacing(5)
    .padding(10);

    match state.get_osc_app_icon(app) {
        Some(icon) => stack![
            col,
            row![
                space::horizontal(),
                image(icon)
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
