// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    homebrew::homebrew_app::HomebrewApp,
    message::Message,
    ui::components::{my_button::my_button, my_card::my_card},
};
use iced::{
    Alignment, Element,
    widget::{column, image, row, space, text, tooltip},
};
use iced_palace::widget::ellipsized_text;
use lucide_iced::{
    bytes::{CHECK, CLOUD_DOWNLOAD, INFO, MESSAGE_CIRCLE_WARNING, TAG, TRASH},
    themed_icon,
};

pub fn homebrew_app_card(app: &HomebrewApp) -> Element<'_, Message> {
    let icon: Element<'_, Message> = match app.icon() {
        Some(icon) => image(icon).height(96).into(),
        None => space().height(96).into(),
    };

    let version_icon: Element<'_, Message> = if let Some(osc_app) = app.osc_app() {
        if osc_app.version() == app.version() {
            tooltip(
                themed_icon(CHECK, 14.),
                my_card("Up to date"),
                tooltip::Position::Bottom,
            )
            .into()
        } else {
            tooltip(
                themed_icon(MESSAGE_CIRCLE_WARNING, 14.),
                my_card(text!("Update available ({})", osc_app.version())),
                tooltip::Position::Bottom,
            )
            .into()
        }
    } else {
        themed_icon(TAG, 14.)
    };

    let update_button = app
        .osc_app()
        .as_ref()
        .filter(|osc_app| osc_app.version() != app.version())
        .map(|osc_app| {
            tooltip(
                my_button()
                    .icon(CLOUD_DOWNLOAD)
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
                    .icon(INFO)
                    .expand_width()
                    .on_press_with(|| Message::OpenHomebrewAppInfo(app.clone())),
                update_button,
                my_button()
                    .icon(TRASH)
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
