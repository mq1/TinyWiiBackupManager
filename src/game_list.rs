// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{config::SortBy, game::Game, message::Message, state::State};
use anyhow::Result;
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use iced::{Task, futures::TryFutureExt};
use itertools::Itertools;
use size::Size;
use smol::{fs, stream::StreamExt};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct GameList {
    list: Box<[Game]>,
    total_size: Size,
    wii_indices: Box<[usize]>,
    wii_size: Size,
    gc_indices: Box<[usize]>,
    gc_size: Size,
    filtered_indices: Box<[usize]>,
}

impl GameList {
    #[inline(always)]
    pub fn empty() -> Self {
        Self {
            list: Box::new([]),
            total_size: Size::from_bytes(0),
            wii_indices: Box::new([]),
            wii_size: Size::from_bytes(0),
            gc_indices: Box::new([]),
            gc_size: Size::from_bytes(0),
            filtered_indices: Box::new([]),
        }
    }

    #[inline(always)]
    pub fn new(games: Vec<Game>) -> Self {
        let mut wii_indices = Vec::new();
        let mut wii_size = Size::from_bytes(0);

        let mut gc_indices = Vec::new();
        let mut gc_size = Size::from_bytes(0);

        for (i, game) in games.iter().enumerate() {
            if game.id().is_wii() {
                wii_indices.push(i);
                wii_size += game.size();
            } else if game.id().is_gc() {
                gc_indices.push(i);
                gc_size += game.size();
            }
        }

        Self {
            list: games.into_boxed_slice(),
            total_size: wii_size + gc_size,
            wii_indices: wii_indices.into_boxed_slice(),
            wii_size,
            gc_indices: gc_indices.into_boxed_slice(),
            gc_size,
            filtered_indices: Box::new([]),
        }
    }

    #[inline(always)]
    pub fn get(&self, i: usize) -> Option<&Game> {
        self.list.get(i)
    }

    #[inline(always)]
    pub fn get_mut(&mut self, i: usize) -> Option<&mut Game> {
        self.list.get_mut(i)
    }

    #[inline(always)]
    pub fn get_unchecked(&self, i: usize) -> &Game {
        &self.list[i]
    }

    #[inline(always)]
    pub fn get_unchecked_mut(&mut self, i: usize) -> &mut Game {
        &mut self.list[i]
    }

    #[inline(always)]
    pub fn iter(&self) -> impl Iterator<Item = &Game> {
        self.list.iter()
    }

    #[inline(always)]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Game> {
        self.list.iter_mut()
    }

    #[inline(always)]
    pub fn wii_indices(&self) -> impl Iterator<Item = usize> {
        self.wii_indices.iter().copied()
    }

    #[inline(always)]
    pub fn gc_indices(&self) -> impl Iterator<Item = usize> {
        self.gc_indices.iter().copied()
    }

    #[inline(always)]
    pub fn iter_wii(&self) -> impl Iterator<Item = &Game> {
        self.wii_indices().map(|i| &self.list[i])
    }

    #[inline(always)]
    pub fn iter_gc(&self) -> impl Iterator<Item = &Game> {
        self.gc_indices().map(|i| &self.list[i])
    }

    #[inline(always)]
    pub fn total_count(&self) -> usize {
        self.list.len()
    }

    #[inline(always)]
    pub fn wii_count(&self) -> usize {
        self.iter_wii().count()
    }

    #[inline(always)]
    pub fn gc_count(&self) -> usize {
        self.iter_gc().count()
    }

    #[inline(always)]
    pub fn total_size(&self) -> Size {
        self.total_size
    }

    #[inline(always)]
    pub fn wii_size(&self) -> Size {
        self.wii_size
    }

    #[inline(always)]
    pub fn gc_size(&self) -> Size {
        self.gc_size
    }

    #[inline(always)]
    pub fn filtered_indices(&self) -> impl Iterator<Item = usize> {
        self.filtered_indices.iter().copied()
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

            (SortBy::SizeAscending, SortBy::NameAscending)
            | (SortBy::SizeDescending, SortBy::NameAscending)
            | (SortBy::None, SortBy::NameAscending) => {
                self.list.sort_unstable_by(|a, b| a.title().cmp(b.title()));
            }

            (SortBy::SizeAscending, SortBy::NameDescending)
            | (SortBy::SizeDescending, SortBy::NameDescending)
            | (SortBy::None, SortBy::NameDescending) => {
                self.list.sort_unstable_by(|a, b| b.title().cmp(a.title()));
            }

            (SortBy::NameAscending, SortBy::SizeAscending)
            | (SortBy::NameDescending, SortBy::SizeAscending)
            | (SortBy::None, SortBy::SizeAscending) => {
                self.list.sort_unstable_by_key(|a| a.size());
            }

            (SortBy::NameAscending, SortBy::SizeDescending)
            | (SortBy::NameDescending, SortBy::SizeDescending)
            | (SortBy::None, SortBy::SizeDescending) => {
                self.list
                    .sort_unstable_by_key(|a| std::cmp::Reverse(a.size()));
            }
        }

        // Indices lists need to be recalculated
        let mut wii_indices = Vec::new();
        let mut gc_indices = Vec::new();
        for (i, game) in self.list.iter().enumerate() {
            if game.id().is_wii() {
                wii_indices.push(i);
            } else if game.id().is_gc() {
                gc_indices.push(i);
            }
        }
        self.wii_indices = wii_indices.into_boxed_slice();
        self.gc_indices = gc_indices.into_boxed_slice();
    }

    pub fn fuzzy_search(&mut self, query: &str) {
        let matcher = SkimMatcherV2::default();

        self.filtered_indices = self
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
            .sorted_unstable_by_key(|(_, score)| *score)
            .map(|(i, _)| i)
            .collect();
    }
}

pub fn get_list_games_task(state: &State) -> Task<Message> {
    let drive_path = state.config.mount_point().to_path_buf();

    Task::perform(
        list(drive_path).map_err(|e| e.to_string()),
        Message::GotGameList,
    )
}

async fn list(drive_path: PathBuf) -> Result<GameList> {
    let wii_path = drive_path.join("wbfs");
    let gc_path = drive_path.join("games");

    let mut games = Vec::new();
    read_game_dir(wii_path, &mut games).await?;
    read_game_dir(gc_path, &mut games).await?;

    let game_list = GameList::new(games);
    Ok(game_list)
}

async fn read_game_dir(game_dir: PathBuf, games: &mut Vec<Game>) -> Result<()> {
    if !game_dir.exists() {
        return Ok(());
    }

    let mut entries = fs::read_dir(game_dir).await?;

    while let Some(entry) = entries.try_next().await? {
        if let Some(game) = Game::from_path(entry.path()).await {
            games.push(game);
        }
    }

    Ok(())
}
