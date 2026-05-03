// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::{Config, TxtCodesSource},
    game_id::GameID,
    http, id_map,
};
use anyhow::{Result, bail};
use arrayvec::ArrayString;
use std::{fmt::Write, fs};

pub fn download_cheats(game_id: GameID, config: &Config) -> Result<()> {
    let code = match config.contents.txt_codes_source {
        TxtCodesSource::WebArchive => {
            let url = format!(
                "https://raw.githubusercontent.com/mq1/GeckoArchive/refs/heads/main/codes/{game_id}.txt"
            );
            http::get_string(&url)?
        }
        TxtCodesSource::GameHacking => {
            let Some(ghid) = id_map::get(game_id).and_then(|entry| entry.ghid) else {
                bail!("Could not find gamehacking id");
            };

            let mut filename = ArrayString::<6>::new();
            write!(filename, "{game_id}")?;

            let mut gam_id = ArrayString::<10>::new();
            write!(gam_id, "{ghid}")?;

            let form = [
                ("format", "Text"),
                ("filename", filename.as_str()),
                ("sysID", "22"),
                ("gamID", gam_id.as_str()),
                ("download", "true"),
            ];

            http::send_form("https://gamehacking.org/inc/sub.exportCodes.php", form)?
        }
        TxtCodesSource::Rc24 => {
            let url = format!("https://codes.rc24.xyz/txt.php?txt={game_id}");
            http::get_string(&url)?
        }
    };

    let parent_dir = config.contents.mount_point.join("txtcodes");
    fs::create_dir_all(&parent_dir)?;

    let mut filename = ArrayString::<10>::new();
    write!(filename, "{game_id}.txt")?;
    let out_path = parent_dir.join(filename.as_str());

    fs::write(out_path, code)?;

    Ok(())
}
