// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::app::App;
use crate::messages::Message;
use crate::{games::GameID, http};
use anyhow::Result;
use egui_phosphor::regular as ph;
use std::{fs, path::Path};

fn cache_cover3d(id: GameID, cache_dir: &Path) -> Result<bool> {
    let path = cache_dir.join(id.as_str()).with_extension("png");
    if path.exists() {
        return Ok(false);
    }

    let url = format!(
        "https://art.gametdb.com/wii/cover3D/{}/{}.png",
        id.get_wiitdb_lang(),
        id.as_str()
    );

    http::download_file(&url, &path)?;

    Ok(true)
}

fn download_cover3d(id: GameID, mount_point: &Path) -> Result<()> {
    let images_dir = mount_point.join("apps").join("usbloader_gx").join("images");
    fs::create_dir_all(&images_dir)?;

    let path = images_dir.join(id.as_str()).with_extension("png");
    if path.exists() {
        return Ok(());
    }

    let url = format!(
        "https://art.gametdb.com/wii/cover3D/{}/{}.png",
        id.get_wiitdb_lang(),
        id.as_str()
    );

    http::download_file(&url, &path)?;

    Ok(())
}

fn download_cover2d(id: GameID, mount_point: &Path) -> Result<()> {
    let images_dir = mount_point
        .join("apps")
        .join("usbloader_gx")
        .join("images")
        .join("2D");
    fs::create_dir_all(&images_dir)?;

    let path = images_dir.join(id.as_str()).with_extension("png");
    if path.exists() {
        return Ok(());
    }

    let url = format!(
        "https://art.gametdb.com/wii/cover/{}/{}.png",
        id.get_wiitdb_lang(),
        id.as_str()
    );

    http::download_file(&url, &path)?;

    Ok(())
}

fn download_coverfull(id: GameID, mount_point: &Path) -> Result<()> {
    let images_dir = mount_point
        .join("apps")
        .join("usbloader_gx")
        .join("images")
        .join("full");
    fs::create_dir_all(&images_dir)?;

    let path = images_dir.join(id.as_str()).with_extension("png");
    if path.exists() {
        return Ok(());
    }

    let url = format!(
        "https://art.gametdb.com/wii/coverfull/{}/{}.png",
        id.get_wiitdb_lang(),
        id.as_str()
    );

    http::download_file(&url, &path)?;

    Ok(())
}

fn download_disc_cover(id: GameID, mount_point: &Path) -> Result<()> {
    let images_dir = mount_point
        .join("apps")
        .join("usbloader_gx")
        .join("images")
        .join("disc");
    fs::create_dir_all(&images_dir)?;

    let path = images_dir.join(id.as_str()).with_extension("png");
    if path.exists() {
        return Ok(());
    }

    let url = format!(
        "https://art.gametdb.com/wii/disc/{}/{}.png",
        id.get_wiitdb_lang(),
        id.as_str()
    );

    http::download_file(&url, &path)?;

    Ok(())
}

fn download_wiiflow_boxcover(id: GameID, mount_point: &Path) -> Result<()> {
    let cover_dir = mount_point.join("wiiflow").join("boxcovers");
    fs::create_dir_all(&cover_dir)?;

    let path = cover_dir.join(format!("{}.png", id.as_str()));
    if path.exists() {
        return Ok(());
    }

    let url = format!(
        "https://art.gametdb.com/wii/coverfull/{}/{}.png",
        id.get_wiitdb_lang(),
        id.as_str()
    );

    http::download_file(&url, &path)?;

    Ok(())
}

fn download_wiiflow_cover(id: GameID, mount_point: &Path) -> Result<()> {
    let cover_dir = mount_point.join("wiiflow").join("covers");
    fs::create_dir_all(&cover_dir)?;

    let path = cover_dir.join(format!("{}.png", id.as_str()));
    if path.exists() {
        return Ok(());
    }

    let url = format!(
        "https://art.gametdb.com/wii/cover/{}/{}.png",
        id.get_wiitdb_lang(),
        id.as_str()
    );

    http::download_file(&url, &path)?;

    Ok(())
}

// Fail safe, ignores errors, no popup notification
pub fn spawn_cache_covers_task(app: &App) {
    let covers_dir = app.data_dir.join("covers");
    let games = app.games.clone();

    app.task_processor.spawn(move |msg_sender| {
        fs::create_dir_all(&covers_dir)?;

        msg_sender.send(Message::UpdateStatus(format!(
            "{} Downloading covers...",
            ph::IMAGE
        )))?;

        let len = games.len();
        for (i, game) in games.into_iter().enumerate() {
            msg_sender.send(Message::UpdateStatus(format!(
                "{} Downloading covers... ({}/{})",
                ph::IMAGE,
                i + 1,
                len
            )))?;

            match cache_cover3d(game.id, &covers_dir) {
                Ok(true) => msg_sender.send(Message::TriggerRefreshImage(game.image_uri))?,
                Ok(false) => {}
                Err(e) => log::error!("Failed to download cover: {}", e),
            }
        }

        msg_sender.send(Message::NotifyInfo(format!(
            "{} Covers downloaded",
            ph::IMAGE
        )))?;

        Ok(())
    });
}

pub fn spawn_download_all_covers_task(app: &App) {
    let mount_point = app.config.contents.mount_point.clone();
    let games = app.games.clone();

    app.task_processor.spawn(move |msg_sender| {
        msg_sender.send(Message::UpdateStatus(format!(
            "{} Downloading covers...",
            ph::IMAGE
        )))?;

        let len = games.len();
        for (i, game) in games.into_iter().enumerate() {
            msg_sender.send(Message::UpdateStatus(format!(
                "{} Downloading covers... ({}/{})",
                ph::IMAGE,
                i + 1,
                len
            )))?;

            if let Err(e) = download_cover3d(game.id, &mount_point) {
                msg_sender.send(Message::NotifyError(e))?;
            }

            if let Err(e) = download_cover2d(game.id, &mount_point) {
                msg_sender.send(Message::NotifyError(e))?;
            }

            if let Err(e) = download_coverfull(game.id, &mount_point) {
                msg_sender.send(Message::NotifyError(e))?;
            }

            if let Err(e) = download_disc_cover(game.id, &mount_point) {
                msg_sender.send(Message::NotifyError(e))?;
            }
        }

        msg_sender.send(Message::NotifyInfo(format!(
            "{} Covers downloaded",
            ph::IMAGE
        )))?;

        Ok(())
    });
}

pub fn spawn_download_wiiflow_covers_task(app: &App) {
    let mount_point = app.config.contents.mount_point.clone();
    let games = app.games.clone();

    app.task_processor.spawn(move |msg_sender| {
        msg_sender.send(Message::UpdateStatus(format!(
            "{} Downloading covers...",
            ph::IMAGE
        )))?;

        let len = games.len();
        for (i, game) in games.into_iter().enumerate() {
            msg_sender.send(Message::UpdateStatus(format!(
                "{} Downloading covers... ({}/{})",
                ph::IMAGE,
                i + 1,
                len
            )))?;

            if let Err(e) = download_wiiflow_boxcover(game.id, &mount_point) {
                msg_sender.send(Message::NotifyError(e))?;
            }

            if let Err(e) = download_wiiflow_cover(game.id, &mount_point) {
                msg_sender.send(Message::NotifyError(e))?;
            }
        }

        msg_sender.send(Message::NotifyInfo(format!(
            "{} Covers downloaded",
            ph::IMAGE
        )))?;

        Ok(())
    });
}
