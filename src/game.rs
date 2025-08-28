// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::INPUT_EXTENSIONS_PATTERN;
use crate::region::Region;
use anyhow::{Context, Result, anyhow};
use glob::glob;
use lazy_regex::{Lazy, Regex, lazy_regex};
use nod::read::{DiscMeta, DiscOptions, DiscReader};
use std::fs;
use std::path::PathBuf;

include!(concat!(env!("OUT_DIR"), "/titles.rs"));

static GAME_DIR_RE: Lazy<Regex> = lazy_regex!(r"^(.*)\[(.*)\]$");

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
    pub id: String,
    pub console: ConsoleType,
    pub title: String,
    pub display_title: String,
    pub path: PathBuf,
    pub region: Region,
    pub info_url: String,
    pub image_url: String,
    pub size: u64,
    disc_meta: DiscMetaState,
}

impl Game {
    /// Creates a new `Game` instance by parsing metadata from a given file path.
    ///
    /// The path is expected to be a directory containing the game files, with a name
    /// format like "My Game Title [GAMEID]".
    pub fn from_path(path: PathBuf, console: ConsoleType) -> Result<Self> {
        let dir_name = path.file_name().unwrap().to_string_lossy();

        let caps = GAME_DIR_RE
            .captures(&dir_name)
            .ok_or_else(|| anyhow!("Invalid game directory name: {dir_name}"))?;

        let id = caps.get(2).unwrap().as_str().to_string();
        let title = caps.get(1).unwrap().as_str().to_string();

        // Use the title from the GameTDB database if available, otherwise, fall back to the
        // parsed title from the file name.
        let display_title = GAME_TITLES.get(&id).copied().unwrap_or(&title).to_string();

        let region = Region::from_id(&id);
        let lang = region.to_lang();

        let info_url = format!("https://www.gametdb.com/Wii/{id}");
        let image_url = format!("https://art.gametdb.com/wii/cover3D/{lang}/{id}.png");

        let size = fs_extra::dir::get_size(&path)
            .with_context(|| format!("Failed to get size of dir: {}", path.display()))?;

        Ok(Self {
            id,
            console,
            title,
            display_title,
            path,
            region,
            info_url,
            image_url,
            size,
            disc_meta: DiscMetaState::NotLoaded,
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
