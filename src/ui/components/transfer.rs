// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{message::Message, state::State, ui::style};
use iced::{
    Alignment, Element, Length,
    widget::{Column, button, column, row, scrollable, text},
};
use lucide_icons::iced::{icon_hard_drive_download, icon_x};

pub fn view(state: &State) -> Element<'_, Message> {
    let mut col = Column::new().spacing(5).padding(10);
    for (i, game) in state.transfer_stack.iter().enumerate().rev() {
        col = col.push(
            row![
                button(icon_x())
                    .style(style::rounded_danger_button)
                    .on_press(Message::CancelTransfer(i)),
                text(game.to_string_lossy())
            ]
            .align_y(Alignment::Center)
            .spacing(5),
        );
    }

    column![
        row![
            icon_hard_drive_download().size(18),
            text("Games to transfer").size(18)
        ]
        .spacing(5)
        .padding(10),
        scrollable(col).width(Length::Fill).height(Length::Fill)
    ]
    .into()
}
