// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{message::Message, state::State, ui::components};
use iced::{
    Alignment, Element, Length,
    widget::{column, space},
};

pub fn view(state: &State) -> Element<'_, Message> {
    let mut col = column![space::vertical()]
        .width(Length::Fill)
        .align_x(Alignment::End)
        .spacing(10)
        .padding(10);

    for notification in state.notifications.list.iter().rev() {
        col = col.push(components::notification::view(notification));
    }

    col.into()
}
