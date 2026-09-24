// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    homebrew::homebrew_app::HomebrewApp, messages::Message, ui::components::my_card::my_card,
};
use iced::{
    Element, Length, padding,
    widget::{button, row, rule, space, text, tooltip},
};
use lucide_icons::iced::{
    icon_check, icon_cloud_download, icon_info, icon_message_circle_warning, icon_tag, icon_trash,
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
            tooltip(
                button(icon_cloud_download().center())
                    .padding(0)
                    .on_press_with(|| Message::InstallOscApp(osc_app.clone()))
                    .style(button::text)
                    .width(20)
                    .height(20),
                my_card("Update (from oscwii.org)"),
                tooltip::Position::Top,
            )
        });

    row![
        text!("{} ({})", app.name(), app.version()),
        version_icon,
        space::horizontal(),
        text(app.size_str()),
        rule::vertical(1),
        tooltip(
            button(icon_trash().center())
                .padding(0)
                .on_press_with(|| Message::AskDeleteDir(app.path().to_path_buf()))
                .style(button::text)
                .width(20)
                .height(20),
            my_card("Delete app"),
            tooltip::Position::Top
        ),
        update_button,
        tooltip(
            button(icon_info().center())
                .padding(0)
                .on_press_with(|| Message::OpenHomebrewAppInfo(app.clone()))
                .style(button::text)
                .width(20)
                .height(20),
            my_card("App info"),
            tooltip::Position::Top
        ),
    ]
    .spacing(5)
    .height(Length::Shrink)
    .padding(padding::all(2).left(5))
    .into()
}
