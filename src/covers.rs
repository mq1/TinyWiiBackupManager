// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{TaskType, http::AGENT, tasks::TaskProcessor};
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

pub fn download_covers(mount_point: &Path, task_processor: &Arc<TaskProcessor>) -> Result<()> {
    let images_dir = mount_point.join("apps").join("usbloader_gx").join("images");
    fs::create_dir_all(&images_dir)?;

    let game_dirs = [mount_point.join("wbfs"), mount_point.join("games")];

    let ids = game_dirs
        .iter()
        .map(fs::read_dir)
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter_map(|name| name.split_once('[').map(|(_, id)| id.to_string()))
        .filter_map(|id| id.strip_suffix(']').map(|id| id.to_string()))
        .collect::<Vec<_>>();

    for id in ids {
        let path = images_dir.join(&id).with_extension("png");

        task_processor.spawn(Box::new(move |weak| {
            weak.upgrade_in_event_loop(move |handle| {
                handle.set_task_type(TaskType::DownloadingCovers);
            })?;

            if !path.exists() {
                let url = format!(
                    "https://art.gametdb.com/wii/cover3D/{}/{}.png",
                    get_lang(&id),
                    &id
                );
                let bytes = AGENT.get(&url).call()?.body_mut().read_to_vec()?;
                fs::write(&path, bytes)?;
            }

            Ok(())
        }))?;
    }

    Ok(())
}
