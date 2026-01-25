// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    message::Message,
    state::State,
    ui::{components, style},
};
use iced::{
    Alignment, Element, Length,
    widget::{button, column, row, space},
};
use lucide_icons::iced::icon_brush_cleaning;

pub fn view(state: &State) -> Element<'_, Message> {
    let mut col = column![space::vertical()]
        .width(Length::Fill)
        .align_x(Alignment::End)
        .spacing(10)
        .padding(10);

    for notification in state.notifications.iter() {
        col = col.push(components::notification::view(notification));
    }

    if state.notifications.count() > 10 {
        col = col.push(
            button(row![icon_brush_cleaning(), "Close all notifications"].spacing(5))
                .style(style::rounded_button)
                .on_press(Message::CloseAllNotifications),
        );
    }

    col.into()
}
