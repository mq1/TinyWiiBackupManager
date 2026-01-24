// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    message::Message,
    state::State,
    ui::{components::my_tooltip, style},
};
use iced::{
    Alignment, Element, Length,
    widget::{Column, button, column, row, rule, scrollable, text},
};
use lucide_icons::iced::{icon_hard_drive_download, icon_x};

pub fn view(state: &State) -> Element<'_, Message> {
    let mut col = Column::new().spacing(5);

    let last_i = state.transfer_queue.len().saturating_sub(1);
    for (i, op) in state.transfer_queue.iter().enumerate() {
        col = col.push(
            row![
                my_tooltip::view(
                    button(icon_x().center())
                        .height(20)
                        .width(20)
                        .style(style::rounded_background_button)
                        .on_press(Message::CancelTransfer(i)),
                    "Cancel operation"
                ),
                text(op.display_str())
            ]
            .height(20)
            .align_y(Alignment::Center)
            .spacing(5),
        );

        if i != last_i {
            col = col.push(rule::horizontal(1));
        }
    }

    column![
        row![
            icon_hard_drive_download().size(18),
            text("Transfer queue").size(18)
        ]
        .spacing(5),
        scrollable(col).width(Length::Fill).height(Length::Fill),
    ]
    .height(Length::Fill)
    .spacing(10)
    .padding(10)
    .into()
}
