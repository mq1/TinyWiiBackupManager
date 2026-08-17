// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    messages::Message, notifications::notification::NotificationLevel, state::AppState,
    ui::components::my_card::my_card,
};
use iced::{
    Alignment, Element, Theme, border,
    widget::{Column, button, row, text},
};
use lucide_icons::iced::{icon_alert_triangle, icon_check, icon_info, icon_x};

pub fn view(state: &AppState) -> Element<'_, Message> {
    let mut notifications = state
        .notifications
        .iter()
        .enumerate()
        .map(|(i, notification)| {
            let icon = match notification.level {
                NotificationLevel::Info => icon_info().style(|theme: &Theme| text::Style {
                    color: Some(theme.palette().primary),
                }),
                NotificationLevel::Warning => {
                    icon_alert_triangle().style(|theme: &Theme| text::Style {
                        color: Some(theme.palette().warning),
                    })
                }
                NotificationLevel::Error => icon_x().style(|theme: &Theme| text::Style {
                    color: Some(theme.palette().danger),
                }),
                NotificationLevel::Success => icon_check().style(|theme: &Theme| text::Style {
                    color: Some(theme.palette().success),
                }),
            };

            my_card(
                row![
                    icon,
                    text(&notification.label),
                    button(icon_x().center())
                        .on_press(Message::CloseNotification(i))
                        .padding(0)
                        .width(22)
                        .height(22)
                        .style(|theme, status| button::Style {
                            border: border::rounded(11),
                            ..button::subtle(theme, status)
                        })
                ]
                .align_y(Alignment::Center)
                .spacing(10),
            )
            .into()
        })
        .collect::<Column<'_, _>>();

    if !state.status.is_empty() {
        notifications = notifications.push(my_card(state.status.as_str()));
    }

    notifications
        .padding(10)
        .spacing(10)
        .align_x(Alignment::End)
        .into()
}
