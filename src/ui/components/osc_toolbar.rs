// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{message::Message, state::State, ui::style};
use iced::{
    Alignment, Element,
    widget::{container, row, space, text_input},
};
use iced_fonts::lucide;

pub fn view(state: &State) -> Element<'_, Message> {
    row![
        container(
            row![
                space(),
                lucide::search(),
                text_input("Search by Name", &state.osc_filter)
                    .width(200)
                    .style(style::search_bar)
                    .on_input(Message::UpdateOscFilter),
            ]
            .spacing(10)
            .padding(5)
            .align_y(Alignment::Center)
        )
        .style(style::card),
        space::horizontal(),
    ]
    .spacing(10)
    .padding(10)
    .align_y(Alignment::Center)
    .into()
}
