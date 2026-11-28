// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;
use std::path::Path;
use twbm_http::download_file_with_fallback;
use wii_disc_info::game_id::GameID;

pub fn download_banner(game_id: GameID, root_dir: &Path) -> Result<()> {
    let uri = format!("https://banner.rc24.xyz/{game_id}.bnr");

    let dest = root_dir
        .join("cache_bnr")
        .join(game_id.as_str())
        .with_added_extension("bnr");

    let fallback = format!("https://banner.rc24.xyz/{}.bnr", game_id.as_partial_str());

    let _ = download_file_with_fallback(&uri, &dest, &fallback)?;
    Ok(())
}

fn is_ngc(game_id: &GameID) -> bool {
    let system_code = game_id.system_code();
    system_code == b'D' || system_code == b'G'
}

pub fn download_all_banners(game_ids: Box<[GameID]>, root_dir: &Path) -> Vec<GameID> {
    let mut errored = Vec::new();

    for game_id in game_ids.into_iter().filter(is_ngc) {
        if download_banner(game_id, root_dir).is_err() {
            errored.push(game_id);
        }
    }

    errored
}
