// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use anyhow::{Context, Result};
use std::path::PathBuf;

use crate::titles::GAME_TITLES;

#[derive(Clone)]
pub struct Game {
    pub id: String,
    pub display_title: String,

    // games are in the format "GAME TITLE [GAMEID]/"
    pub path: PathBuf,
}

impl Game {
    pub fn from_path(path: PathBuf) -> Result<Self> {
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .context("Failed to get file name from path")?;

        let id = {
            let start = file_name
                .rfind('[')
                .context("Could not find '[' in file name")? + 1;
            let end = file_name
                .rfind(']')
                .context("Could not find ']' in file name")?;
            file_name
                .get(start..end)
                .context("Could not extract ID")?
                .to_string()
        };

        let title = path
            .file_stem()
            .and_then(|name| name.to_str())
            .map(|name| name.trim_end_matches(&format!(" [{}]", id)))
            .context("Failed to extract title from path")?
            .to_string();

        let display_title = GAME_TITLES
            .get(id.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("{} [{}]", title, id));

        Ok(Self {
            id,
            display_title,
            path,
        })
    }
}
