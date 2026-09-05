// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::errors::Error;
use smol::{
    fs::File,
    stream::{Stream, StreamExt},
};
use std::path::{Path, PathBuf};
use wii_disc_info::game_id::GameID;
use zip::ZipArchive;

pub mod covers;
pub mod disc_reader;
pub mod game;
pub mod game_list;
pub mod import;

async fn get_id(path: &Path) -> Result<GameID, Error> {
    let meta = if path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"))
    {
        smol::unblock({
            let path = path.to_path_buf();

            move || {
                let file = std::fs::File::open(path)?;
                let mut zip = ZipArchive::new(file)?;
                let mut entry = zip.by_index(0)?;
                let meta = wii_disc_info::Meta::read(&mut entry)?;
                Ok::<_, Error>(meta)
            }
        })
        .await?
    } else {
        let mut file = File::open(path).await?;
        wii_disc_info::Meta::read_async(&mut file).await?
    };

    Ok(meta.game_id())
}

pub async fn keep_valid_games(
    games: impl Stream<Item = PathBuf>,
    existing_ids: Vec<GameID>,
) -> Vec<PathBuf> {
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
        .filter_map(std::convert::identity)
        .collect()
        .await
}
