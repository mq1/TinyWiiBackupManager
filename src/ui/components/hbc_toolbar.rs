// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{message::Message, state::State, ui::style};
use iced::{
    Alignment, Element,
    widget::{container, row, space, text_input},
};
use lucide_icons::iced::icon_search;

pub fn view(state: &State) -> Element<'_, Message> {
    row![
        container(
            row![
                space(),
                icon_search(),
                text_input("Search by Name", &state.hbc_filter)
                    .width(200)
                    .style(style::search_bar)
                    .on_input(Message::UpdateHbcFilter),
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
