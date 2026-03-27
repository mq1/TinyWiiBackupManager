// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{GameID, id_map::ID_MAP};
use anyhow::{Result, anyhow};
use derive_getters::Getters;
use size::Size;
use std::{
    ffi::{OsStr, OsString},
    fs,
    path::PathBuf,
};

#[derive(Debug, Clone, Getters)]
pub struct Game {
    path: PathBuf,
    is_wii: bool,
    size: Size,
    title: String,
    id: GameID,
}

impl PartialEq for Game {
    fn eq(&self, other: &Self) -> bool {
        self.path() == other.path()
    }
}

impl Eq for Game {}

impl Game {
    pub fn maybe_from_path(path: PathBuf, is_wii: bool) -> Option<Self> {
        if !path.is_dir() {
            return None;
        }

        let filename = path.file_name()?.to_str()?;
        if filename.starts_with('.') {
            return None;
        }

        let (title_str, id_str) = filename.split_once(" [")?;
        let id_str = id_str.strip_suffix(']')?;
        let id = GameID::try_from(id_str).ok()?;

        let title = ID_MAP
            .get(&id)
            .map_or(title_str, |e| e.title().as_str())
            .to_string();

        let size = fs_extra::dir::get_size(&path).unwrap_or(0);

        Some(Self {
            path,
            is_wii,
            size: Size::from_bytes(size),
            title,
            id,
        })
    }

    pub fn get_disc_path(&self) -> Result<PathBuf> {
        let entries = fs::read_dir(&self.path)?;

        for entry in entries.filter_map(Result::ok) {
            if !entry.file_type().is_ok_and(|t| t.is_file()) {
                continue;
            }

            let path = entry.path();

            let Some(filename) = path.file_name().and_then(OsStr::to_str) else {
                continue;
            };

            if filename.starts_with('.') {
                continue;
            }

            if filename.ends_with(".part1.iso") {
                continue;
            }

            let Some(ext) = path.extension() else {
                continue;
            };

            if ext.eq_ignore_ascii_case("iso")
                || ext.eq_ignore_ascii_case("wbfs")
                || ext.eq_ignore_ascii_case("ciso")
            {
                return Ok(path);
            }
        }

        Err(anyhow!("No disc found"))
    }

    pub fn get_path_uri(&self) -> OsString {
        self.path.as_os_str().to_os_string()
    }

    pub fn get_gametdb_uri(&self) -> OsString {
        format!("https://www.gametdb.com/Wii/{}", &self.id.inner).into()
    }
}
