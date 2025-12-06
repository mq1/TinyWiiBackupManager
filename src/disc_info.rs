// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::extensions::SUPPORTED_INPUT_EXTENSIONS;
use crate::games::GameID;
use crate::{convert::get_disc_opts, overflow_reader::get_main_file};
use anyhow::{Result, anyhow, bail};
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
        let main_disc_path = get_main_file(game_dir).ok_or(anyhow!("No disc found"))?;
        Self::from_main_file(main_disc_path)
    }

    pub fn from_main_file(main_disc_path: PathBuf) -> Result<DiscInfo> {
        if main_disc_path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ["zip", "ZIP"].contains(&ext))
        {
            let file_reader = BufReader::new(File::open(&main_disc_path)?);
            let mut archive = ZipArchive::new(file_reader)?;

            // for now, only the first file is read
            let mut disc_file = archive.by_index(0)?;
            let title = disc_file.name().to_string();

            if !SUPPORTED_INPUT_EXTENSIONS
                .iter()
                .any(|ext| title.ends_with(ext))
            {
                bail!(
                    "{} Unsupported disc extension: {}",
                    egui_phosphor::regular::DISC,
                    &title
                );
            }

            let format = DiscReader::detect(&mut disc_file)?.ok_or(anyhow!(
                "{} Failed to detect disc format",
                egui_phosphor::regular::DISC
            ))?;

            Ok(Self {
                main_disc_path,
                format,
                title,
                ..Default::default()
            })
        } else {
            let mut disc_info = Self::from_file(File::open(&main_disc_path)?)?;
            disc_info.main_disc_path = main_disc_path;
            Ok(disc_info)
        }
    }

    pub fn from_file(file: File) -> Result<DiscInfo> {
        let disc =
            DiscReader::new_from_non_cloneable_read(file.try_clone().unwrap(), &get_disc_opts())?;

        let header = disc.header();
        let meta = disc.meta();

        let is_worth_stripping = is_worth_stripping(file);

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
pub fn is_worth_stripping(file: File) -> bool {
    if let Ok(disc) = DiscReader::new_from_non_cloneable_read(file, &get_disc_opts())
        && disc.meta().format == Format::Wbfs
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
