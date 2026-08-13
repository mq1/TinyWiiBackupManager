// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    messages::Message, notifications::notification::NotificationLevel, state::AppState,
    ui::components::my_card::my_card,
};
use iced::{
    Element, Theme, border,
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
