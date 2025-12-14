// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::config::SortBy;
use crate::id_map;
use anyhow::Result;
use anyhow::anyhow;
use anyhow::bail;
use egui_phosphor::regular as ph;
use path_slash::PathExt;
use size::Size;
use std::fs;
use std::path::{Path, PathBuf};

pub fn list(mount_point: &Path) -> Vec<Game> {
    if mount_point.as_os_str().is_empty() {
        return vec![];
    }

    let mut games = Vec::new();

    for (dir_name, is_wii) in &[("wbfs", true), ("games", false)] {
        if let Ok(entries) = fs::read_dir(mount_point.join(dir_name)) {
            games.extend(
                entries.filter_map(|entry| {
                    Game::from_dir(entry.ok()?.path(), mount_point, *is_wii).ok()
                }),
            );
        }
    }

    games
}

impl Game {
    pub fn from_dir(dir: PathBuf, mount_point: &Path, is_wii: bool) -> Result<Self> {
        if !dir.is_dir() {
            bail!("{} {} is not a directory", ph::FILE_X, dir.display());
        }

        let file_name = dir
            .file_name()
            .ok_or(anyhow!("{} No file name found", ph::FILE_X))?
            .to_str()
            .ok_or(anyhow!("{} Invalid file name", ph::FILE_X))?;

        if file_name.starts_with('.') {
            bail!(
                "{} Skipping hidden directory {}",
                ph::FOLDER_DASHED,
                dir.display()
            );
        }

        // Extract title and ID from the directory name, e.g., "Game Title [GAMEID]"
        let (title, id_part) = file_name
            .split_once(" [")
            .ok_or(anyhow!("{} Invalid directory name", ph::FOLDER))?;

        let id = id_part
            .strip_suffix(']')
            .map(GameID::from)
            .ok_or(anyhow!("{} Invalid directory name", ph::FOLDER))?;

        let display_title = id_map::get_title(id.0).unwrap_or(title).to_string();

        // Get the directory size
        let size = Size::from_bytes(fs_extra::dir::get_size(&dir).unwrap_or(0));

        // Construct the path to the game's cover image
        let image_path = mount_point
            .join("apps")
            .join("usbloader_gx")
            .join("images")
            .join(id.as_str())
            .with_extension("png");

        let image_uri = format!("file://{}", image_path.to_slash_lossy());

        let search_str = (display_title.clone() + id.as_str()).to_lowercase();

        // Construct the Game object
        Ok(Self {
            path: dir.clone(),
            id,
            is_wii,
            display_title,
            size,
            image_uri,
            search_str,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Game {
    pub path: PathBuf,
    pub id: GameID,
    pub is_wii: bool,
    pub display_title: String,
    pub size: Size,
    pub image_uri: String,
    pub search_str: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct GameID(pub [u8; 6]);

impl Default for GameID {
    fn default() -> Self {
        Self([b'?'; 6])
    }
}

impl From<&str> for GameID {
    fn from(id: &str) -> Self {
        let mut id_bytes = [0u8; 6];
        let bytes = id.as_bytes();
        let len = bytes.len().min(6);
        id_bytes[..len].copy_from_slice(&bytes[..len]);

        Self(id_bytes)
    }
}

impl GameID {
    pub fn get_region_display(&self) -> &'static str {
        match self.0[3] {
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
            b'X' => "Europe alternate languages / US special releases",
            b'Y' => "Europe alternate languages / US special releases",
            b'Z' => "Europe alternate languages / US special releases",
            _ => "Unknown",
        }
    }

    pub fn get_wiitdb_lang(&self) -> &'static str {
        match self.0[3] {
            b'E' | b'N' => "US",
            b'J' => "JA",
            b'K' | b'Q' | b'T' => "KO",
            b'R' => "RU",
            b'W' => "ZH",
            _ => "EN",
        }
    }

    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.0).unwrap_or("invalid")
    }

    pub fn as_partial(&self) -> &str {
        std::str::from_utf8(&self.0[..3]).unwrap_or("invalid")
    }
}

pub fn sort(games: &mut [Game], prev_sort_by: SortBy, sort_by: SortBy) {
    match (prev_sort_by, sort_by) {
        (SortBy::NameAscending, SortBy::NameAscending)
        | (SortBy::NameDescending, SortBy::NameDescending)
        | (SortBy::SizeAscending, SortBy::SizeAscending)
        | (SortBy::SizeDescending, SortBy::SizeDescending)
        | (_, SortBy::None) => {
            // Do nothing, already sorted
        }

        (SortBy::NameDescending, SortBy::NameAscending)
        | (SortBy::NameAscending, SortBy::NameDescending)
        | (SortBy::SizeDescending, SortBy::SizeAscending)
        | (SortBy::SizeAscending, SortBy::SizeDescending) => {
            games.reverse();
        }

        (SortBy::SizeAscending, SortBy::NameAscending)
        | (SortBy::SizeDescending, SortBy::NameAscending)
        | (SortBy::None, SortBy::NameAscending) => {
            games.sort_unstable_by(|a, b| a.display_title.cmp(&b.display_title));
        }

        (SortBy::SizeAscending, SortBy::NameDescending)
        | (SortBy::SizeDescending, SortBy::NameDescending)
        | (SortBy::None, SortBy::NameDescending) => {
            games.sort_unstable_by(|a, b| b.display_title.cmp(&a.display_title));
        }

        (SortBy::NameAscending, SortBy::SizeAscending)
        | (SortBy::NameDescending, SortBy::SizeAscending)
        | (SortBy::None, SortBy::SizeAscending) => {
            games.sort_unstable_by(|a, b| a.size.cmp(&b.size));
        }

        (SortBy::NameAscending, SortBy::SizeDescending)
        | (SortBy::NameDescending, SortBy::SizeDescending)
        | (SortBy::None, SortBy::SizeDescending) => {
            games.sort_unstable_by(|a, b| b.size.cmp(&a.size));
        }
    }
}
