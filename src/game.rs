// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::base_dir::BaseDir;
use crate::messages::BackgroundMessage;
use crate::settings::ArchiveFormat;
use crate::task::TaskProcessor;
use crate::util;
use anyhow::{Context, Result};
use nod::read::DiscMeta;
use path_slash::PathBufExt;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use strum::{AsRefStr, Display};

include!(concat!(env!("OUT_DIR"), "/metadata.rs"));

const WIITDB_BYTES: &'static [u8] = include_bytes!(concat!(env!("OUT_DIR"), "/wiitdb.bin.zst"));

static WIITDB: LazyLock<HashMap<[u8; 6], GameInfo>> = LazyLock::new(|| {
    let mut buffer = [0; DECOMPRESSED_SIZE];
    zstd::bulk::decompress_to_buffer(WIITDB_BYTES, &mut buffer).expect("failed to decompress");
    postcard::from_bytes(&buffer).expect("failed to deserialize")
});

#[rustfmt::skip]
#[derive(Deserialize, Debug, Clone, Copy, AsRefStr, Display)]
pub enum Language { En, Fr, De, Es, It, Ja, Nl, Se, Dk, No, Ko, Pt, Zhtw, Zhcn, Fi, Tr, Gr, Ru }

#[rustfmt::skip]
#[derive(Deserialize, Debug, Clone, Copy, AsRefStr, Display)]
pub enum Region { NtscJ, NtscU, NtscK, NtscT, Pal, PalR }

fn get_locale(region: Region) -> &'static str {
    match region {
        Region::NtscJ => "JA",
        Region::NtscU => "US",
        Region::NtscK => "KO",
        Region::NtscT => "ZH",
        Region::Pal => "EN",
        Region::PalR => "RU",
    }
}

/// Data from WiiTDB XML
#[derive(Deserialize, Debug, Clone)]
pub struct GameInfo {
    pub name: String,
    pub region: Region,
    pub languages: Vec<Language>,
    pub crc_list: Vec<u32>,
}

/// Represents the console type for a game
#[derive(Clone, Copy, Debug, PartialEq, AsRefStr)]
pub enum ConsoleType {
    #[strum(serialize = "🎾 Wii")]
    Wii,
    #[strum(serialize = "🎲 GC")]
    GameCube,
}

/// Represents a single game, containing its metadata and file system information.
#[derive(Debug, Clone)]
pub struct Game {
    pub id: [u8; 6],
    pub id_str: String,
    pub title: String,
    pub path: PathBuf,
    pub size: u64,
    pub console: ConsoleType,
    pub info: Option<GameInfo>,
    pub display_title: String,
    pub info_url: String,
    pub info_opened: bool,
    pub is_verified: Option<bool>,
    pub is_corrupt: Option<bool>,
    pub disc_meta: DiscMeta,
}

impl Game {
    /// Creates a new `Game` instance by parsing metadata from a given file path.
    ///
    /// The path is expected to be a directory containing the game files, with a name
    /// format like "My Game Title [GAMEID]".
    pub fn from_path(path: impl AsRef<Path>, console: ConsoleType) -> Result<Self> {
        let (header, meta) = util::meta::read_header_and_meta(&path)?;

        let info = WIITDB.get(&header.game_id);

        let display_title = info
            .and_then(|info| Some(info.name.clone()))
            .unwrap_or(header.game_title_str().to_string());

        // Verify the game using the embedded CRC from the disc metadata
        // This verifies if the game is a good redump dump
        let is_verified = if let Some(crc32) = meta.crc32
            && let Some(info) = info
        {
            Some(info.crc_list.contains(&crc32))
        } else {
            None
        };

        // Check if the game is corrupt
        // If the metadata is not found, cross-reference Redump
        let is_corrupt = if let Some(finalization) = util::checksum::cache_get(header.game_id)
            && let Some(crc32) = finalization.crc32
            && let Some(info) = info
        {
            Some(!info.crc_list.contains(&crc32))
        } else {
            None
        };

        Ok(Self {
            id: header.game_id,
            id_str: header.game_id_str().to_string(),
            title: header.game_title_str().to_string(),
            path: path.as_ref().to_path_buf(),
            size: fs_extra::dir::get_size(path)?,
            console,
            info: info.cloned(),
            display_title: display_title.to_string(),
            info_url: format!("https://www.gametdb.com/Wii/{}", header.game_title_str()),
            info_opened: false,
            is_verified,
            is_corrupt,
            disc_meta: meta,
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

    pub fn get_local_cover_uri(&self, images_dir: impl AsRef<Path>) -> String {
        let path = images_dir.as_ref().to_owned();
        let file = path.join(&self.id_str).with_extension("png");

        format!("file://{}", file.to_slash_lossy())
    }

    pub fn download_cover(&self, base_dir: &BaseDir) -> Result<bool> {
        let locale = if let Some(info) = &self.info {
            get_locale(info.region)
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

        let locale = if let Some(info) = &self.info {
            get_locale(info.region)
        } else {
            "EN"
        };

        // Cover3D
        let cover3d = self.download_cover(&base_dir)?;

        // Cover2D
        let url = format!("https://art.gametdb.com/wii/cover/{locale}/{id}.png");
        let cover =
            base_dir.download_file(&url, "apps/usbloader_gx/images/2D", &format!("{id}.png"))?;

        // CoverFull
        let url = format!("https://art.gametdb.com/wii/coverfull/{locale}/{id}.png");
        let full =
            base_dir.download_file(&url, "apps/usbloader_gx/images/full", &format!("{id}.png"))?;

        // CoverFull for WiiFlow lite
        let full_path = base_dir.cover_dir().join("full").join(format!("{id}.png"));
        let wiiflow_cover_dir = base_dir.wiiflow_cover_dir();
        fs::create_dir_all(&wiiflow_cover_dir)?;
        let dest = wiiflow_cover_dir.join(format!("{id}.png"));
        if full_path.exists() && !dest.exists() {
            fs::copy(&full_path, &dest)?;
        }

        // Disc
        let url = format!("https://art.gametdb.com/wii/disc/{locale}/{id}.png");
        let disc =
            base_dir.download_file(&url, "apps/usbloader_gx/images/disc", &format!("{id}.png"))?;

        Ok(cover3d || cover || full || disc)
    }

    pub fn toggle_info(&mut self) {
        self.info_opened = !self.info_opened;
    }

    pub fn spawn_integrity_check_task(&self, task_processor: &TaskProcessor) {
        let game = self.clone();

        task_processor.spawn_task(move |ui_sender| {
            let display_title = &game.display_title;

            let already_cached = util::checksum::all(&game, |progress, total| {
                let msg = format!(
                    "🔎  {:02.0}%  {}",
                    progress as f32 / total as f32 * 100.0,
                    &display_title
                );
                let _ = ui_sender.send(BackgroundMessage::UpdateStatus(msg));
            })?;

            let msg = match already_cached {
                true => format!("{display_title} hashes are already cached"),
                false => format!("{display_title} integrity check completed and cached"),
            };

            let _ = ui_sender.send(BackgroundMessage::Info(msg));

            // Refresh the game list
            let _ = ui_sender.send(BackgroundMessage::DirectoryChanged);

            Ok(())
        });
    }

    pub fn spawn_archive_task(
        &self,
        task_processor: &TaskProcessor,
        archive_format: ArchiveFormat,
    ) {
        let output_dir = rfd::FileDialog::new()
            .set_title("Select Output Directory")
            .pick_folder();

        if let Some(output_dir) = output_dir {
            let game = self.clone();

            task_processor.spawn_task(move |ui_sender| {
                let display_title = &game.display_title;

                let out_path =
                    util::archive::game(&game, &output_dir, archive_format, |progress, total| {
                        let msg = format!(
                            "🖴➡📄  {:02.0}%  {}... ",
                            progress as f32 / total as f32 * 100.0,
                            display_title,
                        );
                        let _ = ui_sender.send(BackgroundMessage::UpdateStatus(msg));
                    })?;

                let _ = ui_sender.send(BackgroundMessage::Info(format!(
                    "{} archived to {}",
                    display_title,
                    out_path.display()
                )));

                Ok(())
            });
        }
    }
}
