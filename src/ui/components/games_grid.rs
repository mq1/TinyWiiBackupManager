// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{message::Message, state::State, ui::components};
use iced::{
    Alignment, Element, Length, padding,
    widget::{Row, column, row, space, text},
};
use lucide_icons::iced::{icon_box, icon_hard_drive, icon_pointer};

#[allow(clippy::too_many_lines)]
pub fn view(state: &State) -> Element<'_, Message> {
    if !state.games_filter.is_empty() {
        let cards = state
            .game_list
            .iter_filtered()
            .map(|g| components::game_card::view(state, g));

        let row = Row::from_iter(cards)
            .width(Length::Fill)
            .spacing(10)
            .padding(10)
            .wrap();

        return row.into();
    }

    match (state.show_wii, state.show_gc) {
        (false, false) => row![
            text("").size(18),
            space::horizontal(),
            icon_hard_drive(),
            text(&state.drive_usage).size(16),
        ]
        .align_y(Alignment::Center)
        .spacing(5)
        .padding(padding::left(15).right(25).top(10))
        .into(),
        (true, false) => {
            let cards = state
                .game_list
                .iter_wii()
                .map(|g| components::game_card::view(state, g));

            let row = Row::from_iter(cards)
                .width(Length::Fill)
                .spacing(10)
                .padding(10)
                .wrap();

            column![
                row![
                    icon_pointer().size(18),
                    text(format!(
                        "Wii Games: {} found ({})",
                        state.game_list.wii_count(),
                        state.game_list.wii_size()
                    ))
                    .size(18),
                    space::horizontal(),
                    icon_hard_drive(),
                    text(&state.drive_usage).size(16),
                ]
                .align_y(Alignment::Center)
                .spacing(5)
                .padding(padding::left(15).right(25).top(10)),
                row
            ]
            .into()
        }
        (false, true) => {
            let cards = state
                .game_list
                .iter_gc()
                .map(|g| components::game_card::view(state, g));

            let row = Row::from_iter(cards)
                .width(Length::Fill)
                .spacing(10)
                .padding(10)
                .wrap();

            column![
                row![
                    icon_box().size(18),
                    text(format!(
                        "GameCube Games: {} found ({})",
                        state.game_list.gc_count(),
                        state.game_list.gc_size()
                    ))
                    .size(18),
                    space::horizontal(),
                    icon_hard_drive(),
                    text(&state.drive_usage).size(16),
                ]
                .align_y(Alignment::Center)
                .spacing(5)
                .padding(padding::left(15).right(25).top(10)),
                row
            ]
            .into()
        }
        (true, true) => {
            let wii_cards = state
                .game_list
                .iter_wii()
                .map(|g| components::game_card::view(state, g));

            let wii_row = Row::from_iter(wii_cards)
                .width(Length::Fill)
                .spacing(10)
                .padding(10)
                .wrap();

            let gc_cards = state
                .game_list
                .iter_gc()
                .map(|g| components::game_card::view(state, g));

            let gc_row = Row::from_iter(gc_cards)
                .width(Length::Fill)
                .spacing(10)
                .padding(10)
                .wrap();

            column![
                row![
                    icon_pointer().size(18),
                    text(format!(
                        "Wii Games: {} found ({})",
                        state.game_list.wii_count(),
                        state.game_list.wii_size()
                    ))
                    .size(18),
                    space::horizontal(),
                    icon_hard_drive(),
                    text(&state.drive_usage).size(16),
                ]
                .align_y(Alignment::Center)
                .spacing(5)
                .padding(padding::left(15).right(25).top(10)),
                wii_row,
                space().height(20),
                row![
                    icon_box().size(18),
                    text(format!(
                        "GameCube Games: {} found ({})",
                        state.game_list.gc_count(),
                        state.game_list.gc_size()
                    ))
                    .size(18),
                ]
                .align_y(Alignment::Center)
                .spacing(5)
                .padding(padding::left(15).right(25)),
                gc_row
            ]
            .into()
        }
    }
}
