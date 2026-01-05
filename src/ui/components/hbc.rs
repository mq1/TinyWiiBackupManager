// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{message::Message, state::State};
use iced::{
    Element, Length,
    widget::{container, row, text},
};
use iced_fonts::lucide;

pub fn view(state: &State) -> Element<'_, Message> {
    if !state.config.valid_mount_point() {
        return container(
            row![
                lucide::arrow_down_left(),
                text("Click on"),
                lucide::hard_drive(),
                text("to select a Drive/Mount Point")
            ]
            .spacing(5),
        )
        .center(Length::Fill)
        .into();
    }

    text("HBC Apps").into()
}
