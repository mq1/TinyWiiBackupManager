// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{Game, GameList, SortBy, game};
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use slint::{Model, ModelRc, SharedString, VecModel};
use std::{fs, path::Path, rc::Rc};

pub fn fuzzy_search(games: &ModelRc<Game>, query: &str) -> ModelRc<Game> {
    let matcher = SkimMatcherV2::default();

    let mut filtered_games = Vec::new();
    for game in games.iter() {
        let title_score = matcher.fuzzy_match(&game.title, query);
        let id_score = matcher.fuzzy_match(&game.id, query);

        let score = match (title_score, id_score) {
            (Some(a), Some(b)) => a.saturating_add(b),
            (Some(a), None) | (None, Some(a)) => a,
            (None, None) => continue,
        };

        filtered_games.push((game, score));
    }

    filtered_games.sort_unstable_by_key(|(_, score)| -*score);

    let filtered_games = filtered_games
        .into_iter()
        .map(|(game, _)| game)
        .collect::<VecModel<_>>();

    ModelRc::from(Rc::new(filtered_games))
}
