// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::game::Game;
use crate::util::concurrency::get_threads_num;
use crate::util::fs::find_disc;
use anyhow::{Context, Result};
use nod::read::{DiscOptions, DiscReader, PartitionEncryption};
use nod::write::{DiscWriter, FormatOptions, ProcessOptions};
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hash {
    CRC32(u32),
    MD5([u8; 16]),
    SHA1([u8; 20]),
    XXH64(u64),
}

impl Hash {
    pub fn get_crc32(&self) -> Option<u32> {
        match self {
            Hash::CRC32(crc32) => Some(*crc32),
            _ => None,
        }
    }
}

static HASH_CACHE: LazyLock<Mutex<HashMap<[u8; 6], Vec<Hash>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn cache_insert(key: [u8; 6], val: Hash) {
    if let Ok(mut map) = HASH_CACHE.lock() {
        map.entry(key).or_default().push(val);
    }
}

pub fn cache_get(key: [u8; 6]) -> Option<Vec<Hash>> {
    HASH_CACHE.lock().ok()?.get(&key).cloned()
}

fn cache_remove(key: [u8; 6]) -> Option<Vec<Hash>> {
    HASH_CACHE.lock().ok()?.remove(&key)
}

/// Returns true if the checksum was already cached, false if it was calculated now
pub fn all(game: &Game, mut progress_callback: impl FnMut(u64, u64)) -> Result<bool> {
    if cache_get(game.id).is_some() {
        return Ok(true);
    }

    let input_path = find_disc(&game)?;
    let (preloader_threads, processor_threads) = get_threads_num();

    let disc = DiscReader::new(
        &input_path,
        &DiscOptions {
            partition_encryption: PartitionEncryption::Original,
            preloader_threads,
        },
    )
    .with_context(|| format!("Failed to read disc image: {}", input_path.display()))?;

    let disc_writer = DiscWriter::new(disc, &FormatOptions::default())
        .context("Failed to initialize disc writer")?;

    let finalization = disc_writer
        .process(
            |_, progress, total| {
                progress_callback(progress, total);
                Ok(())
            },
            &ProcessOptions {
                processor_threads,
                digest_crc32: true,
                digest_md5: true,
                digest_sha1: false,
                digest_xxh64: false,
            },
        )
        .context("Failed to process disc for checksum calculation")?;

    let crc32 = finalization
        .crc32
        .context("Failed to calculate CRC32 checksum")?;
    cache_insert(game.id, Hash::CRC32(crc32));

    let md5 = finalization
        .md5
        .context("Failed to calculate MD5 checksum")?;
    cache_insert(game.id, Hash::MD5(md5));

    //let sha1 = finalization
    //    .sha1
    //    .context("Failed to calculate SHA1 checksum")?;
    //cache_insert(game.id, Hash::SHA1(sha1));

    //let xxh64 = finalization
    //    .xxh64
    //    .context("Failed to calculate XXH64 checksum")?;
    //cache_insert(game.id, Hash::XXH64(xxh64));

    Ok(false)
}
