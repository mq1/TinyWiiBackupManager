// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::DiscInfo;
use anyhow::{Result, anyhow, bail};
use nod::{
    common::{Format, PartitionKind},
    read::{DiscOptions, DiscReader, PartitionEncryption, PartitionOptions},
};
use slint::{SharedString, ToSharedString};
use std::{
    ffi::OsStr,
    fs::{self, File},
    path::Path,
};

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

        let mut f = File::open(disc_path)?;
        let meta = wii_disc_info::Meta::read(&mut f)?;
        let is_worth_scrubbing = is_worth_scrubbing(disc_path);

        let crc32_path = disc_path.with_file_name(format!("{}.crc32", meta.game_id()));
        let crc32 = fs::read_to_string(crc32_path).unwrap_or_default();

        Ok(Self {
            format: meta.format().to_shared_string(),
            game_id: meta.game_id().to_shared_string(),
            game_title: meta.game_title().to_shared_string(),
            region: meta.region().to_shared_string(),
            is_wii: meta.is_wii(),
            is_gc: meta.is_gc(),
            disc_number: meta.disc_number().into(),
            disc_version: meta.disc_version().into(),
            is_worth_scrubbing,
            crc32: crc32.trim().to_shared_string(),
            err: SharedString::new(),
        })
    }
}

// Returns true if the disc is worth scrubbing (update partion)
// Currently checks if the update partition is >= 8 MiB
pub fn is_worth_scrubbing(disc_path: &Path) -> bool {
    let Ok(disc) = DiscReader::new(disc_path, &DISC_OPTS) else {
        return false;
    };

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
