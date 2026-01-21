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
pub struct GameList(Box<[Game]>);

impl GameList {
    #[inline(always)]
    pub fn empty() -> Self {
        Self(Box::new([]))
    }

    #[inline(always)]
    pub fn new(games: Vec<Game>) -> Self {
        Self(games.into_boxed_slice())
    }

    #[inline(always)]
    pub fn get(&self, i: usize) -> Option<&Game> {
        self.0.get(i)
    }

    #[inline(always)]
    pub fn get_mut(&mut self, i: usize) -> Option<&mut Game> {
        self.0.get_mut(i)
    }

    #[inline(always)]
    pub fn get_unchecked(&self, i: usize) -> &Game {
        &self.0[i]
    }

    #[inline(always)]
    pub fn get_unchecked_mut(&mut self, i: usize) -> &mut Game {
        &mut self.0[i]
    }

    #[inline(always)]
    pub fn iter(&self) -> impl Iterator<Item = &Game> {
        self.0.iter()
    }

    #[inline(always)]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Game> {
        self.0.iter_mut()
    }

    #[inline(always)]
    pub fn iter_wii(&self) -> impl Iterator<Item = &Game> {
        self.0.iter().filter(|g| g.id().is_wii())
    }

    #[inline(always)]
    pub fn iter_gc(&self) -> impl Iterator<Item = &Game> {
        self.0.iter().filter(|g| g.id().is_gc())
    }

    #[inline(always)]
    pub fn wii_indices(&self) -> impl Iterator<Item = usize> {
        self.0
            .iter()
            .enumerate()
            .filter(|(_, g)| g.id().is_wii())
            .map(|(i, _)| i)
    }

    #[inline(always)]
    pub fn gc_indices(&self) -> impl Iterator<Item = usize> {
        self.0
            .iter()
            .enumerate()
            .filter(|(_, g)| g.id().is_gc())
            .map(|(i, _)| i)
    }

    #[inline(always)]
    pub fn total_count(&self) -> usize {
        self.0.len()
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
        self.iter().fold(Size::default(), |acc, g| acc + g.size())
    }

    #[inline(always)]
    pub fn wii_size(&self) -> Size {
        self.iter_wii()
            .fold(Size::default(), |acc, g| acc + g.size())
    }

    #[inline(always)]
    pub fn gc_size(&self) -> Size {
        self.iter_gc()
            .fold(Size::default(), |acc, g| acc + g.size())
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
                self.0.reverse();
            }

            (SortBy::SizeAscending, SortBy::NameAscending)
            | (SortBy::SizeDescending, SortBy::NameAscending)
            | (SortBy::None, SortBy::NameAscending) => {
                self.0.sort_unstable_by(|a, b| a.title().cmp(b.title()));
            }

            (SortBy::SizeAscending, SortBy::NameDescending)
            | (SortBy::SizeDescending, SortBy::NameDescending)
            | (SortBy::None, SortBy::NameDescending) => {
                self.0.sort_unstable_by(|a, b| b.title().cmp(a.title()));
            }

            (SortBy::NameAscending, SortBy::SizeAscending)
            | (SortBy::NameDescending, SortBy::SizeAscending)
            | (SortBy::None, SortBy::SizeAscending) => {
                self.0.sort_unstable_by_key(|a| a.size());
            }

            (SortBy::NameAscending, SortBy::SizeDescending)
            | (SortBy::NameDescending, SortBy::SizeDescending)
            | (SortBy::None, SortBy::SizeDescending) => {
                self.0.sort_unstable_by_key(|a| std::cmp::Reverse(a.size()))
            }
        }
    }

    pub fn fuzzy_search(&self, query: &str) -> Box<[usize]> {
        let matcher = SkimMatcherV2::default();

        self.iter()
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
            .collect()
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
