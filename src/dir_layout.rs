// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{disc_info::DiscInfo, overflow_writer::get_overflow_path, util::SANITIZE_OPTS};
use anyhow::Result;
use nod::common::Format;
use sanitize_filename::sanitize_with_options;
use std::{fs, path::Path};
use walkdir::WalkDir;

pub fn normalize_games_dir(games_dir: &Path) -> Result<()> {
    for entry in WalkDir::new(games_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_dir())
    {
        let path = entry.path();

        if let Ok(disc_info) = DiscInfo::from_game_dir(path) {
            let game_id = disc_info.id.as_str();

            let new_disc_name = match disc_info.format {
                Format::Wbfs => format!("{}.wbfs", game_id),
                Format::Ciso => match disc_info.disc_num {
                    0 => "game.ciso".to_string(),
                    n => format!("disc{}.ciso", n + 1),
                },
                Format::Iso => match disc_info.is_wii {
                    true => format!("{}.iso", game_id),
                    false => match disc_info.disc_num {
                        0 => "game.iso".to_string(),
                        n => format!("disc{}.iso", n + 1),
                    },
                },
                _ => continue,
            };
            let new_disc_path = path.join(new_disc_name);

            if disc_info.disc_path != new_disc_path {
                fs::rename(&disc_info.disc_path, &new_disc_path)?;
            }

            if let Some(overflow_path) = get_overflow_path(&disc_info.disc_path)
                && overflow_path.exists()
                && let Some(new_overflow_path) = get_overflow_path(&new_disc_path)
                && overflow_path != new_overflow_path
            {
                fs::rename(overflow_path, new_overflow_path)?;
            }

            let new_game_dir = games_dir.join(format!(
                "{} [{}]",
                sanitize_with_options(&disc_info.title, SANITIZE_OPTS),
                disc_info.id.as_str()
            ));

            if path != new_game_dir {
                fs::rename(path, new_game_dir)?;
            }
        }
    }

    Ok(())
}

pub fn normalize_paths(mount_point: &Path) -> Result<()> {
    normalize_games_dir(&mount_point.join("wbfs"))?;
    normalize_games_dir(&mount_point.join("games"))?;

    Ok(())
}
