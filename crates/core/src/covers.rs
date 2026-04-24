// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::game_id::GameID;
use crate::http;
use anyhow::{Result, bail};
use arrayvec::ArrayString;
use std::fmt::Write;
use std::{fs, path::Path};

#[must_use]
fn lang_str(game_id: GameID) -> &'static str {
    let mut s = ArrayString::<6>::new();
    write!(s, "{game_id}").unwrap();

    match s.chars().nth(3) {
        Some('E' | 'N') => "US",
        Some('J') => "JA",
        Some('K' | 'Q' | 'T') => "KO",
        Some('R') => "RU",
        Some('W') => "ZH",
        _ => "EN",
    }
}

pub fn download_cover(game_id: GameID, data_dir: &Path) -> Result<()> {
    let cover_path = data_dir.join(format!("covers/{game_id}.png"));

    if cover_path.exists() {
        bail!("Cover already exists");
    }

    let cover_url = format!(
        "https://art.gametdb.com/wii/cover3D/{}/{}.png",
        lang_str(game_id),
        game_id,
    );

    let body = http::get_vec(&cover_url)?;
    fs::write(&cover_path, &body)?;

    Ok(())
}
