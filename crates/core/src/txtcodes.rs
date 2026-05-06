// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::{Config, TxtCodesSource},
    game_id::GameID,
    id_map,
    util::AGENT,
};
use anyhow::{Result, bail};
use std::fs;

pub fn download_cheats(game_id: GameID, config: &Config) -> Result<()> {
    let code = match config.contents.txt_codes_source {
        TxtCodesSource::WebArchive => {
            let url = format!(
                "https://raw.githubusercontent.com/mq1/GeckoArchive/refs/heads/main/codes/{game_id}.txt"
            );

            AGENT.get(url).call()?.body_mut().read_to_string()?
        }
        TxtCodesSource::GameHacking => {
            let Some(ghid) = id_map::get(game_id).and_then(|entry| entry.ghid) else {
                bail!("Could not find gamehacking id");
            };

            let form = [
                ("format", "Text"),
                ("filename", &game_id.to_string()),
                ("sysID", "22"),
                ("gamID", &ghid.to_string()),
                ("download", "true"),
            ];

            AGENT
                .post("https://gamehacking.org/inc/sub.exportCodes.php")
                .send_form(form)?
                .body_mut()
                .read_to_string()?
        }
        TxtCodesSource::Rc24 => {
            let url = format!("https://codes.rc24.xyz/txt.php?txt={game_id}");

            AGENT.get(url).call()?.body_mut().read_to_string()?
        }
    };

    let parent_dir = config.contents.mount_point.join("txtcodes");
    fs::create_dir_all(&parent_dir)?;

    let filename = format!("{game_id}.txt");
    let out_path = parent_dir.join(filename);

    fs::write(out_path, code)?;

    Ok(())
}
