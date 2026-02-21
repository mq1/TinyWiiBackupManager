// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{message::Message, ui::style};
use iced::{
    Element, Length,
    widget::{Text, button, column, container, row, rule, space},
};
use lucide_icons::iced::{icon_circle_x, icon_info, icon_triangle_alert};

pub enum Level {
    Info,
    Warning,
    Error,
}

fn get_icon<'a>(level: Level) -> Text<'a> {
    match level {
        Level::Info => icon_info(),
        Level::Warning => icon_triangle_alert(),
        Level::Error => icon_circle_x(),
    }
}

fn get_style(level: Level) -> style::RoundedButton {
    match level {
        Level::Info => style::rounded_button,
        Level::Warning => style::rounded_warning_button,
        Level::Error => style::rounded_danger_button,
    }
}

pub fn view<'a>(
    title: &'a str,
    description: &'a str,
    level: Level,
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
                description,
                space::vertical(),
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
