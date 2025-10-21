// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::config::SortBy;
use crate::titles::Titles;
use anyhow::Result;
use anyhow::anyhow;
use anyhow::bail;
use path_slash::PathExt;
use size::Size;
use std::fs;
use std::path::{Path, PathBuf};

pub fn list(mount_point: &Path, titles: &Option<Titles>) -> Result<Vec<Game>> {
    let mut games = Vec::new();

    for dir_name in ["wbfs", "games"] {
        let dir = mount_point.join(dir_name);
        if !dir.exists() || !dir.is_dir() {
            continue;
        }

        for entry in fs::read_dir(&dir)?.filter_map(Result::ok) {
            if let Ok(game) = Game::from_dir(entry.path(), titles, mount_point) {
                games.push(game);
            }
        }
    }

    Ok(games)
}

impl Game {
    pub fn from_dir(dir: PathBuf, titles: &Option<Titles>, mount_point: &Path) -> Result<Self> {
        if !dir.is_dir() {
            bail!("{} is not a directory", dir.display());
        }

        let file_name = dir
            .file_name()
            .ok_or(anyhow!("No file name found"))?
            .to_str()
            .ok_or(anyhow!("Invalid file name"))?;

        if file_name.starts_with('.') {
            bail!("Skipping hidden directory {}", dir.display());
        }

        // Extract title and ID from the directory name, e.g., "Game Title [GAMEID]"
        let (title, id_part) = file_name
            .split_once(" [")
            .ok_or(anyhow!("Invalid directory name"))?;

        let id = id_part
            .strip_suffix(']')
            .ok_or(anyhow!("Invalid directory name"))?
            .into();

        // Determine the console from the parent directory ("wbfs" or "games")
        let is_wii = dir
            .parent()
            .ok_or(anyhow!("No parent directory found"))?
            .file_name()
            .ok_or(anyhow!("No file name found"))?
            .to_str()
            .ok_or(anyhow!("Invalid file name"))?
            == "wbfs";

        let display_title = titles
            .as_ref()
            .and_then(|titles| titles.get(id))
            .unwrap_or(title)
            .to_string();

        // Get the directory size
        let size = Size::from_bytes(fs_extra::dir::get_size(&dir).unwrap_or(0));

        // Construct the path to the game's cover image
        let image_path = mount_point
            .join("apps")
            .join("usbloader_gx")
            .join("images")
            .join(id.as_ref())
            .with_extension("png");

        let image_uri = format!("file://{}", image_path.to_slash_lossy());

        let search_str = (display_title.clone() + id.as_ref()).to_lowercase();

        // Construct the Game object
        Ok(Self {
            path: dir.clone(),
            id,
            display_title,
            size,
            image_uri,
            is_wii,
            search_str,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Game {
    pub path: PathBuf,
    pub id: GameID,
    pub display_title: String,
    pub size: Size,
    pub image_uri: String,
    pub is_wii: bool,
    pub search_str: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct GameID(pub [u8; 6]);

impl GameID {
    pub fn get_region_display(&self) -> &'static str {
        match &self.0[3] {
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
        match &self.0[3] {
            b'E' | b'N' => "US",
            b'J' => "JA",
            b'K' | b'Q' | b'T' => "KO",
            b'R' => "RU",
            b'W' => "ZH",
            _ => "EN",
        }
    }
}

impl From<&str> for GameID {
    fn from(id: &str) -> Self {
        let mut id_bytes = [0u8; 6];
        let bytes = id.as_bytes();
        let len = bytes.len().min(6);
        id_bytes[..len].copy_from_slice(&bytes[..len]);
        GameID(id_bytes)
    }
}

impl AsRef<str> for GameID {
    fn as_ref(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(&self.0) }
    }
}

pub fn sort(games: &mut Vec<Game>, sort_by: &SortBy) {
    match sort_by {
        SortBy::NameAscending => {
            games.sort_by(|a, b| a.display_title.cmp(&b.display_title));
        }
        SortBy::NameDescending => {
            games.sort_by(|a, b| b.display_title.cmp(&a.display_title));
        }
        SortBy::SizeAscending => {
            games.sort_by(|a, b| a.size.cmp(&b.size));
        }
        SortBy::SizeDescending => {
            games.sort_by(|a, b| b.size.cmp(&a.size));
        }
    }
}
