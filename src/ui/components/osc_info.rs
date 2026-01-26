// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    hbc::osc::{Flag, OscAppMeta, Peripheral, Platform},
    message::Message,
    state::State,
    ui::{components, style},
};
use iced::{
    Element, Length, padding,
    widget::{button, column, container, image, row, rule, scrollable, space, stack, text},
};
use itertools::Itertools;
use lucide_icons::iced::{
    icon_calendar, icon_clipboard_list, icon_cloud_download, icon_computer, icon_file_text,
    icon_flag, icon_folders, icon_globe, icon_info, icon_monitor_up, icon_package, icon_store,
    icon_tag, icon_usb, icon_users, icon_weight,
};

#[allow(clippy::too_many_lines)]
pub fn view<'a>(state: &State, app: &'a OscAppMeta) -> Element<'a, Message> {
    let details = column![
        row![icon_info(), "Details"].spacing(10),
        rule::horizontal(1),
        space(),
        row![icon_tag(), text!("Version: {}", app.version())].spacing(5),
        row![
            components::developers::get_icon(app.author()),
            text!("Author: {}", app.author())
        ]
        .spacing(5),
        row![icon_users(), text!("Authors: {}", app.authors().join(", "))].spacing(5),
        row![icon_tag(), text!("Category: {}", app.category())].spacing(5),
        row![
            icon_users(),
            text!("Contributors: {}", app.contributors().join(", "))
        ]
        .spacing(5),
        row![
            icon_cloud_download(),
            text!("Downloads: {}", app.downloads())
        ]
        .spacing(5),
        row![
            icon_flag(),
            text!(
                "Flags: {}",
                app.flags()
                    .iter()
                    .map(Flag::as_str)
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        ]
        .spacing(5),
        row![
            icon_package(),
            text!("Package Type: {}", app.package_type().as_str())
        ]
        .spacing(5),
        row![
            icon_usb(),
            text!(
                "Peripherals: {}",
                app.peripherals().iter().map(Peripheral::as_str).join(", ")
            )
        ]
        .spacing(5),
        row![
            icon_calendar(),
            text!("Release Date: {}", app.release_date().date())
        ]
        .spacing(5),
        row![
            icon_folders(),
            text!("Subdirectories: {}", app.subdirectories().join(", "))
        ]
        .spacing(5),
        row![
            icon_computer(),
            text!(
                "Supported Platforms: {}",
                app.supported_platforms()
                    .iter()
                    .map(Platform::as_str)
                    .join(", ")
            )
        ]
        .spacing(5),
        row![
            icon_weight(),
            text!("Uncompressed Size: {}", app.uncompressed_size())
        ]
        .spacing(5),
        row![
            icon_clipboard_list(),
            text!("Short description: {}", app.description().short())
        ]
        .spacing(5),
    ]
    .padding(10)
    .spacing(5)
    .width(Length::Fill);

    let long_description = column![
        row![icon_file_text(), "Long description"].spacing(5),
        rule::horizontal(1),
        space(),
        text(app.description().long()),
    ]
    .padding(10)
    .spacing(5)
    .width(Length::Fill);

    let mut actions = row![
        button(row![icon_globe(), text("Open OSC Page")].spacing(5))
            .style(style::rounded_button)
            .on_press_with(|| Message::OpenThat(app.get_oscwii_uri())),
        button(row![icon_monitor_up(), text("Send via Wiiload")].spacing(5))
            .style(style::rounded_button)
            .on_press_with(|| Message::WiiloadOsc(app.clone())),
    ]
    .spacing(5)
    .padding(5);

    if state.config.is_mount_point_valid() {
        actions = actions.push(
            button(row![icon_cloud_download(), text("Install")].spacing(5))
                .style(style::rounded_button)
                .on_press_with(|| Message::AskInstallOscApp(app.clone())),
        );
    } else {
        actions = actions.push(
            button(row![icon_cloud_download(), text("Install")].spacing(5))
                .style(style::rounded_button),
        );
    }

    let col = column![
        row![icon_store().size(18), text(app.name()).size(18)]
            .spacing(5)
            .padding(10),
        scrollable(
            column![
                container(details).style(style::card),
                container(long_description).style(style::card)
            ]
            .spacing(10)
            .padding(padding::horizontal(10))
        )
        .spacing(1)
        .height(Length::Fill),
        actions
    ];

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
