// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::DiscInfo;
use anyhow::{Result, anyhow, bail};
use nod::{
    common::{Format, PartitionKind},
    read::{DiscOptions, DiscReader, PartitionEncryption, PartitionOptions},
};
use size::Size;
use slint::ToSharedString;
use std::{ffi::OsStr, fs, path::Path};

const DISC_OPTS: DiscOptions = DiscOptions {
    partition_encryption: PartitionEncryption::Original,
    preloader_threads: 0,
};

impl DiscInfo {
    pub fn try_from_game_dir(game_dir: &Path) -> Result<Self> {
        if !game_dir.is_dir() {
            bail!("Not a directory");
        }

        let Some(filename) = game_dir.file_name().and_then(OsStr::to_str) else {
            bail!("No file name");
        };

        if filename.starts_with('.') {
            bail!("Hidden directory");
        }

        for entry in fs::read_dir(game_dir)?.filter_map(Result::ok) {
            let disc_path = entry.path();

            if let Ok(disc_info) = Self::try_from_path(&disc_path) {
                return Ok(disc_info);
            }

            eprintln!("Skipping non-disc file: {}", disc_path.display());
        }

        Err(anyhow!("No disc file found in {}", game_dir.display()))
    }

    pub fn try_from_path(disc_path: &Path) -> Result<Self> {
        if !disc_path.is_file() {
            bail!("Not a file");
        }

        let Some(filename) = disc_path.file_name().and_then(OsStr::to_str) else {
            bail!("No file name");
        };

        if filename.starts_with('.') {
            bail!("Hidden file");
        }

        if filename.ends_with(".part1.iso") {
            bail!("Part 1 file");
        }

        let Some(ext) = disc_path.extension() else {
            bail!("No file extension");
        };

        if !ext.eq_ignore_ascii_case("iso")
            && !ext.eq_ignore_ascii_case("wbfs")
            && !ext.eq_ignore_ascii_case("ciso")
        {
            bail!("Unsupported file extension");
        }

        let disc = DiscReader::new(disc_path, &DISC_OPTS)?;
        let is_worth_stripping = is_worth_stripping(&disc);

        let header = disc.header();
        let meta = disc.meta();

        let block_size = match meta.block_size {
            Some(size) => Size::from_bytes(size).to_shared_string(),
            None => "N/A".to_shared_string(),
        };

        let disc_size = match meta.disc_size {
            Some(size) => Size::from_bytes(size).to_shared_string(),
            None => "N/A".to_shared_string(),
        };

        let crc32 = match meta.crc32 {
            Some(crc) => format!("{crc:02x}").to_shared_string(),
            None => "N/A".to_shared_string(),
        };

        let md5 = match meta.md5 {
            Some(md5) => hex::encode(md5).to_shared_string(),
            None => "N/A".to_shared_string(),
        };

        let sha1 = match meta.sha1 {
            Some(sha) => hex::encode(sha).to_shared_string(),
            None => "N/A".to_shared_string(),
        };

        let xxh64 = match meta.xxh64 {
            Some(xxh) => format!("{xxh:02x}").to_shared_string(),
            None => "N/A".to_shared_string(),
        };

        Ok(Self {
            // discheader
            id: header.game_id_str().to_shared_string(),
            title: header.game_title_str().to_shared_string(),
            is_wii: header.is_wii(),
            is_gc: header.is_gamecube(),
            disc_num: header.disc_num.into(),
            disc_version: header.disc_version.into(),

            // discmeta
            format: meta.format.to_shared_string(),
            compression: meta.compression.to_shared_string(),
            block_size,
            decrypted: meta.decrypted,
            needs_hash_recovery: meta.needs_hash_recovery,
            lossless: meta.lossless,
            disc_size,
            crc32,
            md5,
            sha1,
            xxh64,

            // misc
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

        let mut block_buf = vec![0u8; 2 * 1024 * 1024].into_boxed_slice(); // 2 MB
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
