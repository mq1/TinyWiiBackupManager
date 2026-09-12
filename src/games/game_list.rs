// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use super::game::Game;
use crate::{config::SortBy, errors::Error, util::misc::contains_ignore_case};
use either::Either;
use size::Size;
use smol::{
    fs,
    stream::{self, Stream, StreamExt},
};
use std::{
    ops::Add,
    path::{Path, PathBuf},
};
use wii_disc_info::game_id::GameID;

#[derive(Debug, Clone)]
pub struct GameFilter {
    pub show_wii: bool,
    pub show_ngc: bool,
    pub search_term: String,
}

impl Default for GameFilter {
    fn default() -> Self {
        Self {
            show_wii: true,
            show_ngc: true,
            search_term: String::new(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct GameList {
    games: Vec<Game>,
    total_size: Size,
    order_by_name: Vec<usize>,
    order_by_size: Vec<usize>,
    pub filter: GameFilter,
}

impl GameList {
    pub async fn new(root_path: PathBuf) -> Result<Self, Error> {
        let wii_dir = root_path.join("wbfs");
        let ngc_dir = root_path.join("games");

        let wii_games = scan_dir(&wii_dir, true).await;
        let ngc_games = scan_dir(&ngc_dir, false).await;

        let games = wii_games.chain(ngc_games).collect::<Vec<_>>().await;
        let total_size = games
            .iter()
            .map(|game| game.size)
            .fold(Size::from_bytes(0), Add::add);

        let mut order_by_name = (0..games.len()).collect::<Vec<_>>();
        let mut order_by_size = order_by_name.clone();

        order_by_name.sort_by_key(|&i| &games[i].title);
        order_by_size.sort_by_key(|&i| games[i].size);

        Ok(Self {
            games,
            total_size,
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
            (self.filter.show_wii && game.is_wii) || (self.filter.show_ngc && !game.is_wii)
        };

        let matches_search =
            |game: &&Game| contains_ignore_case(&game.title, &self.filter.search_term);

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
        if let Some(game) = self.games.iter_mut().find(|game| game.id == game_id) {
            game.load_cover_blocking(data_dir);
        }
    }

    pub fn reload_all_covers(&mut self, data_dir: &Path) {
        for game in &mut self.games {
            game.load_cover_blocking(data_dir);
        }
    }

    pub fn get_all_game_ids(&self) -> Vec<GameID> {
        self.games.iter().map(|game| game.id).collect()
    }

    pub fn count(&self) -> usize {
        self.games.len()
    }

    pub fn total_size(&self) -> Size {
        self.total_size
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
