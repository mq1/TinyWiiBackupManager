// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::titles::Titles;
use crate::{Console, Game};
use anyhow::Result;
use parking_lot::Mutex;
use size::Size;
use slint::{Image, ToSharedString};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub fn list(mount_point: &Path, titles: &Arc<Mutex<Titles>>) -> Result<Vec<Game>> {
    if mount_point.as_os_str().is_empty() {
        return Ok(vec![]);
    }

    let game_dirs = [mount_point.join("wbfs"), mount_point.join("games")];

    // Create dirs
    game_dirs.iter().try_for_each(fs::create_dir_all)?;

    let titles = titles.lock();

    let games = game_dirs
        .iter()
        .map(fs::read_dir)
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flat_map(|rd| rd.filter_map(Result::ok))
        .filter_map(|entry| Game::from_dir(entry.path(), &titles))
        .collect::<Vec<_>>();

    Ok(games)
}

impl Game {
    pub fn from_dir(dir: PathBuf, titles: &Titles) -> Option<Self> {
        // Ensure the path is a directory and not hidden
        if !dir.is_dir() {
            return None;
        }
        let file_name = dir.file_name()?.to_str()?;
        if file_name.starts_with('.') {
            return None;
        }

        // Extract title and ID from the directory name, e.g., "Game Title [GAMEID]"
        let (title, id_part) = file_name.split_once(" [")?;
        let id = id_part.strip_suffix(']')?;

        // Determine the console from the parent directory ("wbfs" or "games")
        let console = match dir.parent()?.file_name()?.to_str()? {
            "wbfs" => Console::Wii,
            "games" => Console::GameCube,
            _ => Console::Unknown,
        };

        let display_title = titles.get(id).unwrap_or(title);

        // Get the directory size
        let size = Size::from_bytes(fs_extra::dir::get_size(&dir).unwrap_or(0));

        // Construct the path to the game's cover image
        let base = dir.parent()?.parent()?;
        let image_path = base
            .join("apps")
            .join("usbloader_gx")
            .join("images")
            .join(id)
            .with_extension("png");

        let image = if image_path.exists()
            && let Ok(image) = Image::load_from_path(&image_path)
        {
            image
        } else {
            Image::load_from_svg_data(include_bytes!("../mdi/image-frame.svg"))
                .expect("Failed to load default icon")
        };

        let search_str = (display_title.to_string() + id)
            .to_lowercase()
            .to_shared_string();

        // Construct the Game object
        Some(Self {
            path: dir.to_str()?.to_shared_string(),
            id: id.to_shared_string(),
            display_title: display_title.to_shared_string(),
            size: size.to_shared_string(),
            size_mib: (size.bytes() / 1024 / 1024) as i32,
            image,
            console,
            search_str,
        })
    }
}
