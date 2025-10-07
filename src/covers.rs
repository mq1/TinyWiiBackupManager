// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{games, http::AGENT, titles::Titles};
use anyhow::Result;
use std::{fs, path::Path, sync::Arc};

fn get_lang(id: &str) -> &'static str {
    let region_code = id.chars().nth(3).unwrap_or_default();

    match region_code {
        'E' | 'N' => "US",
        'J' => "JA",
        'K' | 'Q' | 'T' => "KO",
        'R' => "RU",
        'W' => "ZH",
        _ => "EN",
    }
}

fn download_cover3d(id: &str, mount_point: &Path) -> Result<()> {
    let images_dir = mount_point.join("apps").join("usbloader_gx").join("images");
    fs::create_dir_all(&images_dir)?;

    let path = images_dir.join(id).with_extension("png");

    let url = format!(
        "https://art.gametdb.com/wii/cover3D/{}/{}.png",
        get_lang(id),
        id
    );

    if !path.exists() {
        let bytes = AGENT.get(&url).call()?.body_mut().read_to_vec()?;
        fs::write(&path, bytes)?;
    }

    Ok(())
}

// Fail safe, ignores errors
pub fn download_covers(mount_point: &Path) -> Result<()> {
    let empty_titles = Arc::new(Titles::empty());

    let games = games::list(mount_point, &empty_titles)?;
    for game in games {
        let _ = download_cover3d(&game.id, mount_point);
    }

    Ok(())
}
