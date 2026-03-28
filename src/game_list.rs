// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{Game, GameList};
use anyhow::Result;
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use slint::{Model, ModelRc, VecModel};
use std::{
    fs,
    path::{Path, PathBuf},
};

impl GameList {
    #[must_use]
    pub fn new(drive_path: &Path) -> Self {
        let wii_path = drive_path.join("wbfs");
        let gc_path = drive_path.join("games");

        let mut games = Vec::new();
        let _ = read_game_dir(wii_path, true, &mut games);
        let _ = read_game_dir(gc_path, false, &mut games);

        let mut wii_count = 0;
        let mut wii_size = 0.;

        let mut gc_count = 0;
        let mut gc_size = 0.;

        for game in &games {
            if game.is_wii {
                wii_count += 1;
                wii_size += game.size;
            } else {
                gc_count += 1;
                gc_size += game.size;
            }
        }

        Self {
            games: ModelRc::from(games.as_slice()),
            filtered_games: ModelRc::from([]),
            total_size: wii_size + gc_size,
            wii_count,
            wii_size,
            gc_count,
            gc_size,
        }
    }

    pub fn sort(&mut self, sort_by: &str) {
        let f: fn(&Game, &Game) -> std::cmp::Ordering = match sort_by {
            "name_ascending" => |a, b| a.title.cmp(&b.title),
            "name_descending" => |a, b| b.title.cmp(&a.title),
            "size_ascending" => |a, b| a.size.total_cmp(&b.size),
            "size_descending" => |a, b| b.size.total_cmp(&a.size),
            _ => |_, _| std::cmp::Ordering::Equal,
        };

        let mut games = self.games.iter().collect::<Vec<_>>();
        games.sort_by(f);

        self.games
            .as_any()
            .downcast_ref::<VecModel<Game>>()
            .unwrap()
            .set_vec(games);
    }

    pub fn fuzzy_search(&mut self, query: &str) {
        let matcher = SkimMatcherV2::default();

        let mut games = Vec::new();
        for game in self.games.iter() {
            let title_score = matcher.fuzzy_match(&game.title, query);
            let id_score = matcher.fuzzy_match(&game.id, query);

            if title_score.is_none() && id_score.is_none() {
                continue;
            }

            let score = title_score
                .unwrap_or(0)
                .saturating_add(id_score.unwrap_or(0));

            games.push((game, score));
        }

        games.sort_unstable_by_key(|(_, score)| *score);

        let games = games.into_iter().map(|(game, _)| game).collect::<Vec<_>>();

        self.filtered_games
            .as_any()
            .downcast_ref::<VecModel<Game>>()
            .unwrap()
            .set_vec(games);
    }
}

fn read_game_dir(game_dir: PathBuf, is_wii: bool, games: &mut Vec<Game>) -> Result<()> {
    if !game_dir.exists() {
        return Ok(());
    }

    let entries = fs::read_dir(game_dir)?;
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if let Some(game) = Game::maybe_from_path(&path, is_wii) {
            games.push(game);
        }
    }

    Ok(())
}
