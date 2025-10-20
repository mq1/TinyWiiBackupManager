// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    games::{self, GameID},
    http::AGENT,
    tasks::TaskProcessor,
};
use anyhow::Result;
use parking_lot::Mutex;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

fn download_cover3d(id: &GameID, mount_point: &Path) -> Result<()> {
    let images_dir = mount_point.join("apps").join("usbloader_gx").join("images");
    fs::create_dir_all(&images_dir)?;

    let path = images_dir.join(id.as_ref()).with_extension("png");

    let url = format!(
        "https://art.gametdb.com/wii/cover3D/{}/{}.png",
        id.get_wiitdb_lang(),
        id.as_ref()
    );

    if !path.exists() {
        let bytes = AGENT.get(&url).call()?.body_mut().read_to_vec()?;
        fs::write(&path, bytes)?;
    }

    Ok(())
}

fn download_cover2d(id: &GameID, mount_point: &Path) -> Result<()> {
    let images_dir = mount_point
        .join("apps")
        .join("usbloader_gx")
        .join("images")
        .join("2D");
    fs::create_dir_all(&images_dir)?;

    let path = images_dir.join(id.as_ref()).with_extension("png");

    let url = format!(
        "https://art.gametdb.com/wii/cover/{}/{}.png",
        id.get_wiitdb_lang(),
        id.as_ref()
    );

    if !path.exists() {
        let bytes = AGENT.get(&url).call()?.body_mut().read_to_vec()?;
        fs::write(&path, bytes)?;
    }

    Ok(())
}

fn download_coverfull(id: &GameID, mount_point: &Path) -> Result<()> {
    let images_dir = mount_point
        .join("apps")
        .join("usbloader_gx")
        .join("images")
        .join("full");
    fs::create_dir_all(&images_dir)?;

    let path = images_dir.join(id.as_ref()).with_extension("png");

    let url = format!(
        "https://art.gametdb.com/wii/coverfull/{}/{}.png",
        id.get_wiitdb_lang(),
        id.as_ref()
    );

    if !path.exists() {
        let bytes = AGENT.get(&url).call()?.body_mut().read_to_vec()?;
        fs::write(&path, bytes)?;
    }

    // for WiiFlow lite
    let wiiflow_cover_dir = mount_point.join("wiiflow").join("boxcovers");
    fs::create_dir_all(&wiiflow_cover_dir)?;
    let dest = wiiflow_cover_dir.join(format!("{}.png", id.as_ref()));
    if !dest.exists() {
        fs::copy(&path, &dest)?;
    }

    Ok(())
}

fn download_disc_cover(id: &GameID, mount_point: &Path) -> Result<()> {
    let images_dir = mount_point
        .join("apps")
        .join("usbloader_gx")
        .join("images")
        .join("disc");
    fs::create_dir_all(&images_dir)?;

    let path = images_dir.join(id.as_ref()).with_extension("png");

    let url = format!(
        "https://art.gametdb.com/wii/disc/{}/{}.png",
        id.get_wiitdb_lang(),
        id.as_ref()
    );

    if !path.exists() {
        let bytes = AGENT.get(&url).call()?.body_mut().read_to_vec()?;
        fs::write(&path, bytes)?;
    }

    Ok(())
}

// Fail safe, ignores errors, no popup notification
pub fn spawn_download_covers_task(
    mount_point: PathBuf,
    task_processor: &TaskProcessor,
    should_refresh_images: Arc<Mutex<bool>>,
) {
    task_processor.spawn(move |status, toasts| {
        *status.lock() = "🖻 Downloading covers...".to_string();

        let games = games::list(&mount_point, &None)?;
        let len = games.len();
        for (i, game) in games.iter().enumerate() {
            *status.lock() = format!("🖻 Downloading covers... ({}/{})", i + 1, len);
            let _ = download_cover3d(&game.id, &mount_point);
        }

        toasts.lock().info("🖻 Covers downloaded".to_string());
        *should_refresh_images.lock() = true;

        Ok(())
    });
}

pub fn spawn_download_all_covers_task(
    mount_point: PathBuf,
    task_processor: &TaskProcessor,
    should_refresh_images: Arc<Mutex<bool>>,
) {
    task_processor.spawn(move |status, toasts| {
        *status.lock() = "🖻 Downloading covers...".to_string();

        let games = games::list(&mount_point, &None)?;
        let len = games.len();
        for (i, game) in games.iter().enumerate() {
            *status.lock() = format!("🖻 Downloading covers... ({}/{})", i + 1, len);

            let _ = download_cover3d(&game.id, &mount_point);
            let _ = download_cover2d(&game.id, &mount_point);
            let _ = download_coverfull(&game.id, &mount_point);
            let _ = download_disc_cover(&game.id, &mount_point);
        }

        toasts.lock().info("🖻 Covers downloaded".to_string());
        *should_refresh_images.lock() = true;

        Ok(())
    });
}
