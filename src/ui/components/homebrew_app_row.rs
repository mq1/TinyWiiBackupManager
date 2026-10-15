// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    homebrew::homebrew_app::HomebrewApp,
    message::Message,
    ui::components::{my_card::my_card, row_button::row_button},
};
use iced::{
    Element, Length, padding,
    widget::{row, rule, space, text, tooltip},
};
use lucide_iced::{
    bytes::{CHECK, CLOUD_DOWNLOAD, INFO, MESSAGE_CIRCLE_WARNING, TAG, TRASH},
    themed_icon,
};

pub fn homebrew_app_row(app: &HomebrewApp) -> Element<'_, Message> {
    let version_icon: Element<'_, Message> = if let Some(osc_app) = app.osc_app() {
        if osc_app.version() == app.version() {
            tooltip(
                themed_icon(CHECK, 14.),
                my_card("Up to date"),
                tooltip::Position::Top,
            )
            .into()
        } else {
            tooltip(
                themed_icon(MESSAGE_CIRCLE_WARNING, 14.),
                my_card(text!("Update available ({})", osc_app.version())),
                tooltip::Position::Top,
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
            row_button(
                CLOUD_DOWNLOAD,
                "Update (from oscwii.org)",
                Some(|| Message::InstallOscApp(osc_app.clone())),
            )
        });

    row![
        text!("{} ({})", app.name(), app.version()),
        version_icon,
        space::horizontal(),
        text(app.size_str()),
        rule::vertical(1),
        row_button(
            TRASH,
            "Delete app",
            Some(|| Message::AskDeleteDir(app.path().to_path_buf()))
        ),
        update_button,
        row_button(
            INFO,
            "App info",
            Some(|| Message::OpenHomebrewAppInfo(app.clone()))
        ),
    ]
    .spacing(5)
    .height(Length::Shrink)
    .padding(padding::all(2).left(5))
    .into()
}
