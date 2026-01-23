// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::games::game_id::GameID;
use anyhow::{Result, anyhow, bail};
use nod::common::{Compression, Format, PartitionKind};
use nod::read::{DiscOptions, DiscReader, PartitionEncryption, PartitionOptions};
use size::Size;
use smol::fs;
use smol::stream::StreamExt;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

const DISC_OPTS: DiscOptions = DiscOptions {
    partition_encryption: PartitionEncryption::Original,
    preloader_threads: 0,
};

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone)]
pub struct DiscInfo {
    pub game_dir: PathBuf,
    pub disc_path: PathBuf,

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
    pub block_size: Option<Size>,
    pub decrypted: bool,
    pub needs_hash_recovery: bool,
    pub lossless: bool,
    pub disc_size: Option<Size>,
    pub crc32: Option<u32>,
    pub md5: Option<[u8; 16]>,
    pub sha1: Option<[u8; 20]>,
    pub xxh64: Option<u64>,

    // misc
    pub is_worth_stripping: bool,
}

pub async fn get_main_file(game_dir: &Path) -> Result<PathBuf> {
    let mut entries = fs::read_dir(game_dir).await?;

    while let Some(entry) = entries.try_next().await? {
        let entry_path = entry.path();

        #[allow(clippy::case_sensitive_file_extension_comparisons)]
        if entry_path
            .file_name()
            .and_then(OsStr::to_str)
            .is_some_and(|file_name| {
                file_name.ends_with(".wbfs")
                    || file_name.ends_with(".ciso")
                    || (file_name.ends_with(".iso") && !file_name.ends_with(".part1.iso"))
            })
        {
            return Ok(entry_path);
        }
    }

    Err(anyhow!("No disc file found"))
}

impl DiscInfo {
    pub async fn from_game_dir(game_dir: PathBuf) -> Result<Self> {
        if !game_dir.is_dir() {
            bail!("Not a directory");
        }

        let disc_path = get_main_file(&game_dir).await?;
        Self::from_path(disc_path)
    }

    pub fn from_path(disc_path: PathBuf) -> Result<Self> {
        if !disc_path.is_file() {
            bail!("Not a file");
        }

        let parent_dir = disc_path.parent().ok_or(anyhow!("No parent directory"))?;
        let disc = DiscReader::new(&disc_path, &DISC_OPTS)?;
        let is_worth_stripping = is_worth_stripping(&disc);

        let header = disc.header();
        let meta = disc.meta();

        Ok(Self {
            game_dir: parent_dir.to_path_buf(),
            disc_path,

            // discheader
            id: header.game_id.into(),
            title: header.game_title_str().to_string(),
            is_wii: header.is_wii(),
            is_gc: header.is_gamecube(),
            disc_num: header.disc_num,
            disc_version: header.disc_version,

            // discmeta
            format: meta.format,
            compression: meta.compression,
            block_size: meta.block_size.map(Size::from_bytes),
            decrypted: meta.decrypted,
            needs_hash_recovery: meta.needs_hash_recovery,
            lossless: meta.lossless,
            disc_size: meta.disc_size.map(Size::from_bytes),
            crc32: meta.crc32,
            md5: meta.md5,
            sha1: meta.sha1,
            xxh64: meta.xxh64,

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
