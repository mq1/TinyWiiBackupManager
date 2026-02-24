// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{config::SortBy, games::game::Game, message::Message, state::State};
use anyhow::Result;
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use iced::{Task, futures::TryFutureExt};
use size::Size;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub struct GameList {
    list: Box<[Game]>,
    total_size: Size,
    wii_count: usize,
    wii_size: Size,
    gc_count: usize,
    gc_size: Size,
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

    pub fn new(games: impl Into<Box<[Game]>>) -> Self {
        let list = games.into();

        let mut wii_count = 0;
        let mut wii_size = Size::from_bytes(0);

        let mut gc_count = 0;
        let mut gc_size = Size::from_bytes(0);

        for game in &list {
            if game.is_wii() {
                wii_count += 1;
                wii_size += game.size();
            } else {
                gc_count += 1;
                gc_size += game.size();
            }
        }

        Self {
            list,
            total_size: wii_size + gc_size,
            wii_count,
            wii_size,
            gc_count,
            gc_size,
            filtered_indices: Box::new([]),
        }
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

    #[inline]
    pub fn wii_count(&self) -> usize {
        self.wii_count
    }

    #[inline]
    pub fn gc_count(&self) -> usize {
        self.gc_count
    }

    #[inline]
    pub fn total_size(&self) -> Size {
        self.total_size
    }

    #[inline]
    pub fn wii_size(&self) -> Size {
        self.wii_size
    }

    #[inline]
    pub fn gc_size(&self) -> Size {
        self.gc_size
    }

    pub fn sort(&mut self, prev_sort_by: SortBy, sort_by: SortBy) {
        match (prev_sort_by, sort_by) {
            (SortBy::NameAscending, SortBy::NameAscending)
            | (SortBy::NameDescending, SortBy::NameDescending)
            | (SortBy::SizeAscending, SortBy::SizeAscending)
            | (SortBy::SizeDescending, SortBy::SizeDescending)
            | (_, SortBy::None) => {
                // Do nothing, already sorted
            }

            (SortBy::NameDescending, SortBy::NameAscending)
            | (SortBy::NameAscending, SortBy::NameDescending)
            | (SortBy::SizeDescending, SortBy::SizeAscending)
            | (SortBy::SizeAscending, SortBy::SizeDescending) => {
                self.list.reverse();
            }

            (
                SortBy::SizeAscending | SortBy::SizeDescending | SortBy::None,
                SortBy::NameAscending,
            ) => {
                self.list.sort_unstable_by(|a, b| a.title().cmp(b.title()));
            }

            (
                SortBy::SizeAscending | SortBy::SizeDescending | SortBy::None,
                SortBy::NameDescending,
            ) => {
                self.list.sort_unstable_by(|a, b| b.title().cmp(a.title()));
            }

            (
                SortBy::NameAscending | SortBy::NameDescending | SortBy::None,
                SortBy::SizeAscending,
            ) => {
                self.list.sort_unstable_by_key(super::game::Game::size);
            }

            (
                SortBy::NameAscending | SortBy::NameDescending | SortBy::None,
                SortBy::SizeDescending,
            ) => {
                self.list
                    .sort_unstable_by_key(|a| std::cmp::Reverse(a.size()));
            }
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
                let id_score = matcher.fuzzy_match(game.id().as_str(), query);

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

pub fn get_list_games_task(state: &State) -> Task<Message> {
    let drive_path = state.config.mount_point().clone();

    Task::perform(
        async move { list(&drive_path) }.map_err(|e| format!("Failed to list games: {e:#}")),
        Message::GotGameList,
    )
}

fn list(drive_path: &Path) -> Result<GameList> {
    let wii_path = drive_path.join("wbfs");
    let gc_path = drive_path.join("games");

    let mut games = Vec::new();
    read_game_dir(wii_path, true, &mut games)?;
    read_game_dir(gc_path, false, &mut games)?;

    let game_list = GameList::new(games);
    Ok(game_list)
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
