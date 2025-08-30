// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::SUPPORTED_INPUT_EXTENSIONS;
use crate::base_dir::BaseDir;
use crate::messages::BackgroundMessage;
use crate::task::TaskProcessor;
use anyhow::{Context, Result, anyhow, bail};
use nod::read::{DiscMeta, DiscOptions, DiscReader, PartitionEncryption};
use nod::write::{DiscWriter, FormatOptions, ProcessOptions};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use strum::{AsRefStr, Display};

include!(concat!(env!("OUT_DIR"), "/wiitdb_data.rs"));

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

#[derive(Default, Debug, Clone)]
pub struct Hashes {
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
    pub is_verified: bool,
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

        // Find the positions of the brackets to extract the title and ID
        let Some(id_start) = dir_name.rfind('[') else {
            bail!("Could not find '[' in file name: '{}'", dir_name);
        };

        let Some(id_end) = dir_name.rfind(']') else {
            bail!("Could not find ']' in file name: '{}'", dir_name);
        };

        let title = &dir_name[..id_start].trim_end();
        let id_str = &dir_name[id_start + 1..id_end];

        let id = game_id_to_u64(id_str);

        let info = GAMES.get(&id).cloned().cloned();

        let size = fs_extra::dir::get_size(&path)
            .with_context(|| format!("Failed to get size of dir: {}", path.display()))?;

        let display_title = info.as_ref().map(|i| i.name).unwrap_or(title);

        let info_url = format!("https://www.gametdb.com/Wii/{id_str}");

        // Manually parse hashes
        let hashes_str = fs::read_to_string(path.join("hashes.txt")).unwrap_or_default();
        let mut hashes = Hashes::default();
        for line in hashes_str.lines() {
            if let Some((key, value)) = line.split_once('=')
                && key.trim() == "crc32"
            {
                hashes.crc32 = u32::from_str_radix(value.trim(), 16).ok();
            }
        }

        // Verify the game by cross-referencing WiiTDB
        let is_verified = if let Some(info) = info
            && let Some(crc32) = hashes.crc32
        {
            info.crc_list.contains(&crc32)
        } else {
            false
        };

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
            is_verified,
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

    pub fn save_hashes(&self, hashes: &Hashes) -> Result<()> {
        let path = self.path.join("hashes.txt");
        let mut content = String::new();

        if let Some(crc32) = hashes.crc32 {
            content.push_str(&format!("crc32 = {:x}\n", crc32));
        }

        fs::write(&path, content)
            .with_context(|| format!("Failed to write hashes file: {}", path.display()))
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

    /// Returns true if at least one cover was downloaded.
    pub fn download_all_covers(&self, base_dir: BaseDir) -> Result<bool> {
        let id = &self.id_str;

        // temp fix for NTSC-U
        let locale = if let Some(info) = self.info
            && matches!(info.region, Region::NtscU)
        {
            "US"
        } else {
            "EN"
        };

        // Cover3D is already downloaded (automatically)

        // Cover2D
        let url = format!("https://art.gametdb.com/wii/cover/{locale}/{id}.png");
        let cover =
            base_dir.download_file(&url, "apps/usbloader_gx/images/2D", &format!("{id}.png"))?;

        // CoverFull
        let url = format!("https://art.gametdb.com/wii/coverfull/{locale}/{id}.png");
        let full =
            base_dir.download_file(&url, "apps/usbloader_gx/images/full", &format!("{id}.png"))?;

        // Disc
        let url = format!("https://art.gametdb.com/wii/disc/{locale}/{id}.png");
        let disc =
            base_dir.download_file(&url, "apps/usbloader_gx/images/disc", &format!("{id}.png"))?;

        Ok(cover || full || disc)
    }

    pub fn toggle_info(&mut self) {
        self.info_opened = !self.info_opened;
    }

    pub fn spawn_verify_task(
        &self,
        current_index: usize,
        total_files: usize,
        task_processor: &TaskProcessor,
    ) {
        let disc_path = self.find_disc_image_file();
        let display_title = self.display_title.clone();
        let game_clone = self.clone();

        task_processor.spawn_task(move |ui_sender| {
            let disc_path = disc_path?;

            // Open the disc
            let disc = DiscReader::new(
                &disc_path,
                &DiscOptions {
                    partition_encryption: PartitionEncryption::Original,
                    preloader_threads: 1,
                },
            )?;
            let disc_writer = DiscWriter::new(disc, &FormatOptions::default())?;

            let display_title_truncated = display_title.chars().take(20).collect::<String>();

            // Process the disc to calculate hashes
            let finalization = disc_writer.process(
                |_, done, total| {
                    let msg = format!(
                        "🔎 {}... {:02.0}% ({}/{})",
                        display_title_truncated,
                        done as f32 / total as f32 * 100.0,
                        current_index + 1,
                        total_files
                    );
                    let _ = ui_sender.send(BackgroundMessage::UpdateStatus(Some(msg)));

                    Ok(())
                },
                &ProcessOptions {
                    digest_crc32: true,
                    ..Default::default()
                },
            )?;

            let crc32 = finalization
                .crc32
                .ok_or(anyhow!("Failed to calculate CRC32"))?;

            let hashes = Hashes { crc32: Some(crc32) };
            game_clone.save_hashes(&hashes)?;

            let _ = ui_sender.send(BackgroundMessage::Info(format!(
                "{display_title} is verified"
            )));

            // Refresh the game list
            let _ = ui_sender.send(BackgroundMessage::DirectoryChanged);

            Ok(())
        });
    }

    /// Removes the hashes.txt file
    pub fn remove_meta(&self) -> Result<()> {
        let path = self.path.join("hashes.txt");

        if path.exists() {
            fs::remove_file(&path)
                .with_context(|| format!("Failed to remove hashes file: {}", path.display()))?;
        };

        Ok(())
    }
}
