// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{config::SortBy, games::game::Game};
use compact_str::{CompactString, ToCompactString};
use itertools::Either;
use smol::{
    fs,
    stream::{self, Stream, StreamExt},
};
use std::path::{Path, PathBuf};
use wii_disc_info::game_id::GameID;

#[derive(Debug, Clone)]
pub struct GameFilter {
    show_wii: bool,
    show_ngc: bool,
    search_term: CompactString,
}

impl Default for GameFilter {
    fn default() -> Self {
        Self {
            show_wii: true,
            show_ngc: true,
            search_term: CompactString::default(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct GameList {
    games: Box<[Game]>,
    order_by_name: Box<[usize]>,
    order_by_size: Box<[usize]>,
    filter: GameFilter,
}

#[derive(Debug, Clone, Default)]
pub enum GamesState {
    #[default]
    NotLoaded,
    Loading,
    Loaded(GameList),
    Errored(CompactString),
}

impl GamesState {
    pub async fn load(root_path: PathBuf) -> Self {
        let res = async move {
            let wii_dir = root_path.join("wbfs");
            let ngc_dir = root_path.join("games");

            let wii_games = scan_dir(&wii_dir, true).await;
            let ngc_games = scan_dir(&ngc_dir, false).await;

            let games = wii_games
                .chain(ngc_games)
                .collect::<Vec<_>>()
                .await
                .into_boxed_slice();

            let mut order_by_name = (0..games.len()).collect::<Box<[_]>>();
            let mut order_by_size = order_by_name.clone();

            order_by_name.sort_by_key(|&i| games[i].title());
            order_by_size.sort_by_key(|&i| games[i].size());

            let game_list = GameList {
                games,
                order_by_name,
                order_by_size,
                filter: GameFilter::default(),
            };

            Ok::<_, anyhow::Error>(game_list)
        }
        .await;

        match res {
            Ok(game_list) => GamesState::Loaded(game_list),
            Err(err) => GamesState::Errored(err.to_compact_string()),
        }
    }
}

impl GameList {
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
            .filter(matches_console)
            .filter(matches_search);

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

    pub fn get_all_game_ids(&self) -> impl Iterator<Item = GameID> {
        self.games.iter().map(Game::id)
    }

    pub fn count(&self) -> usize {
        self.games.len()
    }

    pub fn search_term(&self) -> &str {
        &self.filter.search_term
    }

    pub fn show_wii(&self) -> bool {
        self.filter.show_wii
    }

    pub fn show_ngc(&self) -> bool {
        self.filter.show_ngc
    }

    pub fn set_search_term(&mut self, search_term: CompactString) {
        self.filter.search_term = search_term;
    }

    pub fn set_show_wii(&mut self, show_wii: bool) {
        self.filter.show_wii = show_wii;
    }

    pub fn set_show_ngc(&mut self, show_ngc: bool) {
        self.filter.show_ngc = show_ngc;
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
