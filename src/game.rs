// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::SUPPORTED_INPUT_EXTENSIONS;
use crate::base_dir::BaseDir;
use crate::messages::BackgroundMessage;
use crate::task::TaskProcessor;
use anyhow::{Context, Result, anyhow, bail};
use lazy_regex::{Lazy, Regex, lazy_regex};
use nod::read::{DiscMeta, DiscOptions, DiscReader, PartitionEncryption};
use nod::write::{DiscWriter, FormatOptions, ProcessOptions};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock, RwLock};
use strum::{AsRefStr, Display};

include!(concat!(env!("OUT_DIR"), "/wiitdb_data.rs"));

static GAME_DIR_RE: Lazy<Regex> = lazy_regex!(r"^(.*)\[(.*)\]$");

#[rustfmt::skip]
#[derive(Debug, Clone, Copy, AsRefStr, Display)]
pub enum Region { NtscJ, NtscU, NtscK, NtscT, Pal, PalR }

#[rustfmt::skip]
#[allow(clippy::upper_case_acronyms)]
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

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TwbmMeta {
    crc32: Option<u32>,
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
    pub info_opened: bool,
    disc_meta: Arc<OnceLock<Result<DiscMeta>>>,
    is_verified_cache: Arc<RwLock<Option<bool>>>,
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
            disc_meta: Arc::new(OnceLock::new()),
            display_title: display_title.to_string(),
            info_url,
            info_opened: false,
            is_verified_cache: Arc::new(RwLock::new(None)),
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

    pub fn load_twbm_meta(&self) -> Result<TwbmMeta> {
        let path = self.path.join(".twbm.ron");
        if path.exists() {
            let raw = fs::read_to_string(&path)
                .with_context(|| format!("Failed to read twbm meta file: {}", path.display()))?;
            let meta = ron::from_str(&raw)
                .with_context(|| format!("Failed to parse twbm meta file: {}", path.display()))?;
            Ok(meta)
        } else {
            Err(anyhow!("Twbm meta file not found: {}", path.display()))
        }
    }

    pub fn save_twbm_meta(&self, meta: &TwbmMeta) -> Result<()> {
        let path = self.path.join(".twbm.ron");

        let raw = ron::to_string(meta)
            .with_context(|| format!("Failed to serialize twbm meta file: {}", path.display()))?;

        fs::write(&path, raw)
            .with_context(|| format!("Failed to write twbm meta file: {}", path.display()))
    }

    pub fn download_cover(&self, base_dir: BaseDir) -> Result<bool> {
        // temp fix for NTSC-U
        let locale = if let Some(info) = self.info
            && matches!(info.region, Region::NtscU)
        {
            "US"
        } else {
            "EN"
        };

        let id = &self.id_str;

        let url = format!("https://art.gametdb.com/wii/cover3D/{locale}/{id}.png");
        base_dir.download_file(&url, "apps/usbloader_gx/images", &format!("{id}.png"))
    }

    pub fn toggle_info(&mut self) {
        self.info_opened = !self.info_opened;
    }

    pub fn spawn_verify_task(&self, task_processor: &TaskProcessor) {
        let id = self.id;
        let disc_path = self.find_disc_image_file();
        let display_title = self.display_title.clone();
        let game_clone = self.clone();

        task_processor.spawn_task(move |ui_sender| {
            let disc_path = disc_path?;

            let _ = ui_sender.send(BackgroundMessage::UpdateStatus(format!(
                "Verifying {display_title}..."
            )));

            // Open the disc
            let disc = DiscReader::new(
                &disc_path,
                &DiscOptions {
                    partition_encryption: PartitionEncryption::Original,
                    preloader_threads: 1,
                },
            )?;
            let disc_writer = DiscWriter::new(disc, &FormatOptions::default())?;

            // Process the disc to calculate hashes
            let finalization = disc_writer.process(
                |_, _, _| Ok(()),
                &ProcessOptions {
                    digest_crc32: true,
                    ..Default::default()
                },
            )?;

            // check if the calculated hashes match the expected values
            let game_info = GAMES.get(&id).ok_or(anyhow!("Could not find game"))?;
            let crc32 = finalization
                .crc32
                .ok_or(anyhow!("Could not calculate CRC32"))?;

            // Load the current meta, update the crc32, and save it.
            let mut meta = game_clone.load_twbm_meta().unwrap_or_default();
            meta.crc32 = Some(crc32);
            game_clone.save_twbm_meta(&meta)?;

            // Update the cache to reflect the new state.
            *game_clone.is_verified_cache.write().unwrap() = Some(true);

            if !game_info.crc_list.contains(&crc32) {
                bail!("CRC crc32 does not match");
            }

            let _ = ui_sender.send(BackgroundMessage::Info(format!(
                "{display_title} is verified"
            )));

            Ok(())
        });
    }

    pub fn is_verified(&self) -> Result<bool> {
        // First, check the cache with a read lock.
        let read_guard = self
            .is_verified_cache
            .read()
            .map_err(|e| anyhow!("Failed to acquire read lock: {e}"))?;

        if let Some(verified) = *read_guard {
            return Ok(verified);
        }
        drop(read_guard);

        // If the cache is empty, acquire a write lock.
        let mut write_guard = self
            .is_verified_cache
            .write()
            .map_err(|e| anyhow!("Failed to acquire write lock: {e}"))?;

        // Check again in case another thread populated it while we waited.
        if let Some(verified) = *write_guard {
            return Ok(verified);
        }

        // The cache is empty and we have the lock. Load meta, propagating real errors.
        let meta = self.load_twbm_meta().unwrap_or_default();
        let verified = meta.crc32.is_some();

        // Store the result in the cache and return it.
        *write_guard = Some(verified);
        Ok(verified)
    }
}
