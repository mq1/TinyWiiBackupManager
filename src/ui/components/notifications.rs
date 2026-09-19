// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    games::conversion_state::ConversionState,
    messages::Message,
    notifications::notification::{Notification, NotificationLevel},
    state::AppState,
    ui::components::my_card::my_card,
};
use iced::{
    Alignment, Element, Theme, border,
    widget::{Column, button, row, text},
};
use itertools::Either;
use lucide_icons::iced::{icon_alert_triangle, icon_check, icon_info, icon_x};
use std::iter::{empty, once};

fn item((i, notification): (usize, &Notification)) -> Element<'_, Message> {
    let icon = match notification.level() {
        NotificationLevel::Info => icon_info().style(|theme: &Theme| text::Style {
            color: Some(theme.palette().primary),
        }),
        NotificationLevel::Warning => icon_alert_triangle().style(|theme: &Theme| text::Style {
            color: Some(theme.palette().warning),
        }),
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
            text(notification.label()),
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
}

fn progress(conversion: &ConversionState) -> impl Iterator<Item = Element<'_, Message>> {
    match conversion {
        ConversionState::Progress(progress) => {
            Either::Left(once(Element::from(my_card(text(progress.as_str())))))
        }
        _ => Either::Right(empty()),
    }
}

pub fn notifications(state: &AppState) -> Element<'_, Message> {
    state
        .notifications
        .iter()
        .enumerate()
        .map(item)
        .chain(progress(&state.importing))
        .chain(progress(&state.exporting))
        .chain(progress(&state.hashing))
        .collect::<Column<'_, Message>>()
        .padding(10)
        .spacing(10)
        .align_x(Alignment::End)
        .into()
}
