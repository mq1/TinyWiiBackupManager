// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{convert::get_disc_opts, games::GameID, overflow_reader::get_main_file};
use anyhow::{Result, anyhow};
use nod::{
    common::{Compression, Format},
    read::DiscReader,
};
use size::Size;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default)]
pub struct DiscInfo {
    pub game_dir: PathBuf,
    pub game_id: GameID,
    pub game_title: String,
    pub is_gamecube: bool,
    pub is_wii: bool,
    pub disc_num: u8,
    pub disc_version: u8,
    pub format: Format,
    pub compression: Compression,
    pub block_size: Size,
    pub decrypted: bool,
    pub needs_hash_recovery: bool,
    pub lossless: bool,
    pub disc_size: Size,
    pub crc32: String,
    pub md5: String,
    pub sha1: String,
    pub xxh64: String,
}

impl DiscInfo {
    pub fn from_game_dir(game_dir: &Path) -> Result<DiscInfo> {
        let path = get_main_file(game_dir).ok_or(anyhow!("No disc found"))?;

        let disc = DiscReader::new(&path, &get_disc_opts())?;

        // Header
        let header = disc.header();
        let game_id = GameID(header.game_id);
        let game_title = header.game_title_str().to_string();
        let is_gamecube = header.is_gamecube();
        let is_wii = header.is_wii();
        let disc_num = header.disc_num;
        let disc_version = header.disc_version;

        // Meta
        let meta = disc.meta();
        let format = meta.format;
        let compression = meta.compression;
        let block_size = Size::from_bytes(meta.block_size.unwrap_or(0));

        let decrypted = meta.decrypted;
        let needs_hash_recovery = meta.needs_hash_recovery;
        let lossless = meta.lossless;
        let disc_size = Size::from_bytes(meta.disc_size.unwrap_or(0));

        let crc32 = meta
            .crc32
            .map(|hash| format!("{:08x}", hash).to_string())
            .unwrap_or("Unknown".to_string());
        let md5 = meta
            .md5
            .map(|hash| hex::encode(hash).to_string())
            .unwrap_or("Unknown".to_string());
        let sha1 = meta
            .sha1
            .map(|hash| hex::encode(hash).to_string())
            .unwrap_or("Unknown".to_string());
        let xxh64 = meta
            .xxh64
            .map(|hash| format!("{:08x}", hash).to_string())
            .unwrap_or("Unknown".to_string());

        Ok(Self {
            game_dir: game_dir.to_path_buf(),
            game_id,
            game_title,
            is_gamecube,
            is_wii,
            disc_num,
            disc_version,
            format,
            compression,
            block_size,
            decrypted,
            needs_hash_recovery,
            lossless,
            disc_size,
            crc32,
            md5,
            sha1,
            xxh64,
        })
    }
}
