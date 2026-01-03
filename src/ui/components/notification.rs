// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    message::Message,
    notifications::{Notification, NotificationLevel},
    ui::style,
};
use iced::{
    Alignment, Element,
    widget::{button, column, container, row, space, text},
};
use lucide_icons::iced::{
    icon_message_circle_heart, icon_message_circle_warning, icon_message_circle_x, icon_x,
};

pub fn view(notification: &Notification) -> Element<'_, Message> {
    let icon = match notification.level {
        NotificationLevel::Info => icon_message_circle_warning(),
        NotificationLevel::Error => icon_message_circle_x(),
        NotificationLevel::Success => icon_message_circle_heart(),
    };

    container(
        column![
            row![
                space(),
                icon,
                space(),
                text(&notification.text),
                button(icon_x().center())
                    .height(20)
                    .width(20)
                    .style(style::rounded_background_button)
                    .on_press(Message::CloseNotification(notification.id))
            ]
            .spacing(5)
            .align_y(Alignment::Center),
            text(notification.get_life_str())
        ]
        .align_x(Alignment::End),
    )
    .padding(10)
    .style(style::card)
    .into()
}
