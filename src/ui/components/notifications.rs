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

fn progress<'a>(
    (op_state, kind): &'a (LongOperationState, LongOperationKind),
    state: &AppState,
) -> impl Iterator<Item = Element<'a, Message>> {
    match op_state {
        LongOperationState::Progress(progress) => {
            let mut content = row![text(progress.as_str())]
                .align_y(Alignment::Center)
                .spacing(10);
            if *kind == LongOperationKind::Import
                && let Some(run) = &state.active_import
            {
                content = content.push(
                    button(if state.is_import_cancelling() {
                        "Cancelling..."
                    } else {
                        "Cancel import"
                    })
                    .on_press_maybe(
                        (!state.is_import_cancelling()).then_some(Message::CancelImportRun(run.id)),
                    )
                    .style(button::danger),
                );
            }
            Either::Left(once(Element::from(my_card(content))))
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
        .chain(
            state
                .long_operations
                .iter()
                .flat_map(|operation| progress(operation, state)),
        )
        .collect::<Column<'_, Message>>()
        .padding(10)
        .spacing(10)
        .align_x(Alignment::End)
        .into()
}
