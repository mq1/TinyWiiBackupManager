// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    game_id::GameID,
    message::Message,
    state::State,
    ui::{Screen, style},
};
use iced::{
    Element, Length,
    widget::{button, container, row, table, text},
};
use lucide_icons::iced::{icon_info, icon_trash};

pub fn view(state: &State) -> Element<'_, Message> {
    let t_columns = vec![
        table::column(text("Title").size(16), |i: usize| {
            text(&state.games[i].title)
        }),
        table::column(text("ID").size(16), |i: usize| {
            text(state.games[i].id.as_str())
        }),
        table::column(text("Console").size(16), |i: usize| {
            text(if state.games[i].is_wii {
                "Wii"
            } else {
                "GameCube"
            })
        }),
        table::column(text("Size").size(16), |i: usize| {
            text(state.games[i].size.to_string())
        }),
        table::column(text("Actions").size(16), |i| {
            row![
                button(row![icon_info(), text("Info")].spacing(5))
                    .style(style::rounded_secondary_button)
                    .on_press(Message::NavigateTo(Screen::GameInfo(i))),
                button(row![icon_trash(), text("Delete")].spacing(5))
                    .style(style::rounded_secondary_button)
                    .on_press(Message::AskDeleteGame(i))
            ]
            .spacing(5)
        }),
    ];

    let table = if !state.games_filter.is_empty() {
        let indices = state.filtered_game_indices.iter().copied();
        table(t_columns, indices).width(Length::Fill)
    } else {
        let indices = 0..state.games.len();
        table(t_columns, indices).width(Length::Fill)
    };

    container(table).padding(10).into()
}
