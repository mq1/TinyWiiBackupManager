// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    message::Message,
    osc::{Flag, Peripheral, Platform},
    state::State,
    ui::style,
};
use iced::{
    Element,
    widget::{button, column, image, row, rule, scrollable, space, text},
};
use iced_fonts::lucide;

pub fn view(state: &State, osc_i: usize) -> Element<'_, Message> {
    let app = &state.osc_apps[osc_i];

    let mut row = row![
        column![
            row![
                lucide::shopping_bag().size(18),
                text(&app.meta.name).size(18)
            ]
            .spacing(5),
            rule::horizontal(1),
            row![lucide::tag(), text("Version:"), text(&app.meta.version)].spacing(5),
            row![lucide::user(), text("Author:"), text(&app.meta.author)].spacing(5),
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
                text(&app.meta.release_date)
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
            scrollable(text(&app.meta.description.long)),
            space::vertical(),
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
    ]
    .padding(10);

    if let Some(icon) = state.get_osc_app_icon(app) {
        row = row.push(
            button(image(icon).height(50))
                .style(button::text)
                .on_press(Message::OpenOscIcon(osc_i)),
        );
    }

    row.into()
}
