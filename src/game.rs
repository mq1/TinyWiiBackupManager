// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{Game, SortBy, id_map::ID_MAP, util::GIB};
use anyhow::{Result, anyhow};
use slint::{Image, ToSharedString};
use std::{
    ffi::{OsStr, OsString},
    fs,
    path::{Path, PathBuf},
};

impl Game {
    pub fn maybe_from_path(path: &Path, is_wii: bool, data_dir: &Path) -> Option<Self> {
        if !path.is_dir() {
            return None;
        }

        let filename = path.file_name()?.to_str()?;
        if filename.starts_with('.') {
            return None;
        }

        let (title_str, id_str) = filename.split_once('[')?;
        let id = id_str.strip_suffix(']')?;
        if !matches!(id.len(), 4 | 6) {
            return None;
        }

        let title = match ID_MAP.get(id) {
            Some(e) => e.title.to_shared_string(),
            None => title_str.trim().to_shared_string(),
        };

        let size = fs_extra::dir::get_size(path).unwrap_or(0);

        #[allow(clippy::cast_precision_loss)]
        let size_gib = (size as f32 / GIB * 100.).round() / 100.;

        let cover_path = data_dir.join("covers").join(format!("{id}.png"));
        let cover = Image::load_from_path(&cover_path).unwrap_or_default();

        Some(Self {
            path: path.to_string_lossy().to_shared_string(),
            is_wii,
            size_gib,
            title,
            id: id.to_shared_string(),
            cover,
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
    pub fn lang_str(&self) -> &'static str {
        match self.id.as_bytes()[3] {
            b'E' | b'N' => "US",
            b'J' => "JA",
            b'K' | b'Q' | b'T' => "KO",
            b'R' => "RU",
            b'W' => "ZH",
            _ => "EN",
        }
    }
}

pub fn region(id: [u8; 6]) -> &'static str {
    match id[3] {
        b'A' => "System Wii Channels (i.e. Mii Channel)",
        b'B' => "Ufouria: The Saga (NA)",
        b'D' => "Germany",
        b'E' => "USA",
        b'F' => "France",
        b'H' => "Netherlands / Europe alternate languages",
        b'I' => "Italy",
        b'J' => "Japan",
        b'K' => "Korea",
        b'L' => "Japanese import to Europe, Australia and other PAL regions",
        b'M' => "American import to Europe, Australia and other PAL regions",
        b'N' => "Japanese import to USA and other NTSC regions",
        b'P' => "Europe and other PAL regions such as Australia",
        b'Q' => "Japanese Virtual Console import to Korea",
        b'R' => "Russia",
        b'S' => "Spain",
        b'T' => "American Virtual Console import to Korea",
        b'U' => "Australia / Europe alternate languages",
        b'V' => "Scandinavia",
        b'W' => "Republic of China (Taiwan) / Hong Kong / Macau",
        b'X' | b'Y' | b'Z' => "Europe alternate languages / US special releases",
        _ => "Unknown",
    }
}

pub fn get_compare_fn(sort_by: SortBy) -> fn(&Game, &Game) -> std::cmp::Ordering {
    match sort_by {
        SortBy::NameAscending => |a, b| a.title.cmp(&b.title),
        SortBy::NameDescending => |a, b| b.title.cmp(&a.title),
        SortBy::SizeAscending => |a, b| a.size_gib.total_cmp(&b.size_gib),
        SortBy::SizeDescending => |a, b| b.size_gib.total_cmp(&a.size_gib),
    }
}
