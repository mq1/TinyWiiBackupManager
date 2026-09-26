// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    homebrew::homebrew_app::HomebrewApp,
    messages::Message,
    ui::components::{my_card::my_card, row_button::row_button},
};
use iced::{
    Element, Length, padding,
    widget::{row, rule, space, text, tooltip},
};
use lucide_icons::{
    Icon,
    iced::{icon_check, icon_message_circle_warning, icon_tag},
};

pub fn homebrew_app_row(app: &HomebrewApp) -> Element<'_, Message> {
    let version_icon: Element<'_, Message> = if let Some(osc_app) = app.osc_app() {
        if osc_app.version() == app.version() {
            tooltip(icon_check(), my_card("Up to date"), tooltip::Position::Top).into()
        } else {
            tooltip(
                icon_message_circle_warning(),
                my_card(text!("Update available ({})", osc_app.version())),
                tooltip::Position::Top,
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
            row_button(
                Icon::CloudDownload,
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
            Icon::Trash,
            "Delete app",
            Some(|| Message::AskDeleteDir(app.path().to_path_buf()))
        ),
        update_button,
        row_button(
            Icon::Info,
            "App info",
            Some(|| Message::OpenHomebrewAppInfo(app.clone()))
        ),
    ]
    .spacing(5)
    .height(Length::Shrink)
    .padding(padding::all(2).left(5))
    .into()
}
