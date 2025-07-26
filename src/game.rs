// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::titles::GAME_TITLES;
use anyhow::{Context, Result};
use std::path::PathBuf;

#[derive(Clone)]
pub struct Game {
    pub id: String,
    pub display_title: String,
    pub path: PathBuf,
}

impl Game {
    pub fn from_path(path: PathBuf) -> Result<Self> {
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .context("Invalid file name")?;

        let (id_start, id_end) = (
            file_name.rfind('[').context("No '[' in file name")? + 1,
            file_name.rfind(']').context("No ']' in file name")?,
        );

        let id = file_name[id_start..id_end].to_string();
        let title = path
            .file_stem()
            .and_then(|n| n.to_str())
            .map(|n| n.trim_end_matches(&format!(" [{id}]")))
            .context("Failed to get title")?;

        let display_title = GAME_TITLES
            .get(&*id)
            .map_or_else(|| format!("{title} [{id}]"), |&s| s.into());

        Ok(Self {
            id,
            display_title,
            path,
        })
    }
}