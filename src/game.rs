// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::titles::GAME_TITLES;
use anyhow::{Context, Result};
use std::path::PathBuf;

/// Represents a game in the WBFS directory.
#[derive(Clone)]
pub struct Game {
    pub id: String,
    pub display_title: String,
    pub path: PathBuf,
}

impl Game {
    /// Creates a Game instance from its directory path.
    pub fn from_path(path: PathBuf) -> Result<Self> {
        // Extract the file name from the path
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .context("Failed to get file name from path")?;

        // Find the start and end positions of the game ID within square brackets
        let id_start = file_name
            .rfind('[')
            .context("Could not find '[' in file name")?
            + 1;
        let id_end = file_name
            .rfind(']')
            .context("Could not find ']' in file name")?;

        // Extract the game ID
        let id = file_name
            .get(id_start..id_end)
            .context("Could not extract ID")?
            .to_string();

        // Extract the base title (without ID suffix)
        let title = path
            .file_stem()
            .and_then(|name| name.to_str())
            .map(|name| name.trim_end_matches(&format!(" [{id}]")))
            .context("Failed to extract title from path")?
            .to_string();

        // Look up the display title in the predefined titles map, or use a fallback
        let display_title = GAME_TITLES
            .get(id.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("{title} [{id}]"));

        // Return the constructed Game struct
        Ok(Self {
            id,
            display_title,
            path,
        })
    }
}