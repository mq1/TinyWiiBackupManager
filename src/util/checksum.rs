// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::game::Game;
use crate::util::concurrency::get_threads_num;
use crate::util::fs::find_disc;
use anyhow::{Context, Result};
use nod::read::{DiscOptions, DiscReader, PartitionEncryption};
use nod::write::{DiscFinalization, DiscWriter, FormatOptions, ProcessOptions};
use std::collections::{HashMap, HashSet};
use std::sync::{LazyLock, Mutex};

static HASH_CACHE: LazyLock<Mutex<HashMap<[u8; 6], DiscFinalization>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn cache_insert(id: [u8; 6], finalization: DiscFinalization) {
    if let Ok(mut map) = HASH_CACHE.lock() {
        map.insert(id, finalization);
    }
}

pub fn cache_get(id: [u8; 6]) -> Option<DiscFinalization> {
    HASH_CACHE.lock().ok()?.get(&id).cloned()
}

#[allow(dead_code)]
fn cache_remove(id: [u8; 6]) -> Option<DiscFinalization> {
    HASH_CACHE.lock().ok()?.remove(&id)
}

/// Syncs the cache with the games
/// Removes games that are not in the list
/// So the cache only contains games that are in the list
pub fn sync_games(games: &[Game]) {
    if let Ok(mut cache) = HASH_CACHE.lock() {
        // Get the set of game IDs we want to keep
        let game_ids: HashSet<_> = games.iter().map(|g| g.id).collect();
        
        // Remove all entries that aren't in our current games list
        cache.retain(|id, _| game_ids.contains(id));
    }
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
            strip_partitions: vec![],
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
                digest_md5: false,
                digest_sha1: false,
                digest_xxh64: false,
            },
        )
        .context("Failed to process disc for checksum calculation")?;

    cache_insert(game.id, finalization);

    Ok(false)
}
