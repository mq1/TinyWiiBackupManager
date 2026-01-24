// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    games::game::Game,
    message::Message,
    state::State,
    ui::{Screen, components::my_tooltip},
};
use iced::{
    Alignment, Element, Length, padding,
    widget::{button, column, container, row, space, table, text},
};
use lucide_icons::iced::{
    icon_gamepad_2, icon_hard_drive, icon_hard_drive_download, icon_info, icon_trash,
};

pub fn view(state: &State) -> Element<'_, Message> {
    let t_columns = vec![
        table::column(text("Title").size(16), |game: &Game| text(game.title())),
        table::column(text("ID").size(16), |game: &Game| {
            text(game.id().as_str().to_string())
        }),
        table::column(text("Console").size(16), |game: &Game| {
            text(if game.id().is_wii() {
                "Wii"
            } else {
                "GameCube"
            })
        }),
        table::column(text("Size").size(16), |game: &Game| {
            text(game.size().to_string())
        }),
        table::column(text("Actions").size(16), |game: &Game| {
            row![
                my_tooltip::view(
                    button(row![icon_info(), text("Info")].spacing(5))
                        .padding(0)
                        .style(button::text)
                        .on_press_with(|| Message::NavTo(Screen::GameInfo(game.clone()))),
                    "Show game info"
                ),
                text('•'),
                my_tooltip::view(
                    button(row![icon_hard_drive_download(), text("Archive")].spacing(5))
                        .padding(0)
                        .style(button::text)
                        .on_press_with(|| Message::ChooseArchiveDest(game.clone())),
                    "Archive game to PC"
                ),
                text('•'),
                my_tooltip::view(
                    button(row![icon_trash(), text("Delete")].spacing(5))
                        .padding(0)
                        .style(button::text)
                        .on_press_with(|| Message::AskDeleteDirConfirmation(game.path().clone())),
                    "Delete game"
                )
            ]
            .spacing(10)
            .align_y(Alignment::Center)
        }),
    ];

    if !state.games_filter.is_empty() {
        let iter = state.game_list.iter_filtered();

        return container(table(t_columns, iter).width(Length::Fill))
            .padding(10)
            .into();
    }

    let iter = state.game_list.iter();

    column![
        row![
            icon_gamepad_2().size(18),
            text!(
                "Games: {} found ({})",
                state.game_list.total_count(),
                state.game_list.total_size()
            )
            .size(18),
            space::horizontal(),
            icon_hard_drive(),
            text(&state.drive_usage).size(16),
        ]
        .align_y(Alignment::Center)
        .spacing(5)
        .padding(padding::left(15).right(25).top(10)),
        container(table(t_columns, iter).width(Length::Fill)).padding(10),
    ]
    .into()
}
