// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::disc_info::DiscInfo;
use anyhow::Result;
use sanitize_filename::sanitize;
use std::{fs, path::Path};
use walkdir::WalkDir;

pub fn normalize_games_dir(games_dir: &Path) -> Result<()> {
    for entry in WalkDir::new(games_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_dir())
    {
        let path = entry.path();

        if let Ok(disc_info) = DiscInfo::from_game_dir(path) {
            let new_game_dir = games_dir.join(format!(
                "{} [{}]",
                sanitize(&disc_info.title),
                disc_info.id.as_str()
            ));

            fs::rename(path, new_game_dir)?;
        }
    }

    Ok(())
}

pub fn normalize_paths(mount_point: &Path) -> Result<()> {
    normalize_games_dir(&mount_point.join("wbfs"))?;
    normalize_games_dir(&mount_point.join("games"))?;

    Ok(())
}
