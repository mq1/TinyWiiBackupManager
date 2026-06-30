// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{config::SortBy, games::game::Game};
use anyhow::Result;
use smol::{fs, stream::StreamExt};
use std::path::Path;

pub mod game;
pub mod game_id;

async fn scan_dir(dir_path: &Path, is_wii: bool) -> Result<Vec<Game>> {
    let mut games = Vec::new();

    if !fs::metadata(dir_path)
        .await
        .map(|m| m.is_dir())
        .unwrap_or(false)
    {
        return Ok(games);
    }

    while let Some(entry) = fs::read_dir(dir_path).await?.next().await {
        if let Ok(entry) = entry
            && let Ok(game) = Game::try_from_path(entry.path(), is_wii).await
        {
            games.push(game);
        }
    }

    Ok(games)
}

pub async fn list(root_path: impl AsRef<Path>, sort_by: SortBy) -> Result<Vec<Game>> {
    let root_path = root_path.as_ref();

    let wii_dir = root_path.join("wbfs");
    let ngc_dir = root_path.join("games");

    let wii_games = scan_dir(&wii_dir, true).await?;
    let mut ngc_games = scan_dir(&ngc_dir, false).await?;

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
