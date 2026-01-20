// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{message::Message, state::State, ui::Screen};
use iced::{
    Alignment, Element, Length, padding,
    widget::{button, column, container, row, space, table, text},
};
use lucide_icons::iced::{icon_gamepad_2, icon_hard_drive, icon_info, icon_trash};
use size::Size;

pub fn view(state: &State) -> Element<'_, Message> {
    let t_columns = vec![
        table::column(text("Title").size(16), |i: usize| {
            text(&state.games[i].title)
        }),
        table::column(text("ID").size(16), |i: usize| {
            text(state.games[i].id.as_str())
        }),
        table::column(text("Console").size(16), |i: usize| {
            text(if state.games[i].id.is_wii() {
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
                    .padding(0)
                    .style(button::text)
                    .on_press(Message::NavigateTo(Screen::GameInfo(i))),
                text('•'),
                button(row![icon_trash(), text("Delete")].spacing(5))
                    .padding(0)
                    .style(button::text)
                    .on_press_with(move || Message::AskDeleteDirConfirmation(
                        state.games[i].path.clone()
                    ))
            ]
            .spacing(10)
            .align_y(Alignment::Center)
        }),
    ];

    if !state.games_filter.is_empty() {
        let indices = state.filtered_game_indices.iter().copied();
        container(table(t_columns, indices).width(Length::Fill))
            .padding(10)
            .into()
    } else {
        let count = state.games.len();
        let indices = 0..count;
        let total_size = state
            .games
            .iter()
            .fold(Size::from_bytes(0), |a, b| a + b.size);

        column![
            row![
                icon_gamepad_2().size(18),
                text!("Games: {} found ({})", count, total_size).size(18),
                space::horizontal(),
                icon_hard_drive(),
                text(&state.drive_usage).size(16),
            ]
            .align_y(Alignment::Center)
            .spacing(5)
            .padding(padding::left(15).right(25).top(10)),
            container(table(t_columns, indices).width(Length::Fill)).padding(10),
        ]
        .into()
    }
}
