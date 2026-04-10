// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{Game, GameList, SortBy, game};
use anyhow::Result;
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use slint::{Model, ModelRc, SharedString, VecModel};
use std::{fs, path::Path, rc::Rc};

impl GameList {
    #[must_use]
    pub fn new(drive_path: &Path, data_dir: &Path, sort_by: SortBy) -> Self {
        let wii_path = drive_path.join("wbfs");
        let gc_path = drive_path.join("games");

        let mut games = Vec::new();
        let _ = read_game_dir(&wii_path, true, &mut games, data_dir);
        let _ = read_game_dir(&gc_path, false, &mut games, data_dir);

        let total_size = games.iter().fold(0., |acc, game| acc + game.size_gib);

        games.sort_by(game::get_compare_fn(sort_by));
        let model = VecModel::from(games);

        Self {
            games: ModelRc::from(Rc::new(model)),
            filter: SharedString::new(),
            filtered_games: ModelRc::default(),
            total_size,
        }
    }
}

fn read_game_dir(
    game_dir: &Path,
    is_wii: bool,
    games: &mut Vec<Game>,
    data_dir: &Path,
) -> Result<()> {
    if !game_dir.exists() {
        return Ok(());
    }

    let entries = fs::read_dir(game_dir)?;
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if let Some(game) = Game::maybe_from_path(&path, is_wii, data_dir) {
            games.push(game);
        }
    }

    Ok(())
}

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

    filtered_games.sort_unstable_by_key(|(_, score)| *score);

    let filtered_games = filtered_games
        .into_iter()
        .map(|(game, _)| game)
        .collect::<VecModel<_>>();

    ModelRc::from(Rc::new(filtered_games))
}
