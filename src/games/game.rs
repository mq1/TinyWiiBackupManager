// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{games::game_id::GameID, util};
use anyhow::{Result, anyhow, bail};
use size::Size;
use std::{ffi::OsStr, path::PathBuf};

#[derive(Debug, Clone)]
pub struct Game {
    path: PathBuf,
    id: GameID,
    pub title: String,
    pub size: Size,
    is_wii: bool,
}

impl Game {
    pub fn try_from_path(path: impl Into<PathBuf>, is_wii: bool) -> Result<Self> {
        let path = path.into();

        // Check if the path is a directory
        if !path.is_dir() {
            bail!("Path is not a directory");
        }

        // Get the directory name
        let dir_name = path
            .file_name()
            .and_then(OsStr::to_str)
            .ok_or_else(|| anyhow!("Invalid directory name"))?;

        // Extract title and id from the directory name
        let (title_raw, id_raw) = dir_name
            .split_once('[')
            .ok_or_else(|| anyhow!("Invalid directory name"))?;

        // Check if the id is enclosed in square brackets
        if !id_raw.ends_with(']') {
            bail!("Invalid directory name");
        }

        // Parse the id
        let id = GameID::new(&id_raw[..id_raw.len() - 1])
            .ok_or_else(|| anyhow!("Invalid directory name"))?;

        let title = title_raw.trim().to_string();
        let size = util::misc::get_dir_size(&path)?;

        Ok(Self {
            path,
            id,
            title,
            size,
            is_wii,
        })
    }
}

impl PartialEq for Game {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl Eq for Game {}
