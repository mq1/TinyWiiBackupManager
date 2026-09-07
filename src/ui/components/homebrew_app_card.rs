// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    homebrew::homebrew_app::HomebrewApp,
    messages::Message,
    ui::components::{my_button::my_button, my_card::my_card},
};
use iced::{
    Alignment, Element,
    widget::{column, image, row, space, text},
};
use iced_palace::widget::ellipsized_text;
use lucide_icons::{Icon, iced::icon_tag};

pub fn homebrew_app_card(app: &HomebrewApp) -> Element<'_, Message> {
    my_card(
        column![
            row![
                icon_tag(),
                app.meta.version.as_str(),
                space::horizontal(),
                text!("{}", app.size)
            ]
            .spacing(5),
            image(&app.icon).height(96),
            ellipsized_text(&app.meta.name).wrapping(text::Wrapping::None),
            row![
                my_button()
                    .label("Info")
                    .icon(Icon::Info)
                    .expand_width()
                    .on_press_with(|| Message::OpenHomebrewAppInfo(app.clone())),
                my_button()
                    .icon(Icon::Trash)
                    .on_press_with(|| Message::AskDeleteDir(app.path.clone()))
            ]
            .spacing(5)
        ]
        .align_x(Alignment::Center)
        .padding(5)
        .spacing(10),
    )
    .width(172)
    .into()
}
