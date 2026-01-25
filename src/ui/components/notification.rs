// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    message::Message,
    notifications::{Notification, NotificationLevel},
    ui::style,
};
use iced::{
    Alignment, Element,
    widget::{button, container, row, space, text},
};
use lucide_icons::iced::{icon_check, icon_info, icon_triangle_alert, icon_x};

pub fn view(notification: &Notification) -> Element<'_, Message> {
    let icon = match notification.level() {
        NotificationLevel::Info => icon_info().style(text::primary),
        NotificationLevel::Error => icon_triangle_alert().style(text::danger),
        NotificationLevel::Success => icon_check().style(text::success),
    };

    container(
        row![
            space(),
            icon,
            space(),
            text(notification.text()),
            button(icon_x().center())
                .height(20)
                .width(20)
                .style(style::rounded_background_button)
                .on_press(Message::CloseNotification(notification.id()))
        ]
        .spacing(5)
        .align_y(Alignment::Center),
    )
    .padding(10)
    .style(style::heavy_card)
    .into()
}
