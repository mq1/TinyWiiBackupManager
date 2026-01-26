// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::games::extensions::SUPPORTED_DISC_EXTENSIONS;
use crate::games::game_id::GameID;
use anyhow::{Result, anyhow, bail};
use derive_getters::Getters;
use nod::common::{Compression, Format, PartitionKind};
use nod::read::{DiscOptions, DiscReader, PartitionEncryption, PartitionOptions};
use size::Size;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

const DISC_OPTS: DiscOptions = DiscOptions {
    partition_encryption: PartitionEncryption::Original,
    preloader_threads: 0,
};

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Getters)]
pub struct DiscInfo {
    game_dir: PathBuf,
    disc_path: PathBuf,

    // discheader
    #[getter(copy)]
    id: GameID,
    title: String,
    #[getter(copy)]
    is_wii: bool,
    #[getter(copy)]
    is_gc: bool,
    #[getter(copy)]
    disc_num: u8,
    #[getter(copy)]
    disc_version: u8,

    // discmeta
    #[getter(copy)]
    format: Format,
    #[getter(copy)]
    compression: Compression,
    #[getter(copy)]
    block_size: Option<Size>,
    #[getter(copy)]
    decrypted: bool,
    #[getter(copy)]
    needs_hash_recovery: bool,
    #[getter(copy)]
    lossless: bool,
    #[getter(copy)]
    disc_size: Option<Size>,
    #[getter(copy)]
    crc32: Option<u32>,
    #[getter(copy)]
    md5: Option<[u8; 16]>,
    #[getter(copy)]
    sha1: Option<[u8; 20]>,
    #[getter(copy)]
    xxh64: Option<u64>,

    // misc
    #[getter(copy)]
    is_worth_stripping: bool,
}

pub fn get_main_disc_file_in_dir(dir: &Path) -> Result<PathBuf> {
    let entries = fs::read_dir(dir)?;
    for entry in entries.filter_map(Result::ok) {
        if let Ok(file_type) = entry.file_type()
            && file_type.is_file()
            && let Some(filename) = entry.file_name().to_str()
            && !filename.ends_with(".part1.iso")
            && (filename.ends_with(".wbfs")
                || filename.ends_with(".ciso")
                || (filename.ends_with(".iso")))
        {
            return Ok(entry.path());
        }
    }

    Err(anyhow!("No disc file found"))
}

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

            if let Ok(disc_info) = Self::try_from_path(disc_path) {
                return Ok(disc_info);
            }
        }

        Err(anyhow!("No disc file found"))
    }

    pub fn try_from_path(disc_path: PathBuf) -> Result<Self> {
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

        let Some(ext) = disc_path.extension().and_then(OsStr::to_str) else {
            bail!("No file extension");
        };

        if !SUPPORTED_DISC_EXTENSIONS.contains(&ext) {
            bail!("Unsupported file extension");
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
