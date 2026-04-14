// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{USER_AGENT, data_dir::DATA_DIR};
use anyhow::Result;
use std::fs;

#[must_use]
pub fn lang_str(game_id: &str) -> &'static str {
    let region_char = game_id.chars().nth(3).unwrap_or('\0');

    match region_char {
        'E' | 'N' => "US",
        'J' => "JA",
        'K' | 'Q' | 'T' => "KO",
        'R' => "RU",
        'W' => "ZH",
        _ => "EN",
    }
}

pub fn cache_cover(game_id: &str) -> Result<()> {
    let parent = DATA_DIR.join("covers");
    if !parent.exists() {
        fs::create_dir_all(&parent)?;
    }

    let cover_path = parent.join(format!("{game_id}.png"));

    if !cover_path.exists() {
        let cover_url = format!(
            "https://art.gametdb.com/wii/cover3D/{}/{}.png",
            lang_str(game_id),
            game_id,
        );

        let resp = minreq::get(cover_url)
            .with_header("User-Agent", USER_AGENT)
            .send()?;

        fs::write(cover_path, resp.as_bytes())?;
    }

    Ok(())
}
