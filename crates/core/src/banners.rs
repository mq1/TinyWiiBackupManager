// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{game_id::GameID, util::AGENT};
use anyhow::Result;
use std::{fs, path::Path};

fn download_banner_for_game(mount_point: &Path, game_id: GameID) -> Result<()> {
    let parent = mount_point.join("cache_bnr");
    let filename = format!("{game_id}.bnr");
    let path = parent.join(filename);

    if path.exists() {
        return Ok(());
    }

    fn get(url: &str) -> Result<Vec<u8>, ureq::Error> {
        AGENT.get(url).call()?.body_mut().read_to_vec()
    }

    let url = format!("https://banner.rc24.xyz/{game_id}.bnr");

    let bytes = match get(&url) {
        Ok(bytes) => bytes,
        Err(_) => {
            let url = format!("https://banner.rc24.xyz/{}.bnr", game_id.partial());
            get(&url)?
        }
    };

    fs::create_dir_all(&parent)?;
    fs::write(&path, bytes)?;

    Ok(())
}

pub fn download_banners(mount_point: &Path, game_ids: &[GameID]) -> Result<Vec<GameID>> {
    let mut failed_ids = Vec::new();

    for game_id in game_ids {
        if download_banner_for_game(mount_point, *game_id).is_err() {
            failed_ids.push(*game_id);
        }
    }

    Ok(failed_ids)
}
