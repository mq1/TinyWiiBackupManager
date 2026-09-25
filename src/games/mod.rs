// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;
use smol::{
    fs::File,
    stream::{Stream, StreamExt},
};
use std::{
    convert::identity,
    path::{Path, PathBuf},
};
use wii_disc_info::game_id::GameID;
use zip::ZipArchive;

pub mod calc_sha1;
pub mod conversion_state;
pub mod covers;
pub mod disc_reader;
pub mod export;
pub mod game;
pub mod games_state;
pub mod import;
pub mod txtcodes;

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

pub fn keep_valid_games(
    games: impl Stream<Item = PathBuf>,
    existing_ids: &[GameID],
) -> impl Stream<Item = PathBuf> {
    games
        .then(|p| async {
            match get_id(&p).await {
                Ok(id) => {
                    let exists = existing_ids.contains(&id);
                    let filename = p.file_name().unwrap_or_default().to_string_lossy();

                    (!exists || filename.contains("(Disc 1)") || filename.contains("(Disc 2)"))
                        .then_some(p)
                }
                _ => None,
            }
        })
        .filter_map(identity)
}
