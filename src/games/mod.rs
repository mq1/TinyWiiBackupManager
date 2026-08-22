// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use async_zip::base::read::seek::ZipFileReader;
use smol::{
    fs::File,
    io::BufReader,
    stream::{Stream, StreamExt},
};
use std::path::{Path, PathBuf};

pub mod covers;
pub mod disc_reader;
pub mod game;
pub mod game_id;
pub mod game_list;
pub mod import;

async fn is_game(path: &PathBuf) -> bool {
    let Ok(mut file) = File::open(path).await else {
        return false;
    };

    if path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"))
    {
        let mut reader = BufReader::new(file);

        let Ok(mut zip) = ZipFileReader::new(&mut reader).await else {
            return false;
        };

        let Ok(mut entry) = zip.reader_without_entry(0).await else {
            return false;
        };

        wii_disc_info::Meta::read(&mut entry).await.is_ok()
    } else {
        wii_disc_info::Meta::read(&mut file).await.is_ok()
    }
}

pub async fn keep_valid_games(games: impl Stream<Item = PathBuf>) -> Vec<PathBuf> {
    games
        .then(|p| async { is_game(&p).await.then_some(p) })
        .filter_map(std::convert::identity)
        .collect()
        .await
}
