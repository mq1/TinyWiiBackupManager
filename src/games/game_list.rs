// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use super::game::Game;
use crate::config::SortBy;
use anyhow::Result;
use itertools::Either;
use smol::{
    fs,
    stream::{self, Stream, StreamExt},
};
use smol_str::SmolStr;
use std::path::{Path, PathBuf};
use wii_disc_info::game_id::GameID;

#[derive(Debug, Clone)]
pub struct GameFilter {
    pub show_wii: bool,
    pub show_ngc: bool,
    pub search_term: SmolStr,
}

impl Default for GameFilter {
    fn default() -> Self {
        Self {
            show_wii: true,
            show_ngc: true,
            search_term: SmolStr::default(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct GameList {
    games: Vec<Game>,
    order_by_name: Vec<usize>,
    order_by_size: Vec<usize>,
    pub filter: GameFilter,
}

impl GameList {
    pub async fn new(root_path: PathBuf) -> Result<Self> {
        let wii_dir = root_path.join("wbfs");
        let ngc_dir = root_path.join("games");

        let wii_games = scan_dir(&wii_dir, true).await;
        let ngc_games = scan_dir(&ngc_dir, false).await;

        let games = wii_games.chain(ngc_games).collect::<Vec<_>>().await;

        let mut order_by_name = (0..games.len()).collect::<Vec<_>>();
        let mut order_by_size = order_by_name.clone();

        order_by_name.sort_by_key(|&i| games[i].title());
        order_by_size.sort_by_key(|&i| games[i].size());

        Ok(Self {
            games,
            order_by_name,
            order_by_size,
            filter: GameFilter::default(),
        })
    }

    pub fn iter_by(&self, sort_by: SortBy) -> impl Iterator<Item = &Game> {
        let (order, reversed) = match sort_by {
            SortBy::NameAscending => (&self.order_by_name, false),
            SortBy::NameDescending => (&self.order_by_name, true),
            SortBy::SizeAscending => (&self.order_by_size, false),
            SortBy::SizeDescending => (&self.order_by_size, true),
        };

        let matches_console = |game: &&Game| {
            (self.filter.show_wii && game.is_wii()) || (self.filter.show_ngc && game.is_ngc())
        };

        let matches_search = |game: &&Game| game.matches_search(&self.filter.search_term);

        let iter = order
            .iter()
            .map(|&i| &self.games[i])
            .filter(move |game| matches_console(game) && matches_search(game));

        if reversed {
            Either::Right(iter.rev())
        } else {
            Either::Left(iter)
        }
    }

    pub fn reload_cover(&mut self, game_id: GameID, data_dir: &Path) {
        if let Some(game) = self.games.iter_mut().find(|game| game.id() == game_id) {
            game.load_cover_blocking(data_dir);
        }
    }

    pub fn reload_all_covers(&mut self, data_dir: &Path) {
        for game in &mut self.games {
            game.load_cover_blocking(data_dir);
        }
    }

    pub fn get_all_game_ids(&self) -> Vec<GameID> {
        self.games.iter().map(Game::id).collect()
    }

    pub fn count(&self) -> usize {
        self.games.len()
    }
}

pub async fn scan_dir(dir_path: &Path, is_wii: bool) -> impl Stream<Item = Game> {
    stream::iter(fs::read_dir(dir_path).await.ok())
        .flatten()
        .then(move |entry| async move {
            let path = entry.ok()?.path();
            Game::try_from_path(path, is_wii).await.ok()
        })
        .filter_map(std::convert::identity)
}
