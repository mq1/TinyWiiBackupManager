// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::games::GameID;
use crate::{convert::get_disc_opts, overflow_reader::get_main_file};
use anyhow::{Result, anyhow};
use nod::common::{Compression, Format, PartitionKind};
use nod::read::{DiscReader, PartitionOptions};
use size::Size;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use zip::ZipArchive;

#[derive(Debug, Clone, Default)]
pub struct DiscInfo {
    pub main_disc_path: PathBuf,

    // discheader
    pub id: GameID,
    pub title: String,
    pub is_wii: bool,
    pub is_gc: bool,
    pub disc_num: u8,
    pub disc_version: u8,

    // discmeta
    pub format: Format,
    pub compression: Compression,
    pub block_size: String,
    pub decrypted: bool,
    pub needs_hash_recovery: bool,
    pub lossless: bool,
    pub disc_size: String,
    pub crc32: Option<u32>,
    pub md5: Option<[u8; 16]>,
    pub sha1: Option<[u8; 20]>,
    pub xxh64: Option<u64>,

    // misc
    pub is_worth_stripping: bool,
}

impl DiscInfo {
    pub fn from_game_dir(game_dir: &Path) -> Result<DiscInfo> {
        let path = get_main_file(game_dir).ok_or(anyhow!("No disc found"))?;
        let disc = DiscReader::new(path, &get_disc_opts())?;
        Self::from_disc(&disc)
    }

    pub fn from_path(path: &Path) -> Result<DiscInfo> {
        let disc = DiscReader::new(path, &get_disc_opts())?;
        Self::from_disc(&disc)
    }

    pub fn from_zip_file(zip_file: &Path) -> Result<DiscInfo> {
        let zip_file_reader = BufReader::new(File::open(zip_file)?);
        let mut archive = ZipArchive::new(zip_file_reader)?;
        let disc_file = archive.by_index(0)?;

        let disc_path = disc_file
            .enclosed_name()
            .ok_or(anyhow!("No disc file found in ZIP archive"))?;

        let file_name = disc_path
            .file_name()
            .ok_or(anyhow!("No file name"))?
            .to_str()
            .ok_or(anyhow!("Invalid file name"))?;

        Ok(Self {
            main_disc_path: zip_file.to_path_buf(),
            title: file_name.to_string(),
            ..Self::default()
        })
    }

    pub fn from_disc(disc: &DiscReader) -> Result<DiscInfo> {
        let is_worth_stripping = is_worth_stripping(disc);

        let header = disc.header();
        let meta = disc.meta();

        Ok(Self {
            main_disc_path: PathBuf::new(),

            // discheader
            id: GameID(header.game_id),
            title: header.game_title_str().to_string(),
            is_wii: header.is_wii(),
            is_gc: header.is_gamecube(),
            disc_num: header.disc_num,
            disc_version: header.disc_version,

            // discmeta
            format: meta.format,
            compression: meta.compression,
            block_size: meta
                .block_size
                .map(|bytes| Size::from_bytes(bytes).to_string())
                .unwrap_or_else(|| "N/A".to_string()),
            decrypted: meta.decrypted,
            needs_hash_recovery: meta.needs_hash_recovery,
            lossless: meta.lossless,
            disc_size: meta
                .disc_size
                .map(|bytes| Size::from_bytes(bytes).to_string())
                .unwrap_or_else(|| "N/A".to_string()),
            crc32: meta.crc32,
            md5: meta.md5,
            sha1: meta.sha1,
            xxh64: meta.xxh64,
            is_worth_stripping,
        })
    }
}

// Returns true if the disc is worth stripping
// Currently checks if the update partition is >= 8 MiB
pub fn is_worth_stripping(disc: &DiscReader) -> bool {
    if disc.meta().format == Format::Wbfs
        && let Ok(mut update_reader) =
            disc.open_partition_kind(PartitionKind::Update, &PartitionOptions::default())
    {
        let mut non_empty_blocks = 0u8;

        let mut block_buf = vec![0u8; 0x200000].into_boxed_slice(); // 2 MB
        while update_reader.read_exact(&mut block_buf[..]).is_ok() {
            if block_buf.iter().any(|b| *b != 0) {
                non_empty_blocks += 1;
                if non_empty_blocks > 4 {
                    return true;
                }
            }
        }
    }

    false
}
