// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{message::Message, state::State, ui::components};
use iced::{
    Element, Length,
    widget::{Column, Row, column, container, row, scrollable, space, text},
};
use lucide_icons::iced::{icon_arrow_down_left, icon_box, icon_hard_drive, icon_pointer};
use size::Size;

pub fn view(state: &State) -> Element<'_, Message> {
    if !state.config.valid_mount_point() {
        return container(
            row![
                icon_arrow_down_left(),
                text("Click on"),
                icon_hard_drive(),
                text("to select a Drive/Mount Point")
            ]
            .spacing(5),
        )
        .center(Length::Fill)
        .into();
    }

    let filter = state.games_filter.to_lowercase();

    let mut wii_row = Row::new().width(Length::Fill).spacing(10);
    let mut wii_count = 0usize;
    let mut wii_total_size = Size::from_bytes(0);
    let mut gc_row = Row::new().width(Length::Fill).spacing(10);
    let mut gc_count = 0usize;
    let mut gc_total_size = Size::from_bytes(0);

    for (i, game) in state.games.iter().enumerate() {
        if game.is_wii {
            wii_count += 1;
            wii_total_size += game.size;

            if state.show_wii && game.matches_search(&filter) {
                wii_row = wii_row.push(components::game_card::view(state, i));
            }
        } else {
            gc_count += 1;
            gc_total_size += game.size;

            if state.show_gc && game.matches_search(&filter) {
                gc_row = gc_row.push(components::game_card::view(state, i));
            }
        }
    }

    let mut col = Column::new().spacing(10).padding(10);

    if state.show_wii {
        col = col
            .push(
                row![
                    icon_pointer().size(18),
                    text(format!(
                        "Wii Games: {} found ({})",
                        wii_count, wii_total_size
                    ))
                    .size(18)
                ]
                .spacing(5),
            )
            .push(wii_row.wrap());
    }

    if state.show_wii && state.show_gc {
        col = col.push(space());
    }

    if state.show_gc {
        col = col
            .push(
                row![
                    icon_box().size(18),
                    text(format!(
                        "GameCube Games: {} found ({})",
                        gc_count, gc_total_size
                    ))
                    .size(18)
                ]
                .spacing(5),
            )
            .push(gc_row.wrap());
    }

    column![components::games_toolbar::view(state), scrollable(col)].into()
}
