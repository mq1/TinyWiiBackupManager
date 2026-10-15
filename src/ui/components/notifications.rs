// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    long_operation::{LongOperationKind, LongOperationState},
    message::Message,
    notifications::notification::{Notification, NotificationLevel},
    state::AppState,
    ui::components::my_card::my_card,
};
use iced::{
    Alignment, Element, Theme, border,
    widget::{Column, button, row, svg, text},
};
use itertools::Either;
use lucide_iced::{
    bytes::{CHECK, INFO, TRIANGLE_ALERT, X},
    themed_icon,
};
use std::iter::{empty, once};

fn item((i, notification): (usize, &Notification)) -> Element<'_, Message> {
    let icon_bytes = match notification.level() {
        NotificationLevel::Info => INFO,
        NotificationLevel::Warning => TRIANGLE_ALERT,
        NotificationLevel::Error => X,
        NotificationLevel::Success => CHECK,
    };

    let color = |theme: &Theme| match notification.level() {
        NotificationLevel::Info => theme.palette().primary,
        NotificationLevel::Warning => theme.palette().warning,
        NotificationLevel::Error => theme.palette().danger,
        NotificationLevel::Success => theme.palette().success,
    };

    let icon = svg(svg::Handle::from_memory(icon_bytes))
        .width(14.)
        .height(14.)
        .style(move |theme, _status| svg::Style {
            color: Some(color(theme)),
        });

    my_card(
        row![
            icon,
            text(notification.label()),
            button(themed_icon(X, 14.))
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

fn progress(
    (op_state, _): &(LongOperationState, LongOperationKind),
) -> impl Iterator<Item = Element<'_, Message>> {
    match op_state {
        LongOperationState::Progress(progress) => {
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
        .chain(state.long_operations.iter().flat_map(progress))
        .collect::<Column<'_, Message>>()
        .padding(10)
        .spacing(10)
        .align_x(Alignment::End)
        .into()
}
