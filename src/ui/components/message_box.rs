// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{message::Message, ui::style};
use iced::{
    Element, Length,
    widget::{Text, button, column, container, row, rule, scrollable, space},
};
use lucide_icons::iced::{icon_circle_x, icon_info, icon_triangle_alert};
use rfd::MessageLevel;

fn get_icon<'a>(level: MessageLevel) -> Text<'a> {
    match level {
        MessageLevel::Info => icon_info(),
        MessageLevel::Warning => icon_triangle_alert(),
        MessageLevel::Error => icon_circle_x(),
    }
}

fn get_style(level: MessageLevel) -> fn(&iced::Theme, button::Status) -> button::Style {
    match level {
        MessageLevel::Info => style::rounded_button,
        MessageLevel::Warning => style::rounded_warning_button,
        MessageLevel::Error => style::rounded_danger_button,
    }
}

pub fn view<'a>(
    title: &'a str,
    description: &'a str,
    level: MessageLevel,
    message: Option<&'a Message>,
) -> Element<'a, Message> {
    let actions = if let Some(msg) = message {
        row![
            space::horizontal(),
            button("Cancel")
                .style(style::rounded_secondary_button)
                .on_press(Message::CloseMessageBox(None)),
            button("OK")
                .style(get_style(level))
                .on_press_with(|| Message::CloseMessageBox(Some(Box::new(msg.clone())))),
        ]
        .spacing(5)
    } else {
        row![
            space::horizontal(),
            button("OK")
                .style(style::rounded_button)
                .on_press(Message::CloseMessageBox(None))
        ]
    };

    container(
        container(
            column![
                row![get_icon(level), title].spacing(5),
                rule::horizontal(1),
                space(),
                scrollable(description)
                    .width(Length::Fill)
                    .height(Length::Fill),
                actions
            ]
            .spacing(5)
            .padding(10),
        )
        .style(style::card)
        .center_x(600)
        .center_y(400),
    )
    .center(Length::Fill)
    .style(style::root_container)
    .into()
}
