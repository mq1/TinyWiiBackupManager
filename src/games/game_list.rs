// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use futures::{Stream, StreamExt, TryFutureExt};
use smol::fs;
use std::{ops::Index, path::Path};

use super::game::Game;
use crate::{config::SortBy, errors::Error};

/// Functional wrapper over a list of games
#[derive(Debug, Clone, Default)]
pub struct GameList {
    inner: Vec<Game>,
    sorted_by_name: Vec<usize>,
    sorted_by_size: Vec<usize>,
}

impl GameList {
    pub async fn new(
        data_dir: impl AsRef<Path>,
        root_path: impl AsRef<Path>,
    ) -> Result<Self, Error> {
        let covers_dir = data_dir.as_ref().join("covers");
        fs::create_dir_all(&covers_dir).await?;

        let wii_dir = root_path.as_ref().join("wbfs");
        let ngc_dir = root_path.as_ref().join("games");

        let wii_games = scan_dir(&covers_dir, &wii_dir, true).await;
        let ngc_games = scan_dir(&covers_dir, &ngc_dir, false).await;

        let all_games = wii_games.chain(ngc_games).collect::<Vec<_>>().await;

        let mut sorted_by_name = (0..all_games.len()).collect::<Vec<_>>();
        sorted_by_name.sort_by(|&a, &b| all_games[a].title().cmp(all_games[b].title()));

        let mut sorted_by_size = (0..all_games.len()).collect::<Vec<_>>();
        sorted_by_size.sort_by(|&a, &b| all_games[a].size().cmp(&all_games[b].size()));

        Ok(Self {
            inner: all_games,
            sorted_by_name,
            sorted_by_size,
        })
    }

    pub fn iter_by(&self, sort_by: SortBy) -> impl Iterator<Item = &Game> {
        let get_i = move |i| match sort_by {
            SortBy::NameDescending => self.sorted_by_name[i],
            SortBy::NameAscending => self.sorted_by_name[self.sorted_by_name.len() - 1 - i],
            SortBy::SizeDescending => self.sorted_by_size[i],
            SortBy::SizeAscending => self.sorted_by_size[self.sorted_by_size.len() - 1 - i],
        };

        (0..self.inner.len()).map(move |idx| &self.inner[get_i(idx)])
    }

    pub fn entry(&self, path: &Path) -> (usize, &Game) {
        self.inner
            .iter()
            .position(|game| game.path() == path)
            .map(|idx| (idx, &self.inner[idx]))
            .unwrap()
    }
}

async fn scan_dir(covers_dir: &Path, dir_path: &Path, is_wii: bool) -> impl Stream<Item = Game> {
    fs::read_dir(dir_path)
        .try_flatten_stream()
        .filter_map(move |entry| async move {
            let path = entry.ok()?.path();
            Game::try_from_path(path, is_wii, covers_dir).await.ok()
        })
}

impl Index<usize> for GameList {
    type Output = Game;

    fn index(&self, index: usize) -> &Self::Output {
        &self.inner[index]
    }
}
