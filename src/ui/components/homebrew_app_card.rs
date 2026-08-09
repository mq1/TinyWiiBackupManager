// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{homebrew::homebrew_app::HomebrewApp, messages::Message, ui::components};
use iced::{
    Alignment, Element, Length,
    widget::{column, image, row, space, text},
};
use iced_palace::widget::ellipsized_text;
use lucide_icons::{Icon, iced::icon_tag};
use std::sync::Arc;

pub fn view(app: &Arc<HomebrewApp>) -> Element<'_, Message> {
    components::card::view(
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
                components::my_button::view(Some("Info"), Some(Icon::Info))
                    .width(Length::Fill)
                    .on_press_with(|| Message::OpenHomebrewAppInfo(app.clone())),
                components::my_button::view(None, Some(Icon::Trash))
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
