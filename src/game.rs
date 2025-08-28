// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::INPUT_EXTENSIONS_PATTERN;
use anyhow::{Context, Result, anyhow};
use glob::glob;
use lazy_regex::{Lazy, Regex, lazy_regex};
use nod::read::{DiscMeta, DiscOptions, DiscReader};
use std::fs;
use std::path::PathBuf;

include!(concat!(env!("OUT_DIR"), "/wiitdb_data.rs"));

static GAME_DIR_RE: Lazy<Regex> = lazy_regex!(r"^(.*)\[(.*)\]$");

/// Data from WiiTDB XML
#[derive(Debug, Clone, Copy)]
pub struct GameInfo {
    pub name: &'static str,
    pub region: &'static str, // TODO: Make this an enum
    pub languages: &'static [&'static str],
}

/// Represents the console type for a game
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ConsoleType {
    Wii,
    GameCube,
}

/// Represents the state of disc metadata loading
#[derive(Clone)]
enum DiscMetaState {
    /// Metadata has not been loaded yet
    NotLoaded,
    /// Metadata was loaded successfully
    Loaded(DiscMeta),
    /// Metadata loading was attempted but failed
    Failed,
}

/// Represents a single game, containing its metadata and file system information.
#[derive(Clone)]
pub struct Game {
    pub id: [char; 6],
    pub title: String,
    pub path: PathBuf,
    pub size: u64,
    pub console: ConsoleType,
    pub info: Option<GameInfo>,
    pub display_title: String,
    pub info_url: String,
    pub image_url: String,
    disc_meta: DiscMetaState,
}

impl Game {
    /// Creates a new `Game` instance by parsing metadata from a given file path.
    ///
    /// The path is expected to be a directory containing the game files, with a name
    /// format like "My Game Title [GAMEID]".
    pub fn from_path(path: PathBuf, console: ConsoleType) -> Result<Self> {
        let dir_name = path
            .file_name()
            .ok_or_else(|| anyhow!("Invalid game directory name: {}", path.display()))?
            .to_string_lossy();

        let caps = GAME_DIR_RE
            .captures(&dir_name)
            .ok_or_else(|| anyhow!("Invalid game directory name: {dir_name}"))?;

        let id_str = caps
            .get(2)
            .ok_or_else(|| anyhow!("Could not find game ID in directory name: {dir_name}"))?
            .as_str();

        let title = caps
            .get(1)
            .ok_or_else(|| anyhow!("Could not find game title in directory name: {dir_name}"))?
            .as_str();

        let id: [char; 6] = std::array::from_fn(|i| id_str.chars().nth(i).unwrap_or('\0'));

        let info = GAMES.get(&id).cloned().cloned();

        let size = fs_extra::dir::get_size(&path)
            .with_context(|| format!("Failed to get size of dir: {}", path.display()))?;

        let display_title = info.as_ref().map(|i| i.name).unwrap_or_else(|| title);

        let language = info
            .as_ref()
            .map(|i| i.languages.first().cloned().unwrap_or("EN"))
            .unwrap_or("EN");

        let info_url = format!("https://www.gametdb.com/Wii/{id_str}");
        let image_url = format!("https://art.gametdb.com/wii/cover3D/{language}/{id_str}.png");

        Ok(Self {
            id,
            console,
            title: title.to_string(),
            path: path.clone(),
            size,
            info,
            disc_meta: DiscMetaState::NotLoaded,
            display_title: display_title.to_string(),
            info_url,
            image_url,
        })
    }

    /// Prompts the user for confirmation and then permanently deletes the game's directory.
    pub fn remove(&self) -> Result<()> {
        let res = rfd::MessageDialog::new()
            .set_title("Remove Game")
            .set_description(format!(
                "Are you sure you want to remove {}?",
                self.display_title
            ))
            .set_buttons(rfd::MessageButtons::YesNo)
            .show();

        if res == rfd::MessageDialogResult::Yes {
            fs::remove_dir_all(&self.path)
                .with_context(|| format!("Failed to remove game: {}", self.path.display()))?;
        }

        Ok(())
    }

    fn find_disc_image_file(&self) -> Option<PathBuf> {
        let binding = self.path.join(INPUT_EXTENSIONS_PATTERN);
        let pattern = binding.to_string_lossy();

        glob(&pattern)
            .ok()
            .and_then(|mut entries| entries.next())
            .map(|entry| entry.unwrap().into())
    }

    /// Lazily loads disc metadata when needed
    pub fn load_disc_meta(&mut self) -> Option<&DiscMeta> {
        // Check if we need to load the metadata
        let should_load = matches!(self.disc_meta, DiscMetaState::NotLoaded);

        if should_load {
            // Find the disc file first
            let disc_file_path = self.find_disc_image_file();

            let meta = disc_file_path.as_ref().and_then(|disc_file_path| {
                DiscReader::new(disc_file_path, &DiscOptions::default())
                    .ok()
                    .map(|d| d.meta())
            });

            self.disc_meta = match meta {
                Some(meta) => DiscMetaState::Loaded(meta),
                None => DiscMetaState::Failed,
            };
        }

        // Return a reference to the metadata if available
        match &self.disc_meta {
            DiscMetaState::Loaded(meta) => Some(meta),
            _ => None,
        }
    }
}
