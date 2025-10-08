// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{TaskType, games, http::AGENT, tasks::TaskProcessor, titles::Titles};
use anyhow::Result;
use slint::ToSharedString;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

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

fn download_cover2d(id: &str, mount_point: &Path) -> Result<()> {
    let images_dir = mount_point
        .join("apps")
        .join("usbloader_gx")
        .join("images")
        .join("2D");
    fs::create_dir_all(&images_dir)?;

    let path = images_dir.join(id).with_extension("png");

    let url = format!(
        "https://art.gametdb.com/wii/cover/{}/{}.png",
        get_lang(id),
        id
    );

    if !path.exists() {
        let bytes = AGENT.get(&url).call()?.body_mut().read_to_vec()?;
        fs::write(&path, bytes)?;
    }

    Ok(())
}

fn download_coverfull(id: &str, mount_point: &Path) -> Result<()> {
    let images_dir = mount_point
        .join("apps")
        .join("usbloader_gx")
        .join("images")
        .join("full");
    fs::create_dir_all(&images_dir)?;

    let path = images_dir.join(id).with_extension("png");

    let url = format!(
        "https://art.gametdb.com/wii/coverfull/{}/{}.png",
        get_lang(id),
        id
    );

    if !path.exists() {
        let bytes = AGENT.get(&url).call()?.body_mut().read_to_vec()?;
        fs::write(&path, bytes)?;
    }

    // for WiiFlow lite
    let wiiflow_cover_dir = mount_point.join("wiiflow").join("boxcovers");
    fs::create_dir_all(&wiiflow_cover_dir)?;
    let dest = wiiflow_cover_dir.join(format!("{id}.png"));
    if !dest.exists() {
        fs::copy(&path, &dest)?;
    }

    Ok(())
}

fn download_disc_cover(id: &str, mount_point: &Path) -> Result<()> {
    let images_dir = mount_point
        .join("apps")
        .join("usbloader_gx")
        .join("images")
        .join("disc");
    fs::create_dir_all(&images_dir)?;

    let path = images_dir.join(id).with_extension("png");

    let url = format!(
        "https://art.gametdb.com/wii/disc/{}/{}.png",
        get_lang(id),
        id
    );

    if !path.exists() {
        let bytes = AGENT.get(&url).call()?.body_mut().read_to_vec()?;
        fs::write(&path, bytes)?;
    }

    Ok(())
}

// Fail safe, ignores errors, no popup notification
pub fn download_covers(mount_point: PathBuf, task_processor: &Arc<TaskProcessor>) -> Result<()> {
    task_processor.spawn(Box::new(move |weak| {
        weak.upgrade_in_event_loop(move |handle| {
            handle.set_status("Downloading covers...".to_shared_string());
            handle.set_task_type(TaskType::DownloadingCovers);
        })?;

        let empty_titles = Arc::new(Titles::empty());
        let games = games::list(&mount_point, &empty_titles)?;
        let len = games.len();
        for (i, game) in games.iter().enumerate() {
            weak.upgrade_in_event_loop(move |handle| {
                let status = format!("Downloading covers... ({}/{})", i + 1, len);
                handle.set_status(status.to_shared_string());
            })?;

            let _ = download_cover3d(&game.id, &mount_point);
        }

        Ok(String::new())
    }))
}

pub fn download_all_covers(
    mount_point: PathBuf,
    task_processor: &Arc<TaskProcessor>,
) -> Result<()> {
    task_processor.spawn(Box::new(move |weak| {
        weak.upgrade_in_event_loop(move |handle| {
            handle.set_status("Downloading covers...".to_shared_string());
            handle.set_task_type(TaskType::DownloadingCovers);
        })?;

        let empty_titles = Arc::new(Titles::empty());
        let games = games::list(&mount_point, &empty_titles)?;
        let len = games.len();
        for (i, game) in games.iter().enumerate() {
            weak.upgrade_in_event_loop(move |handle| {
                let status = format!("Downloading covers... ({}/{})", i + 1, len);
                handle.set_status(status.to_shared_string());
            })?;

            let _ = download_cover3d(&game.id, &mount_point);
            let _ = download_cover2d(&game.id, &mount_point);
            let _ = download_coverfull(&game.id, &mount_point);
            let _ = download_disc_cover(&game.id, &mount_point);
        }

        Ok("Covers downloaded".to_string())
    }))
}
