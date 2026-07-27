// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{errors::Error, games::game_id::GameID, util};
use size::Size;
use smol::fs;
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub struct Game {
    pub path: PathBuf,
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
    ) -> Result<Self, Error> {
        let path = path.into();

        // Check if the path is a directory
        if !fs::metadata(&path).await?.is_dir() {
            return Err(Error::NotADir);
        }

        // Get the directory name
        let dir_name = path
            .file_name()
            .and_then(OsStr::to_str)
            .ok_or(Error::InvalidDirName)?;

        if dir_name.starts_with('.') {
            return Err(Error::HiddenDir);
        }

        // Extract title and id from the directory name
        let (title_raw, id_raw) = dir_name.split_once('[').ok_or(Error::InvalidDirName)?;

        // Check if the id is enclosed in square brackets
        if !id_raw.ends_with(']') {
            return Err(Error::InvalidDirName);
        }

        // Parse the id
        let id = id_raw[..id_raw.len() - 1]
            .parse::<GameID>()
            .map_err(|()| Error::InvalidDirName)?;

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

    pub async fn get_disc_path(&self) -> Option<PathBuf> {
        let wii_wbfs = format!("{}.wbfs", self.id);
        let wii_iso = format!("{}.iso", self.id);
        let wii_part0_iso = format!("{}.part0.iso", self.id);

        let possible_filenames = [
            wii_wbfs.as_str(),
            wii_iso.as_str(),
            wii_part0_iso.as_str(),
            "game.iso",
            "game.ciso",
        ];

        for filename in possible_filenames {
            let path = self.path.join(filename);

            if fs::metadata(&path).await.is_ok_and(|meta| meta.is_file()) {
                return Some(path);
            }
        }

        None
    }
}

impl PartialEq for Game {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl Eq for Game {}
