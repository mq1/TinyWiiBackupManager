// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    messages::Message,
    osc::osc_app::OscApp,
    ui::components::{my_button::my_button, my_card::my_card},
};
use iced::{
    Alignment, Element,
    widget::{column, image, row, space, text, tooltip},
};
use iced_palace::widget::ellipsized_text;
use lucide_icons::{Icon, iced::icon_tag};

pub fn osc_app_card(app: &OscApp) -> Element<'_, Message> {
    let icon: Element<'_, Message> = match app.icon() {
        Some(icon) => image(icon).height(96).into(),
        None => space().height(96).into(),
    };

    my_card(
        column![
            row![
                icon_tag(),
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
                    .on_press_with(|| Message::OpenOscAppInfo(app.clone())),
                tooltip(
                    my_button()
                        .icon(Icon::CloudDownload)
                        .on_press_with(|| Message::InstallOscApp(app.clone())),
                    my_card("Install"),
                    tooltip::Position::Bottom
                )
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
