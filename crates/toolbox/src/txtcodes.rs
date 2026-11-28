// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{ToolboxGroup, ToolboxItem};
use anyhow::{Context, Result, bail};
use itertools::Itertools;
use twbm_common::config_contents::{ConfigContents, TxtCodesSource};
use twbm_http::{download_file, post_then_download_file};
use wii_disc_info::game_id::GameID;

pub fn download_cheats(game_id: GameID, config: &ConfigContents) -> Result<()> {
    let mount_point = config
        .mount_point
        .as_deref()
        .context("Invalid mount point")?;

    let dest = mount_point.join("txtcodes").join(format!("{game_id}.txt"));

    match config.txt_codes_source {
        TxtCodesSource::WebArchive => {
            let uri = format!(
                "https://raw.githubusercontent.com/mq1/GeckoArchive/refs/heads/main/codes/{game_id}.txt"
            );

            download_file(&uri, &dest)?;
        }
        TxtCodesSource::GameHacking => {
            let Some(ghid) = twbm_idmap::get_ghid(game_id) else {
                bail!("Could not find gamehacking id");
            };

            let uri = "https://gamehacking.org/inc/sub.exportCodes.php".to_string();

            let data =
                format!("format=Text&filename={game_id}&sysID=22&gamID={ghid}&download=true");

            post_then_download_file(&uri, data, &dest)?;
        }
        TxtCodesSource::Rc24 => {
            let uri = format!("https://codes.rc24.xyz/txt.php?txt={game_id}");

            download_file(&uri, &dest)?;
        }
    };

    Ok(())
}

pub const ALL: &[ToolboxGroup] = {
    &[ToolboxGroup {
        label: "Cheats",
        icon: "skull",
        items: &[ToolboxItem {
            label: "Download cheats for all games (source configurable in settings)",
            run: |ctx| {
                let mut failed = Vec::with_capacity(ctx.game_ids.len());

                for game_id in ctx.game_ids {
                    if download_cheats(game_id, &ctx.config).is_err() {
                        failed.push(game_id);
                    }
                }

                let msg = if failed.is_empty() {
                    "Downloaded cheats for all games".to_string()
                } else {
                    format!(
                        "Downloaded cheats for all games except for: {}",
                        failed.iter().format(", ")
                    )
                };

                Ok(msg)
            },
        }],
    }]
};
