// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{message::Message, state::State, ui::components};
use iced::{
    Element, Length,
    widget::{Row, column, row, scrollable, space, text},
};
use lucide_icons::iced::{icon_box, icon_pointer};
use size::Size;

pub fn view(state: &State) -> Element<'_, Message> {
    let filter = state.games_filter.to_lowercase();
    let wii_games = state
        .games
        .iter()
        .filter(|g| g.is_wii && g.matches_search(&filter));
    let gc_games = state
        .games
        .iter()
        .filter(|g| !g.is_wii && g.matches_search(&filter));

    let mut col = column![components::games_toolbar::view(state)]
        .spacing(10)
        .padding(10);

    if state.show_wii {
        let mut wii_row = Row::new().width(Length::Fill).spacing(10);
        let mut wii_count = 0usize;
        let mut wii_total_size = Size::from_bytes(0);

        for game in wii_games {
            wii_row = wii_row.push(components::game_card::view(state, game));
            wii_count += 1;
            wii_total_size += game.size;
        }

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
        let mut gc_row = Row::new().width(Length::Fill).spacing(10);
        let mut gc_count = 0usize;
        let mut gc_total_size = Size::from_bytes(0);

        for game in gc_games {
            gc_row = gc_row.push(components::game_card::view(state, game));
            gc_count += 1;
            gc_total_size += game.size;
        }

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

    scrollable(col).into()
}
