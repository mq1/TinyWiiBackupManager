// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{messages::Message, state::AppState, ui::components::my_card::my_card};
use iced::{
    Alignment, Border, Element, Theme,
    widget::{row, text_input},
};
use lucide_icons::iced::icon_search;

fn text_input_style(theme: &Theme, status: text_input::Status) -> text_input::Style {
    text_input::Style {
        border: Border::default(),
        ..text_input::default(theme, status)
    }
}

pub fn game_search(state: &AppState) -> Element<'_, Message> {
    my_card(
        row![
            icon_search(),
            text_input("Search by Title/ID", &state.games.filter.search_term)
                .width(200)
                .on_input(Message::SearchGames)
                .style(text_input_style),
        ]
        .align_y(Alignment::Center)
        .spacing(5),
    )
    .into()
}
