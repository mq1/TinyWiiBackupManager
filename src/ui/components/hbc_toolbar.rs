// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    message::Message,
    state::State,
    ui::{
        components::{self, my_tooltip},
        style,
    },
};
use iced::{
    Alignment, Element,
    widget::{button, container, row, space, text_input},
};
use lucide_icons::iced::{icon_plus, icon_rotate_cw, icon_search};

pub fn view(state: &State) -> Element<'_, Message> {
    let mut row = row![
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
        space::horizontal()
    ];

    if state.hbc_filter.is_empty() {
        row = row.push(components::sort_by::view(state));
    }

    row.push(components::view_as::view(state))
        .push(space())
        .push(my_tooltip::view(
            button(icon_rotate_cw().size(18).center())
                .width(35)
                .height(35)
                .style(style::rounded_button)
                .on_press(Message::RefreshGamesAndApps),
            "Refresh Apps",
        ))
        .push(my_tooltip::view(
            button(icon_plus().size(18).center())
                .width(35)
                .height(35)
                .style(style::rounded_button)
                .on_press(Message::PickHbcApps),
            "Add Apps (.zip)",
        ))
        .spacing(10)
        .padding(10)
        .align_y(Alignment::Center)
        .into()
}
