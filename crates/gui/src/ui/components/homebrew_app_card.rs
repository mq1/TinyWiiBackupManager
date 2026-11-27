// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    message::Message,
    osc::osc_app::OscApp,
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
use twbm_homebrew::homebrew_app::HomebrewApp;

pub fn homebrew_app_card<'a>(
    app: &'a HomebrewApp,
    osc_app: Option<&'a OscApp>,
) -> Element<'a, Message> {
    let icon: Element<'_, Message> = match app.icon() {
        Some(icon) => image(icon).height(96).into(),
        None => space().height(96).into(),
    };

    let (version_icon, newer_osc_app): (Element<'_, Message>, _) = if let Some(osc_app) = osc_app {
        if osc_app.version() == app.meta().version() {
            (
                tooltip(
                    icon_check(),
                    my_card("Up to date"),
                    tooltip::Position::Bottom,
                )
                .into(),
                Some(osc_app),
            )
        } else {
            (
                tooltip(
                    icon_message_circle_warning(),
                    my_card(text!("Update available ({})", osc_app.version())),
                    tooltip::Position::Bottom,
                )
                .into(),
                None,
            )
        }
    } else {
        (icon_tag().into(), None)
    };

    let update_button = newer_osc_app.map(|osc_app| {
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
                ellipsized_text(app.meta().version())
                    .wrapping(text::Wrapping::None)
                    .width(60),
                space::horizontal(),
                text!("{}", app.size().unwrap_or_default()).wrapping(text::Wrapping::None)
            ]
            .spacing(5),
            icon,
            ellipsized_text(app.meta().name()).wrapping(text::Wrapping::None),
            row![
                my_button()
                    .label("Info")
                    .icon(Icon::Info)
                    .expand_width()
                    .on_press_with(|| Message::OpenHomebrewAppInfo(app.clone().into())),
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
