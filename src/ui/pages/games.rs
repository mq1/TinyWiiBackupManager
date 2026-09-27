// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::ViewAs,
    games::{conversion_state::ConversionState, games_state::GamesState},
    message::Message,
    state::AppState,
    ui::{
        components::{
            game_card::game_card, game_row::game_row, games_titlebar::games_titlebar,
            my_card::my_card,
        },
        pages::{errored::errored, loading::loading, no_drive::no_drive},
    },
};
use iced::{
    Element, Length, padding,
    widget::{Column, Row, column, container, rule, scrollable},
};
use itertools::Itertools;

pub fn games(state: &AppState) -> Element<'_, Message> {
    if state.config.mount_point().as_os_str().is_empty() {
        return no_drive();
    }

    match &state.games {
        GamesState::NotLoaded | GamesState::Loading => loading(),
        GamesState::Errored(e) => errored(e),
        GamesState::Loaded(games) => {
            let is_exporting = matches!(state.exporting, ConversionState::Progress(_));

            let content: Element<'_, Message> = match state.config.view_as() {
                ViewAs::Grid => games
                    .iter_by(state.config.sort_by())
                    .map(|game| game_card(game, is_exporting))
                    .collect::<Row<'_, _>>()
                    .spacing(10)
                    .wrap()
                    .into(),

                ViewAs::Table => my_card(
                    games
                        .iter_by(state.config.sort_by())
                        .map(|game| game_row(game, is_exporting))
                        .intersperse_with(|| rule::horizontal(1).into())
                        .collect::<Column<'_, _>>(),
                )
                .padding(0)
                .into(),
            };

            column![
                games_titlebar(games, state),
                scrollable(container(content).padding(padding::all(10).right(20).top(0)))
                    .width(Length::Fill),
            ]
            .into()
        }
    }
}
