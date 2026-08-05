// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    messages::Message,
    notifications::NotificationLevel,
    state::AppState,
    ui::{
        components,
        my_palette::{BLUE, GREEN, RED, YELLOW},
    },
};
use iced::{
    Element, border,
    widget::{Column, button, row, text},
};
use lucide_icons::iced::{icon_alert_triangle, icon_check, icon_info, icon_x};

pub fn view(state: &AppState) -> Element<'_, Message> {
    state
        .notifications
        .iter()
        .enumerate()
        .map(|(i, notification)| {
            let icon = match notification.level {
                NotificationLevel::Info => icon_info().color(BLUE),
                NotificationLevel::Warning => icon_alert_triangle().color(YELLOW),
                NotificationLevel::Error => icon_x().color(RED),
                NotificationLevel::Success => icon_check().color(GREEN),
            };

            components::card::view(
                row![
                    icon,
                    text(&notification.label),
                    button(icon_x().center())
                        .on_press(Message::CloseNotification(i))
                        .padding(0)
                        .width(22)
                        .height(22)
                        .style(|theme, status| {
                            let mut base = button::subtle(theme, status);
                            base.border = border::rounded(11);
                            base
                        })
                ]
                .spacing(10),
            )
            .into()
        })
        .collect::<Column<'_, _>>()
        .padding(10)
        .spacing(10)
        .into()
}
