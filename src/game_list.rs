// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{Game, GameList};
use anyhow::Result;
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use slint::{ModelRc, VecModel};
use std::{
    fs::{self},
    path::{Path, PathBuf},
    rc::Rc,
};

impl GameList {
    #[must_use]
    pub fn new(drive_path: &Path, data_dir: &Path, sort_by: &str) -> Self {
        let wii_path = drive_path.join("wbfs");
        let gc_path = drive_path.join("games");

        let mut games = Vec::new();
        let _ = read_game_dir(wii_path, true, &mut games, data_dir);
        let _ = read_game_dir(gc_path, false, &mut games, data_dir);

        let mut wii_count = 0;
        let mut wii_size = 0.;

        let mut gc_count = 0;
        let mut gc_size = 0.;

        for game in &games {
            if game.is_wii {
                wii_count += 1;
                wii_size += game.size_gib;
            } else {
                gc_count += 1;
                gc_size += game.size_gib;
            }
        }

        sort(&mut games, sort_by);
        let model = VecModel::from(games);

        Self {
            games: ModelRc::from(Rc::new(model)),
            total_size: wii_size + gc_size,
            wii_count,
            wii_size,
            gc_count,
            gc_size,
        }
    }
}

fn read_game_dir(
    game_dir: PathBuf,
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

pub fn fuzzy_search<G: IntoIterator<Item = Game>>(games: G, query: &str) -> Vec<Game> {
    let matcher = SkimMatcherV2::default();

    let mut filtered_games = Vec::new();
    for game in games {
        let title_score = matcher.fuzzy_match(&game.title, query);
        let id_score = matcher.fuzzy_match(&game.id, query);

        if title_score.is_none() && id_score.is_none() {
            continue;
        }

        let score = title_score
            .unwrap_or(0)
            .saturating_add(id_score.unwrap_or(0));

        filtered_games.push((game, score));
    }

    filtered_games.sort_unstable_by_key(|(_, score)| *score);

    filtered_games.into_iter().map(|(game, _)| game).collect()
}
