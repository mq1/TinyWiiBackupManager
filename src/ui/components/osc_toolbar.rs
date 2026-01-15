// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    message::Message,
    state::State,
    ui::{components, style},
};
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
        components::view_as::view(state),
    ]
    .spacing(10)
    .padding(10)
    .align_y(Alignment::Center)
    .into()
}
