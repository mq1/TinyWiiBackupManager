// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::Logic;
use anyhow::Result;
use slint::{SharedString, ToSharedString, Weak};
use std::fs;
use twbm_core::{
    config::PreferredLanguage,
    covers::{CoverType, download_cover},
    data_dir::DATA_DIR,
    game_id::GameID,
};

pub fn download_covers(
    ids: Vec<GameID>,
    preferred_language: PreferredLanguage,
    weak: &Weak<Logic<'static>>,
) -> Result<()> {
    let covers_dir = DATA_DIR.join("covers");
    fs::create_dir_all(&covers_dir)?;

    for (i, game_id) in ids.into_iter().enumerate() {
        if download_cover(game_id, CoverType::Cover3D, &covers_dir, preferred_language)
            .unwrap_or(false)
        {
            let _ = weak.upgrade_in_event_loop(move |logic| {
                logic.invoke_dispatch(crate::Action::ReloadCover, i.to_shared_string());
            });
        }
    }

    let _ = weak.upgrade_in_event_loop(move |logic| {
        logic.invoke_dispatch(
            crate::Action::FinishedDownloadingCovers,
            SharedString::new(),
        );
    });

    Ok(())
}
