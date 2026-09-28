// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;
use smol::{
    fs::File,
    stream::{Stream, StreamExt},
};
use std::{
    convert::identity,
    ffi::OsStr,
    path::{Path, PathBuf},
};
use wii_disc_info::game_id::GameID;
use zip::ZipArchive;

pub mod banners;
pub mod calc_sha1;
pub mod conversion_state;
pub mod convert;
pub mod covers;
pub mod disc_reader;
pub mod export;
pub mod game;
pub mod games_state;
pub mod import;
pub mod txtcodes;

#[derive(Debug, Clone)]
struct Entry {
    path: PathBuf,
    game_id: GameID,
    fingerprint: [u8; 5],
}

impl Entry {
    fn new(path: PathBuf, game_id: GameID) -> Self {
        let mut fingerprint = [0u8; 5];

        let id_compact = game_id.to_u32().to_be_bytes();
        fingerprint[0..4].copy_from_slice(&id_compact);

        let is_disc1 = is_x(&path, "(Disc 1)");
        let is_disc2 = is_x(&path, "(Disc 2)");

        let flags = (is_disc1 as u8) << 1 | (is_disc2 as u8);
        fingerprint[4] = flags;

        Self {
            path,
            game_id,
            fingerprint,
        }
    }

    fn is_multidisc(&self) -> bool {
        self.fingerprint[4] != 0
    }
}

impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        self.fingerprint == other.fingerprint
    }
}

impl Eq for Entry {}

impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Entry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.fingerprint.cmp(&other.fingerprint)
    }
}

async fn get_id(path: &Path) -> Result<GameID> {
    let is_zip = path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"));

    if is_zip {
        smol::unblock({
            let path = path.to_path_buf();

            move || {
                let f = std::fs::File::open(path)?;
                let mut archive = ZipArchive::new(f)?;
                let mut first_entry = archive.by_index(0)?;
                let meta = wii_disc_info::Meta::read(&mut first_entry)?;
                Ok(meta)
            }
        })
        .await
    } else {
        let mut f = File::open(path).await?;
        let meta = wii_disc_info::Meta::read_async(&mut f).await?;
        Ok(meta)
    }
    .map(|meta| meta.game_id())
}

fn is_x(path: &Path, x: &str) -> bool {
    path.file_name()
        .and_then(OsStr::to_str)
        .is_some_and(|s| s.contains(x))
}

pub async fn keep_valid_games(
    games: impl Stream<Item = PathBuf>,
    existing_ids: &[GameID],
) -> Vec<PathBuf> {
    let mut entries = games
        .then(|path| async {
            match get_id(&path).await {
                Ok(id) => Some(Entry::new(path, id)),
                _ => None,
            }
        })
        .filter_map(identity)
        .collect::<Vec<_>>()
        .await;

    // remove duplicates
    entries.sort_unstable();
    entries.dedup();

    // filter out existing games
    entries.retain(|entry| {
        let exists = existing_ids.contains(&entry.game_id);

        // if it's a dual disc, try anyways
        !exists || entry.is_multidisc()
    });

    entries.into_iter().map(|entry| entry.path).collect()
}
