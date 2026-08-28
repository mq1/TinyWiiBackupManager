// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::errors::Error;
use smol::{
    fs::File,
    stream::{Stream, StreamExt},
};
use std::path::PathBuf;
use zip::ZipArchive;

pub mod covers;
pub mod disc_reader;
pub mod game;
pub mod game_list;
pub mod import;

async fn is_game(path: &PathBuf) -> bool {
    if path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"))
    {
        let path = path.clone();
        smol::unblock(move || {
            let file = std::fs::File::open(path)?;
            let mut zip = ZipArchive::new(file)?;
            let mut entry = zip.by_index(0)?;
            let _ = wii_disc_info::Meta::read(&mut entry)?;
            Ok::<(), Error>(())
        })
        .await
        .is_ok()
    } else {
        let Ok(mut file) = File::open(path).await else {
            return false;
        };

        wii_disc_info::Meta::read_async(&mut file).await.is_ok()
    }
}

pub async fn keep_valid_games(games: impl Stream<Item = PathBuf>) -> Vec<PathBuf> {
    games
        .then(|p| async { is_game(&p).await.then_some(p) })
        .filter_map(std::convert::identity)
        .collect()
        .await
}
