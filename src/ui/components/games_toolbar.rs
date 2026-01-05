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
use iced_fonts::lucide;

pub fn view(state: &State) -> Element<'_, Message> {
    row![
        container(
            row![
                space(),
                lucide::search(),
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
                checkbox(state.show_wii)
                    .style(style::toolbar_checkbox)
                    .on_toggle(Message::ShowWii),
                my_tooltip::view(lucide::pointer(), "Show Wii Games"),
                text("  |  "),
                checkbox(state.show_gc)
                    .style(style::toolbar_checkbox)
                    .on_toggle(Message::ShowGc),
                my_tooltip::view(lucide::r#box(), "Show GameCube Games"),
            ]
            .align_y(Alignment::Center)
            .spacing(5)
            .padding(10)
        )
        .style(style::card),
        space(),
        my_tooltip::view(
            button(lucide::rotate_cw().size(18).center())
                .width(35)
                .height(35)
                .style(style::rounded_button)
                .on_press(Message::RefreshGamesAndApps),
            "Refresh Games"
        ),
        button(lucide::plus().size(18).center())
            .width(35)
            .height(35)
            .style(style::rounded_button)
    ]
    .spacing(10)
    .padding(10)
    .align_y(Alignment::Center)
    .into()
}
