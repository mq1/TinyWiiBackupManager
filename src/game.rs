// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

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
    pub fn from_path(path: PathBuf) -> Self {
        let id = path
            .file_name()
            .and_then(|name| name.to_str())
            .and_then(|name| {
                let (start, end) = (name.rfind('[')? + 1, name.rfind(']')?);
                name.get(start..end).map(String::from)
            })
            .unwrap_or_else(|| "Unknown".to_string());

        let title = path
            .file_stem()
            .and_then(|name| name.to_str())
            .map(|name| name.trim_end_matches(&format!(" [{}]", id)))
            .unwrap_or("Unknown Title")
            .to_string();

        let display_title = GAME_TITLES
            .get(id.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("{} [{}]", title, id));

        Self {
            id,
            display_title,
            path,
        }
    }
}
