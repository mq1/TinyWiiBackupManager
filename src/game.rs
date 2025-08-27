// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::SUPPORTED_INPUT_EXTENSIONS;
use crate::titles::GAME_TITLES;
use crate::util::{gametdb::GameTDB, redump, regions::REGION_TO_LANG};
use anyhow::{Context, Result, bail};
use filetime::FileTime;
use nod::read::{DiscMeta, DiscOptions, DiscReader};
use std::path::{Path, PathBuf};
use std::{fs, io};

/// Represents the console type for a game
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ConsoleType {
    Wii,
    GameCube,
}

/// Represents the state of disc metadata loading
#[derive(Clone, Debug)]
enum DiscMetaState {
    /// Metadata has not been loaded yet
    NotLoaded,
    /// Metadata was loaded successfully
    Loaded(DiscMeta),
    /// Metadata loading was attempted but failed
    Failed,
}

/// Calculated hashes from a full verification
#[derive(Clone, Debug)]
pub struct CalculatedHashes {
    pub crc32: Option<u32>,
    pub sha1: Option<[u8; 20]>,
    pub xxh64: Option<u64>,
}

impl CalculatedHashes {
    /// Convert these calculated hashes into a verification status by checking against the Redump database
    pub fn into_verification_status(self) -> VerificationStatus {
        if let Some(crc32) = self.crc32 {
            if let Some(redump_entry) = redump::find_by_crc32(crc32) {
                // Check if SHA1 also matches
                if self.sha1.is_some_and(|sha| sha == redump_entry.sha1) {
                    VerificationStatus::FullyVerified(redump_entry, self)
                } else {
                    VerificationStatus::Failed(
                        format!(
                            "Partial match: {} (CRC32 matches, file differs - likely NKit v1)",
                            redump_entry.name
                        ),
                        Some(self),
                    )
                }
            } else {
                VerificationStatus::Failed("Not in Redump database".to_string(), Some(self))
            }
        } else {
            VerificationStatus::Failed("Failed to calculate hashes".to_string(), None)
        }
    }
}

/// Represents the verification status of a game
#[derive(Clone, Debug)]
pub enum VerificationStatus {
    /// Not yet verified
    NotVerified,
    /// Has embedded hashes that match Redump (quick check)
    EmbeddedMatch(redump::GameResult),
    /// Fully verified and matches Redump
    FullyVerified(redump::GameResult, CalculatedHashes),
    /// Verification failed or doesn't match Redump
    Failed(String, Option<CalculatedHashes>),
}

/// Tracks verification status along with the file state when it was verified
#[derive(Clone, Debug)]
pub struct VerificationData {
    pub status: VerificationStatus,
    /// Size of directory when verified
    pub verified_size: u64,
    /// Latest modification time when verified
    pub verified_mtime: FileTime,
}

/// Represents a single game, containing its metadata and file system information.
#[derive(Clone, Debug)]
pub struct Game {
    pub id: String,
    pub console: ConsoleType,
    pub title: String,
    pub display_title: String,
    pub path: PathBuf,
    pub language: String,
    pub info_url: String,
    pub image_url: String,
    /// State of disc metadata loading
    disc_meta: DiscMetaState,
    pub size: u64,
    /// Verification data (status + file state when verified)
    pub verification_data: Option<VerificationData>,
}

impl Game {
    /// Creates a new `Game` instance by parsing metadata from a given file path.
    ///
    /// The path is expected to be a directory containing the game files, with a name
    /// format like "My Game Title [GAMEID]".
    pub fn from_path(path: PathBuf, console: ConsoleType, base_dir: Option<&Path>) -> Result<Self> {
        let (id, title) = Self::parse_filename(&path)?;

        // Try GameTDB first, then fall back to built-in titles, then the parsed title
        let display_title = if let Some(base_dir) = base_dir
            && let Some(gametdb) = GameTDB::load_from_base_dir(base_dir)
            && let Some(gametdb_title) = gametdb.get_title(&id, None)
        {
            gametdb_title
        } else {
            GAME_TITLES.get(&id).copied().unwrap_or(&title).to_string()
        };

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

        let size = fs_extra::dir::get_size(&path)
            .with_context(|| format!("Failed to get size of dir: {}", path.display()))?;

        Ok(Self {
            id,
            console,
            title,
            display_title,
            path,
            language,
            info_url,
            image_url,
            disc_meta: DiscMetaState::NotLoaded, // Will be loaded on demand
            size,
            verification_data: None,
        })
    }

    /// Parses the game ID and title from the directory name.
    /// Assumes a format like "Game Title [ID]".
    fn parse_filename(path: &Path) -> Result<(String, String)> {
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

        Ok((id, title))
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

        if res == rfd::MessageDialogResult::No {
            return Ok(());
        }

        fs::remove_dir_all(&self.path)
            .with_context(|| format!("Failed to remove game: {}", self.path.display()))
    }

    /// Lazily loads disc metadata when needed
    pub fn load_disc_meta(&mut self) -> Option<&DiscMeta> {
        // Check if we need to load the metadata
        let should_load = matches!(self.disc_meta, DiscMetaState::NotLoaded);

        if should_load {
            // Find the disc file first
            let disc_file_path = find_disc_image_file(&self.path);

            let meta = disc_file_path.as_ref().and_then(|disc_file_path| {
                DiscReader::new(disc_file_path, &DiscOptions::default())
                    .ok()
                    .map(|d| d.meta())
            });

            self.disc_meta = match meta {
                Some(meta) => {
                    // Check embedded hashes against Redump if not already verified
                    if self.verification_data.is_none() {
                        self.check_embedded_hashes(&meta);
                    }
                    DiscMetaState::Loaded(meta)
                }
                None => DiscMetaState::Failed,
            };
        }

        // Return a reference to the metadata if available
        match &self.disc_meta {
            DiscMetaState::Loaded(meta) => Some(meta),
            _ => None,
        }
    }

    /// Check embedded hashes against Redump database
    fn check_embedded_hashes(&mut self, meta: &DiscMeta) {
        if let Some(crc32) = meta.crc32 {
            if let Some(redump_entry) = redump::find_by_crc32(crc32) {
                // Check if other hashes match too
                if meta.sha1.is_none_or(|sha| sha == redump_entry.sha1) {
                    self.set_verification_status(VerificationStatus::EmbeddedMatch(redump_entry));
                } else {
                    // CRC32 matches but SHA1 doesn't - likely an NKit v1 scrubbed disc
                    self.set_verification_status(VerificationStatus::Failed(
                        format!(
                            "Partial match: {} (CRC32 matches, file differs - likely NKit v1)",
                            redump_entry.name
                        ),
                        None,
                    ));
                }
            } else {
                // Has embedded hashes but not in Redump database
                self.set_verification_status(VerificationStatus::Failed(
                    "Embedded hashes not found in Redump database".to_string(),
                    None,
                ));
            }
        }
    }

    /// Get the disc file path for this game
    pub fn get_disc_file_path(&self) -> Option<PathBuf> {
        find_disc_image_file(&self.path)
    }

    /// Get the latest modification time of any file in the game directory
    pub fn get_latest_mtime(&self) -> Option<FileTime> {
        get_latest_mtime(&self.path)
    }

    /// Get the current verification status
    pub fn get_verification_status(&self) -> &VerificationStatus {
        if let Some(ref data) = self.verification_data {
            &data.status
        } else {
            &VerificationStatus::NotVerified
        }
    }

    /// Update verification status with current file state
    pub fn set_verification_status(&mut self, status: VerificationStatus) {
        let mtime = self.get_latest_mtime().unwrap_or_else(FileTime::zero);
        self.verification_data = Some(VerificationData {
            status,
            verified_size: self.size,
            verified_mtime: mtime,
        });
    }
}

/// Finds the first valid disc image file within a given game directory.
/// Returns `None` if no disc image is found.
fn find_disc_image_file(game_dir: &Path) -> Option<PathBuf> {
    let entries = fs::read_dir(game_dir).ok()?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file()
            && let Some(ext) = path.extension()
            && let Some(ext_str) = ext.to_str()
            && SUPPORTED_INPUT_EXTENSIONS.contains(&ext_str)
        {
            return Some(path);
        }
    }

    None
}

/// Get the latest modification time of any file in a directory (recursively)
fn get_latest_mtime(dir: &Path) -> Option<FileTime> {
    let mut latest = FileTime::zero();

    fn visit_dir(dir: &Path, latest: &mut FileTime) -> io::Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            let mtime = FileTime::from_last_modification_time(&metadata);

            if mtime > *latest {
                *latest = mtime;
            }

            if entry.path().is_dir() {
                visit_dir(&entry.path(), latest)?;
            }
        }
        Ok(())
    }

    visit_dir(dir, &mut latest).ok()?;

    if latest == FileTime::zero() {
        None
    } else {
        Some(latest)
    }
}
