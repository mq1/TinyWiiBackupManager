// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{message::Message, ui::style};
use blocking_dialog::BlockingDialogLevel;
use iced::{
    Element, Length,
    widget::{Text, button, column, container, row, rule, space},
};
use lucide_icons::iced::{icon_circle_x, icon_info, icon_triangle_alert};

fn get_icon<'a>(level: BlockingDialogLevel) -> Text<'a> {
    match level {
        BlockingDialogLevel::Info => icon_info(),
        BlockingDialogLevel::Warning => icon_triangle_alert(),
        BlockingDialogLevel::Error => icon_circle_x(),
    }
}

pub fn view<'a>(
    title: &'a str,
    description: &'a str,
    level: BlockingDialogLevel,
) -> Element<'a, Message> {
    container(
        container(
            column![
                row![get_icon(level), title].spacing(5),
                rule::horizontal(1),
                space(),
                description,
                space::vertical(),
                row![
                    space::horizontal(),
                    button("OK").on_press(Message::CloseAlert)
                ]
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
