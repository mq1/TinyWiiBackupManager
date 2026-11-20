// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::Config,
    util::http::{download_file, post_then_download_file},
};
use anyhow::{Context, Result, bail};
use twbm_common::config_contents::TxtCodesSource;
use wii_disc_info::game_id::GameID;

pub async fn download_cheats(game_id: GameID, config: &Config) -> Result<()> {
    let mount_point = config
        .contents
        .mount_point
        .as_deref()
        .context("Invalid mount point")?;

    let dest = mount_point
        .join("txtcodes")
        .join(game_id.as_str())
        .with_added_extension("txt");

    match config.contents.txt_codes_source {
        TxtCodesSource::WebArchive => {
            let uri = format!(
                "https://raw.githubusercontent.com/mq1/GeckoArchive/refs/heads/main/codes/{game_id}.txt"
            );

            download_file(&uri, &dest).await?;
        }
        TxtCodesSource::GameHacking => {
            let Some(ghid) = twbm_idmap::get_ghid(game_id) else {
                bail!("Could not find gamehacking id");
            };

            let uri = "https://gamehacking.org/inc/sub.exportCodes.php".to_string();

            let data =
                format!("format=Text&filename={game_id}&sysID=22&gamID={ghid}&download=true");

            post_then_download_file(&uri, data, &dest).await?;
        }
        TxtCodesSource::Rc24 => {
            let uri = format!("https://codes.rc24.xyz/txt.php?txt={game_id}");

            download_file(&uri, &dest).await?;
        }
    };

    Ok(())
}
