// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::ViewAs,
    games::conversion_state::ConversionState,
    messages::Message,
    state::AppState,
    ui::components::{
        game_card::game_card, game_row::game_row, games_other_toolbar::games_other_toolbar,
        games_titlebar::games_titlebar, my_card::my_card,
    },
};
use iced::{
    Element,
    widget::{Column, Row, column, rule},
};
use itertools::Itertools;
use tap::Pipe;

pub fn games(state: &AppState) -> Element<'_, Message> {
    let is_exporting = matches!(state.exporting, ConversionState::Progress(_));
    let games = state.games.iter_by(state.config.sort_by);

    let content: Element<'_, Message> = match state.config.view_as {
        ViewAs::Grid => games
            .map(|game| game_card(game, is_exporting))
            .collect::<Row<'_, _>>()
            .spacing(10)
            .wrap()
            .into(),

        ViewAs::Table => games
            .map(|game| game_row(game, is_exporting))
            .intersperse_with(|| rule::horizontal(1).into())
            .collect::<Column<'_, _>>()
            .pipe(my_card)
            .padding(0)
            .into(),
    };

    column![games_titlebar(state), games_other_toolbar(state), content]
        .padding(10)
        .spacing(10)
        .into()
}
