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

        let all_games = wii_games.chain(ngc_games).collect().await;

        Ok(Self { inner: all_games })
    }

    pub fn sorted_by(mut self, sort_by: SortBy) -> Self {
        let compare: fn(&Game, &Game) -> _ = match sort_by {
            SortBy::NameDescending => |a, b| a.title().cmp(b.title()),
            SortBy::NameAscending => |a, b| b.title().cmp(a.title()),
            SortBy::SizeDescending => |a, b| a.size().cmp(&b.size()),
            SortBy::SizeAscending => |a, b| b.size().cmp(&a.size()),
        };

        self.inner.sort_unstable_by(compare);
        self
    }

    pub fn iter(&self) -> impl Iterator<Item = &Game> {
        self.inner.iter()
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
