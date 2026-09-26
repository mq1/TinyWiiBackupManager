// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::util::http::download_file_with_fallback;
use anyhow::Result;
use smol::stream::{self, StreamExt};
use std::{convert::identity, path::Path};
use tap::Pipe;
use wii_disc_info::game_id::GameID;

pub async fn download_banner(game_id: GameID, root_dir: &Path) -> Result<()> {
    let uri = format!("https://banner.rc24.xyz/{game_id}.bnr");

    let dest = root_dir
        .join("cache_bnr")
        .join(game_id.as_str())
        .with_added_extension("bnr");

    let fallback = format!("https://banner.rc24.xyz/{}.bnr", game_id.as_partial_str());

    download_file_with_fallback(&uri, &dest, &fallback).await
}

fn is_ngc(game_id: &GameID) -> bool {
    let system_code = game_id.to_bytes()[0];
    system_code == b'D' || system_code == b'G'
}

pub async fn download_all_banners(
    game_ids: impl IntoIterator<Item = GameID>,
    root_dir: &Path,
) -> Vec<GameID> {
    game_ids
        .into_iter()
        .filter(is_ngc)
        .pipe(stream::iter)
        .then(move |game_id| async move {
            download_banner(game_id, root_dir)
                .await
                .is_err()
                .then_some(game_id)
        })
        .filter_map(identity)
        .collect::<Vec<_>>()
        .await
}
