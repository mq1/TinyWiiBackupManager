// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{config::SortBy, errors::Error, games::game::Game};
use smol::{fs, stream::StreamExt};
use std::path::Path;

pub mod covers;
pub mod game;
pub mod game_id;

async fn scan_dir(covers_dir: &Path, dir_path: &Path, is_wii: bool) -> Vec<Game> {
    let Ok(entries) = fs::read_dir(dir_path).await else {
        return Vec::new();
    };

    entries
        .then(async |entry| {
            let path = entry?.path();
            Game::try_from_path(path, is_wii, covers_dir).await
        })
        .filter_map(Result::ok)
        .collect::<Vec<_>>()
        .await
}

pub async fn list(data_dir: &Path, root_path: &Path, sort_by: SortBy) -> Result<Vec<Game>, Error> {
    let covers_dir = data_dir.join("covers");
    fs::create_dir_all(&covers_dir).await?;

    let wii_dir = root_path.join("wbfs");
    let ngc_dir = root_path.join("games");

    let wii_games = scan_dir(&covers_dir, &wii_dir, true).await;
    let mut ngc_games = scan_dir(&covers_dir, &ngc_dir, false).await;

    let mut all_games = wii_games;
    all_games.append(&mut ngc_games);

    sort(&mut all_games, sort_by);

    Ok(all_games)
}

pub fn sort(games: &mut [Game], sort_by: SortBy) {
    games.sort_unstable_by(|a, b| match sort_by {
        SortBy::NameDescending => a.title.cmp(&b.title),
        SortBy::NameAscending => b.title.cmp(&a.title),
        SortBy::SizeDescending => a.size.cmp(&b.size),
        SortBy::SizeAscending => b.size.cmp(&a.size),
    });
}
