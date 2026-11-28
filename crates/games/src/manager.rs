// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::game::Game;
use itertools::Either;
use std::{fs, io, path::Path};
use twbm_common::config_contents::SortBy;
use wii_disc_info::game_id::GameID;

#[derive(Debug, Clone)]
pub struct GameFilter {
    show_wii: bool,
    show_ngc: bool,
    search_term: String,
}

#[derive(Debug, Clone)]
pub struct GameList {
    games: Vec<Game>,
    order_by_name: Box<[usize]>,
    order_by_size: Box<[usize]>,
    filter: GameFilter,
}

#[derive(Debug, Clone)]
pub enum GameManager {
    Loading,
    Loaded(GameList),
    Errored(String),
}

impl GameManager {
    pub fn load(root_path: &Path, data_dir: &Path) -> Self {
        let wii_dir = root_path.join("wbfs");
        let ngc_dir = root_path.join("games");

        let mut games = Vec::new();

        if let Err(e) = scan_dir(&wii_dir, true, &mut games, data_dir)
            && e.kind() != io::ErrorKind::NotFound
        {
            return Self::Errored(e.to_string());
        }

        if let Err(e) = scan_dir(&ngc_dir, false, &mut games, data_dir)
            && e.kind() != io::ErrorKind::NotFound
        {
            return Self::Errored(e.to_string());
        }

        let mut order_by_name = (0..games.len()).collect::<Box<[_]>>();
        let mut order_by_size = order_by_name.clone();

        order_by_name.sort_by_key(|&i| games[i].title());
        order_by_size.sort_by_key(|&i| games[i].size());

        Self::Loaded(GameList {
            games,
            order_by_name,
            order_by_size,
            filter: GameFilter {
                show_wii: true,
                show_ngc: true,
                search_term: "".into(),
            },
        })
    }
}

impl GameList {
    pub fn iter_by(&self, sort_by: SortBy) -> impl Iterator<Item = &Game> {
        let (order, reversed) = match sort_by {
            SortBy::NameAscending => (self.order_by_name.as_ref(), false),
            SortBy::NameDescending => (self.order_by_name.as_ref(), true),
            SortBy::SizeAscending => (self.order_by_size.as_ref(), false),
            SortBy::SizeDescending => (self.order_by_size.as_ref(), true),
        };

        let iter = order.iter().map(|&i| &self.games[i]).filter(|game| {
            ((self.filter.show_wii && game.is_wii()) || (self.filter.show_ngc && game.is_ngc()))
                && game.matches_search(&self.filter.search_term)
        });

        if reversed {
            Either::Right(iter.rev())
        } else {
            Either::Left(iter)
        }
    }

    pub fn reload_cover(&mut self, game_id: GameID, data_dir: &Path) {
        if let Some(game) = self.games.iter_mut().find(|game| game.id() == game_id) {
            game.reset_cover_path(data_dir);
        }
    }

    pub fn reload_all_covers(&mut self, data_dir: &Path) {
        for game in &mut self.games {
            game.reset_cover_path(data_dir);
        }
    }

    pub fn get_all_game_ids(&self) -> Box<[GameID]> {
        self.games.iter().map(Game::id).collect()
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

    pub fn set_search_term(&mut self, search_term: String) {
        self.filter.search_term = search_term;
    }

    pub fn set_show_wii(&mut self, show_wii: bool) {
        self.filter.show_wii = show_wii;
    }

    pub fn set_show_ngc(&mut self, show_ngc: bool) {
        self.filter.show_ngc = show_ngc;
    }
}

pub fn scan_dir(
    dir_path: &Path,
    is_wii: bool,
    games: &mut Vec<Game>,
    data_dir: &Path,
) -> io::Result<()> {
    for entry in fs::read_dir(dir_path)? {
        if let Ok(game) = Game::try_from_path(entry?.path(), is_wii, data_dir) {
            games.push(game);
        }
    }

    Ok(())
}
