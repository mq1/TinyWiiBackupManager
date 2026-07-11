// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{games::game_id::GameID, util};
use anyhow::{Result, anyhow, bail};
use serde::Deserialize;
use size::Size;
use smol::fs;
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Deserialize)]
pub struct Game {
    path: PathBuf,
    pub id: GameID,
    pub title: String,
    pub size: Size,
    pub is_wii: bool,
    pub cached_cover_path: PathBuf,
}

impl Game {
    pub async fn try_from_path(
        path: impl Into<PathBuf>,
        is_wii: bool,
        covers_dir: &Path,
    ) -> Result<Self> {
        let path = path.into();

        // Check if the path is a directory
        if !fs::metadata(&path).await?.is_dir() {
            bail!("Path is not a directory");
        }

        // Get the directory name
        let dir_name = path
            .file_name()
            .and_then(OsStr::to_str)
            .ok_or_else(|| anyhow!("Invalid directory name"))?;

        if dir_name.starts_with('.') {
            bail!("Hidden path");
        }

        // Extract title and id from the directory name
        let (title_raw, id_raw) = dir_name
            .split_once('[')
            .ok_or_else(|| anyhow!("Invalid directory name"))?;

        // Check if the id is enclosed in square brackets
        if !id_raw.ends_with(']') {
            bail!("Invalid directory name");
        }

        // Parse the id
        let id = id_raw[..id_raw.len() - 1]
            .parse::<GameID>()
            .map_err(|_| anyhow!("Invalid directory name"))?;

        let title = title_raw.trim().to_string();
        let size = util::misc::get_dir_size(&path).await;

        let cached_cover_path = covers_dir.join(id.as_str()).with_extension("png");

        Ok(Self {
            path,
            id,
            title,
            size,
            is_wii,
            cached_cover_path,
        })
    }
}

impl PartialEq for Game {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl Eq for Game {}
