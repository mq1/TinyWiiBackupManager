// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::app::SUPPORTED_INPUT_EXTENSIONS;
use crate::titles::GAME_TITLES;
use anyhow::{Context, Result, bail};
use nod::read::{DiscMeta, DiscOptions, DiscReader};
use phf::phf_map;
use std::fs;
use std::path::{Path, PathBuf};

// A static map to convert the region character from a game's ID to a language code
// used by the GameTDB API for fetching cover art.
static REGION_TO_LANG: phf::Map<char, &'static str> = phf_map! {
    'A' => "EN", // System Wii Channels (i.e. Mii Channel)
    'B' => "EN", // Ufouria: The Saga (NA)
    'D' => "DE", // Germany
    'E' => "US", // USA
    'F' => "FR", // France
    'H' => "NL", // Netherlands
    'I' => "IT", // Italy
    'J' => "JA", // Japan
    'K' => "KO", // Korea
    'L' => "EN", // Japanese import to Europe, Australia and other PAL regions
    'M' => "EN", // American import to Europe, Australia and other PAL regions
    'N' => "US", // Japanese import to USA and other NTSC regions
    'P' => "EN", // Europe and other PAL regions such as Australia
    'Q' => "KO", // Japanese Virtual Console import to Korea
    'R' => "RU", // Russia
    'S' => "ES", // Spain
    'T' => "KO", // American Virtual Console import to Korea
    'U' => "EN", // Australia / Europe alternate languages
    'V' => "EN", // Scandinavia
    'W' => "ZH", // Republic of China (Taiwan) / Hong Kong / Macau
    'X' => "EN", // Europe alternate languages / US special releases
    'Y' => "EN", // Europe alternate languages / US special releases
    'Z' => "EN", // Europe alternate languages / US special releases
};

/// Represents a single game, containing its metadata and file system information.
#[derive(Clone)]
pub struct Game {
    pub id: String,
    pub is_gc: bool,
    pub title: String,
    pub display_title: String,
    pub path: PathBuf,
    pub language: String,
    pub info_url: String,
    pub image_url: String,
    pub disc_meta: Option<DiscMeta>,
    pub size: u64,
}

struct GameIdInfo {
    id: String,
    title: String,
}

impl Game {
    /// Creates a new `Game` instance by parsing metadata from a given file path.
    ///
    /// The path is expected to be a directory containing the game files, with a name
    /// format like "My Game Title [GAMEID]".
    pub fn from_path(path: PathBuf) -> Result<Self> {
        let GameIdInfo { id, title } = Self::parse_filename(&path)?;

        let is_gc = id.starts_with('G');

        // Use the title from the GameTDB database if available, otherwise, fall back to the
        // parsed title from the file name.
        let display_title = GAME_TITLES.get(&id).copied().unwrap_or(&title).to_string();

        // The 4th character in a Wii/GameCube ID represents the region.
        // We use this to determine the language for fetching the correct cover art.
        let region_code = id
            .chars()
            .nth(3)
            .context("Game ID is missing a region code")?;
        let language = REGION_TO_LANG
            .get(&region_code)
            .copied()
            .unwrap_or("EN")
            .to_string();

        let info_url = format!("https://www.gametdb.com/Wii/{id}");
        let image_url = format!("https://art.gametdb.com/wii/cover3D/{language}/{id}.png");

        let disc_meta = read_disc_metadata(&path);
        let size = fs_extra::dir::get_size(&path)
            .with_context(|| format!("Failed to get size of dir: {}", path.display()))?;

        Ok(Self {
            id,
            is_gc,
            title,
            display_title,
            path,
            language,
            info_url,
            image_url,
            disc_meta,
            size,
        })
    }

    /// Parses the game ID and title from the directory name.
    /// Assumes a format like "Game Title [ID]".
    fn parse_filename(path: &Path) -> Result<GameIdInfo> {
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .context("Invalid file name")?;

        // Find the last pair of square brackets to extract the ID
        let Some(id_start) = file_name.rfind('[') else {
            bail!("Could not find '[' in file name: '{}'", file_name);
        };

        let Some(id_end) = file_name.rfind(']') else {
            bail!("Could not find ']' in file name: '{}'", file_name);
        };

        if id_start >= id_end {
            bail!("Invalid ID format in file name: '{}'", file_name);
        }

        let id = file_name[id_start + 1..id_end].to_string();
        let title = file_name[..id_start].trim().to_string();

        Ok(GameIdInfo { id, title })
    }

    /// Prompts the user for confirmation and then permanently deletes the game's directory.
    pub fn remove(&self) -> Result<()> {
        let res = rfd::MessageDialog::new()
            .set_title("Remove Game")
            .set_description(&format!(
                "Are you sure you want to remove {}?",
                self.display_title
            ))
            .set_buttons(rfd::MessageButtons::YesNo)
            .show();

        if res == rfd::MessageDialogResult::No {
            return Ok(());
        }

        fs::remove_dir_all(&self.path)
            .with_context(|| format!("Failed to remove game: {}", self.path.display()))
    }
}

/// Finds the first valid disc image file within a given game directory.
fn find_disc_image_file(game_dir: &Path) -> Option<PathBuf> {
    fs::read_dir(game_dir)
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .find(|path| {
            path.is_file()
                && path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| SUPPORTED_INPUT_EXTENSIONS.contains(&ext))
                    .unwrap_or(false)
        })
}

/// Reads disc metadata from the first disc image file found in the game directory.
/// Returns `None` if no disc image is found or if the metadata cannot be read.
fn read_disc_metadata(game_dir: &Path) -> Option<DiscMeta> {
    let disc_file = find_disc_image_file(game_dir)?;
    DiscReader::new(&disc_file, &DiscOptions::default())
        .ok()
        .map(|d| d.meta())
}
