// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    message::Message,
    state::State,
    ui::{components::my_tooltip, style},
};
use iced::{
    Alignment, Element,
    widget::{button, checkbox, container, row, space, text, text_input},
};
use lucide_icons::iced::{icon_box, icon_plus, icon_pointer, icon_rotate_cw, icon_search};

pub fn view(state: &State) -> Element<'_, Message> {
    row![
        container(
            row![
                space(),
                icon_search(),
                text_input("Search by Title/ID", &state.games_filter)
                    .width(200)
                    .style(style::search_bar)
                    .on_input(Message::UpdateGamesFilter),
            ]
            .spacing(10)
            .padding(5)
            .align_y(Alignment::Center)
        )
        .style(style::card),
        space::horizontal(),
        container(
            row![
                checkbox(state.show_wii).on_toggle(Message::ShowWii),
                my_tooltip::view(icon_pointer(), "Show Wii Games"),
                text("  |  "),
                checkbox(state.show_gc).on_toggle(Message::ShowGc),
                my_tooltip::view(icon_box(), "Show GameCube Games"),
            ]
            .align_y(Alignment::Center)
            .spacing(5)
            .padding(10)
        )
        .style(style::card),
        space(),
        my_tooltip::view(
            button(icon_rotate_cw().size(18).center())
                .width(35)
                .height(35)
                .style(style::rounded_button)
                .on_press(Message::RefreshGames),
            "Refresh Games"
        ),
        button(icon_plus().size(18).center())
            .width(35)
            .height(35)
            .style(style::rounded_button)
    ]
    .spacing(10)
    .align_y(Alignment::Center)
    .into()
}
