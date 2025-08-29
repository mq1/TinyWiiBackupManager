// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::SUPPORTED_INPUT_EXTENSIONS;
use crate::base_dir::BaseDir;
use anyhow::{Context, Result, anyhow};
use lazy_regex::{Lazy, Regex, lazy_regex};
use nod::read::{DiscMeta, DiscOptions, DiscReader};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::OnceLock;
use strum::{AsRefStr, Display};

include!(concat!(env!("OUT_DIR"), "/wiitdb_data.rs"));

static GAME_DIR_RE: Lazy<Regex> = lazy_regex!(r"^(.*)\[(.*)\]$");

#[rustfmt::skip]
#[derive(Debug, Clone, Copy, AsRefStr, Display)]
pub enum Region { NtscJ, NtscU, NtscK, NtscT, Pal, PalR }

#[rustfmt::skip]
#[derive(Debug, Clone, Copy, AsRefStr, Display)]
pub enum Language { EN, FR, DE, ES, IT, JA, NL, SE, DK, NO, KO, PT, ZHTW, ZHCN, FI, TR, GR, RU }

/// Data from WiiTDB XML
#[derive(Debug, Clone, Copy)]
pub struct GameInfo {
    pub name: &'static str,
    pub region: Region,
    pub languages: &'static [Language],
    pub crc_list: &'static [u32],
}

/// Represents the console type for a game
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ConsoleType {
    Wii,
    GameCube,
}

/// Represents a single game, containing its metadata and file system information.
#[derive(Clone)]
pub struct Game {
    pub id: u64,
    pub id_str: String,
    pub title: String,
    pub path: PathBuf,
    pub size: u64,
    pub console: ConsoleType,
    pub info: Option<GameInfo>,
    pub display_title: String,
    pub info_url: String,
    disc_meta: Arc<OnceLock<Result<DiscMeta>>>,
}

/// Converts a string slice (up to 8 chars) into a u64.
///
/// It effectively treats the string's bytes as a big-endian integer.
/// For example, "ABCD" becomes 0x41424344.
fn game_id_to_u64(id: &str) -> u64 {
    id.bytes().fold(0, |acc, byte| (acc << 8) | u64::from(byte))
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

        let id = game_id_to_u64(id_str);

        let info = GAMES.get(&id).cloned().cloned();

        let size = fs_extra::dir::get_size(&path)
            .with_context(|| format!("Failed to get size of dir: {}", path.display()))?;

        let display_title = info.as_ref().map(|i| i.name).unwrap_or(title);

        let info_url = format!("https://www.gametdb.com/Wii/{id_str}");

        Ok(Self {
            id,
            id_str: id_str.to_string(),
            console,
            title: title.to_string(),
            path: path.clone(),
            size,
            info,
            // Initialize with a placeholder that will be filled on first access
            disc_meta: Arc::new(OnceLock::new()),
            display_title: display_title.to_string(),
            info_url,
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

    fn find_disc_image_file(&self) -> Result<PathBuf> {
        // Read the directory entries, returning an error if it fails
        let entries = fs::read_dir(&self.path)
            .with_context(|| format!("Failed to read directory: {}", self.path.display()))?;

        // Iterate over the directory entries, looking for the first match
        for entry in entries {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();

            // Skip if not a file
            if !path.is_file() {
                continue;
            }

            // Check if the file's extension is in the supported list
            if let Some(ext) = path.extension()
                && let Some(ext_str) = ext.to_str()
                && SUPPORTED_INPUT_EXTENSIONS.contains(&ext_str)
            {
                return Ok(path);
            }
        }

        Err(anyhow!(
            "No supported disc image file found in: {}",
            self.path.display()
        ))
    }

    /// Lazily loads disc metadata when needed.
    ///
    /// Returns:
    /// - `Ok(&DiscMeta)` if metadata was successfully loaded.
    /// - `Err(&anyhow::Error)` if no disc image file was found or an error occurred during reading.
    pub fn load_disc_meta(&self) -> &Result<DiscMeta> {
        // Use get_or_init with a closure that computes the value if not already set
        self.disc_meta.get_or_init(|| {
            let file = self.find_disc_image_file()?;
            let reader = DiscReader::new(&file, &DiscOptions::default())?;
            Ok(reader.meta())
        })
    }

    pub fn download_cover(&self, base_dir: BaseDir) -> Result<()> {
        let language = self
            .info
            .as_ref()
            .and_then(|i| i.languages.first())
            .unwrap_or(&Language::EN);

        let id = &self.id_str;

        let url = format!("https://art.gametdb.com/wii/cover3D/{language}/{id}.png");
        base_dir.download_file(&url, "apps/usbloader_gx/images", &format!("{id}.png"))
    }
}
