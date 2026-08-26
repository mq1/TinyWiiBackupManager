// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use super::game::Game;
use crate::{config::SortBy, errors::Error};
use either::Either;
use smol::{
    fs,
    stream::{self, Stream, StreamExt},
};
use std::{path::Path, sync::Arc};

/// Functional wrapper over a list of games
#[derive(Debug, Clone, Default)]
pub struct GameList {
    sorted_by_name: Vec<Arc<Game>>,
    sorted_by_size: Vec<Arc<Game>>,
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

        let all_games = wii_games
            .chain(ngc_games)
            .map(Arc::new)
            .collect::<Vec<_>>()
            .await;

        let mut sorted_by_name = all_games.clone();
        let mut sorted_by_size = all_games;

        sorted_by_name.sort_by(|a, b| a.title.cmp(&b.title));
        sorted_by_size.sort_by_key(|g| g.size);

        Ok(Self {
            sorted_by_name,
            sorted_by_size,
        })
    }

    pub fn iter_by(&self, sort_by: SortBy) -> impl Iterator<Item = &Arc<Game>> {
        match sort_by {
            SortBy::NameAscending => Either::Left(self.sorted_by_name.iter()),
            SortBy::NameDescending => Either::Right(self.sorted_by_name.iter().rev()),
            SortBy::SizeAscending => Either::Left(self.sorted_by_size.iter()),
            SortBy::SizeDescending => Either::Right(self.sorted_by_size.iter().rev()),
        }
    }
}

async fn scan_dir(covers_dir: &Path, dir_path: &Path, is_wii: bool) -> impl Stream<Item = Game> {
    stream::iter(fs::read_dir(dir_path).await.ok())
        .flatten()
        .then(move |entry| async move {
            let path = entry.ok()?.path();
            Game::try_from_path(path, is_wii, covers_dir).await.ok()
        })
        .filter_map(std::convert::identity)
}
