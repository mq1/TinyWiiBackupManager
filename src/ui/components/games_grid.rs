// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{message::Message, state::State, ui::components};
use iced::{
    Alignment, Element, Length, padding,
    widget::{Row, column, row, space, text},
};
use lucide_icons::iced::{icon_box, icon_hard_drive, icon_pointer};
use size::Size;

pub fn view(state: &State) -> Element<'_, Message> {
    if !state.games_filter.is_empty() {
        let mut row = Row::new().width(Length::Fill).spacing(10).padding(10);

        for i in state.filtered_game_indices.iter().copied() {
            row = row.push(components::game_card::view(state, i));
        }

        row.wrap().into()
    } else {
        let mut wii_row = Row::new().width(Length::Fill).spacing(10).padding(10);
        let mut wii_count = 0usize;
        let mut wii_total_size = Size::from_bytes(0);
        let mut gc_row = Row::new().width(Length::Fill).spacing(10).padding(10);
        let mut gc_count = 0usize;
        let mut gc_total_size = Size::from_bytes(0);

        for (i, game) in state.games.iter().enumerate() {
            if game.is_wii {
                wii_count += 1;
                wii_total_size += game.size;

                if state.show_wii {
                    wii_row = wii_row.push(components::game_card::view(state, i));
                }
            } else {
                gc_count += 1;
                gc_total_size += game.size;

                if state.show_gc {
                    gc_row = gc_row.push(components::game_card::view(state, i));
                }
            }
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
            (true, false) => column![
                row![
                    icon_pointer().size(18),
                    text(format!(
                        "Wii Games: {} found ({})",
                        wii_count, wii_total_size
                    ))
                    .size(18),
                    space::horizontal(),
                    icon_hard_drive(),
                    text(&state.drive_usage).size(16),
                ]
                .align_y(Alignment::Center)
                .spacing(5)
                .padding(padding::left(15).right(25).top(10)),
                wii_row.wrap()
            ]
            .into(),
            (false, true) => column![
                row![
                    icon_box().size(18),
                    text(format!(
                        "GameCube Games: {} found ({})",
                        gc_count, gc_total_size
                    ))
                    .size(18),
                    space::horizontal(),
                    icon_hard_drive(),
                    text(&state.drive_usage).size(16),
                ]
                .align_y(Alignment::Center)
                .spacing(5)
                .padding(padding::left(15).right(25).top(10)),
                gc_row.wrap()
            ]
            .into(),
            (true, true) => column![
                row![
                    icon_pointer().size(18),
                    text(format!(
                        "Wii Games: {} found ({})",
                        wii_count, wii_total_size
                    ))
                    .size(18),
                    space::horizontal(),
                    icon_hard_drive(),
                    text(&state.drive_usage).size(16),
                ]
                .align_y(Alignment::Center)
                .spacing(5)
                .padding(padding::left(15).right(25).top(10)),
                wii_row.wrap(),
                space().height(20),
                row![
                    icon_box().size(18),
                    text(format!(
                        "GameCube Games: {} found ({})",
                        gc_count, gc_total_size
                    ))
                    .size(18),
                ]
                .align_y(Alignment::Center)
                .spacing(5)
                .padding(padding::left(15).right(25)),
                gc_row.wrap()
            ]
            .into(),
        }
    }
}
