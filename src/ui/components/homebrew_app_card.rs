// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    homebrew::homebrew_app::HomebrewApp,
    messages::Message,
    ui::components::{my_button::my_button, my_card::my_card},
};
use iced::{
    Alignment, Element,
    widget::{column, image, row, space, text, tooltip},
};
use iced_palace::widget::ellipsized_text;
use lucide_icons::{
    Icon,
    iced::{icon_check, icon_message_circle_warning, icon_tag},
};

pub fn homebrew_app_card(app: &HomebrewApp) -> Element<'_, Message> {
    let icon: Element<'_, Message> = match app.icon() {
        Some(icon) => image(icon).height(96).into(),
        None => space().height(96).into(),
    };

    let version_icon: Element<'_, Message> = if let Some(osc_app) = app.osc_app() {
        if osc_app.version() == app.version() {
            tooltip(
                icon_check(),
                my_card("Up to date"),
                tooltip::Position::Bottom,
            )
            .into()
        } else {
            tooltip(
                icon_message_circle_warning(),
                my_card(text!("Update available ({})", osc_app.version())),
                tooltip::Position::Bottom,
            )
            .into()
        }
    } else {
        icon_tag().into()
    };

    let update_button = app
        .osc_app()
        .as_ref()
        .filter(|osc_app| osc_app.version() != app.version())
        .map(|osc_app| {
            tooltip(
                my_button()
                    .icon(Icon::CloudDownload)
                    .on_press_with(|| Message::InstallOscApp(osc_app.clone())),
                my_card("Update (from oscwii.org)"),
                tooltip::Position::Bottom,
            )
        });

    my_card(
        column![
            row![
                version_icon,
                ellipsized_text(app.version())
                    .wrapping(text::Wrapping::None)
                    .width(60),
                space::horizontal(),
                text(app.size_str()).wrapping(text::Wrapping::None)
            ]
            .spacing(5),
            icon,
            ellipsized_text(app.name()).wrapping(text::Wrapping::None),
            row![
                my_button()
                    .label("Info")
                    .icon(Icon::Info)
                    .expand_width()
                    .on_press_with(|| Message::OpenHomebrewAppInfo(app.clone())),
                update_button,
                my_button()
                    .icon(Icon::Trash)
                    .on_press_with(|| Message::AskDeleteDir(app.path().to_path_buf()))
            ]
            .spacing(5)
        ]
        .align_x(Alignment::Center)
        .padding(5)
        .spacing(10),
    )
    .width(169.5)
    .into()
}
