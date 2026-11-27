// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    message::Message,
    osc::osc_app::OscApp,
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
use twbm_homebrew::homebrew_app::HomebrewApp;

pub fn homebrew_app_row<'a>(
    app: &'a HomebrewApp,
    osc_app: Option<&'a OscApp>,
) -> Element<'a, Message> {
    let (version_icon, newer_osc_app): (Element<'_, Message>, _) = if let Some(osc_app) = osc_app {
        if osc_app.version() == app.meta().version() {
            (
                tooltip(icon_check(), my_card("Up to date"), tooltip::Position::Top).into(),
                Some(osc_app),
            )
        } else {
            (
                tooltip(
                    icon_message_circle_warning(),
                    my_card(text!("Update available ({})", osc_app.version())),
                    tooltip::Position::Top,
                )
                .into(),
                None,
            )
        }
    } else {
        (icon_tag().into(), None)
    };

    let update_button = newer_osc_app.map(|osc_app| {
        row_button(
            Icon::CloudDownload,
            "Update (from oscwii.org)",
            Some(|| Message::InstallOscApp(osc_app.clone())),
        )
    });

    row![
        text!("{} ({})", app.meta().name(), app.meta().version()),
        version_icon,
        space::horizontal(),
        text!("{}", app.size().unwrap_or_default()),
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
            Some(|| Message::OpenHomebrewAppInfo(app.clone().into()))
        ),
    ]
    .spacing(5)
    .height(Length::Shrink)
    .padding(padding::all(2).left(5))
    .into()
}
