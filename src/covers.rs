// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{game::Game, game_id::GameID, http_util, message::Message, state::State};
use anyhow::Result;
use iced::{Task, futures::TryFutureExt};
use std::{
    fs,
    path::{Path, PathBuf},
};

async fn cache_cover3d(id: [u8; 6], cache_dir: &Path) -> Result<()> {
    let path = cache_dir.join(id.as_str()).with_extension("png");
    if path.exists() {
        return Ok(());
    }

    let url = format!(
        "https://art.gametdb.com/wii/cover3D/{}/{}.png",
        id.as_lang_str(),
        id.as_str()
    );

    http_util::download_file(url, &path).await?;

    Ok(())
}

async fn download_cover3d(id: [u8; 6], mount_point: &Path) -> Result<()> {
    let images_dir = mount_point.join("apps").join("usbloader_gx").join("images");
    fs::create_dir_all(&images_dir)?;

    let path = images_dir.join(id.as_str()).with_extension("png");
    if path.exists() {
        return Ok(());
    }

    let url = format!(
        "https://art.gametdb.com/wii/cover3D/{}/{}.png",
        id.as_lang_str(),
        id.as_str()
    );

    http_util::download_file(url, &path).await?;

    Ok(())
}

async fn download_cover2d(id: [u8; 6], mount_point: &Path) -> Result<()> {
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
        id.as_lang_str(),
        id.as_str()
    );

    http_util::download_file(url, &path).await?;

    Ok(())
}

async fn download_coverfull(id: [u8; 6], mount_point: &Path) -> Result<()> {
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
        id.as_lang_str(),
        id.as_str()
    );

    http_util::download_file(url, &path).await?;

    Ok(())
}

async fn download_disc_cover(id: [u8; 6], mount_point: &Path) -> Result<()> {
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
        id.as_lang_str(),
        id.as_str()
    );

    http_util::download_file(url, &path).await?;

    Ok(())
}

async fn download_wiiflow_boxcover(id: [u8; 6], mount_point: &Path) -> Result<()> {
    let cover_dir = mount_point.join("wiiflow").join("boxcovers");
    fs::create_dir_all(&cover_dir)?;

    let path = cover_dir.join(format!("{}.png", id.as_str()));
    if path.exists() {
        return Ok(());
    }

    let url = format!(
        "https://art.gametdb.com/wii/coverfull/{}/{}.png",
        id.as_lang_str(),
        id.as_str()
    );

    http_util::download_file(url, &path).await?;

    Ok(())
}

async fn download_wiiflow_cover(id: [u8; 6], mount_point: &Path) -> Result<()> {
    let cover_dir = mount_point.join("wiiflow").join("covers");
    fs::create_dir_all(&cover_dir)?;

    let path = cover_dir.join(format!("{}.png", id.as_str()));
    if path.exists() {
        return Ok(());
    }

    let url = format!(
        "https://art.gametdb.com/wii/cover/{}/{}.png",
        id.as_lang_str(),
        id.as_str()
    );

    http_util::download_file(url, &path).await?;

    Ok(())
}

pub fn get_cache_cover3ds_task(state: &State) -> Task<Message> {
    let cache_dir = state.data_dir.join("covers");
    let games = state.games.clone();

    Task::perform(
        cache_cover3ds(cache_dir, games).map_err(|e| e.to_string()),
        Message::EmptyResult,
    )
}

// ignores errors, no popup notifications
async fn cache_cover3ds(cache_dir: PathBuf, games: Box<[Game]>) -> Result<()> {
    fs::create_dir_all(&cache_dir)?;

    for game in games {
        let _ = cache_cover3d(game.id, &cache_dir).await;
    }

    Ok(())
}

#[cfg(false)]
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

#[cfg(false)]
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
