// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{Game, id_map::ID_MAP};
use anyhow::{Result, anyhow};
use slint::ToSharedString;
use std::{
    ffi::{OsStr, OsString},
    fs,
    path::{Path, PathBuf},
};

impl Game {
    pub fn maybe_from_path(path: &Path, is_wii: bool) -> Option<Self> {
        if !path.is_dir() {
            return None;
        }

        let filename = path.file_name()?.to_str()?;
        if filename.starts_with('.') {
            return None;
        }

        let (title_str, id_str) = filename.split_once(" [")?;
        let id = id_str.strip_suffix(']')?;
        if !matches!(id.len(), 4 | 6) {
            return None;
        }

        let title = match ID_MAP.get(id) {
            Some(e) => e.title().to_shared_string(),
            None => title_str.to_shared_string(),
        };

        let size = fs_extra::dir::get_size(path).unwrap_or(0);

        #[allow(clippy::cast_precision_loss)]
        let size = size as f32;

        Some(Self {
            path: path.to_string_lossy().to_shared_string(),
            is_wii,
            size,
            title,
            id: id.to_shared_string(),
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

    #[must_use]
    pub fn get_gametdb_uri(&self) -> OsString {
        format!("https://www.gametdb.com/Wii/{}", &self.id).into()
    }

    #[must_use]
    pub fn partial_id(&self) -> &str {
        &self.id[0..3]
    }

    #[must_use]
    pub fn region_str(&self) -> &'static str {
        let region_char = self.id.chars().nth(3);

        match region_char {
            Some('A') => "System Wii Channels (i.e. Mii Channel)",
            Some('B') => "Ufouria: The Saga (NA)",
            Some('D') => "Germany",
            Some('E') => "USA",
            Some('F') => "France",
            Some('H') => "Netherlands / Europe alternate languages",
            Some('I') => "Italy",
            Some('J') => "Japan",
            Some('K') => "Korea",
            Some('L') => "Japanese import to Europe, Australia and other PAL regions",
            Some('M') => "American import to Europe, Australia and other PAL regions",
            Some('N') => "Japanese import to USA and other NTSC regions",
            Some('P') => "Europe and other PAL regions such as Australia",
            Some('Q') => "Japanese Virtual Console import to Korea",
            Some('R') => "Russia",
            Some('S') => "Spain",
            Some('T') => "American Virtual Console import to Korea",
            Some('U') => "Australia / Europe alternate languages",
            Some('V') => "Scandinavia",
            Some('W') => "Republic of China (Taiwan) / Hong Kong / Macau",
            Some('X' | 'Y' | 'Z') => "Europe alternate languages / US special releases",
            _ => "Unknown",
        }
    }

    #[must_use]
    pub fn lang_str(&self) -> &'static str {
        let region_char = self.id.chars().nth(3);

        match region_char {
            Some('E' | 'N') => "US",
            Some('J') => "JA",
            Some('K' | 'Q' | 'T') => "KO",
            Some('R') => "RU",
            Some('W') => "ZH",
            _ => "EN",
        }
    }
}
