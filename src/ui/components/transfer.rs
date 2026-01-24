// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{message::Message, state::State, ui::style};
use iced::{
    Alignment, Element, Length,
    widget::{Column, button, column, row, scrollable, text},
};
use lucide_icons::iced::{icon_hard_drive_download, icon_x};

pub fn view(state: &State) -> Element<'_, Message> {
    let mut col = Column::new().spacing(5);

    for (i, op) in state.transfer_queue.iter().enumerate() {
        col = col.push(
            row![
                button(icon_x())
                    .style(style::rounded_danger_button)
                    .on_press(Message::CancelTransfer(i)),
                text(op.display_str())
            ]
            .align_y(Alignment::Center)
            .spacing(5),
        );
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
