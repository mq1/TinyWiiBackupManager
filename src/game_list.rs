// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::game::Game;
use anyhow::Result;
use derive_getters::Getters;
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use size::Size;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Getters)]
pub struct GameList {
    #[getter(skip)]
    list: Box<[Game]>,
    total_size: Size,
    wii_count: usize,
    wii_size: Size,
    gc_count: usize,
    gc_size: Size,
    #[getter(skip)]
    filtered_indices: Box<[(usize, i64)]>,
}

impl GameList {
    pub fn empty() -> Self {
        Self {
            list: Box::new([]),
            total_size: Size::from_bytes(0),
            wii_count: 0,
            wii_size: Size::from_bytes(0),
            gc_count: 0,
            gc_size: Size::from_bytes(0),
            filtered_indices: Box::new([]),
        }
    }

    pub fn new(drive_path: &Path) -> Result<Self> {
        let wii_path = drive_path.join("wbfs");
        let gc_path = drive_path.join("games");

        let mut games = Vec::new();
        read_game_dir(wii_path, true, &mut games)?;
        read_game_dir(gc_path, false, &mut games)?;

        let mut wii_count = 0;
        let mut wii_size = Size::from_bytes(0);

        let mut gc_count = 0;
        let mut gc_size = Size::from_bytes(0);

        for game in &games {
            if game.is_wii() {
                wii_count += 1;
                wii_size += game.size();
            } else {
                gc_count += 1;
                gc_size += game.size();
            }
        }

        Ok(Self {
            list: games.into_boxed_slice(),
            total_size: wii_size + gc_size,
            wii_count,
            wii_size,
            gc_count,
            gc_size,
            filtered_indices: Box::new([]),
        })
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &Game> {
        self.list.iter()
    }

    #[inline]
    pub fn iter_filtered(&self) -> impl Iterator<Item = &Game> {
        self.filtered_indices
            .iter()
            .copied()
            .map(|(i, _score)| &self.list[i])
    }

    #[inline]
    pub fn total_count(&self) -> usize {
        self.list.len()
    }

    pub fn sort(&mut self, sort_by: &str) {
        match sort_by {
            "name_ascending" => {
                self.list.sort_unstable_by(|a, b| a.title().cmp(b.title()));
            }
            "name_descending" => {
                self.list.sort_unstable_by(|a, b| b.title().cmp(a.title()));
            }
            "size_ascending" => {
                self.list.sort_unstable_by(|a, b| a.size().cmp(b.size()));
            }
            "size_descending" => {
                self.list.sort_unstable_by(|a, b| b.size().cmp(a.size()));
            }
            _ => {}
        }
    }

    pub fn fuzzy_search(&mut self, query: &str) {
        let matcher = SkimMatcherV2::default();

        self.filtered_indices = self
            .list
            .iter()
            .enumerate()
            .filter_map(|(i, game)| {
                let title_score = matcher.fuzzy_match(game.title(), query);
                let id_score = matcher.fuzzy_match(&game.id().inner, query);

                match (title_score, id_score) {
                    (Some(s1), Some(s2)) => Some((i, s1 + s2)),
                    (Some(s1), None) | (None, Some(s1)) => Some((i, s1)),
                    (None, None) => None,
                }
            })
            .collect();

        self.filtered_indices
            .sort_unstable_by_key(|(_, score)| *score);
    }
}

fn read_game_dir(game_dir: PathBuf, is_wii: bool, games: &mut Vec<Game>) -> Result<()> {
    if !game_dir.exists() {
        return Ok(());
    }

    let entries = fs::read_dir(game_dir)?;
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if let Some(game) = Game::maybe_from_path(path, is_wii) {
            games.push(game);
        }
    }

    Ok(())
}
