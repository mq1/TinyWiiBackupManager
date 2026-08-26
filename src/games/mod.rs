// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::errors::Error;
use async_zip::base::read1::seek::ZipArchiveReader;
use smol::{
    fs::File,
    io::BufReader,
    stream::{Stream, StreamExt},
};
use std::path::PathBuf;

pub mod covers;
pub mod disc_reader;
pub mod game;
pub mod game_list;
pub mod import;

async fn is_game(path: &PathBuf) -> Result<(), Error> {
    let mut file = File::open(path).await?;

    let _meta = if path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"))
    {
        let mut entry = ZipArchiveReader::open(BufReader::new(file))
            .await?
            .file_oneshot(0)
            .await?;

        wii_disc_info::Meta::read(&mut entry).await?
    } else {
        wii_disc_info::Meta::read(&mut file).await?
    };

    Ok(())
}

pub async fn keep_valid_games(games: impl Stream<Item = PathBuf>) -> Vec<PathBuf> {
    games
        .then(|p| async { is_game(&p).await.is_ok().then_some(p) })
        .filter_map(std::convert::identity)
        .collect()
        .await
}
