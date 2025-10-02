// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{Console, Game, config, wiitdb};
use anyhow::{Result, bail};
use size::Size;
use slint::{Image, ToSharedString};
use std::fs;
use std::path::PathBuf;

pub fn list() -> Result<Vec<Game>> {
    let mount_point = config::get().mount_point;
    if mount_point.as_os_str().is_empty() {
        bail!("No base directory set");
    }

    let game_dirs = [mount_point.join("wbfs"), mount_point.join("games")];

    // First, attempt to read all directories. The `collect` will short-circuit
    // and return the first I/O error encountered.
    let mut games = game_dirs
        .into_iter()
        .map(fs::read_dir)
        .collect::<Result<Vec<_>, _>>()? // Propagate any I/O error
        .into_iter()
        .flatten() // Flatten the Vec<ReadDir> into an iterator of DirEntry
        .filter_map(Result::ok)
        .map(|dir| dir.path())
        .filter_map(Game::from_dir)
        .collect::<Vec<_>>();

    games.sort_by(|a, b| a.title.cmp(&b.title));

    Ok(games)
}

impl Game {
    pub fn from_dir(dir: PathBuf) -> Option<Game> {
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
            _ => return None, // Not in a valid console directory
        };

        // Look up game info from the WiiTDB database
        let mut id_bytes = [0u8; 6];
        let bytes = id.as_bytes();
        let len = bytes.len().min(6);
        id_bytes[..len].copy_from_slice(&bytes[..len]);
        let info = wiitdb::lookup(&id_bytes);

        // Use the database title if available, otherwise fall back to the parsed title
        let display_title = info.map_or_else(|| title.to_string(), |info| info.title);

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
        let image = Image::load_from_path(&image_path).unwrap_or_default();

        // Construct the Game object
        Some(Game {
            path: dir.to_str()?.to_shared_string(),
            id: id.to_shared_string(),
            title: title.to_shared_string(),
            display_title: display_title.to_shared_string(),
            size: size.to_shared_string(),
            image,
            console,
        })
    }
}
