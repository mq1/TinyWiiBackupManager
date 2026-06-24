// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{config::SortBy, games::game::Game};
use anyhow::Result;
use std::{fs, path::Path};

pub mod game;
pub mod game_id;

pub fn list(root_path: impl AsRef<Path>, sort_by: SortBy) -> Result<Vec<Game>> {
    let root_path = root_path.as_ref();

    let wii_dir = root_path.join("wbfs");
    let ngc_dir = root_path.join("games");

    let wii_entries = fs::read_dir(&wii_dir)?.filter_map(Result::ok);
    let ngc_entries = fs::read_dir(&ngc_dir)?.filter_map(Result::ok);

    let wii_games = wii_entries.filter_map(|entry| Game::try_from_path(entry.path(), true).ok());
    let ngc_games = ngc_entries.filter_map(|entry| Game::try_from_path(entry.path(), false).ok());

    let mut all_games = wii_games.chain(ngc_games).collect::<Vec<_>>();
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
